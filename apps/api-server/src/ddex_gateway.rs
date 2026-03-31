// ── ddex_gateway.rs ────────────────────────────────────────────────────────────
//! DDEX Gateway — automated ERN (push) and DSR (pull) cycles.
//!
//! V-model (GMP/GLP) approach:
//!   Every operation is a named, sequenced "Gateway Event" with an ISO-8601 timestamp
//!   and a monotonic sequence number.  Events are stored in the audit log and can be
//!   used by auditors to prove "track X was delivered to DSP Y at time T, and revenue
//!   from DSP Y was ingested at time T+Δ."
//!
//! ERN Push cycle:
//!   1. Collect pending release metadata from the pending queue.
//!   2. Build DDEX ERN 4.1 XML (using ddex::build_ern_xml_with_contributors).
//!   3. Write XML to a staging directory.
//!   4. SFTP PUT to each configured DSP endpoint.
//!   5. Record TransferReceipt in the audit log.
//!   6. Move staging file to a "sent" archive.
//!
//! DSR Pull cycle:
//!   1. SFTP LIST the DSP drop directory.
//!   2. For each new file: SFTP GET → local temp dir.
//!   3. Parse with dsr_parser::parse_dsr_file.
//!   4. Emit per-ISRC royalty totals to the royalty pipeline.
//!   5. (Optionally) delete or archive the remote file.
//!   6. Record audit event.

#![allow(dead_code)]

use crate::ddex::{build_ern_xml_with_contributors, DdexContributor};
use crate::dsr_parser::{parse_dsr_path, DspDialect, DsrReport};
use crate::sftp::{sftp_delete, sftp_get, sftp_list, sftp_put, SftpConfig, TransferReceipt};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{error, info, warn};

// ── Sequence counter ──────────────────────────────────────────────────────────

/// Global gateway audit sequence number (monotonically increasing).
static AUDIT_SEQ: AtomicU64 = AtomicU64::new(1);

fn next_seq() -> u64 {
    AUDIT_SEQ.fetch_add(1, Ordering::SeqCst)
}

// ── DSP endpoint registry ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DspId {
    Spotify,
    AppleMusic,
    AmazonMusic,
    YouTubeMusic,
    Tidal,
    Deezer,
    Napster,
    Pandora,
    SoundCloud,
    Custom(String),
}

impl DspId {
    #[zkperf_macros::zkperf]
    pub fn display_name(&self) -> &str {
        match self {
            Self::Spotify => "Spotify",
            Self::AppleMusic => "Apple Music",
            Self::AmazonMusic => "Amazon Music",
            Self::YouTubeMusic => "YouTube Music",
            Self::Tidal => "Tidal",
            Self::Deezer => "Deezer",
            Self::Napster => "Napster",
            Self::Pandora => "Pandora",
            Self::SoundCloud => "SoundCloud",
            Self::Custom(name) => name.as_str(),
        }
    }

    #[zkperf_macros::zkperf]
    pub fn dsr_dialect(&self) -> DspDialect {
        match self {
            Self::Spotify => DspDialect::Spotify,
            Self::AppleMusic => DspDialect::AppleMusic,
            Self::AmazonMusic => DspDialect::Amazon,
            Self::YouTubeMusic => DspDialect::YouTube,
            Self::Tidal => DspDialect::Tidal,
            Self::Deezer => DspDialect::Deezer,
            Self::Napster => DspDialect::Napster,
            Self::Pandora => DspDialect::Pandora,
            Self::SoundCloud => DspDialect::SoundCloud,
            Self::Custom(_) => DspDialect::DdexStandard,
        }
    }
}

// ── Gateway configuration ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DspEndpointConfig {
    pub dsp_id: DspId,
    pub sftp: SftpConfig,
    /// True if this DSP accepts ERN push from us.
    pub accepts_ern: bool,
    /// True if this DSP drops DSR files for us to ingest.
    pub drops_dsr: bool,
    /// Delete DSR files after successful ingestion.
    pub delete_after_ingest: bool,
}

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub endpoints: Vec<DspEndpointConfig>,
    /// Local directory for staging ERN XML before SFTP push.
    pub ern_staging_dir: PathBuf,
    /// Local directory for downloaded DSR files.
    pub dsr_staging_dir: PathBuf,
    /// Minimum bytes a DSR file must contain to be processed (guards against empty drops).
    pub min_dsr_file_bytes: u64,
    pub dev_mode: bool,
}

impl GatewayConfig {
    #[zkperf_macros::zkperf]
    pub fn from_env() -> Self {
        let dev = std::env::var("GATEWAY_DEV_MODE").unwrap_or_default() == "1";
        // Load the "default" DSP from env; real deployments configure per-DSP SFTP creds.
        let default_sftp = SftpConfig::from_env("SFTP");
        let endpoints = vec![
            DspEndpointConfig {
                dsp_id: DspId::Spotify,
                sftp: SftpConfig::from_env("SFTP_SPOTIFY"),
                accepts_ern: true,
                drops_dsr: true,
                delete_after_ingest: false,
            },
            DspEndpointConfig {
                dsp_id: DspId::AppleMusic,
                sftp: SftpConfig::from_env("SFTP_APPLE"),
                accepts_ern: true,
                drops_dsr: true,
                delete_after_ingest: true,
            },
            DspEndpointConfig {
                dsp_id: DspId::AmazonMusic,
                sftp: SftpConfig::from_env("SFTP_AMAZON"),
                accepts_ern: true,
                drops_dsr: true,
                delete_after_ingest: false,
            },
            DspEndpointConfig {
                dsp_id: DspId::YouTubeMusic,
                sftp: SftpConfig::from_env("SFTP_YOUTUBE"),
                accepts_ern: true,
                drops_dsr: true,
                delete_after_ingest: false,
            },
            DspEndpointConfig {
                dsp_id: DspId::Tidal,
                sftp: SftpConfig::from_env("SFTP_TIDAL"),
                accepts_ern: true,
                drops_dsr: true,
                delete_after_ingest: true,
            },
            DspEndpointConfig {
                dsp_id: DspId::Deezer,
                sftp: SftpConfig::from_env("SFTP_DEEZER"),
                accepts_ern: true,
                drops_dsr: true,
                delete_after_ingest: false,
            },
            DspEndpointConfig {
                dsp_id: DspId::SoundCloud,
                sftp: default_sftp,
                accepts_ern: false,
                drops_dsr: true,
                delete_after_ingest: false,
            },
        ];

        Self {
            endpoints,
            ern_staging_dir: PathBuf::from(
                std::env::var("ERN_STAGING_DIR").unwrap_or_else(|_| "/tmp/ern_staging".into()),
            ),
            dsr_staging_dir: PathBuf::from(
                std::env::var("DSR_STAGING_DIR").unwrap_or_else(|_| "/tmp/dsr_staging".into()),
            ),
            min_dsr_file_bytes: std::env::var("MIN_DSR_FILE_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(512),
            dev_mode: dev,
        }
    }
}

// ── Audit event ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct GatewayEvent {
    pub seq: u64,
    pub event_type: GatewayEventType,
    pub dsp: String,
    pub isrc: Option<String>,
    pub detail: String,
    pub timestamp: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum GatewayEventType {
    ErnGenerated,
    ErnDelivered,
    ErnDeliveryFailed,
    DsrDiscovered,
    DsrDownloaded,
    DsrParsed,
    DsrIngestionFailed,
    DsrDeleted,
    RoyaltyEmitted,
}

fn make_event(
    event_type: GatewayEventType,
    dsp: &str,
    isrc: Option<&str>,
    detail: impl Into<String>,
    success: bool,
) -> GatewayEvent {
    GatewayEvent {
        seq: next_seq(),
        event_type,
        dsp: dsp.to_string(),
        isrc: isrc.map(String::from),
        detail: detail.into(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        success,
    }
}

// ── ERN push (outbound) ───────────────────────────────────────────────────────

/// A pending release ready for ERN push.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingRelease {
    pub isrc: String,
    pub title: String,
    pub btfs_cid: String,
    pub contributors: Vec<DdexContributor>,
    pub wikidata: Option<crate::wikidata::WikidataArtist>,
    pub master_fp: Option<shared::master_pattern::PatternFingerprint>,
    /// Which DSPs to push to. Empty = all ERN-capable DSPs.
    pub target_dsps: Vec<String>,
}

/// Result of a single ERN push to one DSP.
#[derive(Debug, Clone, Serialize)]
pub struct ErnDeliveryResult {
    pub dsp: String,
    pub isrc: String,
    pub local_ern_path: String,
    pub receipt: Option<TransferReceipt>,
    pub event: GatewayEvent,
}

/// Push an ERN for a single release to all target DSPs.
///
/// Returns one `ErnDeliveryResult` per DSP attempted.
#[zkperf_macros::zkperf]
pub async fn push_ern(config: &GatewayConfig, release: &PendingRelease) -> Vec<ErnDeliveryResult> {
    let mut results = Vec::new();

    // Build the ERN XML once (same XML goes to all DSPs)
    let wiki = release.wikidata.clone().unwrap_or_default();
    let fp = release.master_fp.clone().unwrap_or_default();
    let xml = build_ern_xml_with_contributors(
        &release.title,
        &release.isrc,
        &release.btfs_cid,
        &fp,
        &wiki,
        &release.contributors,
    );

    // Write to staging dir
    let filename = format!("ERN_{}_{}.xml", release.isrc, next_seq());
    let local_path = config.ern_staging_dir.join(&filename);

    if let Err(e) = tokio::fs::create_dir_all(&config.ern_staging_dir).await {
        warn!(err=%e, "Could not create ERN staging dir");
    }
    if let Err(e) = tokio::fs::write(&local_path, xml.as_bytes()).await {
        error!(err=%e, "Failed to write ERN XML to staging");
        return results;
    }

    let ev = make_event(
        GatewayEventType::ErnGenerated,
        "gateway",
        Some(&release.isrc),
        format!("ERN XML staged: {}", local_path.display()),
        true,
    );
    info!(seq = ev.seq, isrc = %release.isrc, "ERN generated");

    // Push to each target DSP
    for ep in &config.endpoints {
        if !ep.accepts_ern {
            continue;
        }
        let dsp_name = ep.dsp_id.display_name();
        if !release.target_dsps.is_empty()
            && !release
                .target_dsps
                .iter()
                .any(|t| t.eq_ignore_ascii_case(dsp_name))
        {
            continue;
        }

        let result = sftp_put(&ep.sftp, &local_path, &filename).await;
        match result {
            Ok(receipt) => {
                let ev = make_event(
                    GatewayEventType::ErnDelivered,
                    dsp_name,
                    Some(&release.isrc),
                    format!(
                        "Delivered {} bytes, sha256={}",
                        receipt.bytes, receipt.sha256
                    ),
                    true,
                );
                info!(seq = ev.seq, dsp = %dsp_name, isrc = %release.isrc, "ERN delivered");
                results.push(ErnDeliveryResult {
                    dsp: dsp_name.to_string(),
                    isrc: release.isrc.clone(),
                    local_ern_path: local_path.to_string_lossy().into(),
                    receipt: Some(receipt),
                    event: ev,
                });
            }
            Err(e) => {
                let ev = make_event(
                    GatewayEventType::ErnDeliveryFailed,
                    dsp_name,
                    Some(&release.isrc),
                    format!("SFTP push failed: {e}"),
                    false,
                );
                warn!(seq = ev.seq, dsp = %dsp_name, isrc = %release.isrc, err=%e, "ERN delivery failed");
                results.push(ErnDeliveryResult {
                    dsp: dsp_name.to_string(),
                    isrc: release.isrc.clone(),
                    local_ern_path: local_path.to_string_lossy().into(),
                    receipt: None,
                    event: ev,
                });
            }
        }
    }

    results
}

// ── DSR pull (inbound) ────────────────────────────────────────────────────────

/// Result of a single DSR ingestion run from one DSP.
#[derive(Debug, Serialize)]
pub struct DsrIngestionResult {
    pub dsp: String,
    pub files_discovered: usize,
    pub files_processed: usize,
    pub files_rejected: usize,
    pub total_records: usize,
    pub total_revenue_usd: f64,
    pub reports: Vec<DsrReport>,
    pub events: Vec<GatewayEvent>,
}

/// Poll one DSP SFTP drop, download all new DSR files, parse them, and return
/// aggregated royalty data.
#[zkperf_macros::zkperf]
pub async fn ingest_dsr_from_dsp(
    config: &GatewayConfig,
    ep: &DspEndpointConfig,
) -> DsrIngestionResult {
    let dsp_name = ep.dsp_id.display_name();
    let mut events = Vec::new();
    let mut reports = Vec::new();
    let mut files_processed = 0usize;
    let mut files_rejected = 0usize;

    // ── Step 1: discover DSR files ──────────────────────────────────────────
    let file_list = match sftp_list(&ep.sftp).await {
        Ok(list) => list,
        Err(e) => {
            let ev = make_event(
                GatewayEventType::DsrIngestionFailed,
                dsp_name,
                None,
                format!("sftp_list failed: {e}"),
                false,
            );
            warn!(seq = ev.seq, dsp = %dsp_name, err=%e, "DSR discovery failed");
            events.push(ev);
            return DsrIngestionResult {
                dsp: dsp_name.to_string(),
                files_discovered: 0,
                files_processed,
                files_rejected,
                total_records: 0,
                total_revenue_usd: 0.0,
                reports,
                events,
            };
        }
    };

    let files_discovered = file_list.len();
    let ev = make_event(
        GatewayEventType::DsrDiscovered,
        dsp_name,
        None,
        format!("Discovered {files_discovered} DSR file(s)"),
        true,
    );
    info!(seq = ev.seq, dsp = %dsp_name, count = files_discovered, "DSR files discovered");
    events.push(ev);

    // ── Step 2: download + parse each file ──────────────────────────────────
    let dsp_dir = config.dsr_staging_dir.join(dsp_name.replace(' ', "_"));
    for filename in &file_list {
        // LangSec: validate filename before any filesystem ops
        if filename.contains('/') || filename.contains("..") {
            warn!(file = %filename, "DSR filename contains path traversal chars — skipping");
            files_rejected += 1;
            continue;
        }

        let (local_path, receipt) = match sftp_get(&ep.sftp, filename, &dsp_dir).await {
            Ok(r) => r,
            Err(e) => {
                let ev = make_event(
                    GatewayEventType::DsrIngestionFailed,
                    dsp_name,
                    None,
                    format!("sftp_get({filename}) failed: {e}"),
                    false,
                );
                warn!(seq = ev.seq, dsp = %dsp_name, file = %filename, err=%e, "DSR download failed");
                events.push(ev);
                files_rejected += 1;
                continue;
            }
        };

        // Guard against empty / suspiciously small files
        if receipt.bytes < config.min_dsr_file_bytes {
            warn!(
                file = %filename,
                bytes = receipt.bytes,
                "DSR file too small — likely empty drop, skipping"
            );
            files_rejected += 1;
            continue;
        }

        let ev = make_event(
            GatewayEventType::DsrDownloaded,
            dsp_name,
            None,
            format!(
                "Downloaded {} ({} bytes, sha256={})",
                filename, receipt.bytes, receipt.sha256
            ),
            true,
        );
        events.push(ev);

        // Parse
        let report = match parse_dsr_path(&local_path, Some(ep.dsp_id.dsr_dialect())).await {
            Ok(r) => r,
            Err(e) => {
                let ev = make_event(
                    GatewayEventType::DsrIngestionFailed,
                    dsp_name,
                    None,
                    format!("parse_dsr_path({filename}) failed: {e}"),
                    false,
                );
                warn!(seq = ev.seq, dsp = %dsp_name, file = %filename, err=%e, "DSR parse failed");
                events.push(ev);
                files_rejected += 1;
                continue;
            }
        };

        let ev = make_event(
            GatewayEventType::DsrParsed,
            dsp_name,
            None,
            format!(
                "Parsed {} records ({} ISRCs, ${:.2} revenue)",
                report.records.len(),
                report.isrc_totals.len(),
                report.total_revenue_usd
            ),
            true,
        );
        info!(
            seq = ev.seq,
            dsp = %dsp_name,
            records = report.records.len(),
            revenue = report.total_revenue_usd,
            "DSR parsed"
        );
        events.push(ev);
        files_processed += 1;
        reports.push(report);

        // ── Step 3: optionally delete the remote file ───────────────────────
        if ep.delete_after_ingest {
            if let Err(e) = sftp_delete(&ep.sftp, filename).await {
                warn!(dsp = %dsp_name, file = %filename, err=%e, "DSR remote delete failed");
            } else {
                let ev = make_event(
                    GatewayEventType::DsrDeleted,
                    dsp_name,
                    None,
                    format!("Deleted remote file {filename}"),
                    true,
                );
                events.push(ev);
            }
        }
    }

    // ── Aggregate revenue across all parsed reports ──────────────────────────
    let total_records: usize = reports.iter().map(|r| r.records.len()).sum();
    let total_revenue_usd: f64 = reports.iter().map(|r| r.total_revenue_usd).sum();

    DsrIngestionResult {
        dsp: dsp_name.to_string(),
        files_discovered,
        files_processed,
        files_rejected,
        total_records,
        total_revenue_usd,
        reports,
        events,
    }
}

/// Run a full DSR ingestion cycle across ALL configured DSPs that drop DSR files.
#[zkperf_macros::zkperf]
pub async fn run_dsr_cycle(config: &GatewayConfig) -> Vec<DsrIngestionResult> {
    let mut results = Vec::new();
    for ep in &config.endpoints {
        if !ep.drops_dsr {
            continue;
        }
        let result = ingest_dsr_from_dsp(config, ep).await;
        results.push(result);
    }
    results
}

/// Run a full ERN push cycle for a list of pending releases.
#[zkperf_macros::zkperf]
pub async fn run_ern_cycle(
    config: &GatewayConfig,
    releases: &[PendingRelease],
) -> Vec<ErnDeliveryResult> {
    let mut all_results = Vec::new();
    for release in releases {
        let mut results = push_ern(config, release).await;
        all_results.append(&mut results);
    }
    all_results
}

// ── Gateway status snapshot ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct GatewayStatus {
    pub dsp_count: usize,
    pub ern_capable_dsps: Vec<String>,
    pub dsr_capable_dsps: Vec<String>,
    pub audit_seq_watermark: u64,
    pub dev_mode: bool,
}

#[zkperf_macros::zkperf]
pub fn gateway_status(config: &GatewayConfig) -> GatewayStatus {
    let ern_capable: Vec<String> = config
        .endpoints
        .iter()
        .filter(|e| e.accepts_ern)
        .map(|e| e.dsp_id.display_name().to_string())
        .collect();
    let dsr_capable: Vec<String> = config
        .endpoints
        .iter()
        .filter(|e| e.drops_dsr)
        .map(|e| e.dsp_id.display_name().to_string())
        .collect();
    GatewayStatus {
        dsp_count: config.endpoints.len(),
        ern_capable_dsps: ern_capable,
        dsr_capable_dsps: dsr_capable,
        audit_seq_watermark: AUDIT_SEQ.load(Ordering::SeqCst),
        dev_mode: config.dev_mode,
    }
}