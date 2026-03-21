//! LMDB-backed ZK proof cache (heed 0.20).
//!
//! Key: hex(band_byte ‖ SHA-256(n_artists ‖ total_btt ‖ splits_bps)) = 66 hex chars
//! Value: hex-encoded ZK proof bytes (stored as JSON string in LMDB)
//!
//! Eviction policy: none (proofs are deterministic — same inputs always produce
//! the same proof, so stale entries are never harmful, only wasteful).
use sha2::{Digest, Sha256};

pub struct ZkProofCache {
    db: crate::persist::LmdbStore,
}

impl ZkProofCache {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            db: crate::persist::LmdbStore::open(path, "zk_proofs")?,
        })
    }

    /// Build the 33-byte cache key (band byte ‖ SHA-256 of inputs).
    pub fn cache_key(band: u8, n_artists: u32, total_btt: u64, splits_bps: &[u16]) -> [u8; 33] {
        let mut h = Sha256::new();
        h.update(n_artists.to_le_bytes());
        h.update(total_btt.to_le_bytes());
        for bps in splits_bps {
            h.update(bps.to_le_bytes());
        }
        let hash: [u8; 32] = h.finalize().into();
        let mut key = [0u8; 33];
        key[0] = band;
        key[1..].copy_from_slice(&hash);
        key
    }

    fn key_str(band: u8, n_artists: u32, total_btt: u64, splits_bps: &[u16]) -> String {
        hex::encode(Self::cache_key(band, n_artists, total_btt, splits_bps))
    }

    /// Retrieve a cached proof. Returns `None` on miss.
    pub fn get(
        &self,
        band: u8,
        n_artists: u32,
        total_btt: u64,
        splits_bps: &[u16],
    ) -> Option<Vec<u8>> {
        let key = Self::key_str(band, n_artists, total_btt, splits_bps);
        let hex_str: String = self.db.get(&key).ok().flatten()?;
        hex::decode(hex_str).ok()
    }

    /// Store a proof. The proof bytes are hex-encoded to stay JSON-compatible.
    pub fn put(
        &self,
        band: u8,
        n_artists: u32,
        total_btt: u64,
        splits_bps: &[u16],
        proof: Vec<u8>,
    ) {
        let key = Self::key_str(band, n_artists, total_btt, splits_bps);
        let hex_proof = hex::encode(&proof);
        if let Err(e) = self.db.put(&key, &hex_proof) {
            tracing::error!(err=%e, "ZK proof cache write error");
        }
    }

    /// Prometheus-compatible metrics line.
    pub fn metrics_text(&self) -> String {
        let count = self.db.all_values::<String>().map(|v| v.len()).unwrap_or(0);
        format!("retrosync_zk_cache_entries {count}\n")
    }
}
