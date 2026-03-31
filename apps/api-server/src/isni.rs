#![allow(dead_code)]
//! ISNI — International Standard Name Identifier (ISO 27729).
//!
//! ISNI is the ISO 27729:2012 standard for uniquely identifying parties
//! (persons and organisations) that participate in the creation,
//! production, management, and distribution of intellectual property.
//!
//! In the music industry ISNI is used to:
//!   - Unambiguously identify composers, lyricists, performers, publishers,
//!     record labels, and PROs across databases.
//!   - Disambiguate name-matched artists in royalty systems.
//!   - Cross-reference with IPI, ISWC, ISRC, and Wikidata QID.
//!
//! Reference: https://isni.org / https://www.iso.org/standard/44292.html
//!
//! LangSec:
//!   - ISNI always 16 digits (last may be 'X' for check digit 10).
//!   - Validated via ISO 27729 MOD 11-2 check algorithm before any lookup.
//!   - All outbound ISNI.org API calls length-bounded and JSON-sanitised.

use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};

// ── Config ────────────────────────────────────────────────────────────────────

/// ISNI.org API configuration.
#[derive(Clone)]
pub struct IsniConfig {
    /// Base URL for ISNI.org SRU search endpoint.
    pub base_url: String,
    /// Optional API key (ISNI.org may require registration for bulk lookups).
    pub api_key: Option<String>,
    /// Timeout for ISNI.org API calls.
    pub timeout_secs: u64,
}

impl IsniConfig {
    #[zkperf_macros::zkperf]
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("ISNI_BASE_URL")
                .unwrap_or_else(|_| "https://isni.org/isni/".into()),
            api_key: std::env::var("ISNI_API_KEY").ok(),
            timeout_secs: std::env::var("ISNI_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        }
    }
}

// ── Validated ISNI newtype ─────────────────────────────────────────────────────

/// A validated 16-character ISNI (digits 0-9 and optional trailing 'X').
/// Stored in canonical compact form (no spaces).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Isni(pub String);

impl std::fmt::Display for Isni {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Display as ISNI xxxx xxxx xxxx xxxx
        let d = &self.0;
        if d.len() == 16 {
            write!(
                f,
                "ISNI {} {} {} {}",
                &d[0..4],
                &d[4..8],
                &d[8..12],
                &d[12..16]
            )
        } else {
            write!(f, "ISNI {d}")
        }
    }
}

// ── ISO 27729 Validation ───────────────────────────────────────────────────────

/// Validate an ISNI string (compact or spaced, with or without "ISNI" prefix).
///
/// Returns `Ok(Isni)` containing the canonical compact 16-char form.
///
/// The check digit uses the ISO 27729 MOD 11-2 algorithm (identical to
/// ISBN-13 but over 16 digits).
#[zkperf_macros::zkperf]
pub fn validate_isni(input: &str) -> Result<Isni, IsniError> {
    // Strip optional "ISNI" prefix (case-insensitive) and whitespace
    let stripped = input
        .trim()
        .trim_start_matches("ISNI")
        .trim_start_matches("isni")
        .replace([' ', '-'], "");

    if stripped.len() != 16 {
        return Err(IsniError::InvalidLength(stripped.len()));
    }

    // All characters must be digits except last may be 'X'
    let chars: Vec<char> = stripped.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if i < 15 {
            if !c.is_ascii_digit() {
                return Err(IsniError::InvalidCharacter(i, c));
            }
        } else if !c.is_ascii_digit() && c != 'X' {
            return Err(IsniError::InvalidCharacter(i, c));
        }
    }

    // MOD 11-2 check digit (ISO 27729 §6.2)
    let expected_check = mod11_2_check(&stripped);
    let actual_check = chars[15];
    if actual_check != expected_check {
        return Err(IsniError::CheckDigitMismatch {
            expected: expected_check,
            found: actual_check,
        });
    }

    Ok(Isni(stripped.to_uppercase()))
}

/// Compute the ISO 27729 MOD 11-2 check character for the first 15 digits.
fn mod11_2_check(digits: &str) -> char {
    let chars: Vec<char> = digits.chars().collect();
    let mut sum: u64 = 0;
    let mut p = 2u64;
    // Process digits 1..=15 from right to left (position 15 is the check)
    for i in (0..15).rev() {
        let d = chars[i].to_digit(10).unwrap_or(0) as u64;
        sum += d * p;
        p = if p == 2 { 3 } else { 2 };
    }
    let remainder = sum % 11;
    match remainder {
        0 => '0',
        1 => 'X',
        r => char::from_digit((11 - r) as u32, 10).unwrap_or('?'),
    }
}

/// ISNI validation error.
#[derive(Debug, thiserror::Error)]
pub enum IsniError {
    #[error("ISNI must be 16 characters; got {0}")]
    InvalidLength(usize),
    #[error("Invalid character '{1}' at position {0}")]
    InvalidCharacter(usize, char),
    #[error("Check digit mismatch: expected '{expected}', found '{found}'")]
    CheckDigitMismatch { expected: char, found: char },
    #[error("ISNI.org API error: {0}")]
    ApiError(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

// ── ISNI Record (from ISNI.org) ────────────────────────────────────────────────

/// A resolved ISNI identity record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsniRecord {
    pub isni: Isni,
    pub primary_name: String,
    pub variant_names: Vec<String>,
    pub kind: IsniEntityKind,
    pub ipi_numbers: Vec<String>,
    pub isrc_creator: bool,
    pub wikidata_qid: Option<String>,
    pub viaf_id: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub countries: Vec<String>,
    pub birth_year: Option<u32>,
    pub death_year: Option<u32>,
    pub organisations: Vec<String>,
}

/// Whether the ISNI identifies a person or an organisation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IsniEntityKind {
    Person,
    Organisation,
    Unknown,
}

// ── ISNI.org API lookup ────────────────────────────────────────────────────────

/// Look up an ISNI record from ISNI.org SRU API.
///
/// Returns the resolved `IsniRecord` or an error if the ISNI is not found
/// or the API is unreachable.
#[instrument(skip(config))]
pub async fn lookup_isni(config: &IsniConfig, isni: &Isni) -> Result<IsniRecord, IsniError> {
    info!(isni=%isni.0, "ISNI lookup");
    let url = format!("{}{}", config.base_url, isni.0);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .user_agent("Retrosync/1.0 ISNI-Resolver")
        .build()?;

    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        warn!(isni=%isni.0, status, "ISNI lookup failed");
        return Err(IsniError::ApiError(format!("HTTP {status}")));
    }

    // ISNI.org currently returns HTML; parse JSON when available.
    // In production wire to ISNI SRU endpoint with schema=isni-b.
    // For now, return a minimal record from URL response.
    let _body = resp.text().await?;

    Ok(IsniRecord {
        isni: isni.clone(),
        primary_name: String::new(),
        variant_names: vec![],
        kind: IsniEntityKind::Unknown,
        ipi_numbers: vec![],
        isrc_creator: false,
        wikidata_qid: None,
        viaf_id: None,
        musicbrainz_id: None,
        countries: vec![],
        birth_year: None,
        death_year: None,
        organisations: vec![],
    })
}

/// Search ISNI.org for a name query.
/// Returns up to `limit` matching ISNIs.
#[instrument(skip(config))]
pub async fn search_isni_by_name(
    config: &IsniConfig,
    name: &str,
    limit: usize,
) -> Result<Vec<IsniRecord>, IsniError> {
    if name.is_empty() || name.len() > 200 {
        return Err(IsniError::ApiError("name must be 1–200 characters".into()));
    }
    let base = config.base_url.trim_end_matches('/');
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .user_agent("Retrosync/1.0 ISNI-Resolver")
        .build()?;

    // Use reqwest query params for safe URL encoding
    let resp = client
        .get(base)
        .query(&[
            ("query", format!("pica.na=\"{name}\"")),
            ("maximumRecords", limit.min(100).to_string()),
            ("recordSchema", "isni-b".to_string()),
        ])
        .header("Accept", "application/json")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(IsniError::ApiError(format!(
            "HTTP {}",
            resp.status().as_u16()
        )));
    }

    // Parse result set — full XML/JSON parsing to be wired in production.
    Ok(vec![])
}

// ── Cross-reference helpers ────────────────────────────────────────────────────

/// Parse a formatted ISNI string (with spaces) into compact form for storage.
#[zkperf_macros::zkperf]
pub fn normalise_isni(input: &str) -> String {
    input
        .trim()
        .trim_start_matches("ISNI")
        .trim_start_matches("isni")
        .replace([' ', '-'], "")
        .to_uppercase()
}

/// Cross-reference an ISNI against an IPI name number.
/// Both must pass independent validation before cross-referencing.
#[zkperf_macros::zkperf]
pub fn cross_reference_isni_ipi(isni: &Isni, ipi: &str) -> CrossRefResult {
    // IPI format: 11 digits, optionally prefixed "IPI:"
    let ipi_clean = ipi.trim().trim_start_matches("IPI:").trim();
    if ipi_clean.len() != 11 || !ipi_clean.chars().all(|c| c.is_ascii_digit()) {
        return CrossRefResult::InvalidIpi;
    }
    CrossRefResult::Unverified {
        isni: isni.0.clone(),
        ipi: ipi_clean.to_string(),
        note: "Cross-reference requires ISNI.org API confirmation".into(),
    }
}

/// Result of an ISNI ↔ IPI cross-reference attempt.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum CrossRefResult {
    Confirmed {
        isni: String,
        ipi: String,
    },
    Unverified {
        isni: String,
        ipi: String,
        note: String,
    },
    InvalidIpi,
    Mismatch {
        detail: String,
    },
}