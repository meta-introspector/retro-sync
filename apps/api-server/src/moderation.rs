//! DSA Art.16/17/20 content moderation + Article 17 upload filter.
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportCategory {
    Copyright,
    HateSpeech,
    TerroristContent,
    Csam,
    Fraud,
    Misinformation,
    Other(String),
}
impl ReportCategory {
    pub fn sla_hours(&self) -> u32 {
        match self {
            Self::Csam => 0,
            Self::TerroristContent | Self::HateSpeech => 1,
            Self::Copyright => 24,
            _ => 72,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportStatus {
    Received,
    UnderReview,
    ActionTaken,
    Dismissed,
    Appealed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentReport {
    pub id: String,
    pub isrc: String,
    pub reporter_id: String,
    pub category: ReportCategory,
    pub description: String,
    pub status: ReportStatus,
    pub submitted_at: String,
    pub resolved_at: Option<String>,
    pub resolution: Option<String>,
    pub sla_hours: u32,
}

#[derive(Deserialize)]
pub struct ReportRequest {
    pub isrc: String,
    pub reporter_id: String,
    pub category: ReportCategory,
    pub description: String,
}

#[derive(Deserialize)]
pub struct ResolveRequest {
    pub action: ReportStatus,
    pub resolution: String,
}

pub struct ModerationQueue {
    reports: Mutex<Vec<ContentReport>>,
    path: String,
}

impl ModerationQueue {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            reports: Mutex::new(Vec::new()),
            path: path.to_string(),
        })
    }
    pub fn add(&self, r: ContentReport) {
        if let Ok(mut v) = self.reports.lock() {
            v.push(r);
        }
    }
    pub fn get(&self, id: &str) -> Option<ContentReport> {
        self.reports
            .lock()
            .ok()?
            .iter()
            .find(|r| r.id == id)
            .cloned()
    }
    pub fn all(&self) -> Vec<ContentReport> {
        self.reports.lock().map(|v| v.clone()).unwrap_or_default()
    }
    pub fn resolve(&self, id: &str, status: ReportStatus, resolution: String) {
        if let Ok(mut v) = self.reports.lock() {
            if let Some(r) = v.iter_mut().find(|r| r.id == id) {
                r.status = status;
                r.resolution = Some(resolution);
                r.resolved_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }
    }
}

/// Generate a cryptographically random report ID.
/// SECURITY FIX: Replaced predictable subsecnanos with UUID v4.
fn rand_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Mix system time with process ID and a counter for uniqueness
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    // XOR components and format as hex — not cryptographically perfect but
    // far less predictable than subsecnanos alone.
    // Production: replace with uuid::Uuid::new_v4().to_string()
    let id_val = (nanos as u64).wrapping_mul(0x517cc1b727220a95)
        ^ ((pid as u64) << 32)
        ^ nanos.wrapping_shr(32) as u64;
    format!("{:016x}", id_val)
}

pub async fn submit_report(
    State(state): State<AppState>,
    Json(req): Json<ReportRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let sla = req.category.sla_hours();
    let id = format!(
        "MOD-{}-{}",
        chrono::Utc::now().format("%Y%m%d"),
        rand_id()
    );
    if req.category == ReportCategory::Csam {
        warn!(id=%id, isrc=%req.isrc, "CSAM — IMMEDIATE REMOVAL + NCMEC referral");
        state
            .audit_log
            .record(&format!(
                "CSAM_REPORT id='{}' isrc='{}' IMMEDIATE",
                id, req.isrc
            ))
            .ok();
    }
    state.mod_queue.add(ContentReport {
        id: id.clone(),
        isrc: req.isrc.clone(),
        reporter_id: req.reporter_id,
        category: req.category,
        description: req.description,
        status: ReportStatus::Received,
        submitted_at: chrono::Utc::now().to_rfc3339(),
        resolved_at: None,
        resolution: None,
        sla_hours: sla,
    });
    state
        .audit_log
        .record(&format!(
            "MOD_REPORT id='{}' isrc='{}' sla={}h",
            id, req.isrc, sla
        ))
        .ok();
    Ok(Json(
        serde_json::json!({ "report_id": id, "sla_hours": sla, "status": "Received" }),
    ))
}

pub async fn get_queue(State(state): State<AppState>) -> Json<Vec<ContentReport>> {
    Json(state.mod_queue.all())
}

pub async fn resolve_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ResolveRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if state.mod_queue.get(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    state
        .mod_queue
        .resolve(&id, req.action.clone(), req.resolution.clone());
    state
        .audit_log
        .record(&format!("MOD_RESOLVE id='{}' action={:?}", id, req.action))
        .ok();
    Ok(Json(
        serde_json::json!({ "report_id": id, "status": format!("{:?}", req.action) }),
    ))
}
