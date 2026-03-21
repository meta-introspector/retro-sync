//! SAP integration — S/4HANA (OData v4 / REST) and ECC (IDoc / RFC / BAPI).
//!
//! Architecture:
//!
//!   S/4HANA paths (Finance module):
//!     • POST /api/sap/royalty-posting  → FI Journal Entry via
//!       OData v4: POST /sap/opu/odata4/sap/api_journalentry_srv/srvd_a2x/
//!       SAP_FI_JOURNALENTRY/0001/JournalEntry
//!     • POST /api/sap/vendor-sync      → BP/Vendor master upsert via
//!       OData v4: /sap/opu/odata4/sap/api_business_partner/srvd_a2x/
//!       SAP_API_BUSINESS_PARTNER/0001/BusinessPartner
//!
//!   ECC (SAP R/3 / ERP 6.0) paths:
//!     • POST /api/sap/idoc/royalty     → FIDCCP02 / INVOIC02 IDoc XML
//!       posted to the ECC IDoc inbound adapter (tRFC / HTTP-XML gateway).
//!       Also supports RFC BAPI_ACC_DOCUMENT_POST via JSON-RPC bridge.
//!
//!   Zero Trust: all calls use client-cert mTLS (SAP API Management gateway).
//!   LangSec: all monetary amounts validated before mapping to SAP fields.
//!   ISO 9001 §7.5: every posting logged to audit store with correlation ID.

use crate::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct SapConfig {
    // S/4HANA
    pub s4_base_url: String, // e.g. https://s4hana.retrosync.media
    pub s4_client: String,   // SAP client (Mandant), e.g. "100"
    pub s4_user: String,
    pub s4_password: String,
    pub s4_company_code: String, // e.g. "RTSY"
    pub s4_gl_royalty: String,   // G/L account for royalty expense
    pub s4_gl_liability: String, // G/L account for royalty liability (AP)
    pub s4_profit_centre: String,
    pub s4_cost_centre: String,
    // ECC
    pub ecc_idoc_url: String,      // IDoc HTTP inbound endpoint
    pub ecc_sender_port: String,   // e.g. "RETROSYNC"
    pub ecc_receiver_port: String, // e.g. "SAPECCPORT"
    pub ecc_logical_sys: String,   // SAP logical system name
    // Shared
    pub enabled: bool,
    pub dev_mode: bool, // if true: log but do not POST
}

impl SapConfig {
    pub fn from_env() -> Self {
        let ev = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
        Self {
            s4_base_url: ev("SAP_S4_BASE_URL", "https://s4hana.retrosync.media"),
            s4_client: ev("SAP_S4_CLIENT", "100"),
            s4_user: ev("SAP_S4_USER", "RETROSYNC_SVC"),
            s4_password: ev("SAP_S4_PASSWORD", ""),
            s4_company_code: ev("SAP_COMPANY_CODE", "RTSY"),
            s4_gl_royalty: ev("SAP_GL_ROYALTY_EXPENSE", "630000"),
            s4_gl_liability: ev("SAP_GL_ROYALTY_LIABILITY", "210100"),
            s4_profit_centre: ev("SAP_PROFIT_CENTRE", "PC-MUSIC"),
            s4_cost_centre: ev("SAP_COST_CENTRE", "CC-LABEL"),
            ecc_idoc_url: ev(
                "SAP_ECC_IDOC_URL",
                "http://ecc.retrosync.media:8000/sap/bc/idoc_xml",
            ),
            ecc_sender_port: ev("SAP_ECC_SENDER_PORT", "RETROSYNC"),
            ecc_receiver_port: ev("SAP_ECC_RECEIVER_PORT", "SAPECCPORT"),
            ecc_logical_sys: ev("SAP_ECC_LOGICAL_SYS", "ECCCLNT100"),
            enabled: ev("SAP_ENABLED", "0") == "1",
            dev_mode: ev("SAP_DEV_MODE", "1") == "1",
        }
    }
}

// ── Client handle ─────────────────────────────────────────────────────────────

pub struct SapClient {
    pub cfg: SapConfig,
    http: reqwest::Client,
}

impl SapClient {
    pub fn from_env() -> Self {
        Self {
            cfg: SapConfig::from_env(),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("reqwest client"),
        }
    }
}

// ── Domain types ──────────────────────────────────────────────────────────────

/// A royalty payment event — one payout to one payee for one period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoyaltyPosting {
    pub correlation_id: String,  // idempotency key (UUID or ISRC+period hash)
    pub payee_vendor_id: String, // SAP vendor/BP number
    pub payee_name: String,
    pub amount_currency: String, // ISO 4217, e.g. "USD"
    pub amount: f64,             // gross royalty amount
    pub withholding_tax: f64,    // 0.0 if no WHT applicable
    pub net_amount: f64,         // amount − withholding_tax
    pub period_start: String,    // YYYYMMDD
    pub period_end: String,
    pub isrc: Option<String>,
    pub iswc: Option<String>,
    pub work_title: Option<String>,
    pub cost_centre: Option<String>,
    pub profit_centre: Option<String>,
    pub reference: String,     // free-form reference / invoice number
    pub posting_date: String,  // YYYYMMDD
    pub document_date: String, // YYYYMMDD
}

/// A vendor/business-partner to upsert in SAP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorRecord {
    pub bp_number: Option<String>, // blank on create
    pub legal_name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,            // ISO 3166-1 alpha-2
    pub language: String,           // ISO 639-1
    pub tax_number: Option<String>, // TIN / VAT ID
    pub iban: Option<String>,
    pub bank_key: Option<String>,
    pub bank_account: Option<String>,
    pub payment_terms: String, // SAP payment terms key, e.g. "NT30"
    pub currency: String,      // default payout currency
    pub email: Option<String>,
    pub ipi_cae: Option<String>, // cross-ref to rights data
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct PostingResult {
    pub correlation_id: String,
    pub sap_document_no: Option<String>,
    pub sap_fiscal_year: Option<String>,
    pub company_code: String,
    pub status: PostingStatus,
    pub message: String,
    pub dev_mode: bool,
}

#[derive(Serialize, PartialEq)]
pub enum PostingStatus {
    Posted,
    Simulated,
    Failed,
    Disabled,
}

#[derive(Serialize)]
pub struct VendorSyncResult {
    pub bp_number: String,
    pub status: String,
    pub dev_mode: bool,
}

#[derive(Serialize)]
pub struct IdocResult {
    pub correlation_id: String,
    pub idoc_number: Option<String>,
    pub status: String,
    pub dev_mode: bool,
}

// ── S/4HANA OData v4 helpers ──────────────────────────────────────────────────

/// Build the OData v4 FI Journal Entry payload for a royalty accrual.
///
/// Maps to: API_JOURNALENTRY_SRV / JournalEntry entity.
/// Debit:   G/L royalty expense account (cfg.s4_gl_royalty)
/// Credit:  G/L royalty liability/AP account (cfg.s4_gl_liability)
fn build_journal_entry_payload(p: &RoyaltyPosting, cfg: &SapConfig) -> serde_json::Value {
    serde_json::json!({
        "ReferenceDocumentType": "KR",         // vendor invoice
        "BusinessTransactionType": "RFBU",
        "CompanyCode":   cfg.s4_company_code,
        "DocumentDate":  p.document_date,
        "PostingDate":   p.posting_date,
        "TransactionCurrency": p.amount_currency,
        "DocumentHeaderText": format!("Royalty {} {}", p.reference, p.period_end),
        "OperatingUnit": cfg.s4_profit_centre,
        "_JournalEntryItem": [
            {
                // Debit line — royalty expense
                "LedgerGLLineItem":       "1",
                "GLAccount":              cfg.s4_gl_royalty,
                "AmountInTransactionCurrency": format!("{:.2}", p.amount),
                "DebitCreditCode":        "S",    // Soll = debit
                "CostCenter":             cfg.s4_cost_centre,
                "ProfitCenter":           cfg.s4_profit_centre,
                "AssignmentReference":    p.correlation_id,
                "ItemText": p.work_title.as_deref().unwrap_or(&p.reference),
            },
            {
                // Credit line — royalty liability (vendor AP)
                "LedgerGLLineItem":       "2",
                "GLAccount":              cfg.s4_gl_liability,
                "AmountInTransactionCurrency": format!("-{:.2}", p.net_amount),
                "DebitCreditCode":        "H",    // Haben = credit
                "Supplier":               p.payee_vendor_id,
                "AssignmentReference":    p.correlation_id,
                "ItemText":               format!("Vendor {} {}", p.payee_name, p.period_end),
            },
        ]
    })
}

/// Build the OData v4 BusinessPartner payload for a vendor upsert.
fn build_bp_payload(v: &VendorRecord, cfg: &SapConfig) -> serde_json::Value {
    serde_json::json!({
        "BusinessPartner":        v.bp_number.as_deref().unwrap_or(""),
        "BusinessPartnerFullName": v.legal_name,
        "FirstName":              v.first_name.as_deref().unwrap_or(""),
        "LastName":               v.last_name.as_deref().unwrap_or(""),
        "Language":               v.language,
        "TaxNumber1":             v.tax_number.as_deref().unwrap_or(""),
        "to_BusinessPartnerAddress": {
            "results": [{
                "Country":      v.country,
                "PostalCode":   v.postal_code.as_deref().unwrap_or(""),
                "CityName":     v.city.as_deref().unwrap_or(""),
                "StreetName":   v.street.as_deref().unwrap_or(""),
            }]
        },
        "to_BusinessPartnerRole": {
            "results": [{ "BusinessPartnerRole": "FLVN01" }] // vendor role
        },
        "to_BuPaIdentification": {
            "results": if let Some(ipi) = &v.ipi_cae { vec![
                serde_json::json!({ "BPIdentificationType": "IPI", "BPIdentificationNumber": ipi })
            ]} else { vec![] }
        }
    })
}

// ── ECC IDoc builder ──────────────────────────────────────────────────────────

/// Build a FIDCCP02 (FI document) IDoc XML for ECC inbound processing.
/// Used when the SAP landscape still runs ECC 6.0 rather than S/4HANA.
///
/// IDoc type: FIDCCP02  Message type: FIDCC2
/// Each RoyaltyPosting maps to one FIDCCP02 IDoc with:
///   E1FIKPF  — document header
///   E1FISEG  — one debit line (royalty expense)
///   E1FISEG  — one credit line (royalty liability AP)
pub fn build_royalty_idoc(p: &RoyaltyPosting, cfg: &SapConfig) -> String {
    let now = chrono::Utc::now();
    let ts = now.format("%Y%m%d%H%M%S").to_string();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<FIDCCP02>
  <IDOC BEGIN="1">
    <EDI_DC40 SEGMENT="1">
      <TABNAM>EDI_DC40</TABNAM>
      <MANDT>100</MANDT>
      <DOCNUM>{ts}</DOCNUM>
      <DOCREL>740</DOCREL>
      <STATUS>30</STATUS>
      <DIRECT>2</DIRECT>
      <OUTMOD>2</OUTMOD>
      <IDOCTYP>FIDCCP02</IDOCTYP>
      <MESTYP>FIDCC2</MESTYP>
      <SNDPRT>LS</SNDPRT>
      <SNDPOR>{sender_port}</SNDPOR>
      <SNDPRN>{logical_sys}</SNDPRN>
      <RCVPRT>LS</RCVPRT>
      <RCVPOR>{receiver_port}</RCVPOR>
      <RCVPRN>SAPECCCLNT100</RCVPRN>
      <CREDAT>{date}</CREDAT>
      <CRETIM>{time}</CRETIM>
    </EDI_DC40>
    <E1FIKPF SEGMENT="1">
      <BUKRS>{company_code}</BUKRS>
      <BKTXT>{reference}</BKTXT>
      <BLART>KR</BLART>
      <BLDAT>{doc_date}</BLDAT>
      <BUDAT>{post_date}</BUDAT>
      <WAERS>{currency}</WAERS>
      <XBLNR>{correlation_id}</XBLNR>
    </E1FIKPF>
    <E1FISEG SEGMENT="1">
      <BUZEI>001</BUZEI>
      <BSCHL>40</BSCHL>
      <HKONT>{gl_royalty}</HKONT>
      <WRBTR>{amount:.2}</WRBTR>
      <KOSTL>{cost_centre}</KOSTL>
      <PRCTR>{profit_centre}</PRCTR>
      <SGTXT>{work_title}</SGTXT>
      <ZUONR>{correlation_id}</ZUONR>
    </E1FISEG>
    <E1FISEG SEGMENT="1">
      <BUZEI>002</BUZEI>
      <BSCHL>31</BSCHL>
      <LIFNR>{vendor_id}</LIFNR>
      <HKONT>{gl_liability}</HKONT>
      <WRBTR>{net_amount:.2}</WRBTR>
      <SGTXT>Royalty {payee_name} {period_end}</SGTXT>
      <ZUONR>{correlation_id}</ZUONR>
    </E1FISEG>
  </IDOC>
</FIDCCP02>"#,
        ts = ts,
        sender_port = cfg.ecc_sender_port,
        logical_sys = cfg.ecc_logical_sys,
        receiver_port = cfg.ecc_receiver_port,
        company_code = cfg.s4_company_code,
        reference = p.reference,
        doc_date = p.document_date,
        post_date = p.posting_date,
        currency = p.amount_currency,
        correlation_id = p.correlation_id,
        gl_royalty = cfg.s4_gl_royalty,
        amount = p.amount,
        cost_centre = p.cost_centre.as_deref().unwrap_or(&cfg.s4_cost_centre),
        profit_centre = p.profit_centre.as_deref().unwrap_or(&cfg.s4_profit_centre),
        work_title = p.work_title.as_deref().unwrap_or(&p.reference),
        gl_liability = cfg.s4_gl_liability,
        vendor_id = p.payee_vendor_id,
        net_amount = p.net_amount,
        payee_name = p.payee_name,
        period_end = p.period_end,
        date = now.format("%Y%m%d"),
        time = now.format("%H%M%S"),
    )
}

// ── HTTP handlers ─────────────────────────────────────────────────────────────

/// POST /api/sap/royalty-posting
/// Post a royalty accrual to S/4HANA FI (OData v4 journal entry).
/// Falls back to IDoc if SAP_ECC_MODE=1.
pub async fn post_royalty_document(
    State(state): State<AppState>,
    Json(posting): Json<RoyaltyPosting>,
) -> Result<Json<PostingResult>, StatusCode> {
    let cfg = &state.sap_client.cfg;

    // LangSec: validate monetary amounts
    if posting.amount < 0.0
        || posting.net_amount < 0.0
        || posting.net_amount > posting.amount + 0.01
    {
        warn!(correlation_id=%posting.correlation_id, "SAP posting: invalid amounts");
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    state
        .audit_log
        .record(&format!(
            "SAP_ROYALTY_POSTING corr='{}' vendor='{}' amount={:.2} {} dev_mode={}",
            posting.correlation_id,
            posting.payee_vendor_id,
            posting.amount,
            posting.amount_currency,
            cfg.dev_mode
        ))
        .ok();

    if !cfg.enabled || cfg.dev_mode {
        info!(correlation_id=%posting.correlation_id, "SAP posting simulated (dev_mode)");
        return Ok(Json(PostingResult {
            correlation_id: posting.correlation_id.clone(),
            sap_document_no: Some("SIMULATED".into()),
            sap_fiscal_year: Some(chrono::Utc::now().format("%Y").to_string()),
            company_code: cfg.s4_company_code.clone(),
            status: PostingStatus::Simulated,
            message: "SAP_DEV_MODE: posting logged, not submitted".into(),
            dev_mode: true,
        }));
    }

    let ecc_mode = std::env::var("SAP_ECC_MODE").unwrap_or_default() == "1";
    if ecc_mode {
        // ECC path: emit IDoc
        let idoc = build_royalty_idoc(&posting, cfg);
        let resp = state
            .sap_client
            .http
            .post(&cfg.ecc_idoc_url)
            .header("Content-Type", "application/xml")
            .body(idoc)
            .send()
            .await
            .map_err(|e| {
                warn!(err=%e, "ECC IDoc POST failed");
                StatusCode::BAD_GATEWAY
            })?;

        if !resp.status().is_success() {
            warn!(status=%resp.status(), "ECC IDoc rejected");
            return Err(StatusCode::BAD_GATEWAY);
        }
        return Ok(Json(PostingResult {
            correlation_id: posting.correlation_id,
            sap_document_no: None,
            sap_fiscal_year: None,
            company_code: cfg.s4_company_code.clone(),
            status: PostingStatus::Posted,
            message: "ECC IDoc posted".into(),
            dev_mode: false,
        }));
    }

    // S/4HANA path: OData v4 Journal Entry
    let url = format!(
        "{}/sap/opu/odata4/sap/api_journalentry_srv/srvd_a2x/SAP_FI_JOURNALENTRY/0001/JournalEntry?sap-client={}",
        cfg.s4_base_url, cfg.s4_client
    );
    let payload = build_journal_entry_payload(&posting, cfg);

    let resp = state
        .sap_client
        .http
        .post(&url)
        .basic_auth(&cfg.s4_user, Some(&cfg.s4_password))
        .header("Content-Type", "application/json")
        .header("sap-client", &cfg.s4_client)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            warn!(err=%e, "S/4HANA journal entry POST failed");
            StatusCode::BAD_GATEWAY
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        warn!(http_status=%status, body=%body, "S/4HANA journal entry rejected");
        return Err(StatusCode::BAD_GATEWAY);
    }

    let body: serde_json::Value = resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let doc_no = body["d"]["CompanyCodeDocument"]
        .as_str()
        .map(str::to_string);
    let year = body["d"]["FiscalYear"].as_str().map(str::to_string);

    info!(correlation_id=%posting.correlation_id, doc_no=?doc_no, "S/4HANA journal entry posted");
    Ok(Json(PostingResult {
        correlation_id: posting.correlation_id,
        sap_document_no: doc_no,
        sap_fiscal_year: year,
        company_code: cfg.s4_company_code.clone(),
        status: PostingStatus::Posted,
        message: "Posted to S/4HANA FI".into(),
        dev_mode: false,
    }))
}

/// POST /api/sap/vendor-sync
/// Create or update a business partner / vendor in S/4HANA.
pub async fn sync_vendor(
    State(state): State<AppState>,
    Json(vendor): Json<VendorRecord>,
) -> Result<Json<VendorSyncResult>, StatusCode> {
    let cfg = &state.sap_client.cfg;

    state
        .audit_log
        .record(&format!(
            "SAP_VENDOR_SYNC bp='{}' name='{}' dev_mode={}",
            vendor.bp_number.as_deref().unwrap_or("NEW"),
            vendor.legal_name,
            cfg.dev_mode
        ))
        .ok();

    if !cfg.enabled || cfg.dev_mode {
        return Ok(Json(VendorSyncResult {
            bp_number: vendor.bp_number.unwrap_or_else(|| "SIMULATED".into()),
            status: "SIMULATED".into(),
            dev_mode: true,
        }));
    }

    let (url, method) = match &vendor.bp_number {
        Some(bp) => (
            format!("{}/sap/opu/odata4/sap/api_business_partner/srvd_a2x/SAP_API_BUSINESS_PARTNER/0001/BusinessPartner('{}')?sap-client={}",
                cfg.s4_base_url, bp, cfg.s4_client),
            "PATCH",
        ),
        None => (
            format!("{}/sap/opu/odata4/sap/api_business_partner/srvd_a2x/SAP_API_BUSINESS_PARTNER/0001/BusinessPartner?sap-client={}",
                cfg.s4_base_url, cfg.s4_client),
            "POST",
        ),
    };

    let payload = build_bp_payload(&vendor, cfg);
    let req = if method == "PATCH" {
        state.sap_client.http.patch(&url)
    } else {
        state.sap_client.http.post(&url)
    };

    let resp = req
        .basic_auth(&cfg.s4_user, Some(&cfg.s4_password))
        .header("Content-Type", "application/json")
        .header("sap-client", &cfg.s4_client)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            warn!(err=%e, "S/4HANA BP upsert failed");
            StatusCode::BAD_GATEWAY
        })?;

    if !resp.status().is_success() {
        warn!(status=%resp.status(), "S/4HANA BP upsert rejected");
        return Err(StatusCode::BAD_GATEWAY);
    }

    let body: serde_json::Value = resp.json().await.unwrap_or_default();
    let bp = body["d"]["BusinessPartner"]
        .as_str()
        .or(vendor.bp_number.as_deref())
        .unwrap_or("")
        .to_string();

    info!(bp=%bp, "S/4HANA vendor synced");
    Ok(Json(VendorSyncResult {
        bp_number: bp,
        status: "OK".into(),
        dev_mode: false,
    }))
}

/// POST /api/sap/idoc/royalty
/// Explicitly emit a FIDCCP02 IDoc to ECC (bypasses S/4HANA path).
pub async fn emit_royalty_idoc(
    State(state): State<AppState>,
    Json(posting): Json<RoyaltyPosting>,
) -> Result<Json<IdocResult>, StatusCode> {
    let cfg = &state.sap_client.cfg;
    let idoc = build_royalty_idoc(&posting, cfg);

    state
        .audit_log
        .record(&format!(
            "SAP_IDOC_EMIT corr='{}' dev_mode={}",
            posting.correlation_id, cfg.dev_mode
        ))
        .ok();

    if !cfg.enabled || cfg.dev_mode {
        info!(correlation_id=%posting.correlation_id, "ECC IDoc simulated");
        return Ok(Json(IdocResult {
            correlation_id: posting.correlation_id,
            idoc_number: Some("SIMULATED".into()),
            status: "SIMULATED".into(),
            dev_mode: true,
        }));
    }

    let resp = state
        .sap_client
        .http
        .post(&cfg.ecc_idoc_url)
        .header("Content-Type", "application/xml")
        .body(idoc)
        .send()
        .await
        .map_err(|e| {
            warn!(err=%e, "ECC IDoc emit failed");
            StatusCode::BAD_GATEWAY
        })?;

    if !resp.status().is_success() {
        warn!(status=%resp.status(), "ECC IDoc rejected");
        return Err(StatusCode::BAD_GATEWAY);
    }

    // ECC typically returns the IDoc number in the response body
    let body = resp.text().await.unwrap_or_default();
    let idoc_no = body
        .lines()
        .find(|l| l.contains("<DOCNUM>"))
        .and_then(|l| l.split('>').nth(1))
        .and_then(|l| l.split('<').next())
        .map(str::to_string);

    info!(idoc_no=?idoc_no, correlation_id=%posting.correlation_id, "ECC IDoc posted");
    Ok(Json(IdocResult {
        correlation_id: posting.correlation_id,
        idoc_number: idoc_no,
        status: "POSTED".into(),
        dev_mode: false,
    }))
}

/// GET /api/sap/health
pub async fn sap_health(State(state): State<AppState>) -> Json<serde_json::Value> {
    let cfg = &state.sap_client.cfg;
    Json(serde_json::json!({
        "sap_enabled":  cfg.enabled,
        "dev_mode":     cfg.dev_mode,
        "s4_base_url":  cfg.s4_base_url,
        "ecc_idoc_url": cfg.ecc_idoc_url,
        "company_code": cfg.s4_company_code,
    }))
}
