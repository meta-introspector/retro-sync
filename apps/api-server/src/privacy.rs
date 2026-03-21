//! GDPR Art.7 consent · Art.17 erasure · Art.20 portability. CCPA opt-out.
//!
//! Persistence: LMDB via persist::LmdbStore.
//! Per-user auth: callers may only read/modify their own data.
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::{Deserialize, Serialize};
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
    consent_db: crate::persist::LmdbStore,
    deletion_db: crate::persist::LmdbStore,
}

impl PrivacyStore {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        // Two named databases inside the same LMDB directory
        let consent_dir = format!("{}/consents", path);
        let deletion_dir = format!("{}/deletions", path);
        Ok(Self {
            consent_db: crate::persist::LmdbStore::open(&consent_dir, "consents")?,
            deletion_db: crate::persist::LmdbStore::open(&deletion_dir, "deletions")?,
        })
    }

    /// Append a consent record; key = user_id (list of consents per user).
    pub fn record_consent(&self, r: ConsentRecord) {
        if let Err(e) = self.consent_db.append(&r.user_id, r) {
            tracing::error!(err=%e, "Consent persist error");
        }
    }

    /// Return the latest consent value for (user_id, purpose).
    pub fn has_consent(&self, user_id: &str, purpose: &ConsentPurpose) -> bool {
        self.consent_db
            .get_list::<ConsentRecord>(user_id)
            .unwrap_or_default()
            .into_iter()
            .rev()
            .find(|c| &c.purpose == purpose)
            .map(|c| c.granted)
            .unwrap_or(false)
    }

    /// Queue a GDPR deletion request.
    pub fn queue_deletion(&self, r: DeletionRequest) {
        if let Err(e) = self.deletion_db.put(&r.user_id, &r) {
            tracing::error!(err=%e, user=%r.user_id, "Deletion persist error");
        }
    }

    /// Export all consent records for a user (GDPR Art.20 portability).
    pub fn export_user_data(&self, user_id: &str) -> serde_json::Value {
        let consents = self
            .consent_db
            .get_list::<ConsentRecord>(user_id)
            .unwrap_or_default();
        serde_json::json!({ "user_id": user_id, "consents": consents })
    }
}

// ── HTTP handlers ─────────────────────────────────────────────────────────────

pub async fn record_consent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ConsentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // PER-USER AUTH: the caller's wallet address must match the user_id in the request
    let caller = crate::auth::extract_caller(&headers)?;
    if caller.to_ascii_lowercase() != req.user_id.to_ascii_lowercase() {
        warn!(caller=%caller, uid=%req.user_id, "Consent: caller != uid — forbidden");
        return Err(StatusCode::FORBIDDEN);
    }

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
    headers: HeaderMap,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // PER-USER AUTH: caller may only delete their own data
    let caller = crate::auth::extract_caller(&headers)?;
    if caller.to_ascii_lowercase() != user_id.to_ascii_lowercase() {
        warn!(caller=%caller, uid=%user_id, "Privacy delete: caller != uid — forbidden");
        return Err(StatusCode::FORBIDDEN);
    }

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
    headers: HeaderMap,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // PER-USER AUTH: caller may only export their own data
    let caller = crate::auth::extract_caller(&headers)?;
    if caller.to_ascii_lowercase() != user_id.to_ascii_lowercase() {
        warn!(caller=%caller, uid=%user_id, "Privacy export: caller != uid — forbidden");
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(state.privacy_db.export_user_data(&user_id)))
}
