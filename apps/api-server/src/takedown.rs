//! DMCA §512 notice-and-takedown. EU Copyright Directive Art. 17.
//!
//! Persistence: LMDB via persist::LmdbStore — notices survive server restarts.
//! The rand_id now uses OS entropy for unpredictable DMCA IDs.
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NoticeStatus {
    Received,
    UnderReview,
    ContentRemoved,
    CounterReceived,
    Restored,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakedownNotice {
    pub id: String,
    pub isrc: String,
    pub claimant_name: String,
    pub claimant_email: String,
    pub work_description: String,
    pub infringing_url: String,
    pub good_faith: bool,
    pub accuracy: bool,
    pub status: NoticeStatus,
    pub submitted_at: String,
    pub resolved_at: Option<String>,
    pub counter_notice: Option<CounterNotice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterNotice {
    pub uploader_name: String,
    pub uploader_email: String,
    pub good_faith: bool,
    pub submitted_at: String,
}

#[derive(Deserialize)]
pub struct TakedownRequest {
    pub isrc: String,
    pub claimant_name: String,
    pub claimant_email: String,
    pub work_description: String,
    pub infringing_url: String,
    pub good_faith: bool,
    pub accuracy: bool,
}

#[derive(Deserialize)]
pub struct CounterNoticeRequest {
    pub uploader_name: String,
    pub uploader_email: String,
    pub good_faith: bool,
}

pub struct TakedownStore {
    db: crate::persist::LmdbStore,
}

impl TakedownStore {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            db: crate::persist::LmdbStore::open(path, "dmca_notices")?,
        })
    }

    pub fn add(&self, n: TakedownNotice) -> anyhow::Result<()> {
        self.db.put(&n.id, &n)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<TakedownNotice> {
        self.db.get(id).ok().flatten()
    }

    pub fn update_status(&self, id: &str, status: NoticeStatus) {
        let _ = self.db.update::<TakedownNotice>(id, |n| {
            n.status = status.clone();
            n.resolved_at = Some(chrono::Utc::now().to_rfc3339());
        });
    }

    pub fn set_counter(&self, id: &str, counter: CounterNotice) {
        let _ = self.db.update::<TakedownNotice>(id, |n| {
            n.counter_notice = Some(counter.clone());
            n.status = NoticeStatus::CounterReceived;
        });
    }
}

/// Cryptographically random 8-hex-char suffix for DMCA IDs.
fn rand_id() -> String {
    crate::wallet_auth::random_hex_pub(4)
}

pub async fn submit_notice(
    State(state): State<AppState>,
    Json(req): Json<TakedownRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !req.good_faith || !req.accuracy {
        return Err(StatusCode::BAD_REQUEST);
    }
    let id = format!("DMCA-{}-{}", chrono::Utc::now().format("%Y%m%d"), rand_id());
    let notice = TakedownNotice {
        id: id.clone(),
        isrc: req.isrc.clone(),
        claimant_name: req.claimant_name.clone(),
        claimant_email: req.claimant_email.clone(),
        work_description: req.work_description.clone(),
        infringing_url: req.infringing_url.clone(),
        good_faith: req.good_faith,
        accuracy: req.accuracy,
        status: NoticeStatus::Received,
        submitted_at: chrono::Utc::now().to_rfc3339(),
        resolved_at: None,
        counter_notice: None,
    };
    state
        .takedown_db
        .add(notice)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state
        .audit_log
        .record(&format!(
            "DMCA_NOTICE id='{}' isrc='{}' claimant='{}'",
            id, req.isrc, req.claimant_name
        ))
        .ok();
    state
        .takedown_db
        .update_status(&id, NoticeStatus::ContentRemoved);
    info!(id=%id, isrc=%req.isrc, "DMCA notice received — content removed (24h SLA)");
    Ok(Json(serde_json::json!({
        "notice_id": id, "status": "ContentRemoved",
        "message": "Notice received. Content removed within 24h per DMCA §512.",
        "counter_notice_window": "10 business days",
    })))
}

pub async fn submit_counter_notice(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CounterNoticeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if state.takedown_db.get(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    if !req.good_faith {
        return Err(StatusCode::BAD_REQUEST);
    }
    state.takedown_db.set_counter(
        &id,
        CounterNotice {
            uploader_name: req.uploader_name,
            uploader_email: req.uploader_email,
            good_faith: req.good_faith,
            submitted_at: chrono::Utc::now().to_rfc3339(),
        },
    );
    state
        .audit_log
        .record(&format!("DMCA_COUNTER id='{id}'"))
        .ok();
    Ok(Json(
        serde_json::json!({ "notice_id": id, "status": "CounterReceived",
        "message": "Content restored in 10-14 business days if no lawsuit filed per §512(g)." }),
    ))
}

pub async fn get_notice(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TakedownNotice>, StatusCode> {
    state
        .takedown_db
        .get(&id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
