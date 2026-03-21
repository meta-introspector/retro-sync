//! Ledger hardware wallet signer via ethers-rs.
//!
//! Production: connects to physical Ledger device via HID, signs transactions
//! on the secure element. Private key never leaves the device.
//!
//! Dev (LEDGER_DEV_MODE=1): returns a deterministic stub signature so the
//! rest of the pipeline can be exercised without hardware.
//!
//! NOTE: actual transaction signing is now handled in bttc.rs via
//! `SignerMiddleware<Provider<Http>, Ledger>`. This module exposes the
//! lower-level `sign_bytes` helper for use by other callers (e.g. DDEX
//! manifest signing, ISO 9001 audit log sealing).

#[cfg(feature = "ledger")]
use tracing::info;
use tracing::{instrument, warn};

/// Signs arbitrary bytes with the Ledger's Ethereum personal_sign path.
/// For EIP-712 structured data, use the middleware in bttc.rs directly.
#[allow(dead_code)]
#[instrument(skip(payload))]
pub async fn sign_bytes(payload: &[u8]) -> anyhow::Result<Vec<u8>> {
    if std::env::var("LEDGER_DEV_MODE").unwrap_or_default() == "1" {
        warn!("LEDGER_DEV_MODE=1 — returning deterministic stub signature");
        // Deterministic stub: sha256(payload) ++ 65 zero bytes (r,s,v)
        use sha2::{Digest, Sha256};
        let mut sig = Sha256::digest(payload).to_vec();
        sig.resize(32 + 65, 0);
        return Ok(sig);
    }

    #[cfg(feature = "ledger")]
    {
        use ethers::signers::{HDPath, Ledger, Signer};

        let chain_id = std::env::var("BTTC_CHAIN_ID")
            .unwrap_or_else(|_| "199".into()) // BTTC mainnet
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("BTTC_CHAIN_ID must be a u64"))?;

        let ledger = Ledger::new(HDPath::LedgerLive(0), chain_id)
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Cannot open Ledger: {}. Device must be connected, unlocked, \
                 Ethereum app open.",
                    e
                )
            })?;

        let sig = ledger
            .sign_message(payload)
            .await
            .map_err(|e| anyhow::anyhow!("Ledger sign_message failed: {}", e))?;

        let mut out = Vec::with_capacity(65);
        let mut r_bytes = [0u8; 32];
        let mut s_bytes = [0u8; 32];
        sig.r.to_big_endian(&mut r_bytes);
        sig.s.to_big_endian(&mut s_bytes);
        out.extend_from_slice(&r_bytes);
        out.extend_from_slice(&s_bytes);
        out.push(sig.v as u8);

        info!(addr=%ledger.address(), "Ledger signature produced");
        Ok(out)
    }

    #[cfg(not(feature = "ledger"))]
    {
        anyhow::bail!(
            "Ledger feature not enabled. Set LEDGER_DEV_MODE=1 for development \
             or compile with --features ledger for production."
        )
    }
}

/// Returns the Ledger's Ethereum address at `m/44'/60'/0'/0/0`.
/// Used to pre-verify the correct device is connected before submitting.
#[allow(dead_code)]
pub async fn get_address() -> anyhow::Result<String> {
    if std::env::var("LEDGER_DEV_MODE").unwrap_or_default() == "1" {
        return Ok("0xDEV0000000000000000000000000000000000001".into());
    }

    #[cfg(feature = "ledger")]
    {
        use ethers::signers::{HDPath, Ledger, Signer};
        let chain_id = std::env::var("BTTC_CHAIN_ID")
            .unwrap_or_else(|_| "199".into())
            .parse::<u64>()?;
        let ledger = Ledger::new(HDPath::LedgerLive(0), chain_id)
            .await
            .map_err(|e| anyhow::anyhow!("Ledger not found: {}", e))?;
        Ok(format!("{:#x}", ledger.address()))
    }

    #[cfg(not(feature = "ledger"))]
    {
        anyhow::bail!("Ledger feature not compiled in")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dev_mode_stub_is_deterministic() {
        std::env::set_var("LEDGER_DEV_MODE", "1");
        let sig1 = sign_bytes(b"hello retrosync").await.unwrap();
        let sig2 = sign_bytes(b"hello retrosync").await.unwrap();
        assert_eq!(
            sig1, sig2,
            "dev stub must be deterministic for test reproducibility"
        );
        // Different payload → different stub
        let sig3 = sign_bytes(b"different payload").await.unwrap();
        assert_ne!(sig1, sig3);
        std::env::remove_var("LEDGER_DEV_MODE");
    }

    #[tokio::test]
    async fn dev_mode_address_returns_stub() {
        std::env::set_var("LEDGER_DEV_MODE", "1");
        let addr = get_address().await.unwrap();
        assert!(addr.starts_with("0x"), "address must be hex");
        std::env::remove_var("LEDGER_DEV_MODE");
    }
}
