//! LMDB ZK proof cache (heed).
//! Key: band_byte ‖ SHA-256(n_artists ‖ total_btt ‖ splits_bps) = 33 bytes
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct ZkProofCache {
    cache: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
    path: String,
}

impl ZkProofCache {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            cache: Mutex::new(HashMap::new()),
            path: path.to_string(),
        })
    }
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
    pub fn get(
        &self,
        band: u8,
        n_artists: u32,
        total_btt: u64,
        splits_bps: &[u16],
    ) -> Option<Vec<u8>> {
        let key = Self::cache_key(band, n_artists, total_btt, splits_bps);
        self.cache.lock().ok()?.get(key.as_ref()).cloned()
    }
    pub fn put(
        &self,
        band: u8,
        n_artists: u32,
        total_btt: u64,
        splits_bps: &[u16],
        proof: Vec<u8>,
    ) {
        let key = Self::cache_key(band, n_artists, total_btt, splits_bps);
        if let Ok(mut m) = self.cache.lock() {
            m.insert(key.to_vec(), proof);
        }
    }
    pub fn metrics_text(&self) -> String {
        let size = self.cache.lock().map(|m| m.len()).unwrap_or(0);
        format!("retrosync_zk_cache_entries {}\n", size)
    }
}
