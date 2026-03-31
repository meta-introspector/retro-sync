//! KYC/AML — FinCEN, OFAC SDN screening, W-9/W-8BEN, EU AMLD6.
//!
//! Persistence: LMDB via persist::LmdbStore.
//! Per-user auth: callers may only read/write their own KYC record.
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KycTier {
    Tier0Unverified,
    Tier1Basic,
    Tier2Full,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaxForm {
    W9,
    W8Ben,
    W8BenE,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OfacStatus {
    Clear,
    PendingScreening,
    Flagged,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycRecord {
    pub user_id: String,
    pub tier: KycTier,
    pub legal_name: Option<String>,
    pub country_code: Option<String>,
    pub id_type: Option<String>,
    pub tax_form: Option<TaxForm>,
    pub tin_hash: Option<String>,
    pub ofac_status: OfacStatus,
    pub created_at: String,
    pub updated_at: String,
    pub payout_blocked: bool,
}

#[derive(Deserialize)]
pub struct KycSubmission {
    pub legal_name: String,
    pub country_code: String,
    pub id_type: String,
    pub tax_form: TaxForm,
    pub tin_hash: Option<String>,
}

pub struct KycStore {
    db: crate::persist::LmdbStore,
}

impl KycStore {
    #[zkperf_macros::zkperf]
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            db: crate::persist::LmdbStore::open(path, "kyc_records")?,
        })
    }

    #[zkperf_macros::zkperf]
    pub fn get(&self, uid: &str) -> Option<KycRecord> {
        self.db.get(uid).ok().flatten()
    }

    #[zkperf_macros::zkperf]
    pub fn upsert(&self, r: KycRecord) {
        if let Err(e) = self.db.put(&r.user_id, &r) {
            tracing::error!(err=%e, user=%r.user_id, "KYC persist error");
        }
    }

    #[zkperf_macros::zkperf]
    pub fn payout_permitted(&self, uid: &str, amount_usd: f64) -> bool {
        match self.get(uid) {
            None => false,
            Some(r) => {
                if r.payout_blocked {
                    return false;
                }
                if r.ofac_status != OfacStatus::Clear {
                    return false;
                }
                if amount_usd > 3000.0 && r.tier != KycTier::Tier2Full {
                    return false;
                }
                r.tier != KycTier::Tier0Unverified
            }
        }
    }
}

// OFAC sanctioned countries (comprehensive programs, 2025)
const SANCTIONED: &[&str] = &["CU", "IR", "KP", "RU", "SY", "VE"];

async fn screen_ofac(name: &str, country: &str) -> OfacStatus {
    if SANCTIONED.contains(&country) {
        warn!(name=%name, country=%country, "OFAC: sanctioned country");
        return OfacStatus::Flagged;
    }
    // Production: call Refinitiv/ComplyAdvantage/LexisNexis SDN API
    OfacStatus::Clear
}

#[zkperf_macros::zkperf]
pub async fn submit_kyc(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(uid): Path<String>,
    Json(req): Json<KycSubmission>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // PER-USER AUTH: caller must own this uid
    let caller = crate::auth::extract_caller(&headers)?;
    if !caller.eq_ignore_ascii_case(&uid) {
        warn!(caller=%caller, uid=%uid, "KYC submit: caller != uid — forbidden");
        return Err(StatusCode::FORBIDDEN);
    }

    let ofac = screen_ofac(&req.legal_name, &req.country_code).await;
    let blocked = ofac == OfacStatus::Flagged || ofac == OfacStatus::Blocked;
    let tier = if blocked {
        KycTier::Suspended
    } else {
        KycTier::Tier1Basic
    };
    let now = chrono::Utc::now().to_rfc3339();
    state.kyc_db.upsert(KycRecord {
        user_id: uid.clone(),
        tier: tier.clone(),
        legal_name: Some(req.legal_name.clone()),
        country_code: Some(req.country_code.clone()),
        id_type: Some(req.id_type),
        tax_form: Some(req.tax_form),
        tin_hash: req.tin_hash,
        ofac_status: ofac.clone(),
        created_at: now.clone(),
        updated_at: now,
        payout_blocked: blocked,
    });
    state
        .audit_log
        .record(&format!(
            "KYC_SUBMIT user='{uid}' tier={tier:?} ofac={ofac:?}"
        ))
        .ok();
    if blocked {
        warn!(user=%uid, "KYC: payout blocked — OFAC flag");
    }
    Ok(Json(serde_json::json!({
        "user_id": uid, "tier": format!("{:?}", tier),
        "ofac_status": format!("{:?}", ofac), "payout_blocked": blocked,
    })))
}

#[zkperf_macros::zkperf]
pub async fn kyc_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(uid): Path<String>,
) -> Result<Json<KycRecord>, StatusCode> {
    // PER-USER AUTH: caller may only read their own record
    let caller = crate::auth::extract_caller(&headers)?;
    if !caller.eq_ignore_ascii_case(&uid) {
        warn!(caller=%caller, uid=%uid, "KYC status: caller != uid — forbidden");
        return Err(StatusCode::FORBIDDEN);
    }

    state
        .kyc_db
        .get(&uid)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}