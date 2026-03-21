//! Core domain types — all newtypes for LangSec boundary enforcement.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Isrc(pub String);

impl std::fmt::Display for Isrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for BtfsCid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for EvmAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for Bowi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BtfsCid(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EvmAddress(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Bowi(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoyaltySplit {
    pub address: EvmAddress,
    pub bps: u16,
    /// Per-artist BTT amount in wei (18 decimals). Computed off-chain from
    /// `total_btt * bps / 10_000` before submission.
    #[serde(default)]
    pub amount_btt: u128,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    InvalidFormat(String),
    InvalidLength { expected: usize, got: usize },
    InvalidCheckDigit,
    InvalidBowi(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(s) => write!(f, "invalid format: {}", s),
            Self::InvalidLength { expected, got } => {
                write!(f, "expected {} chars, got {}", expected, got)
            }
            Self::InvalidCheckDigit => write!(f, "check digit invalid"),
            Self::InvalidBowi(s) => write!(f, "invalid BOWI identifier: {}", s),
        }
    }
}
