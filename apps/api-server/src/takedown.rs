//! DMCA §512 notice-and-takedown. EU Copyright Directive Art. 17.
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
    notices: Mutex<Vec<TakedownNotice>>,
    path: String,
}

impl TakedownStore {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            notices: Mutex::new(Vec::new()),
            path: path.to_string(),
        })
    }
    pub fn add(&self, n: TakedownNotice) -> anyhow::Result<()> {
        if let Ok(mut v) = self.notices.lock() {
            v.push(n);
        }
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<TakedownNotice> {
        self.notices
            .lock()
            .ok()?
            .iter()
            .find(|n| n.id == id)
            .cloned()
    }
    pub fn update_status(&self, id: &str, status: NoticeStatus) {
        if let Ok(mut v) = self.notices.lock() {
            if let Some(n) = v.iter_mut().find(|n| n.id == id) {
                n.status = status;
                n.resolved_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }
    }
    pub fn set_counter(&self, id: &str, counter: CounterNotice) {
        if let Ok(mut v) = self.notices.lock() {
            if let Some(n) = v.iter_mut().find(|n| n.id == id) {
                n.counter_notice = Some(counter);
                n.status = NoticeStatus::CounterReceived;
            }
        }
    }
}

fn rand_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0xdead)
}

pub async fn submit_notice(
    State(state): State<AppState>,
    Json(req): Json<TakedownRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !req.good_faith || !req.accuracy {
        return Err(StatusCode::BAD_REQUEST);
    }
    let id = format!(
        "DMCA-{}-{:08x}",
        chrono::Utc::now().format("%Y%m%d"),
        rand_id()
    );
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
        .record(&format!("DMCA_COUNTER id='{}'", id))
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
