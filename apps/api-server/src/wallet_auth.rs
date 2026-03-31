//! Wallet challenge-response authentication.
//!
//! Flow:
//!   1. Client: GET /api/auth/challenge/{address}
//!      Server issues a random nonce with 5-minute TTL.
//!
//!   2. Client signs the nonce string with their wallet private key.
//!      - BTTC / EVM wallets: personal_sign (EIP-191 prefix)
//!      - TronLink on Tron: signMessageV2
//!
//!   3. Client: POST /api/auth/verify { challenge_id, address, signature }
//!      Server recovers the signer address from the ECDSA signature and
//!      checks it matches the claimed address.  On success, issues a JWT
//!      (`sub` = wallet address, `exp` = 24h) the client stores and sends
//!      as `Authorization: Bearer <token>` on all subsequent API calls.
//!
//! Security properties:
//!   - Nonce is cryptographically random (OS entropy via /dev/urandom).
//!   - Challenges expire after 5 minutes → replay window is bounded.
//!   - Used challenges are deleted immediately → single-use.
//!   - JWT signed with HMAC-SHA256 using JWT_SECRET env var.

use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{info, warn};

// ── Challenge store (in-memory, short-lived) ──────────────────────────────────

#[derive(Debug)]
struct PendingChallenge {
    address: String,
    nonce: String,
    issued_at: Instant,
}

pub struct ChallengeStore {
    pending: Mutex<HashMap<String, PendingChallenge>>,
}

impl Default for ChallengeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ChallengeStore {
    #[zkperf_macros::zkperf]
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }

    fn issue(&self, address: &str) -> (String, String) {
        let challenge_id = random_hex(16);
        let nonce = format!(
            "Sign in to Retrosync Media Group.\nNonce: {}\nIssued: {}",
            random_hex(32),
            chrono::Utc::now().to_rfc3339()
        );
        if let Ok(mut map) = self.pending.lock() {
            // Purge expired challenges first
            map.retain(|_, v| v.issued_at.elapsed() < Duration::from_secs(300));
            map.insert(
                challenge_id.clone(),
                PendingChallenge {
                    address: address.to_ascii_lowercase(),
                    nonce: nonce.clone(),
                    issued_at: Instant::now(),
                },
            );
        }
        (challenge_id, nonce)
    }

    fn consume(&self, challenge_id: &str) -> Option<PendingChallenge> {
        if let Ok(mut map) = self.pending.lock() {
            let entry = map.remove(challenge_id)?;
            if entry.issued_at.elapsed() > Duration::from_secs(300) {
                warn!(challenge_id=%challenge_id, "Challenge expired — rejecting");
                return None;
            }
            Some(entry)
        } else {
            None
        }
    }
}

/// Public alias for use by other modules (e.g., moderation.rs ID generation).
#[zkperf_macros::zkperf]
pub fn random_hex_pub(n: usize) -> String {
    random_hex(n)
}

/// Cryptographically random hex string of `n` bytes (2n hex chars).
///
/// SECURITY: Uses OS entropy (/dev/urandom / getrandom syscall) exclusively.
/// SECURITY FIX: Removed DefaultHasher fallback — DefaultHasher is NOT
/// cryptographically secure.  If OS entropy is unavailable, we derive bytes
/// from a SHA-256 chain seeded by time + PID + a counter, which is weak but
/// still orders-of-magnitude stronger than DefaultHasher.  A CRITICAL log is
/// emitted so the operator knows to investigate the entropy source.
fn random_hex(n: usize) -> String {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut bytes = vec![0u8; n];

    // Primary: OS entropy — always preferred
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        if f.read_exact(&mut bytes).is_ok() {
            return hex::encode(bytes);
        }
    }

    // Last resort: SHA-256 derivation from time + PID + atomic counter.
    // This is NOT cryptographically secure on its own but is far superior
    // to DefaultHasher and buys time until /dev/urandom is restored.
    tracing::error!(
        "SECURITY CRITICAL: /dev/urandom unavailable — \
         falling back to SHA-256 time/PID derivation. \
         Investigate entropy source immediately."
    );
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let ctr = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let seed = format!(
        "retrosync-entropy:{:?}:{}:{}",
        std::time::SystemTime::now(),
        std::process::id(),
        ctr,
    );
    let mut out = Vec::with_capacity(n);
    let mut round_input = seed.into_bytes();
    while out.len() < n {
        let digest = Sha256::digest(&round_input);
        out.extend_from_slice(&digest);
        round_input = digest.to_vec();
    }
    out.truncate(n);
    hex::encode(out)
}

// ── AppState extension ────────────────────────────────────────────────────────

// The ChallengeStore is embedded in AppState via main.rs

// ── HTTP handlers ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub nonce: String,
    pub expires_in_secs: u64,
    pub instructions: &'static str,
}

#[zkperf_macros::zkperf]
pub async fn issue_challenge(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<ChallengeResponse>, axum::http::StatusCode> {
    // LangSec: wallet addresses have strict length and character constraints.
    // EVM 0x + 40 hex = 42 chars; Tron Base58 = 34 chars.
    // We allow up to 128 chars to accommodate future chains; zero-length is rejected.
    if address.is_empty() || address.len() > 128 {
        warn!(
            len = address.len(),
            "issue_challenge: address length out of range"
        );
        return Err(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    }
    // LangSec: only alphanumeric + 0x-prefix chars; no control chars, spaces, or
    // path-injection sequences are permitted in an address field.
    if !address
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == 'x' || c == 'X')
    {
        warn!(%address, "issue_challenge: address contains invalid characters");
        return Err(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    let address = address.to_ascii_lowercase();
    let (challenge_id, nonce) = state.challenge_store.issue(&address);
    info!(address=%address, challenge_id=%challenge_id, "Wallet challenge issued");
    Ok(Json(ChallengeResponse {
        challenge_id,
        nonce,
        expires_in_secs: 300,
        instructions: "Sign the `nonce` string with your wallet. \
                        For EVM/BTTC: use personal_sign. \
                        For TronLink/Tron: use signMessageV2.",
    }))
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub challenge_id: String,
    pub address: String,
    pub signature: String,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub token: String,
    pub address: String,
    pub expires_in_secs: u64,
}

#[zkperf_macros::zkperf]
pub async fn verify_challenge(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, StatusCode> {
    // LangSec: challenge_id is a hex string produced by random_hex(16) → 32 chars.
    // Cap at 128 to prevent oversized strings from reaching the store lookup.
    if req.challenge_id.is_empty() || req.challenge_id.len() > 128 {
        warn!(
            len = req.challenge_id.len(),
            "verify_challenge: challenge_id length out of range"
        );
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    // LangSec: challenge_id must be hex-only (0-9, a-f); reject control chars.
    if !req
        .challenge_id
        .chars()
        .all(|c| c.is_ascii_hexdigit() || c == '-')
    {
        warn!("verify_challenge: challenge_id contains non-hex characters");
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    // LangSec: signature length sanity — EVM compact sig is 130 hex chars (65 bytes);
    // Tron sigs are similar.  Reject anything absurdly long (>512 chars).
    if req.signature.len() > 512 {
        warn!(
            len = req.signature.len(),
            "verify_challenge: signature field too long"
        );
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let address = req.address.to_ascii_lowercase();

    // Retrieve and consume the challenge (single-use + TTL enforced here)
    let challenge = state
        .challenge_store
        .consume(&req.challenge_id)
        .ok_or_else(|| {
            warn!(challenge_id=%req.challenge_id, "Unknown or expired challenge");
            StatusCode::UNPROCESSABLE_ENTITY
        })?;

    // Verify the claimed address matches the challenge's address
    if challenge.address != address {
        warn!(
            claimed=%address,
            challenge_addr=%challenge.address,
            "Address mismatch in challenge verify"
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // Verify the signature — fail closed by default.
    // The only bypass is WALLET_AUTH_DEV_BYPASS=1, which must be set explicitly
    // and is intended solely for local development against a test wallet.
    let verified =
        verify_evm_signature(&challenge.nonce, &req.signature, &address).unwrap_or(false);

    if !verified {
        let bypass = std::env::var("WALLET_AUTH_DEV_BYPASS").unwrap_or_default() == "1";
        if !bypass {
            warn!(address=%address, "Wallet signature verification failed — rejecting");
            return Err(StatusCode::FORBIDDEN);
        }
        warn!(
            address=%address,
            "Wallet signature not verified — WALLET_AUTH_DEV_BYPASS=1 (dev only, never in prod)"
        );
    }

    // Issue JWT
    let token = issue_jwt(&address).map_err(|e| {
        warn!("JWT issue failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!(address=%address, "Wallet authentication successful — JWT issued");
    Ok(Json(VerifyResponse {
        token,
        address,
        expires_in_secs: 86400,
    }))
}

// ── EVM Signature Verification ────────────────────────────────────────────────

/// Verify an EIP-191 personal_sign signature.
/// The message is prefixed as: `\x19Ethereum Signed Message:\n{len}{msg}`
/// Returns true if the recovered address matches the claimed address.
fn verify_evm_signature(
    message: &str,
    signature_hex: &str,
    claimed_address: &str,
) -> anyhow::Result<bool> {
    // EIP-191 prefix
    let prefixed = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);

    // SHA3-256 (keccak256) of the prefixed message
    let msg_hash = keccak256(prefixed.as_bytes());

    // Decode signature: 65 bytes = r (32) + s (32) + v (1)
    let sig_bytes = hex::decode(signature_hex.trim_start_matches("0x"))
        .map_err(|e| anyhow::anyhow!("Signature hex decode failed: {e}"))?;

    if sig_bytes.len() != 65 {
        anyhow::bail!("Signature must be 65 bytes, got {}", sig_bytes.len());
    }

    let r = &sig_bytes[0..32];
    let s = &sig_bytes[32..64];
    let v = sig_bytes[64];

    // Normalise v: TronLink uses 0/1, Ethereum uses 27/28
    let recovery_id = match v {
        0 | 27 => 0u8,
        1 | 28 => 1u8,
        _ => anyhow::bail!("Invalid recovery id v={v}"),
    };

    // Recover the public key and derive the address
    let recovered = recover_evm_address(&msg_hash, r, s, recovery_id)?;

    Ok(recovered.eq_ignore_ascii_case(claimed_address.trim_start_matches("0x")))
}

/// Keccak-256 hash (Ethereum's hash function), delegated to ethers::utils.
/// NOTE: Ethereum Keccak-256 differs from SHA3-256. Use this only.
fn keccak256(data: &[u8]) -> [u8; 32] {
    ethers_core::utils::keccak256(data)
}

/// ECDSA public key recovery on secp256k1.
/// Uses ethers-signers since ethers is already a dependency.
fn recover_evm_address(
    msg_hash: &[u8; 32],
    r: &[u8],
    s: &[u8],
    recovery_id: u8,
) -> anyhow::Result<String> {
    use ethers_core::types::{Signature, H256, U256};

    let mut r_arr = [0u8; 32];
    let mut s_arr = [0u8; 32];
    r_arr.copy_from_slice(r);
    s_arr.copy_from_slice(s);

    let sig = Signature {
        r: U256::from_big_endian(&r_arr),
        s: U256::from_big_endian(&s_arr),
        v: recovery_id as u64,
    };

    let hash = H256::from_slice(msg_hash);
    let recovered = sig.recover(hash)?;
    Ok(format!("{recovered:#x}"))
}

// ── JWT Issuance ──────────────────────────────────────────────────────────────

/// Issue a 24-hour JWT with `sub` = wallet address.
/// The token is HMAC-SHA256 signed using JWT_SECRET env var.
/// JWT_SECRET must be set — there is no insecure fallback.
#[zkperf_macros::zkperf]
pub fn issue_jwt(wallet_address: &str) -> anyhow::Result<String> {
    let secret = std::env::var("JWT_SECRET").map_err(|_| {
        anyhow::anyhow!("JWT_SECRET is not configured — set it before starting the server")
    })?;

    let now = chrono::Utc::now().timestamp();
    let exp = now + 86400; // 24h

    // Build JWT header + payload
    let header = base64_encode_url(b"{\"alg\":\"HS256\",\"typ\":\"JWT\"}");
    let payload_json = serde_json::json!({
        "sub": wallet_address,
        "iat": now,
        "exp": exp,
        "iss": "retrosync-api",
    });
    let payload = base64_encode_url(payload_json.to_string().as_bytes());

    let signing_input = format!("{header}.{payload}");
    let sig = hmac_sha256(secret.as_bytes(), signing_input.as_bytes());
    let sig_b64 = base64_encode_url(&sig);

    Ok(format!("{header}.{payload}.{sig_b64}"))
}

fn base64_encode_url(bytes: &[u8]) -> String {
    let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };
        out.push(chars[(b0 >> 2) as usize] as char);
        out.push(chars[((b0 & 3) << 4 | b1 >> 4) as usize] as char);
        if chunk.len() > 1 {
            out.push(chars[((b1 & 0xf) << 2 | b2 >> 6) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(chars[(b2 & 0x3f) as usize] as char);
        }
    }
    out.replace('+', "-").replace('/', "_")
}

fn hmac_sha256(key: &[u8], msg: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    const BLOCK: usize = 64;
    let mut k = if key.len() > BLOCK {
        Sha256::digest(key).to_vec()
    } else {
        key.to_vec()
    };
    k.resize(BLOCK, 0);
    let ipad: Vec<u8> = k.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = k.iter().map(|b| b ^ 0x5c).collect();
    let inner = Sha256::digest([ipad.as_slice(), msg].concat());
    Sha256::digest([opad.as_slice(), inner.as_slice()].concat()).to_vec()
}