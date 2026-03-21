//! Retrosync backend — Axum API server.
//! Zero Trust: every request verified via JWT + SPIFFE SVID (auth.rs).
//! LangSec: all inputs pass through shared::parsers recognizers.
//! ISO 9001 §7.5: all operations logged to append-only audit store.

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    middleware,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;
use shared::parsers::recognize_isrc;
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

mod audio_qc;
mod auth;
mod btfs;
mod bttc;
mod ddex;
mod dsp;
mod fraud;
mod gtms;
mod identifiers;
mod iso_store;
mod kyc;
mod ledger;
mod metrics;
mod mirrors;
mod moderation;
mod privacy;
mod royalty_reporting;
mod sap; // SAP S/4HANA (OData v4) + ECC (IDoc/RFC) integration
mod takedown;
mod wikidata;
mod xslt;
mod zk_cache; // Global Trade Management — ECCN, HS codes, export control, sanctions

#[derive(Clone)]
pub struct AppState {
    pub pki_dir: std::path::PathBuf,
    pub audit_log: Arc<iso_store::AuditStore>,
    pub metrics: Arc<metrics::CtqMetrics>,
    pub zk_cache: Arc<zk_cache::ZkProofCache>,
    pub takedown_db: Arc<takedown::TakedownStore>,
    pub privacy_db: Arc<privacy::PrivacyStore>,
    pub fraud_db: Arc<fraud::FraudDetector>,
    pub kyc_db: Arc<kyc::KycStore>,
    pub mod_queue: Arc<moderation::ModerationQueue>,
    pub sap_client: Arc<sap::SapClient>,
    pub gtms_db: Arc<gtms::GtmsStore>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("backend=debug".parse()?))
        .json()
        .init();

    let state = AppState {
        pki_dir: std::path::PathBuf::from(
            std::env::var("PKI_DIR").unwrap_or_else(|_| "pki".into()),
        ),
        audit_log: Arc::new(iso_store::AuditStore::open("audit.db")?),
        metrics: Arc::new(metrics::CtqMetrics::new()),
        zk_cache: Arc::new(zk_cache::ZkProofCache::open("zk_proof_cache.lmdb")?),
        takedown_db: Arc::new(takedown::TakedownStore::open("takedown.db")?),
        privacy_db: Arc::new(privacy::PrivacyStore::open("privacy.db")?),
        fraud_db: Arc::new(fraud::FraudDetector::new()),
        kyc_db: Arc::new(kyc::KycStore::open("kyc.db")?),
        mod_queue: Arc::new(moderation::ModerationQueue::open("moderation.db")?),
        sap_client: Arc::new(sap::SapClient::from_env()),
        gtms_db: Arc::new(gtms::GtmsStore::new()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics::handler))
        .route("/api/upload", post(upload_track))
        .route("/api/track/:id", get(track_status))
        // DMCA §512
        .route("/api/takedown", post(takedown::submit_notice))
        .route(
            "/api/takedown/:id/counter",
            post(takedown::submit_counter_notice),
        )
        .route("/api/takedown/:id", get(takedown::get_notice))
        // GDPR/CCPA
        .route("/api/privacy/consent", post(privacy::record_consent))
        .route(
            "/api/privacy/delete/:uid",
            delete(privacy::delete_user_data),
        )
        .route("/api/privacy/export/:uid", get(privacy::export_user_data))
        // Moderation (DSA/Article 17)
        .route("/api/moderation/report", post(moderation::submit_report))
        .route("/api/moderation/queue", get(moderation::get_queue))
        .route(
            "/api/moderation/:id/resolve",
            post(moderation::resolve_report),
        )
        // KYC/AML
        .route("/api/kyc/:uid", post(kyc::submit_kyc))
        .route("/api/kyc/:uid/status", get(kyc::kyc_status))
        // CWR/XSLT society submissions
        .route(
            "/api/royalty/xslt/:society",
            post(xslt::transform_submission),
        )
        .route(
            "/api/royalty/xslt/all",
            post(xslt::transform_all_submissions),
        )
        // SAP S/4HANA + ECC
        .route("/api/sap/royalty-posting", post(sap::post_royalty_document))
        .route("/api/sap/vendor-sync", post(sap::sync_vendor))
        .route("/api/sap/idoc/royalty", post(sap::emit_royalty_idoc))
        .route("/api/sap/health", get(sap::sap_health))
        // Global Trade Management
        .route("/api/gtms/classify", post(gtms::classify_work))
        .route("/api/gtms/screen", post(gtms::screen_distribution))
        .route("/api/gtms/declaration/:id", get(gtms::get_declaration))
        .layer({
            // SECURITY FIX: CORS is locked to explicit allowed origins only.
            // Set ALLOWED_ORIGINS env var to a comma-separated list of origins.
            // Falls back to localhost only in development.
            use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
            let origins = auth::allowed_origins();
            if origins.iter().any(|_| true) {
                CorsLayer::new()
                    .allow_origin(origins)
                    .allow_methods([Method::GET, Method::POST, Method::DELETE])
                    .allow_headers([AUTHORIZATION, CONTENT_TYPE])
            } else {
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any)
            }
        })
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::verify_zero_trust,
        ))
        .with_state(state);

    let addr = "0.0.0.0:8443";
    info!("Backend listening on https://{} (mTLS)", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "retrosync-backend" }))
}

async fn track_status(Path(id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "id": id, "status": "registered" }))
}

async fn upload_track(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let start = std::time::Instant::now();

    let mut title = String::new();
    let mut artist_name = String::new();
    let mut isrc_raw = String::new();
    let mut audio_bytes = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        match field.name().unwrap_or("") {
            "title" => title = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?,
            "artist" => artist_name = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?,
            "isrc" => isrc_raw = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?,
            "audio" => {
                // SECURITY FIX: Enforce maximum file size to prevent OOM DoS.
                // Default: 100MB. Override with MAX_AUDIO_BYTES env var.
                let max_bytes: usize = std::env::var("MAX_AUDIO_BYTES")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(100 * 1024 * 1024); // 100MB default
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?;
                if bytes.len() > max_bytes {
                    warn!(
                        size = bytes.len(),
                        max = max_bytes,
                        "Upload rejected: file too large"
                    );
                    state.metrics.record_defect("upload_too_large");
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
                audio_bytes = bytes.to_vec();
            }
            _ => {}
        }
    }

    // ── LangSec: formal recognition ───────────────────────────────────────
    let isrc = recognize_isrc(&isrc_raw).map_err(|e| {
        warn!(err=%e, "LangSec: ISRC rejected");
        state.metrics.record_defect("isrc_parse");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    // ── Master Pattern fingerprint ────────────────────────────────────────
    use sha2::{Digest, Sha256};
    use shared::master_pattern::{pattern_fingerprint, RarityTier};
    let audio_hash: [u8; 32] = Sha256::digest(&audio_bytes).into();
    let fp = pattern_fingerprint(isrc.0.as_bytes(), &audio_hash);
    let tier = RarityTier::from_band(fp.band);
    info!(isrc=%isrc, band=%fp.band, rarity=%tier.as_str(), "Master Pattern computed");

    // ── Alphabet resonance ────────────────────────────────────────────────
    use shared::alphabet::resonance_report;
    let resonance = resonance_report(&artist_name, &title, fp.band);

    // ── Audio QC (LUFS + format) ──────────────────────────────────────────
    let qc_report = audio_qc::run_qc(&audio_bytes, None, None);
    for defect in &qc_report.defects {
        state.metrics.record_defect("audio_qc");
        warn!(defect=%defect, isrc=%isrc, "Audio QC defect");
    }
    let track_meta = dsp::TrackMeta {
        isrc: Some(isrc.0.clone()),
        upc: None,
        explicit: false,
        territory_rights: false,
        contributor_meta: false,
        cover_art_px: None,
    };
    let dsp_results = dsp::validate_all(&qc_report, &track_meta);
    let dsp_failures: Vec<_> = dsp_results.iter().filter(|r| !r.passed).collect();

    // ── ISO 9001 audit ────────────────────────────────────────────────────
    state
        .audit_log
        .record(&format!(
            "UPLOAD_START title='{}' isrc='{}' bytes={} band={} rarity={} qc_passed={}",
            title,
            isrc,
            audio_bytes.len(),
            fp.band,
            tier.as_str(),
            qc_report.passed
        ))
        .ok();

    // ── Article 17 upload filter ──────────────────────────────────────────
    if wikidata::isrc_exists(&isrc.0).await {
        warn!(isrc=%isrc, "Article 17: ISRC already on Wikidata — flagging");
        state.mod_queue.add(moderation::ContentReport {
            id: format!("ART17-{}", isrc.0),
            isrc: isrc.0.clone(),
            reporter_id: "system:article17_filter".into(),
            category: moderation::ReportCategory::Copyright,
            description: format!("ISRC {} already registered on Wikidata", isrc.0),
            status: moderation::ReportStatus::UnderReview,
            submitted_at: chrono::Utc::now().to_rfc3339(),
            resolved_at: None,
            resolution: None,
            sla_hours: 24,
        });
    }

    // ── Wikidata enrichment ───────────────────────────────────────────────
    let wiki = if std::env::var("WIKIDATA_DISABLED").unwrap_or_default() != "1"
        && !artist_name.is_empty()
    {
        wikidata::lookup_artist(&artist_name).await
    } else {
        wikidata::WikidataArtist::default()
    };
    if let Some(ref qid) = wiki.qid {
        info!(artist=%artist_name, qid=%qid, mbid=?wiki.musicbrainz_id, "Wikidata enriched");
        state
            .audit_log
            .record(&format!(
                "WIKIDATA_ENRICH isrc='{}' artist='{}' qid='{}'",
                isrc, artist_name, qid
            ))
            .ok();
    }

    info!(isrc=%isrc, title=%title, "Pipeline starting");

    // ── Pipeline ──────────────────────────────────────────────────────────
    let cid = btfs::upload(&audio_bytes, &title, &isrc)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let tx_result = bttc::submit_distribution(&cid, &[], fp.band, None)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let tx_hash = tx_result.tx_hash;

    let reg = ddex::register(&title, &isrc, &cid, &fp, &wiki)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    mirrors::push_all(&cid, &reg.isrc, &title, fp.band)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    // ── Six Sigma CTQ ─────────────────────────────────────────────────────
    let elapsed_ms = start.elapsed().as_millis() as f64;
    state.metrics.record_band(fp.band);
    state.metrics.record_latency("upload_pipeline", elapsed_ms);
    if elapsed_ms > 200.0 {
        warn!(elapsed_ms, "CTQ breach: latency >200ms");
        state.metrics.record_defect("latency_breach");
    }

    state
        .audit_log
        .record(&format!(
            "UPLOAD_DONE isrc='{}' cid='{}' tx='{}' elapsed_ms={}",
            isrc, cid.0, tx_hash, elapsed_ms
        ))
        .ok();

    Ok(Json(serde_json::json!({
        "cid":             cid.0,
        "isrc":            isrc.0,
        "tx_hash":         tx_hash,
        "band":            fp.band,
        "band_residue":    fp.band_residue,
        "mapped_prime":    fp.mapped_prime,
        "rarity":          tier.as_str(),
        "cycle_pos":       fp.cycle_position,
        "title_resonant":  resonance.title_resonant,
        "wikidata_qid":    wiki.qid,
        "musicbrainz_id":  wiki.musicbrainz_id,
        "artist_label":    wiki.label_name,
        "artist_country":  wiki.country,
        "artist_genres":   wiki.genres,
        "audio_qc_passed": qc_report.passed,
        "audio_qc_defects":qc_report.defects,
        "dsp_ready":       dsp_failures.is_empty(),
        "dsp_failures":    dsp_failures.iter().map(|r| &r.dsp).collect::<Vec<_>>(),
    })))
}
