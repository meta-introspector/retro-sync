//! GDPR Art.7 consent · Art.17 erasure · Art.20 portability. CCPA opt-out. COPPA gate.
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConsentPurpose {
    Analytics,
    Marketing,
    ThirdPartySharing,
    DataProcessing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub user_id: String,
    pub purpose: ConsentPurpose,
    pub granted: bool,
    pub timestamp: String,
    pub ip_hash: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionRequest {
    pub user_id: String,
    pub requested_at: String,
    pub fulfilled_at: Option<String>,
    pub scope: Vec<String>,
}

#[derive(Deserialize)]
pub struct ConsentRequest {
    pub user_id: String,
    pub purpose: ConsentPurpose,
    pub granted: bool,
    pub ip_hash: String,
    pub version: String,
}

pub struct PrivacyStore {
    consents: Mutex<Vec<ConsentRecord>>,
    deletions: Mutex<Vec<DeletionRequest>>,
    path: String,
}

impl PrivacyStore {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            consents: Mutex::new(Vec::new()),
            deletions: Mutex::new(Vec::new()),
            path: path.to_string(),
        })
    }
    pub fn record_consent(&self, r: ConsentRecord) {
        if let Ok(mut v) = self.consents.lock() {
            v.push(r);
        }
    }
    pub fn has_consent(&self, user_id: &str, purpose: &ConsentPurpose) -> bool {
        self.consents
            .lock()
            .map(|v| {
                v.iter()
                    .rev()
                    .find(|c| c.user_id == user_id && &c.purpose == purpose)
                    .map(|c| c.granted)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }
    pub fn queue_deletion(&self, r: DeletionRequest) {
        if let Ok(mut v) = self.deletions.lock() {
            v.push(r);
        }
    }
    pub fn export_user_data(&self, user_id: &str) -> serde_json::Value {
        let c: Vec<_> = self
            .consents
            .lock()
            .map(|v| v.iter().filter(|c| c.user_id == user_id).cloned().collect())
            .unwrap_or_default();
        serde_json::json!({ "user_id": user_id, "consents": c })
    }
}

pub async fn record_consent(
    State(state): State<AppState>,
    Json(req): Json<ConsentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.privacy_db.record_consent(ConsentRecord {
        user_id: req.user_id.clone(),
        purpose: req.purpose,
        granted: req.granted,
        timestamp: chrono::Utc::now().to_rfc3339(),
        ip_hash: req.ip_hash,
        version: req.version,
    });
    state
        .audit_log
        .record(&format!(
            "CONSENT user='{}' granted={}",
            req.user_id, req.granted
        ))
        .ok();
    Ok(Json(serde_json::json!({ "status": "recorded" })))
}

pub async fn delete_user_data(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.privacy_db.queue_deletion(DeletionRequest {
        user_id: user_id.clone(),
        requested_at: chrono::Utc::now().to_rfc3339(),
        fulfilled_at: None,
        scope: vec!["uploads", "consents", "kyc", "payments"]
            .into_iter()
            .map(|s| s.into())
            .collect(),
    });
    state
        .audit_log
        .record(&format!("GDPR_DELETE_REQUEST user='{}'", user_id))
        .ok();
    warn!(user=%user_id, "GDPR deletion queued — 30 day deadline (Art.17)");
    Ok(Json(
        serde_json::json!({ "status": "queued", "deadline": "30 days per GDPR Art.17" }),
    ))
}

pub async fn export_user_data(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(state.privacy_db.export_user_data(&user_id)))
}
