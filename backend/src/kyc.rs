//! KYC/AML — FinCEN, OFAC SDN screening, W-9/W-8BEN, EU AMLD6.
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{info, warn};

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
    records: Mutex<HashMap<String, KycRecord>>,
    path: String,
}

impl KycStore {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            records: Mutex::new(HashMap::new()),
            path: path.to_string(),
        })
    }
    pub fn get(&self, uid: &str) -> Option<KycRecord> {
        self.records.lock().ok()?.get(uid).cloned()
    }
    pub fn upsert(&self, r: KycRecord) {
        if let Ok(mut m) = self.records.lock() {
            m.insert(r.user_id.clone(), r);
        }
    }
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

pub async fn submit_kyc(
    State(state): State<AppState>,
    Path(uid): Path<String>,
    Json(req): Json<KycSubmission>,
) -> Result<Json<serde_json::Value>, StatusCode> {
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
            "KYC_SUBMIT user='{}' tier={:?} ofac={:?}",
            uid, tier, ofac
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

pub async fn kyc_status(
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> Result<Json<KycRecord>, StatusCode> {
    state
        .kyc_db
        .get(&uid)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
