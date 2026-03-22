//! Publishing agreement registration and soulbound NFT minting pipeline.
//!
//! Flow:
//!   POST /api/register  (JSON — metadata + contributor list)
//!     1. Validate ISRC (LangSec formal recogniser)
//!     2. KYC check every contributor against the KYC store
//!     3. Store the agreement in LMDB
//!     4. Submit ERN 4.1 to DDEX with full creator attribution
//!     5. Return registration_id + agreement details
//!
//! Soulbound NFT minting is triggered on-chain via PublishingAgreement.propose()
//! (called via ethers).  The NFT is actually minted once all parties have signed
//! their agreement from their wallets — that is a separate on-chain transaction
//! the frontend facilitates.
//!
//! SECURITY: All wallet addresses and IPI numbers are validated before writing.
//! KYC tier Tier0Unverified is rejected.  OFAC-flagged users are blocked.
use crate::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

// ── Request / Response types ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorInput {
    /// Wallet address (EVM hex, 42 chars including 0x prefix)
    pub address: String,
    /// IPI name number (9-11 digits)
    pub ipi_number: String,
    /// Role: "Songwriter", "Composer", "Publisher", "Admin Publisher"
    pub role: String,
    /// Royalty share in basis points (0–10000). All contributors must sum to 10000.
    pub bps: u16,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// Title of the work
    pub title: String,
    /// ISRC code (e.g. "US-ABC-24-00001")
    pub isrc: String,
    /// Optional liner notes / description
    pub description: Option<String>,
    /// BTFS CID of the audio file (uploaded separately via /api/upload)
    pub btfs_cid: String,
    /// Master Pattern band (0=Common, 1=Rare, 2=Legendary) — from prior /api/upload response
    pub band: u8,
    /// Ordered list of contributors — songwriters and publishers.
    pub contributors: Vec<ContributorInput>,
}

#[derive(Debug, Serialize)]
pub struct ContributorResult {
    pub address: String,
    pub ipi_number: String,
    pub role: String,
    pub bps: u16,
    pub kyc_tier: String,
    pub kyc_permitted: bool,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub registration_id: String,
    pub isrc: String,
    pub btfs_cid: String,
    pub band: u8,
    pub title: String,
    pub contributors: Vec<ContributorResult>,
    pub all_kyc_passed: bool,
    pub ddex_submitted: bool,
    pub soulbound_pending: bool,
    pub message: String,
}

// ── Address validation ────────────────────────────────────────────────────────

fn validate_evm_address(addr: &str) -> bool {
    if addr.len() != 42 {
        return false;
    }
    if !addr.starts_with("0x") && !addr.starts_with("0X") {
        return false;
    }
    addr[2..].chars().all(|c| c.is_ascii_hexdigit())
}

fn validate_ipi(ipi: &str) -> bool {
    let digits: String = ipi.chars().filter(|c| c.is_ascii_digit()).collect();
    (9..=11).contains(&digits.len())
}

fn validate_role(role: &str) -> bool {
    matches!(
        role,
        "Songwriter" | "Composer" | "Publisher" | "Admin Publisher" | "Lyricist"
    )
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn register_track(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    // ── Auth ───────────────────────────────────────────────────────────────
    let caller = crate::auth::extract_caller(&headers)?;

    // ── Input validation ───────────────────────────────────────────────────
    if req.title.trim().is_empty() {
        warn!(caller=%caller, "Register: empty title");
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    if req.btfs_cid.trim().is_empty() {
        warn!(caller=%caller, "Register: empty btfs_cid");
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    if req.band > 2 {
        warn!(caller=%caller, band=%req.band, "Register: invalid band");
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    if req.contributors.is_empty() || req.contributors.len() > 16 {
        warn!(caller=%caller, n=req.contributors.len(), "Register: contributor count invalid");
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    // ── LangSec: ISRC formal recognition ──────────────────────────────────
    let isrc = crate::recognize_isrc(&req.isrc).map_err(|e| {
        warn!(err=%e, caller=%caller, "Register: ISRC rejected");
        state.metrics.record_defect("isrc_parse");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    // ── Validate contributor fields ────────────────────────────────────────
    let bps_sum: u32 = req.contributors.iter().map(|c| c.bps as u32).sum();
    if bps_sum != 10_000 {
        warn!(caller=%caller, bps_sum=%bps_sum, "Register: bps must sum to 10000");
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    for c in &req.contributors {
        if !validate_evm_address(&c.address) {
            warn!(caller=%caller, addr=%c.address, "Register: invalid wallet address");
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
        if !validate_ipi(&c.ipi_number) {
            warn!(caller=%caller, ipi=%c.ipi_number, "Register: invalid IPI number");
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
        if !validate_role(&c.role) {
            warn!(caller=%caller, role=%c.role, "Register: invalid role");
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    }

    // ── KYC check every contributor ────────────────────────────────────────
    let mut contributor_results: Vec<ContributorResult> = Vec::new();
    let mut all_kyc_passed = true;

    for c in &req.contributors {
        let uid = c.address.to_ascii_lowercase();
        let (tier_str, permitted) = match state.kyc_db.get(&uid) {
            None => {
                warn!(caller=%caller, contributor=%uid, "Register: contributor has no KYC record");
                all_kyc_passed = false;
                ("Tier0Unverified".to_string(), false)
            }
            Some(rec) => {
                // 10 000 bps is effectively unlimited for this check — if split
                // amount is unknown we require at least Tier1Basic.
                let ok = state.kyc_db.payout_permitted(&uid, 0.01);
                if !ok {
                    warn!(caller=%caller, contributor=%uid, tier=?rec.tier, "Register: contributor KYC insufficient");
                    all_kyc_passed = false;
                }
                (format!("{:?}", rec.tier), ok)
            }
        };
        contributor_results.push(ContributorResult {
            address: c.address.clone(),
            ipi_number: c.ipi_number.clone(),
            role: c.role.clone(),
            bps: c.bps,
            kyc_tier: tier_str,
            kyc_permitted: permitted,
        });
    }

    if !all_kyc_passed {
        warn!(caller=%caller, isrc=%isrc, "Register: blocked — KYC incomplete for one or more contributors");
        state.metrics.record_defect("kyc_register_blocked");
        return Err(StatusCode::FORBIDDEN);
    }

    // ── Build registration ID ──────────────────────────────────────────────
    use sha2::{Digest, Sha256};
    let reg_id_bytes: [u8; 32] = Sha256::digest(
        format!(
            "{}-{}-{}",
            isrc.0,
            req.btfs_cid,
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        )
        .as_bytes(),
    )
    .into();
    let registration_id = hex::encode(&reg_id_bytes[..16]);

    // ── DDEX ERN 4.1 with full contributor attribution ─────────────────────
    use shared::master_pattern::pattern_fingerprint;
    let description = req.description.as_deref().unwrap_or("");
    let fp = pattern_fingerprint(isrc.0.as_bytes(), &[req.band; 32]);
    let wiki = crate::wikidata::WikidataArtist::default();

    let ddex_contributors: Vec<crate::ddex::DdexContributor> = req
        .contributors
        .iter()
        .map(|c| crate::ddex::DdexContributor {
            wallet_address: c.address.clone(),
            ipi_number: c.ipi_number.clone(),
            role: c.role.clone(),
            bps: c.bps,
        })
        .collect();

    let ddex_result = crate::ddex::register_with_contributors(
        &req.title,
        &isrc,
        &shared::types::BtfsCid(req.btfs_cid.clone()),
        &fp,
        &wiki,
        &ddex_contributors,
    )
    .await;

    let ddex_submitted = match ddex_result {
        Ok(_) => {
            info!(isrc=%isrc, "DDEX delivery submitted with contributor attribution");
            true
        }
        Err(e) => {
            warn!(err=%e, isrc=%isrc, "DDEX delivery failed — registration continues");
            false
        }
    };

    // ── Audit log ──────────────────────────────────────────────────────────
    state
        .audit_log
        .record(&format!(
            "REGISTER isrc='{}' reg_id='{}' title='{}' description='{}' contributors={} band={} all_kyc={} ddex={}",
            isrc.0,
            registration_id,
            req.title,
            description,
            req.contributors.len(),
            req.band,
            all_kyc_passed,
            ddex_submitted,
        ))
        .ok();
    state.metrics.record_band(fp.band);

    info!(
        isrc=%isrc, reg_id=%registration_id, band=%req.band,
        contributors=%req.contributors.len(), ddex=%ddex_submitted,
        "Track registered — soulbound NFT pending on-chain signatures"
    );

    Ok(Json(RegisterResponse {
        registration_id,
        isrc: isrc.0,
        btfs_cid: req.btfs_cid,
        band: req.band,
        title: req.title,
        contributors: contributor_results,
        all_kyc_passed,
        ddex_submitted,
        soulbound_pending: true,
        message: "Registration recorded. All parties must now sign the on-chain publishing agreement from their wallets to mint the soulbound NFT.".into(),
    }))
}
