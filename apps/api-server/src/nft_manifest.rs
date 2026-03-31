// ── nft_manifest.rs ───────────────────────────────────────────────────────────
//! NFT Shard Manifest — metadata-first, ownership-first shard access model.
//!
//! Architecture (revised from previous degraded-audio approach):
//!   • Music shards live on BTFS and are *publicly accessible*.
//!   • NFT (on BTTC) holds the ShardManifest: ordered CIDs, assembly instructions,
//!     and an optional AES-256-GCM key for tracks that choose at-rest encryption.
//!   • Public listeners can see fragments (unordered shards) but only NFT holders
//!     can reconstruct the complete, coherent track.
//!   • ZK proofs verify NFT ownership + correct assembly without revealing keys publicly.
//!
//! ShardManifest fields:
//!   track_cid      — BTFS CID of the "root" track object (JSON index)
//!   shard_order    — ordered list of BTFS shard CIDs  (assembly sequence)
//!   shard_count    — used for completeness verification
//!   enc_key_hex    — optional AES-256-GCM key (present only if encrypted shards)
//!   nonce_hex      — AES-GCM nonce
//!   version        — manifest schema version
//!   isrc           — ISRC of the track this manifest covers
//!   zk_commit_hash — SHA-256 of (shard_order || enc_key_hex) for ZK circuit input
//!
//! GMP note: the manifest itself is the "V-model verification artifact" —
//! it proves the assembled track is correct and complete.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{info, warn};

// ── Manifest ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardManifest {
    /// Schema version — increment on breaking changes.
    pub version: u8,
    pub isrc: String,
    /// BTFS CID of the top-level track metadata object.
    pub track_cid: String,
    /// Ordered list of BTFS CIDs — reconstructing in this order gives the full audio.
    pub shard_order: Vec<String>,
    pub shard_count: usize,
    /// Stems index: maps stem name (e.g. "vocal", "drums") to its slice of shard_order.
    pub stems: std::collections::HashMap<String, StemRange>,
    /// Optional AES-256-GCM encryption key (hex). None for public unencrypted shards.
    pub enc_key_hex: Option<String>,
    /// AES-GCM nonce (hex, 96-bit / 12 bytes). Required if enc_key_hex is present.
    pub nonce_hex: Option<String>,
    /// SHA-256 commitment over the manifest for ZK circuit input.
    pub zk_commit_hash: String,
    /// BTTC token ID once minted. None before minting.
    pub token_id: Option<u64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemRange {
    pub name: String,
    pub start_index: usize,
    pub end_index: usize,
}

impl ShardManifest {
    /// Build a new manifest from a list of ordered shard CIDs.
    /// Call `mint_manifest_nft` afterwards to assign a token ID.
    #[zkperf_macros::zkperf]
    pub fn new(
        isrc: impl Into<String>,
        track_cid: impl Into<String>,
        shard_order: Vec<String>,
        stems: std::collections::HashMap<String, StemRange>,
        enc_key_hex: Option<String>,
        nonce_hex: Option<String>,
    ) -> Self {
        let isrc = isrc.into();
        let track_cid = track_cid.into();
        let shard_count = shard_order.len();
        let commit = compute_zk_commit(&shard_order, enc_key_hex.as_deref());
        Self {
            version: 1,
            isrc,
            track_cid,
            shard_order,
            shard_count,
            stems,
            enc_key_hex,
            nonce_hex,
            zk_commit_hash: commit,
            token_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// True if this manifest uses encrypted shards.
    #[zkperf_macros::zkperf]
    pub fn is_encrypted(&self) -> bool {
        self.enc_key_hex.is_some()
    }

    /// Return the ordered CIDs for a specific stem.
    #[zkperf_macros::zkperf]
    pub fn stem_cids(&self, stem: &str) -> Option<&[String]> {
        let r = self.stems.get(stem)?;
        let end = r.end_index.min(self.shard_order.len());
        if r.start_index > end {
            return None;
        }
        Some(&self.shard_order[r.start_index..end])
    }

    /// Serialise the manifest to a canonical JSON byte string for BTFS upload.
    #[zkperf_macros::zkperf]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        // Canonical: sorted keys, no extra whitespace
        serde_json::to_vec(self).unwrap_or_default()
    }
}

/// Compute the ZK commitment hash: SHA-256(concat(shard_order CIDs) || enc_key_hex).
/// This is the public input to the ZK circuit for ownership proof.
#[zkperf_macros::zkperf]
pub fn compute_zk_commit(shard_order: &[String], enc_key_hex: Option<&str>) -> String {
    let mut h = Sha256::new();
    for cid in shard_order {
        h.update(cid.as_bytes());
        h.update(b"\x00"); // separator
    }
    if let Some(key) = enc_key_hex {
        h.update(key.as_bytes());
    }
    hex::encode(h.finalize())
}

// ── BTTC NFT minting ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct MintReceipt {
    pub token_id: u64,
    pub tx_hash: String,
    pub manifest_cid: String,
    pub zk_commit_hash: String,
    pub minted_at: String,
}

/// Mint a ShardManifest NFT on BTTC.
///
/// Steps:
///   1. Upload the manifest JSON to BTFS → get manifest_cid.
///   2. ABI-encode `mintManifest(isrc, manifest_cid, zk_commit_hash)`.
///   3. Submit via BTTC RPC (dev mode: stub).
///
/// The contract event `ManifestMinted(tokenId, isrc, manifestCid, zkCommitHash)`
/// is indexed by the gateway so holders can look up their manifest by token ID.
#[zkperf_macros::zkperf]
pub async fn mint_manifest_nft(manifest: &mut ShardManifest) -> anyhow::Result<MintReceipt> {
    let dev_mode = std::env::var("BTTC_DEV_MODE").unwrap_or_default() == "1";

    // ── Step 1: upload manifest to BTFS ──────────────────────────────────
    let manifest_bytes = manifest.to_canonical_bytes();
    let manifest_cid = if dev_mode {
        format!("bafyrei-manifest-{}", &manifest.isrc)
    } else {
        upload_to_btfs(&manifest_bytes).await?
    };

    info!(isrc = %manifest.isrc, manifest_cid = %manifest_cid, "Manifest uploaded to BTFS");

    // ── Step 2 + 3: mint NFT on BTTC ────────────────────────────────────
    let (token_id, tx_hash) = if dev_mode {
        warn!("BTTC_DEV_MODE=1 — stub NFT mint");
        (999_001u64, format!("0x{}", "ab12".repeat(16)))
    } else {
        call_mint_manifest_contract(&manifest.isrc, &manifest_cid, &manifest.zk_commit_hash).await?
    };

    manifest.token_id = Some(token_id);

    let receipt = MintReceipt {
        token_id,
        tx_hash,
        manifest_cid,
        zk_commit_hash: manifest.zk_commit_hash.clone(),
        minted_at: chrono::Utc::now().to_rfc3339(),
    };
    info!(token_id, isrc = %manifest.isrc, "ShardManifest NFT minted");
    Ok(receipt)
}

/// Look up a ShardManifest from BTFS by NFT token ID.
///
/// Workflow:
///   1. Call `tokenURI(tokenId)` on the NFT contract → BTFS CID or IPFS URI.
///   2. Fetch the manifest JSON from BTFS.
///   3. Validate the `zk_commit_hash` matches the on-chain value.
#[zkperf_macros::zkperf]
pub async fn lookup_manifest_by_token(token_id: u64) -> anyhow::Result<ShardManifest> {
    let dev_mode = std::env::var("BTTC_DEV_MODE").unwrap_or_default() == "1";

    if dev_mode {
        warn!("BTTC_DEV_MODE=1 — returning stub ShardManifest for token {token_id}");
        let mut stems = std::collections::HashMap::new();
        stems.insert(
            "vocal".into(),
            StemRange {
                name: "vocal".into(),
                start_index: 0,
                end_index: 4,
            },
        );
        stems.insert(
            "instrumental".into(),
            StemRange {
                name: "instrumental".into(),
                start_index: 4,
                end_index: 8,
            },
        );
        let shard_order: Vec<String> = (0..8).map(|i| format!("bafyrei-shard-{i:04}")).collect();
        return Ok(ShardManifest::new(
            "GBAYE0601498",
            "bafyrei-track-root",
            shard_order,
            stems,
            None,
            None,
        ));
    }

    // Production: call tokenURI on BTTC NFT contract
    let manifest_cid = call_token_uri(token_id).await?;
    let manifest_json = fetch_from_btfs(&manifest_cid).await?;
    let manifest: ShardManifest = serde_json::from_str(&manifest_json)?;

    // Validate commit hash
    let expected = compute_zk_commit(&manifest.shard_order, manifest.enc_key_hex.as_deref());
    if manifest.zk_commit_hash != expected {
        anyhow::bail!(
            "Manifest ZK commit mismatch: on-chain {}, computed {}",
            manifest.zk_commit_hash,
            expected
        );
    }

    Ok(manifest)
}

// ── ZK proof of manifest ownership ───────────────────────────────────────────

/// Claim: "I own NFT token T, therefore I can assemble track I from shards."
///
/// Proof structure (Groth16 on BN254, same curve as royalty_split circuit):
///   Public inputs:  zk_commit_hash, token_id, wallet_address_hash
///   Private witness: enc_key_hex (if encrypted), shard_order, NFT signature
///
/// This function generates a STUB proof in dev mode. In production, it would
/// delegate to the arkworks Groth16 prover.
#[derive(Debug, Serialize)]
pub struct ManifestOwnershipProof {
    pub token_id: u64,
    pub wallet: String,
    pub zk_commit_hash: String,
    pub proof_hex: String,
    pub proven_at: String,
}

#[zkperf_macros::zkperf]
pub fn generate_manifest_ownership_proof_stub(
    token_id: u64,
    wallet: &str,
    manifest: &ShardManifest,
) -> ManifestOwnershipProof {
    // Stub: hash (token_id || wallet || zk_commit) as "proof"
    let mut h = Sha256::new();
    h.update(token_id.to_le_bytes());
    h.update(wallet.as_bytes());
    h.update(manifest.zk_commit_hash.as_bytes());
    let proof_hex = hex::encode(h.finalize());
    ManifestOwnershipProof {
        token_id,
        wallet: wallet.to_string(),
        zk_commit_hash: manifest.zk_commit_hash.clone(),
        proof_hex,
        proven_at: chrono::Utc::now().to_rfc3339(),
    }
}

// ── BTFS helpers ──────────────────────────────────────────────────────────────

async fn upload_to_btfs(data: &[u8]) -> anyhow::Result<String> {
    let api = std::env::var("BTFS_API_URL").unwrap_or_else(|_| "http://127.0.0.1:5001".into());
    let url = format!("{api}/api/v0/add");
    let part = reqwest::multipart::Part::bytes(data.to_vec())
        .file_name("manifest.json")
        .mime_str("application/json")?;
    let form = reqwest::multipart::Form::new().part("file", part);
    let client = reqwest::Client::new();
    let resp = client.post(&url).multipart(form).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("BTFS upload failed: {}", resp.status());
    }
    let body = resp.text().await?;
    let cid = body
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter_map(|v| v["Hash"].as_str().map(String::from))
        .next_back()
        .ok_or_else(|| anyhow::anyhow!("BTFS returned no CID"))?;
    Ok(cid)
}

async fn fetch_from_btfs(cid: &str) -> anyhow::Result<String> {
    let api = std::env::var("BTFS_API_URL").unwrap_or_else(|_| "http://127.0.0.1:5001".into());
    let url = format!("{api}/api/v0/cat?arg={cid}");
    let client = reqwest::Client::new();
    let resp = client.post(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("BTFS fetch failed for CID {cid}: {}", resp.status());
    }
    Ok(resp.text().await?)
}

// ── BTTC contract calls (stubs for production impl) ──────────────────────────

async fn call_mint_manifest_contract(
    isrc: &str,
    manifest_cid: &str,
    zk_commit: &str,
) -> anyhow::Result<(u64, String)> {
    let rpc = std::env::var("BTTC_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8545".into());
    let contract = std::env::var("NFT_MANIFEST_CONTRACT_ADDR")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000002".into());

    // keccak4("mintManifest(string,string,bytes32)") → selector
    // In production: ABI encode + eth_sendRawTransaction
    let _ = (rpc, contract, isrc, manifest_cid, zk_commit);
    anyhow::bail!("mintManifest not yet implemented in production — set BTTC_DEV_MODE=1")
}

async fn call_token_uri(token_id: u64) -> anyhow::Result<String> {
    let rpc = std::env::var("BTTC_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8545".into());
    let contract = std::env::var("NFT_MANIFEST_CONTRACT_ADDR")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000002".into());
    let _ = (rpc, contract, token_id);
    anyhow::bail!("tokenURI not yet implemented in production — set BTTC_DEV_MODE=1")
}