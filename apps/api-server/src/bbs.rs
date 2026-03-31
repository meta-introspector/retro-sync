#![allow(dead_code)]
//! BBS — Broadcast Blanket Service for background and broadcast music licensing.
//!
//! The Broadcast Blanket Service provides:
//!   - Background music blanket licences for public premises (restaurants,
//!     hotels, retail, gyms, broadcast stations, streaming platforms).
//!   - Per-broadcast cue sheet reporting for TV, radio, and online broadcast.
//!   - Integration with PRO blanket licence pools (PRS, ASCAP, BMI, SOCAN,
//!     GEMA, SACEM, and 150+ worldwide collection societies).
//!   - Real-time broadcast monitoring data ingestion (BMAT, MEDIAGUARD feeds).
//!
//! BBS connects to the Retrosync collection society registry to route royalties
//! automatically to the correct PRO/CMO in each territory based on:
//!   - Work ISWC + territory → mechanical/performance split
//!   - Recording ISRC + territory → neighbouring rights split
//!   - Society agreement priority (reciprocal agreements map)
//!
//! LangSec:
//!   - All ISRCs/ISWCs validated before cue sheet generation.
//!   - Station/venue identifiers limited to 100 chars, ASCII-safe.
//!   - Broadcast duration: u32 seconds, max 7200 (2 hours per cue).
//!   - Cue sheet batches: max 10,000 lines per submission.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct BbsConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub broadcaster_id: String,
    pub timeout_secs: u64,
    pub dev_mode: bool,
}

impl BbsConfig {
    #[zkperf_macros::zkperf]
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("BBS_BASE_URL")
                .unwrap_or_else(|_| "https://api.bbs-licensing.com/v2".into()),
            api_key: std::env::var("BBS_API_KEY").ok(),
            broadcaster_id: std::env::var("BBS_BROADCASTER_ID")
                .unwrap_or_else(|_| "RETROSYNC-DEV".into()),
            timeout_secs: std::env::var("BBS_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            dev_mode: std::env::var("BBS_DEV_MODE")
                .map(|v| v == "1")
                .unwrap_or(false),
        }
    }
}

// ── Licence Types ─────────────────────────────────────────────────────────────

/// Types of BBS blanket licence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BbsLicenceType {
    /// Background music for public premises (non-broadcast)
    BackgroundMusic,
    /// Terrestrial radio broadcast
    RadioBroadcast,
    /// Terrestrial TV broadcast
    TvBroadcast,
    /// Online / internet radio streaming
    OnlineRadio,
    /// Podcast / on-demand audio
    Podcast,
    /// Sync / audiovisual (requires separate sync clearance)
    Sync,
    /// Film / cinema
    Cinema,
}

impl BbsLicenceType {
    #[zkperf_macros::zkperf]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::BackgroundMusic => "Background Music (Public Premises)",
            Self::RadioBroadcast => "Terrestrial Radio Broadcast",
            Self::TvBroadcast => "Terrestrial TV Broadcast",
            Self::OnlineRadio => "Online / Internet Radio",
            Self::Podcast => "Podcast / On-Demand Audio",
            Self::Sync => "Synchronisation / AV",
            Self::Cinema => "Film / Cinema",
        }
    }
}

// ── Blanket Licence ────────────────────────────────────────────────────────────

/// A BBS blanket licence record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BbsBlanketLicence {
    pub licence_id: String,
    pub licensee: String,
    pub licence_type: BbsLicenceType,
    pub territories: Vec<String>,
    pub effective_from: DateTime<Utc>,
    pub effective_to: Option<DateTime<Utc>>,
    pub annual_fee_usd: f64,
    pub repertoire_coverage: Vec<String>,
    pub reporting_frequency: ReportingFrequency,
    pub societies_covered: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportingFrequency {
    Monthly,
    Quarterly,
    Annual,
    PerBroadcast,
}

// ── Cue Sheet (Broadcast Play Report) ─────────────────────────────────────────

const MAX_CUE_DURATION_SECS: u32 = 7_200; // 2 hours
const MAX_CUES_PER_BATCH: usize = 10_000;

/// A single broadcast cue (one music play).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastCue {
    /// ISRC of the sound recording played.
    pub isrc: String,
    /// ISWC of the underlying musical work (if known).
    pub iswc: Option<String>,
    /// Title as broadcast (for matching).
    pub title: String,
    /// Performing artist as broadcast.
    pub artist: String,
    /// Broadcast station or venue ID (max 100 chars).
    pub station_id: String,
    /// Territory ISO 3166-1 alpha-2 code.
    pub territory: String,
    /// UTC timestamp of broadcast/play start.
    pub played_at: DateTime<Utc>,
    /// Duration in seconds (max 7200).
    pub duration_secs: u32,
    /// Usage type for this cue.
    pub use_type: BbsLicenceType,
    /// Whether this was a featured or background performance.
    pub featured: bool,
}

/// A batch of cues for a single reporting period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CueSheetBatch {
    pub batch_id: String,
    pub broadcaster_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub cues: Vec<BroadcastCue>,
    pub submitted_at: DateTime<Utc>,
}

/// Validation error for cue sheet data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CueValidationError {
    pub cue_index: usize,
    pub field: String,
    pub reason: String,
}

/// Validate a batch of broadcast cues.
#[zkperf_macros::zkperf]
pub fn validate_cue_batch(cues: &[BroadcastCue]) -> Vec<CueValidationError> {
    let mut errors = Vec::new();
    if cues.len() > MAX_CUES_PER_BATCH {
        errors.push(CueValidationError {
            cue_index: 0,
            field: "batch".into(),
            reason: format!("batch exceeds max {MAX_CUES_PER_BATCH} cues"),
        });
        return errors;
    }
    for (i, cue) in cues.iter().enumerate() {
        // ISRC length check (full validation done by shared parser upstream)
        if cue.isrc.len() != 12 {
            errors.push(CueValidationError {
                cue_index: i,
                field: "isrc".into(),
                reason: "ISRC must be 12 characters (no hyphens)".into(),
            });
        }
        // Station ID
        if cue.station_id.is_empty() || cue.station_id.len() > 100 {
            errors.push(CueValidationError {
                cue_index: i,
                field: "station_id".into(),
                reason: "station_id must be 1–100 characters".into(),
            });
        }
        // Duration
        if cue.duration_secs == 0 || cue.duration_secs > MAX_CUE_DURATION_SECS {
            errors.push(CueValidationError {
                cue_index: i,
                field: "duration_secs".into(),
                reason: format!("duration must be 1–{MAX_CUE_DURATION_SECS} seconds"),
            });
        }
        // Territory: ISO 3166-1 alpha-2, 2 uppercase letters
        if cue.territory.len() != 2 || !cue.territory.chars().all(|c| c.is_ascii_uppercase()) {
            errors.push(CueValidationError {
                cue_index: i,
                field: "territory".into(),
                reason: "territory must be ISO 3166-1 alpha-2 (2 uppercase letters)".into(),
            });
        }
    }
    errors
}

/// Submit a cue sheet batch to the BBS reporting endpoint.
#[instrument(skip(config))]
pub async fn submit_cue_sheet(
    config: &BbsConfig,
    cues: Vec<BroadcastCue>,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
) -> anyhow::Result<CueSheetBatch> {
    let errors = validate_cue_batch(&cues);
    if !errors.is_empty() {
        anyhow::bail!("Cue sheet validation failed: {} errors", errors.len());
    }

    let batch_id = format!(
        "BBS-{}-{:016x}",
        config.broadcaster_id,
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );

    let batch = CueSheetBatch {
        batch_id: batch_id.clone(),
        broadcaster_id: config.broadcaster_id.clone(),
        period_start,
        period_end,
        cues,
        submitted_at: Utc::now(),
    };

    if config.dev_mode {
        info!(batch_id=%batch_id, cues=%batch.cues.len(), "BBS cue sheet (dev mode, not submitted)");
        return Ok(batch);
    }

    if config.api_key.is_none() {
        anyhow::bail!("BBS_API_KEY not set; cannot submit live cue sheet");
    }

    let url = format!("{}/cue-sheets", config.base_url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .user_agent("Retrosync/1.0 BBS-Client")
        .build()?;

    let resp = client
        .post(&url)
        .header(
            "Authorization",
            format!("Bearer {}", config.api_key.as_deref().unwrap_or("")),
        )
        .header("X-Broadcaster-Id", &config.broadcaster_id)
        .json(&batch)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        warn!(batch_id=%batch_id, status, "BBS cue sheet submission failed");
        anyhow::bail!("BBS API error: HTTP {status}");
    }

    Ok(batch)
}

/// Generate a BMAT-compatible broadcast monitoring report CSV.
#[zkperf_macros::zkperf]
pub fn generate_bmat_csv(cues: &[BroadcastCue]) -> String {
    let mut out = String::new();
    out.push_str(
        "ISRC,ISWC,Title,Artist,Station,Territory,PlayedAt,DurationSecs,UseType,Featured\r\n",
    );
    for cue in cues {
        let iswc = cue.iswc.as_deref().unwrap_or("");
        let featured = if cue.featured { "Y" } else { "N" };
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\r\n",
            cue.isrc,
            iswc,
            csv_field(&cue.title),
            csv_field(&cue.artist),
            csv_field(&cue.station_id),
            cue.territory,
            cue.played_at.format("%Y-%m-%dT%H:%M:%SZ"),
            cue.duration_secs,
            cue.use_type.display_name(),
            featured,
        ));
    }
    out
}

fn csv_field(s: &str) -> String {
    if s.starts_with(['=', '+', '-', '@']) {
        format!("\t{s}")
    } else if s.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// ── Blanket Rate Calculator ────────────────────────────────────────────────────

/// Compute estimated blanket licence fee for a venue/broadcaster.
#[zkperf_macros::zkperf]
pub fn estimate_blanket_fee(
    licence_type: &BbsLicenceType,
    territory: &str,
    annual_hours: f64,
) -> f64 {
    // Simplified rate table (USD) — actual rates negotiated per territory
    let base_rate = match licence_type {
        BbsLicenceType::BackgroundMusic => 600.0,
        BbsLicenceType::RadioBroadcast => 2_500.0,
        BbsLicenceType::TvBroadcast => 8_000.0,
        BbsLicenceType::OnlineRadio => 1_200.0,
        BbsLicenceType::Podcast => 500.0,
        BbsLicenceType::Sync => 0.0, // Negotiated per sync
        BbsLicenceType::Cinema => 3_000.0,
    };
    // GDP-adjusted territory multiplier (simplified)
    let territory_multiplier = match territory {
        "US" | "GB" | "DE" | "JP" | "AU" => 1.0,
        "FR" | "IT" | "CA" | "KR" | "NL" => 0.9,
        "BR" | "MX" | "IN" | "ZA" => 0.4,
        "NG" | "PK" | "BD" => 0.2,
        _ => 0.6,
    };
    // Usage multiplier (1.0 at 2000 hrs/year baseline)
    let usage_multiplier = (annual_hours / 2000.0).clamp(0.1, 10.0);
    base_rate * territory_multiplier * usage_multiplier
}