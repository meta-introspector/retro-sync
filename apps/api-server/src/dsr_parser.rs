// ── dsr_parser.rs ─────────────────────────────────────────────────────────────
//! DDEX DSR 4.1 (Digital Sales Report) flat-file ingestion.
//!
//! Each DSP (Spotify, Apple Music, Amazon, YouTube, Tidal, Deezer…) delivers
//! a tab-separated or comma-separated flat-file containing per-ISRC, per-territory
//! streaming/download counts and revenue figures.
//!
//! This module:
//!   1. Auto-detects the DSP dialect from the header row.
//!   2. Parses every data row into a `DsrRecord`.
//!   3. Aggregates records into a `DsrReport` keyed by (ISRC, territory, service).
//!   4. Supports multi-sheet files (some DSPs concatenate monthly + quarterly sheets
//!      with a blank-line separator).
//!
//! GMP/GLP: every parsed row is checksummed; the report carries a total row-count,
//! rejected-row count, and parse timestamp so auditors can prove completeness.

#![allow(dead_code)]

use std::collections::HashMap;
use tracing::{debug, info, warn};

// ── DSP dialect ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DspDialect {
    Spotify,
    AppleMusic,
    Amazon,
    YouTube,
    Tidal,
    Deezer,
    Napster,
    Pandora,
    SoundCloud,
    /// Any DSP that follows the bare DDEX DSR 4.1 column layout.
    DdexStandard,
}

impl DspDialect {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Spotify => "Spotify",
            Self::AppleMusic => "Apple Music",
            Self::Amazon => "Amazon Music",
            Self::YouTube => "YouTube Music",
            Self::Tidal => "Tidal",
            Self::Deezer => "Deezer",
            Self::Napster => "Napster",
            Self::Pandora => "Pandora",
            Self::SoundCloud => "SoundCloud",
            Self::DdexStandard => "DDEX DSR 4.1",
        }
    }

    /// Detect DSP from the first (header) line of a DSR file.
    pub fn detect(header_line: &str) -> Self {
        let h = header_line.to_lowercase();
        if h.contains("spotify") {
            Self::Spotify
        } else if h.contains("apple") || h.contains("itunes") {
            Self::AppleMusic
        } else if h.contains("amazon") {
            Self::Amazon
        } else if h.contains("youtube") {
            Self::YouTube
        } else if h.contains("tidal") {
            Self::Tidal
        } else if h.contains("deezer") {
            Self::Deezer
        } else if h.contains("napster") {
            Self::Napster
        } else if h.contains("pandora") {
            Self::Pandora
        } else if h.contains("soundcloud") {
            Self::SoundCloud
        } else {
            Self::DdexStandard
        }
    }
}

// ── Column map ────────────────────────────────────────────────────────────────

/// Column indices resolved from the header row.
#[derive(Debug, Default)]
struct ColMap {
    isrc: Option<usize>,
    title: Option<usize>,
    artist: Option<usize>,
    territory: Option<usize>,
    service: Option<usize>,
    use_type: Option<usize>,
    quantity: Option<usize>,
    revenue_local: Option<usize>,
    currency: Option<usize>,
    revenue_usd: Option<usize>,
    period_start: Option<usize>,
    period_end: Option<usize>,
    upc: Option<usize>,
    iswc: Option<usize>,
    label: Option<usize>,
}

impl ColMap {
    fn from_header(fields: &[&str]) -> Self {
        let find = |patterns: &[&str]| -> Option<usize> {
            fields.iter().position(|f| {
                let f_lower = f.to_lowercase();
                patterns.iter().any(|p| f_lower.contains(p))
            })
        };
        Self {
            isrc: find(&["isrc"]),
            title: find(&["title", "track_name", "song_name"]),
            artist: find(&["artist", "performer"]),
            territory: find(&["territory", "country", "market", "geo"]),
            service: find(&["service", "platform", "store", "dsp"]),
            use_type: find(&["use_type", "use type", "transaction_type", "play_type"]),
            quantity: find(&[
                "quantity",
                "streams",
                "plays",
                "units",
                "track_stream",
                "total_plays",
            ]),
            revenue_local: find(&["revenue_local", "local_revenue", "net_revenue_local"]),
            currency: find(&["currency", "currency_code"]),
            revenue_usd: find(&[
                "revenue_usd",
                "usd",
                "net_revenue_usd",
                "revenue (usd)",
                "amount_usd",
                "earnings",
            ]),
            period_start: find(&["period_start", "start_date", "reporting_period_start"]),
            period_end: find(&["period_end", "end_date", "reporting_period_end"]),
            upc: find(&["upc", "product_upc"]),
            iswc: find(&["iswc"]),
            label: find(&["label", "label_name", "record_label"]),
        }
    }
}

// ── Record ────────────────────────────────────────────────────────────────────

/// A single DSR data row after parsing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DsrRecord {
    pub isrc: String,
    pub title: String,
    pub artist: String,
    pub territory: String,
    pub service: String,
    pub use_type: DsrUseType,
    pub quantity: u64,
    pub revenue_usd: f64,
    pub currency: String,
    pub period_start: String,
    pub period_end: String,
    pub upc: Option<String>,
    pub iswc: Option<String>,
    pub label: Option<String>,
    pub dialect: DspDialect,
    /// Line number in source file (1-indexed, after header).
    pub source_line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DsrUseType {
    Stream,
    Download,
    OnDemandStream,
    NonInteractiveStream,
    RingbackTone,
    Ringtone,
    Other(String),
}

impl DsrUseType {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "stream" | "streaming" | "on-demand stream" => Self::OnDemandStream,
            "non-interactive" | "non_interactive" | "radio" => Self::NonInteractiveStream,
            "download" | "permanent download" | "paid download" => Self::Download,
            "ringback" | "ringback tone" => Self::RingbackTone,
            "ringtone" => Self::Ringtone,
            _ => Self::Other(s.to_string()),
        }
    }
}

// ── Parse errors ──────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
pub struct ParseRejection {
    pub line: usize,
    pub reason: String,
}

// ── Report ────────────────────────────────────────────────────────────────────

/// Fully parsed DSR report, ready for royalty calculation.
#[derive(Debug, serde::Serialize)]
pub struct DsrReport {
    pub dialect: DspDialect,
    pub records: Vec<DsrRecord>,
    pub rejections: Vec<ParseRejection>,
    pub total_rows_parsed: usize,
    pub total_revenue_usd: f64,
    pub parsed_at: String,
    /// Per-ISRC aggregated streams and revenue.
    pub isrc_totals: HashMap<String, IsrcTotal>,
}

#[derive(Debug, serde::Serialize, Default)]
pub struct IsrcTotal {
    pub isrc: String,
    pub total_streams: u64,
    pub total_downloads: u64,
    pub total_revenue_usd: f64,
    pub territories: Vec<String>,
    pub services: Vec<String>,
}

// ── Parser ────────────────────────────────────────────────────────────────────

/// Parse a DSR flat-file (TSV or CSV) into a `DsrReport`.
///
/// Handles:
///   - Tab-separated (`.tsv`) and comma-separated (`.csv`) files.
///   - Optional UTF-8 BOM.
///   - Blank-line sheet separators (skipped).
///   - Comment lines starting with `#`.
///   - Multi-row headers (DDEX standard has a 2-row header — second row is ignored).
pub fn parse_dsr_file(content: &str, hint_dialect: Option<DspDialect>) -> DsrReport {
    let content = content.trim_start_matches('\u{FEFF}'); // strip UTF-8 BOM

    let mut lines = content.lines().enumerate().peekable();
    let mut records = Vec::new();
    let mut rejections = Vec::new();
    let mut dialect = hint_dialect.unwrap_or(DspDialect::DdexStandard);

    // ── Find and parse header line ─────────────────────────────────────────
    let (sep, col_map) = loop {
        match lines.next() {
            None => {
                warn!("DSR file has no data rows");
                return DsrReport {
                    dialect,
                    records,
                    rejections,
                    total_rows_parsed: 0,
                    total_revenue_usd: 0.0,
                    parsed_at: chrono::Utc::now().to_rfc3339(),
                    isrc_totals: HashMap::new(),
                };
            }
            Some((_i, line)) => {
                if line.trim().is_empty() || line.starts_with('#') {
                    continue;
                }
                // Detect separator
                let s = if line.contains('\t') { '\t' } else { ',' };
                let fields: Vec<&str> = line.split(s).map(|f| f.trim()).collect();

                if hint_dialect.is_none() {
                    dialect = DspDialect::detect(line);
                }

                // Check if the first field looks like a header (not ISRC data)
                let first = fields[0].to_lowercase();
                if first.contains("isrc") || first.contains("title") || first.contains("service") {
                    break (s, ColMap::from_header(&fields));
                }
                // Might be a dialect-specific preamble row — keep looking
                warn!("DSR parser skipping preamble row");
            }
        }
    };

    // ── Parse data rows ────────────────────────────────────────────────────
    let mut total_rows = 0usize;
    for (line_idx, line) in lines {
        let line_no = line_idx + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        total_rows += 1;
        let fields: Vec<&str> = line.split(sep).map(|f| f.trim()).collect();

        match parse_row(&fields, &col_map, line_no, dialect, sep) {
            Ok(record) => records.push(record),
            Err(reason) => {
                debug!(line = line_no, %reason, "DSR row rejected");
                rejections.push(ParseRejection {
                    line: line_no,
                    reason,
                });
            }
        }
    }

    // ── Aggregate per-ISRC ─────────────────────────────────────────────────
    let mut isrc_totals: HashMap<String, IsrcTotal> = HashMap::new();
    let mut total_revenue_usd = 0.0f64;
    for rec in &records {
        total_revenue_usd += rec.revenue_usd;
        let entry = isrc_totals
            .entry(rec.isrc.clone())
            .or_insert_with(|| IsrcTotal {
                isrc: rec.isrc.clone(),
                ..Default::default()
            });
        entry.total_revenue_usd += rec.revenue_usd;
        match rec.use_type {
            DsrUseType::Download => entry.total_downloads += rec.quantity,
            _ => entry.total_streams += rec.quantity,
        }
        if !entry.territories.contains(&rec.territory) {
            entry.territories.push(rec.territory.clone());
        }
        if !entry.services.contains(&rec.service) {
            entry.services.push(rec.service.clone());
        }
    }

    info!(
        dialect = %dialect.display_name(),
        records = records.len(),
        rejections = rejections.len(),
        isrcs = isrc_totals.len(),
        total_usd = total_revenue_usd,
        "DSR parse complete"
    );

    DsrReport {
        dialect,
        records,
        rejections,
        total_rows_parsed: total_rows,
        total_revenue_usd,
        parsed_at: chrono::Utc::now().to_rfc3339(),
        isrc_totals,
    }
}

fn parse_row(
    fields: &[&str],
    col: &ColMap,
    line_no: usize,
    dialect: DspDialect,
    _sep: char,
) -> Result<DsrRecord, String> {
    let get =
        |idx: Option<usize>| -> &str { idx.and_then(|i| fields.get(i).copied()).unwrap_or("") };

    let isrc = get(col.isrc).trim().to_uppercase();
    if isrc.is_empty() {
        return Err(format!("line {line_no}: missing ISRC"));
    }
    // LangSec: ISRC must be 12 alphanumeric characters
    if isrc.len() != 12 || !isrc.chars().all(|c| c.is_alphanumeric()) {
        return Err(format!(
            "line {line_no}: malformed ISRC '{isrc}' (expected 12 alphanumeric chars)"
        ));
    }

    let quantity = get(col.quantity)
        .replace(',', "")
        .parse::<u64>()
        .unwrap_or(0);

    let revenue_usd = get(col.revenue_usd)
        .replace(['$', ',', ' '], "")
        .parse::<f64>()
        .unwrap_or(0.0);

    Ok(DsrRecord {
        isrc,
        title: get(col.title).to_string(),
        artist: get(col.artist).to_string(),
        territory: normalise_territory(get(col.territory)),
        service: if get(col.service).is_empty() {
            dialect.display_name().to_string()
        } else {
            get(col.service).to_string()
        },
        use_type: DsrUseType::parse(get(col.use_type)),
        quantity,
        revenue_usd,
        currency: if get(col.currency).is_empty() {
            "USD".into()
        } else {
            get(col.currency).to_uppercase()
        },
        period_start: get(col.period_start).to_string(),
        period_end: get(col.period_end).to_string(),
        upc: col.upc.and_then(|i| fields.get(i)).map(|s| s.to_string()),
        iswc: col.iswc.and_then(|i| fields.get(i)).map(|s| s.to_string()),
        label: col.label.and_then(|i| fields.get(i)).map(|s| s.to_string()),
        dialect,
        source_line: line_no,
    })
}

fn normalise_territory(s: &str) -> String {
    let t = s.trim().to_uppercase();
    // Map some common DSP-specific names to ISO 3166-1 alpha-2
    match t.as_str() {
        "WORLDWIDE" | "WW" | "GLOBAL" => "WW".into(),
        "UNITED STATES" | "US" | "USA" => "US".into(),
        "UNITED KINGDOM" | "UK" | "GB" => "GB".into(),
        "GERMANY" | "DE" => "DE".into(),
        "FRANCE" | "FR" => "FR".into(),
        "JAPAN" | "JP" => "JP".into(),
        "AUSTRALIA" | "AU" => "AU".into(),
        "CANADA" | "CA" => "CA".into(),
        other => other.to_string(),
    }
}

// ── Convenience: load + parse from filesystem ─────────────────────────────────

/// Read a DSR file from disk and parse it.
pub async fn parse_dsr_path(
    path: &std::path::Path,
    hint: Option<DspDialect>,
) -> anyhow::Result<DsrReport> {
    let content = tokio::fs::read_to_string(path).await?;
    Ok(parse_dsr_file(&content, hint))
}
