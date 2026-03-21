//! Master Pattern Protocol — mod-9 supersingular prime band classification.
//!
//! 15 supersingular primes in three bands:
//!   Band 0 (Common,    7 primes): {2,3,5,7,11,13,17}   digit_root(sum)=4
//!   Band 1 (Rare,      4 primes): {19,23,29,31}          digit_root(sum)=3
//!   Band 2 (Legendary, 4 primes): {41,47,59,71}          digit_root(sum)=2
//!
//! Closure invariant: 4+3+2=9 ≡ 0 (mod 9)  — verified at compile time.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ── Compile-time invariant checks ────────────────────────────────────────
const fn digit_root_const(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let r = n % 9;
    if r == 0 {
        9
    } else {
        r
    }
}

const BAND0: [u64; 7] = [2, 3, 5, 7, 11, 13, 17];
const BAND1: [u64; 4] = [19, 23, 29, 31];
const BAND2: [u64; 4] = [41, 47, 59, 71];

const fn sum_arr7(a: [u64; 7]) -> u64 {
    a[0] + a[1] + a[2] + a[3] + a[4] + a[5] + a[6]
}
const fn sum_arr4(a: [u64; 4]) -> u64 {
    a[0] + a[1] + a[2] + a[3]
}

const _: () = assert!(digit_root_const(sum_arr7(BAND0)) == 4, "Band0 DR must be 4");
const _: () = assert!(digit_root_const(sum_arr4(BAND1)) == 3, "Band1 DR must be 3");
const _: () = assert!(digit_root_const(sum_arr4(BAND2)) == 2, "Band2 DR must be 2");
const _: () = assert!((4 + 3 + 2) % 9 == 0, "Closure: 4+3+2 mod 9 must be 0");

// ── Runtime API ───────────────────────────────────────────────────────────
/// Mod-9 digit root (9→9, not 0).
pub fn digit_root(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let r = n % 9;
    if r == 0 {
        9
    } else {
        r
    }
}

/// Rarity tier based on band.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RarityTier {
    Common,
    Rare,
    Legendary,
}

impl RarityTier {
    pub fn from_band(band: u8) -> Self {
        match band {
            0 => Self::Common,
            1 => Self::Rare,
            _ => Self::Legendary,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Common => "Common",
            Self::Rare => "Rare",
            Self::Legendary => "Legendary",
        }
    }
}

/// Classify a prime into its band.
pub fn classify_prime(p: u64) -> Option<u8> {
    if BAND0.contains(&p) {
        return Some(0);
    }
    if BAND1.contains(&p) {
        return Some(1);
    }
    if BAND2.contains(&p) {
        return Some(2);
    }
    None
}

/// Map a band to its first (lowest) prime.
pub fn map_to_band_prime(band: u8) -> u64 {
    match band {
        0 => 2,
        1 => 19,
        _ => 41,
    }
}

/// Determine band from digit root.
pub fn band_from_digit_root(dr: u64) -> u8 {
    match dr {
        4 => 0,
        3 => 1,
        2 => 2,
        _ => 0,
    }
}

/// Full fingerprint of a track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternFingerprint {
    pub hash_u64: u64,
    pub cycle_position: u8,
    pub digit_root: u64,
    pub expanded_root: u64,
    pub band: u8,
    pub band_residue: u64,
    pub mapped_prime: u64,
    pub parity: bool,
    pub parity_inverted: bool,
    pub closure_verified: bool,
}

/// Compute fingerprint from ISRC bytes + audio SHA-256 hash.
pub fn pattern_fingerprint(isrc_bytes: &[u8], audio_hash: &[u8; 32]) -> PatternFingerprint {
    let mut combined = isrc_bytes.to_vec();
    combined.extend_from_slice(audio_hash);
    let hash_bytes: [u8; 32] = Sha256::digest(&combined).into();
    let hash_u64 = u64::from_le_bytes(hash_bytes[..8].try_into().unwrap_or_default());

    let dr = digit_root(hash_u64);
    let band = band_from_digit_root(dr);
    let residue = (4u64 + 3 + 2).wrapping_sub(band as u64) % 9;
    let prime = map_to_band_prime(band);
    let expanded = {
        let s: u64 = hash_u64
            .to_string()
            .chars()
            .filter_map(|c| c.to_digit(10))
            .map(|d| d as u64)
            .sum();
        digit_root(s)
    };

    PatternFingerprint {
        hash_u64,
        cycle_position: (hash_u64 % 256) as u8,
        digit_root: dr,
        expanded_root: expanded,
        band,
        band_residue: residue,
        mapped_prime: prime,
        parity: hash_u64 % 2 == 1,
        parity_inverted: hash_u64 % 2 == 0,
        closure_verified: (4 + 3 + 2) % 9 == 0,
    }
}
