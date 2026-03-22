//! DSA Art.16/17/20 content moderation + Article 17 upload filter.
//!
//! Persistence: LMDB via persist::LmdbStore.
//! Report IDs use a cryptographically random 16-byte hex string.
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::warn;

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
    db: crate::persist::LmdbStore,
}

impl ModerationQueue {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            db: crate::persist::LmdbStore::open(path, "mod_reports")?,
        })
    }

    pub fn add(&self, r: ContentReport) {
        if let Err(e) = self.db.put(&r.id, &r) {
            tracing::error!(err=%e, id=%r.id, "Moderation persist error");
        }
    }

    pub fn get(&self, id: &str) -> Option<ContentReport> {
        self.db.get(id).ok().flatten()
    }

    pub fn all(&self) -> Vec<ContentReport> {
        self.db.all_values().unwrap_or_default()
    }

    pub fn resolve(&self, id: &str, status: ReportStatus, resolution: String) {
        let _ = self.db.update::<ContentReport>(id, |r| {
            r.status = status.clone();
            r.resolution = Some(resolution.clone());
            r.resolved_at = Some(chrono::Utc::now().to_rfc3339());
        });
    }
}

/// Generate a cryptographically random report ID using OS entropy.
fn rand_id() -> String {
    crate::wallet_auth::random_hex_pub(16)
}

/// Submit an electronic report to the NCMEC CyberTipline (18 U.S.C. §2258A).
///
/// Requires `NCMEC_API_KEY` env var.  In development (no key set), logs a
/// warning and returns a synthetic report ID so the flow can be tested.
///
/// Production endpoint: https://api.cybertipline.org/v1/reports
/// Sandbox endpoint:    https://sandbox.api.cybertipline.org/v1/reports
/// (Set via `NCMEC_API_URL` env var.)
async fn submit_ncmec_report(report_id: &str, isrc: &str) -> anyhow::Result<String> {
    let api_key = match std::env::var("NCMEC_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            warn!(
                report_id=%report_id,
                "NCMEC_API_KEY not set — CSAM report NOT submitted to NCMEC. \
                 Set NCMEC_API_KEY in production. Manual submission required."
            );
            return Ok(format!("DEV-UNSUBMITTED-{report_id}"));
        }
    };

    let endpoint = std::env::var("NCMEC_API_URL")
        .unwrap_or_else(|_| "https://api.cybertipline.org/v1/reports".into());

    let body = serde_json::json!({
        "reportType": "CSAM",
        "incidentSummary": "Potential CSAM identified during upload fingerprint scan",
        "contentIdentifier": {
            "type": "ISRC",
            "value": isrc
        },
        "reportingEntity": {
            "name": "Retrosync Media Group",
            "type": "ESP",
            "internalReportId": report_id
        },
        "reportedAt": chrono::Utc::now().to_rfc3339(),
        "immediateRemoval": true
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("NCMEC API unreachable: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body_text = resp.text().await.unwrap_or_default();
        anyhow::bail!("NCMEC API returned {status}: {body_text}");
    }

    let result: serde_json::Value = resp.json().await.unwrap_or_else(|_| serde_json::json!({}));

    let ncmec_id = result["reportId"]
        .as_str()
        .or_else(|| result["id"].as_str())
        .unwrap_or(report_id)
        .to_string();

    Ok(ncmec_id)
}

pub async fn submit_report(
    State(state): State<AppState>,
    Json(req): Json<ReportRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let sla = req.category.sla_hours();
    let id = format!("MOD-{}-{}", chrono::Utc::now().format("%Y%m%d"), rand_id());
    if req.category == ReportCategory::Csam {
        warn!(id=%id, isrc=%req.isrc, "CSAM — IMMEDIATE REMOVAL + NCMEC CyberTipline referral");
        state
            .audit_log
            .record(&format!(
                "CSAM_REPORT id='{}' isrc='{}' IMMEDIATE",
                id, req.isrc
            ))
            .ok();
        // LEGAL REQUIREMENT: Electronic report to NCMEC CyberTipline (18 U.S.C. §2258A)
        // Spawn non-blocking so the API call doesn't delay content removal
        let report_id_clone = id.clone();
        let isrc_clone = req.isrc.clone();
        tokio::spawn(async move {
            match submit_ncmec_report(&report_id_clone, &isrc_clone).await {
                Ok(ncmec_id) => {
                    tracing::info!(
                        report_id=%report_id_clone,
                        ncmec_id=%ncmec_id,
                        "NCMEC CyberTipline report submitted successfully"
                    );
                }
                Err(e) => {
                    // Log as CRITICAL — failure to report CSAM is a federal crime
                    tracing::error!(
                        report_id=%report_id_clone,
                        err=%e,
                        "CRITICAL: NCMEC CyberTipline report FAILED — manual submission required immediately"
                    );
                }
            }
        });
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

/// SECURITY FIX: Admin-only endpoint.
///
/// The queue exposes CSAM report details, hate-speech evidence, and reporter
/// identities.  Access is restricted to addresses listed in the
/// `ADMIN_WALLET_ADDRESSES` env var (comma-separated, lower-case 0x or Tron).
///
/// In development (var not set), a warning is logged and access is denied so
/// developers are reminded to configure admin wallets before shipping.
pub async fn get_queue(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Vec<ContentReport>>, axum::http::StatusCode> {
    // Extract the caller's wallet address from the JWT (injected by verify_zero_trust
    // as the X-Wallet-Address header).
    let caller = request
        .headers()
        .get("x-wallet-address")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    let admin_list_raw =
        std::env::var("ADMIN_WALLET_ADDRESSES").unwrap_or_default();

    if admin_list_raw.is_empty() {
        tracing::warn!(
            caller=%caller,
            "ADMIN_WALLET_ADDRESSES not set — denying access to moderation queue. \
             Configure this env var before enabling admin access."
        );
        return Err(axum::http::StatusCode::FORBIDDEN);
    }

    let is_admin = admin_list_raw
        .split(',')
        .map(|a| a.trim().to_ascii_lowercase())
        .any(|a| a == caller);

    if !is_admin {
        tracing::warn!(
            %caller,
            "Unauthorized attempt to access moderation queue — not in ADMIN_WALLET_ADDRESSES"
        );
        return Err(axum::http::StatusCode::FORBIDDEN);
    }

    state
        .audit_log
        .record(&format!("ADMIN_MOD_QUEUE_ACCESS caller='{caller}'"))
        .ok();

    Ok(Json(state.mod_queue.all()))
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
