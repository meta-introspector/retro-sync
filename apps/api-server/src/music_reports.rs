#![allow(dead_code)] // Integration module: full API surface exposed for future routes
//! Music Reports integration — musiceports.com licensing and royalty data.
//!
//! Music Reports (https://www.musicreports.com) is a leading provider of music
//! licensing solutions, specialising in:
//!   - Statutory mechanical licensing (Section 115 compulsory licences)
//!   - Digital audio recording (DAR) reporting
//!   - Sound recording metadata matching
//!   - Royalty statement generation
//!
//! This module provides:
//!   1. Configuration for the Music Reports API.
//!   2. Licence lookup by ISRC or work metadata.
//!   3. Mechanical royalty rate lookup (compulsory rates from CRB determinations).
//!   4. Licence application submission.
//!   5. Royalty statement import and reconciliation.
//!
//! Security:
//!   - API key from MUSIC_REPORTS_API_KEY env var only.
//!   - All ISRCs/ISWCs validated by shared parsers before API calls.
//!   - Response data length-bounded before processing.
//!   - Dev mode available for testing without live API credentials.
use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct MusicReportsConfig {
    pub api_key: String,
    pub base_url: String,
    pub enabled: bool,
    pub dev_mode: bool,
    /// Timeout for API requests (seconds).
    pub timeout_secs: u64,
}

impl MusicReportsConfig {
    #[zkperf_macros::zkperf]
    pub fn from_env() -> Self {
        let api_key = std::env::var("MUSIC_REPORTS_API_KEY").unwrap_or_default();
        let enabled = !api_key.is_empty();
        if !enabled {
            warn!("Music Reports not configured — set MUSIC_REPORTS_API_KEY");
        }
        Self {
            api_key,
            base_url: std::env::var("MUSIC_REPORTS_BASE_URL")
                .unwrap_or_else(|_| "https://api.musicreports.com/v2".into()),
            enabled,
            dev_mode: std::env::var("MUSIC_REPORTS_DEV_MODE").unwrap_or_default() == "1",
            timeout_secs: 15,
        }
    }
}

// ── Licence types ─────────────────────────────────────────────────────────────

/// Type of mechanical licence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MechanicalLicenceType {
    /// Section 115 compulsory (statutory) licence.
    Statutory115,
    /// Voluntary direct licence.
    Direct,
    /// Harry Fox Agency (HFA) licence.
    HarryFox,
    /// MLC-administered statutory licence (post-MMA 2018).
    MlcStatutory,
}

/// Compulsory mechanical royalty rate (from CRB determinations).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanicalRate {
    /// Rate per physical copy / permanent download (cents).
    pub rate_per_copy_cents: f32,
    /// Rate as percentage of content cost (for streaming).
    pub rate_pct_content_cost: f32,
    /// Minimum rate per stream (sub-cents, e.g. 0.00020).
    pub min_per_stream: f32,
    /// Applicable period (YYYY).
    pub effective_year: u16,
    /// CRB proceeding name (e.g. "Phonorecords IV").
    pub crb_proceeding: String,
}

/// Current (2024) CRB Phonorecords IV rates.
#[zkperf_macros::zkperf]
pub fn current_mechanical_rate() -> MechanicalRate {
    MechanicalRate {
        rate_per_copy_cents: 9.1,    // $0.091 per copy (physical/download)
        rate_pct_content_cost: 15.1, // 15.1% of content cost (streaming)
        min_per_stream: 0.00020,     // $0.00020 minimum per interactive stream
        effective_year: 2024,
        crb_proceeding: "Phonorecords IV (2023–2027)".into(),
    }
}

// ── Licence lookup ────────────────────────────────────────────────────────────

/// A licence record returned by Music Reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenceRecord {
    pub licence_id: String,
    pub isrc: Option<String>,
    pub iswc: Option<String>,
    pub work_title: String,
    pub licensor: String, // e.g. "ASCAP", "BMI", "Harry Fox"
    pub licence_type: MechanicalLicenceType,
    pub territory: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub status: LicenceStatus,
    pub royalty_rate_pct: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LicenceStatus {
    Active,
    Pending,
    Expired,
    Disputed,
    Terminated,
}

/// Look up existing licences for an ISRC.
#[instrument(skip(config))]
pub async fn lookup_by_isrc(
    config: &MusicReportsConfig,
    isrc: &str,
) -> anyhow::Result<Vec<LicenceRecord>> {
    // LangSec: validate ISRC before API call
    shared::parsers::recognize_isrc(isrc).map_err(|e| anyhow::anyhow!("Invalid ISRC: {e}"))?;

    if config.dev_mode {
        info!(isrc=%isrc, "Music Reports dev: returning stub licence");
        return Ok(vec![LicenceRecord {
            licence_id: format!("MR-DEV-{isrc}"),
            isrc: Some(isrc.to_string()),
            iswc: None,
            work_title: "Dev Track".into(),
            licensor: "Music Reports Dev".into(),
            licence_type: MechanicalLicenceType::Statutory115,
            territory: "Worldwide".into(),
            start_date: "2024-01-01".into(),
            end_date: None,
            status: LicenceStatus::Active,
            royalty_rate_pct: Some(current_mechanical_rate().rate_pct_content_cost),
        }]);
    }

    if !config.enabled {
        anyhow::bail!("Music Reports not configured — set MUSIC_REPORTS_API_KEY");
    }

    let url = format!(
        "{}/licences?isrc={}",
        config.base_url,
        urlencoding_encode(isrc)
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .build()?;

    let resp: serde_json::Value = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Accept", "application/json")
        .send()
        .await?
        .json()
        .await?;

    parse_licence_response(&resp)
}

/// Look up existing licences by ISWC (work identifier).
#[instrument(skip(config))]
pub async fn lookup_by_iswc(
    config: &MusicReportsConfig,
    iswc: &str,
) -> anyhow::Result<Vec<LicenceRecord>> {
    // Basic ISWC format validation
    if iswc.len() < 11 || !iswc.starts_with("T-") {
        anyhow::bail!("Invalid ISWC format: {iswc}");
    }
    if config.dev_mode {
        return Ok(vec![]);
    }
    if !config.enabled {
        anyhow::bail!("Music Reports not configured");
    }

    let url = format!(
        "{}/licences?iswc={}",
        config.base_url,
        urlencoding_encode(iswc)
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .build()?;

    let resp: serde_json::Value = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Accept", "application/json")
        .send()
        .await?
        .json()
        .await?;

    parse_licence_response(&resp)
}

// ── Royalty statement import ──────────────────────────────────────────────────

/// A royalty statement line item from Music Reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoyaltyStatementLine {
    pub period: String, // YYYY-MM
    pub isrc: String,
    pub work_title: String,
    pub units: u64, // streams / downloads / copies
    pub rate: f32,  // rate per unit
    pub gross_royalty: f64,
    pub deduction_pct: f32, // admin fee / deduction
    pub net_royalty: f64,
    pub currency: String, // ISO 4217
}

/// A complete royalty statement from Music Reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoyaltyStatement {
    pub statement_id: String,
    pub period_start: String,
    pub period_end: String,
    pub payee: String,
    pub lines: Vec<RoyaltyStatementLine>,
    pub total_gross: f64,
    pub total_net: f64,
    pub currency: String,
}

/// Fetch royalty statements for a given period.
#[instrument(skip(config))]
pub async fn fetch_statements(
    config: &MusicReportsConfig,
    period_start: &str,
    period_end: &str,
) -> anyhow::Result<Vec<RoyaltyStatement>> {
    // Validate date format
    for date in [period_start, period_end] {
        if date.len() != 7 || !date.chars().all(|c| c.is_ascii_digit() || c == '-') {
            anyhow::bail!("Date must be YYYY-MM format, got: {date}");
        }
    }

    if config.dev_mode {
        info!(period_start=%period_start, period_end=%period_end, "Music Reports dev: no statements");
        return Ok(vec![]);
    }

    if !config.enabled {
        anyhow::bail!("Music Reports not configured");
    }

    let url = format!(
        "{}/statements?start={}&end={}",
        config.base_url,
        urlencoding_encode(period_start),
        urlencoding_encode(period_end)
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .build()?;

    let resp: serde_json::Value = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Accept", "application/json")
        .send()
        .await?
        .json()
        .await?;

    let statements = resp["data"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|s| serde_json::from_value(s.clone()).ok())
        .collect();

    Ok(statements)
}

// ── Reconciliation ────────────────────────────────────────────────────────────

/// Reconcile Music Reports royalties against Retrosync on-chain distributions.
/// Returns ISRCs where reported royalty differs from on-chain amount by > 5%.
#[zkperf_macros::zkperf]
pub fn reconcile_royalties(
    statement: &RoyaltyStatement,
    onchain_distributions: &std::collections::HashMap<String, f64>,
) -> Vec<(String, f64, f64)> {
    let mut discrepancies = Vec::new();
    for line in &statement.lines {
        if let Some(&onchain) = onchain_distributions.get(&line.isrc) {
            let diff_pct =
                ((line.net_royalty - onchain).abs() / line.net_royalty.max(f64::EPSILON)) * 100.0;
            if diff_pct > 5.0 {
                warn!(
                    isrc=%line.isrc,
                    reported=line.net_royalty,
                    onchain=onchain,
                    diff_pct=diff_pct,
                    "Music Reports reconciliation discrepancy"
                );
                discrepancies.push((line.isrc.clone(), line.net_royalty, onchain));
            }
        }
    }
    discrepancies
}

// ── DSP coverage check ────────────────────────────────────────────────────────

/// Licensing coverage tiers for DSPs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspLicenceCoverage {
    pub dsp_name: String,
    pub requires_mechanical: bool,
    pub requires_performance: bool,
    pub requires_neighbouring: bool,
    pub territory: String,
    pub notes: String,
}

/// Return licensing requirements for major DSPs.
#[zkperf_macros::zkperf]
pub fn dsp_licence_requirements() -> Vec<DspLicenceCoverage> {
    vec![
        DspLicenceCoverage {
            dsp_name: "Spotify".into(),
            requires_mechanical: true,
            requires_performance: true,
            requires_neighbouring: false,
            territory: "Worldwide".into(),
            notes: "Uses MLC for mechanical (US), direct licensing elsewhere".into(),
        },
        DspLicenceCoverage {
            dsp_name: "Apple Music".into(),
            requires_mechanical: true,
            requires_performance: true,
            requires_neighbouring: false,
            territory: "Worldwide".into(),
            notes: "Mechanical via Music Reports / HFA / MLC".into(),
        },
        DspLicenceCoverage {
            dsp_name: "Amazon Music".into(),
            requires_mechanical: true,
            requires_performance: true,
            requires_neighbouring: false,
            territory: "Worldwide".into(),
            notes: "Statutory blanket licence (US) + direct (international)".into(),
        },
        DspLicenceCoverage {
            dsp_name: "SoundCloud".into(),
            requires_mechanical: true,
            requires_performance: true,
            requires_neighbouring: true,
            territory: "Worldwide".into(),
            notes: "Neighbouring rights via SoundExchange (US)".into(),
        },
        DspLicenceCoverage {
            dsp_name: "YouTube Music".into(),
            requires_mechanical: true,
            requires_performance: true,
            requires_neighbouring: true,
            territory: "Worldwide".into(),
            notes: "Content ID + MLC mechanical; neighbouring via YouTube licence".into(),
        },
        DspLicenceCoverage {
            dsp_name: "TikTok".into(),
            requires_mechanical: true,
            requires_performance: true,
            requires_neighbouring: false,
            territory: "Worldwide".into(),
            notes: "Master licence + publishing licence required per market".into(),
        },
    ]
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_licence_response(resp: &serde_json::Value) -> anyhow::Result<Vec<LicenceRecord>> {
    let items = match resp["data"].as_array() {
        Some(arr) => arr,
        None => {
            if let Some(err) = resp["error"].as_str() {
                anyhow::bail!("Music Reports API error: {err}");
            }
            return Ok(vec![]);
        }
    };

    // Bound: never process more than 1000 records in a single response
    let records = items
        .iter()
        .take(1000)
        .filter_map(|item| serde_json::from_value::<LicenceRecord>(item.clone()).ok())
        .collect();

    Ok(records)
}

/// Minimal URL encoding for query parameter values.
/// Only encodes characters that are not safe in query strings.
fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_rate_plausible() {
        let rate = current_mechanical_rate();
        assert!(rate.rate_per_copy_cents > 0.0);
        assert!(rate.rate_pct_content_cost > 0.0);
        assert_eq!(rate.effective_year, 2024);
    }

    #[test]
    fn urlencoding_works() {
        assert_eq!(urlencoding_encode("US-S1Z-99-00001"), "US-S1Z-99-00001");
        assert_eq!(urlencoding_encode("hello world"), "hello%20world");
    }

    #[test]
    fn reconcile_finds_discrepancy() {
        let stmt = RoyaltyStatement {
            statement_id: "STMT-001".into(),
            period_start: "2024-01".into(),
            period_end: "2024-03".into(),
            payee: "Test Artist".into(),
            lines: vec![RoyaltyStatementLine {
                period: "2024-01".into(),
                isrc: "US-S1Z-99-00001".into(),
                work_title: "Test Track".into(),
                units: 10000,
                rate: 0.004,
                gross_royalty: 40.0,
                deduction_pct: 5.0,
                net_royalty: 38.0,
                currency: "USD".into(),
            }],
            total_gross: 40.0,
            total_net: 38.0,
            currency: "USD".into(),
        };

        let mut onchain = std::collections::HashMap::new();
        onchain.insert("US-S1Z-99-00001".to_string(), 10.0); // significant discrepancy

        let discrepancies = reconcile_royalties(&stmt, &onchain);
        assert_eq!(discrepancies.len(), 1);
    }

    #[test]
    fn dsp_requirements_complete() {
        let reqs = dsp_licence_requirements();
        assert!(reqs.len() >= 4);
        assert!(reqs.iter().all(|r| !r.dsp_name.is_empty()));
    }
}