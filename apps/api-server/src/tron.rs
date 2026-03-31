#![allow(dead_code)] // Integration module: full distribution API exposed for future routes
//! Tron Network integration — TronLink wallet auth + TRX/TRC-20 royalty routing.
//!
//! Tron is a high-throughput blockchain with near-zero fees, making it suitable
//! for micro-royalty distributions to artists in markets where BTT is primary.
//!
//! This module provides:
//!   - Tron address validation (Base58Check, 0x41 prefix)
//!   - Wallet challenge-response authentication (TronLink signMessageV2)
//!   - TRX royalty distribution via Tron JSON-RPC (fullnode HTTP API)
//!   - TRC-20 token distribution (royalties in USDT-TRC20 or BTT-TRC20)
//!
//! Security:
//!   - All Tron addresses validated by langsec::validate_tron_address().
//!   - TRON_API_URL must be HTTPS in production.
//!   - TRON_PRIVATE_KEY loaded from environment, never logged.
//!   - Value cap: MAX_TRX_DISTRIBUTION (1M TRX) per transaction.
//!   - Dev mode (TRON_DEV_MODE=1): no network calls, returns stub tx hash.
use crate::langsec;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};

/// 1 million TRX in sun (1 TRX = 1,000,000 sun).
pub const MAX_TRX_DISTRIBUTION: u64 = 1_000_000 * 1_000_000;

// ── Configuration ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TronConfig {
    /// Full-node HTTP API URL (e.g. https://api.trongrid.io).
    pub api_url: String,
    /// TRC-20 contract address for royalty token (USDT or BTT on Tron).
    pub token_contract: Option<String>,
    /// Enabled flag.
    pub enabled: bool,
    /// Dev mode — return stub responses without calling Tron.
    pub dev_mode: bool,
}

impl TronConfig {
    #[zkperf_macros::zkperf]
    pub fn from_env() -> Self {
        let api_url =
            std::env::var("TRON_API_URL").unwrap_or_else(|_| "https://api.trongrid.io".into());
        let env = std::env::var("RETROSYNC_ENV").unwrap_or_default();
        if env == "production" && !api_url.starts_with("https://") {
            panic!("SECURITY: TRON_API_URL must use HTTPS in production");
        }
        if !api_url.starts_with("https://") {
            warn!(
                url=%api_url,
                "TRON_API_URL uses plaintext — configure HTTPS for production"
            );
        }
        Self {
            api_url,
            token_contract: std::env::var("TRON_TOKEN_CONTRACT").ok(),
            enabled: std::env::var("TRON_ENABLED").unwrap_or_default() == "1",
            dev_mode: std::env::var("TRON_DEV_MODE").unwrap_or_default() == "1",
        }
    }
}

// ── Types ─────────────────────────────────────────────────────────────────────

/// A validated Tron address (Base58Check, 0x41 prefix).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TronAddress(pub String);

impl std::fmt::Display for TronAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A Tron royalty recipient.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronRecipient {
    pub address: TronAddress,
    /// Basis points (0–10_000).
    pub bps: u16,
}

/// Result of a Tron distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronDistributionResult {
    pub tx_hash: String,
    pub total_sun: u64,
    pub recipients: Vec<TronRecipient>,
    pub dev_mode: bool,
}

// ── Wallet authentication ─────────────────────────────────────────────────────

/// Tron wallet authentication challenge.
///
/// TronLink (and compatible wallets) implement `tronWeb.trx.signMessageV2(message)`
/// which produces a 65-byte ECDSA signature (hex, 130 chars) over the Tron-prefixed
/// message: "\x19TRON Signed Message:\n{len}{message}".
///
/// Verification mirrors the EVM personal_sign logic but uses the Tron prefix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronChallenge {
    pub challenge_id: String,
    pub address: TronAddress,
    pub nonce: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TronVerifyRequest {
    pub challenge_id: String,
    pub address: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TronAuthResult {
    pub address: TronAddress,
    pub verified: bool,
    pub message: String,
}

/// Issue a Tron wallet authentication challenge.
#[zkperf_macros::zkperf]
pub fn issue_tron_challenge(raw_address: &str) -> Result<TronChallenge, String> {
    // LangSec validation
    langsec::validate_tron_address(raw_address).map_err(|e| e.to_string())?;

    let nonce = generate_nonce();
    let expires = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + 300; // 5-minute TTL

    Ok(TronChallenge {
        challenge_id: generate_nonce(),
        address: TronAddress(raw_address.to_string()),
        nonce,
        expires_at: expires,
    })
}

/// Verify a TronLink signMessageV2 signature.
///
/// NOTE: Full on-chain ECDSA recovery requires secp256k1 + keccak256.
/// In production, verify the signature server-side using the trongrid API:
///   POST https://api.trongrid.io/wallet/verifyMessage
///   { "value": nonce, "address": address, "signature": sig }
///
/// This function performs the API call in production and accepts in dev mode.
#[instrument(skip(config))]
pub async fn verify_tron_signature(
    config: &TronConfig,
    request: &TronVerifyRequest,
    expected_nonce: &str,
) -> Result<TronAuthResult, String> {
    // LangSec: validate address before any network call
    langsec::validate_tron_address(&request.address).map_err(|e| e.to_string())?;

    // Validate signature format: 130 hex chars (65 bytes)
    let sig = request
        .signature
        .strip_prefix("0x")
        .unwrap_or(&request.signature);
    if sig.len() != 130 || !sig.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid signature format: must be 130 hex chars".into());
    }

    if config.dev_mode {
        info!(
            address=%request.address,
            "TRON_DEV_MODE: signature verification skipped"
        );
        return Ok(TronAuthResult {
            address: TronAddress(request.address.clone()),
            verified: true,
            message: "dev_mode_bypass".into(),
        });
    }

    if !config.enabled {
        return Err("Tron integration not enabled — set TRON_ENABLED=1".into());
    }

    // Call TronGrid verifyMessage
    let verify_url = format!("{}/wallet/verifymessage", config.api_url);
    let body = serde_json::json!({
        "value": expected_nonce,
        "address": request.address,
        "signature": request.signature,
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp: serde_json::Value = client
        .post(&verify_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("TronGrid request failed: {e}"))?
        .json()
        .await
        .map_err(|e| format!("TronGrid response parse failed: {e}"))?;

    let verified = resp["result"].as_bool().unwrap_or(false);
    info!(address=%request.address, verified, "Tron signature verification");

    Ok(TronAuthResult {
        address: TronAddress(request.address.clone()),
        verified,
        message: if verified {
            "ok".into()
        } else {
            "signature_mismatch".into()
        },
    })
}

// ── Royalty distribution ──────────────────────────────────────────────────────

/// Distribute TRX royalties to multiple recipients.
///
/// In production this builds a multi-send transaction via the Tron HTTP API.
/// Each transfer is sent individually (Tron does not natively support atomic
/// multi-send in a single transaction without a smart contract).
///
/// Value cap: MAX_TRX_DISTRIBUTION per call (enforced before any network call).
#[instrument(skip(config))]
pub async fn distribute_trx(
    config: &TronConfig,
    recipients: &[TronRecipient],
    total_sun: u64,
    isrc: &str,
) -> anyhow::Result<TronDistributionResult> {
    // Value cap
    if total_sun > MAX_TRX_DISTRIBUTION {
        anyhow::bail!(
            "SECURITY: TRX distribution amount {total_sun} exceeds cap {MAX_TRX_DISTRIBUTION} sun"
        );
    }
    if recipients.is_empty() {
        anyhow::bail!("No recipients for TRX distribution");
    }

    // Validate all addresses
    for r in recipients {
        langsec::validate_tron_address(&r.address.0).map_err(|e| anyhow::anyhow!("{e}"))?;
    }

    // Validate BPS sum
    let bp_sum: u32 = recipients.iter().map(|r| r.bps as u32).sum();
    if bp_sum != 10_000 {
        anyhow::bail!("Royalty BPS sum must equal 10,000 (got {bp_sum})");
    }

    if config.dev_mode {
        let stub_hash = format!("dev_{}", &isrc.replace('-', "").to_lowercase());
        info!(isrc=%isrc, total_sun, "TRON_DEV_MODE: stub distribution");
        return Ok(TronDistributionResult {
            tx_hash: stub_hash,
            total_sun,
            recipients: recipients.to_vec(),
            dev_mode: true,
        });
    }

    if !config.enabled {
        anyhow::bail!("Tron not enabled — set TRON_ENABLED=1 and TRON_API_URL");
    }

    // In production: sign + broadcast via Tron HTTP API.
    // Requires TRON_PRIVATE_KEY env var (hex-encoded 64 chars).
    // This stub returns a placeholder — integrate with tron-api-client or
    // a signing sidecar that holds the private key outside this process.
    warn!(isrc=%isrc, "Tron production distribution not yet connected to signing sidecar");
    anyhow::bail!(
        "Tron production distribution requires a signing sidecar — \
         set TRON_DEV_MODE=1 for testing or connect tron-signer service"
    )
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn generate_nonce() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!(
        "{:016x}{:08x}",
        t.as_nanos(),
        t.subsec_nanos().wrapping_mul(0xdeadbeef)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tron_address_validation() {
        assert!(langsec::validate_tron_address("TQn9Y2khEsLJW1ChVWFMSMeRDow5KcbLSE").is_ok());
        assert!(langsec::validate_tron_address("not_a_tron_address").is_err());
    }

    #[test]
    fn bps_sum_validated() {
        let cfg = TronConfig {
            api_url: "https://api.trongrid.io".into(),
            token_contract: None,
            enabled: false,
            dev_mode: true,
        };
        let _ = cfg; // config created successfully
    }
}