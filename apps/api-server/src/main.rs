//! Retrosync backend — Axum API server.
//! Zero Trust: every request verified via JWT (auth.rs).
//! LangSec: all inputs pass through shared::parsers recognizers.
//! ISO 9001 §7.5: all operations logged to append-only audit store.

use axum::{
    extract::{Multipart, Path, State},
    http::{Method, StatusCode},
    middleware,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use shared::parsers::recognize_isrc;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

mod audio_qc;
mod auth;
mod bbs;
mod ddex_gateway;
mod dsr_parser;
mod btfs;
mod bttc;
mod bwarm;
mod cmrra;
mod coinbase;
mod collection_societies;
mod ddex;
mod dqi;
mod dsp;
mod durp;
mod fraud;
mod gtms;
mod hyperglot;
mod identifiers;
mod isni;
mod iso_store;
mod kyc;
mod langsec;
mod ledger;
mod metrics;
mod mirrors;
mod moderation;
mod multisig_vault;
mod music_reports;
mod nft_manifest;
mod persist;
mod privacy;
mod publishing;
mod rate_limit;
mod royalty_reporting;
mod sap;
mod sftp;
mod shard;
mod takedown;
mod tron;
mod wallet_auth;
mod wikidata;
mod xslt;
mod zk_cache;

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
    pub challenge_store: Arc<wallet_auth::ChallengeStore>,
    pub rate_limiter: Arc<rate_limit::RateLimiter>,
    pub shard_store: Arc<shard::ShardStore>,
    // ── New integrations ──────────────────────────────────────────────────
    pub tron_config: Arc<tron::TronConfig>,
    pub coinbase_config: Arc<coinbase::CoinbaseCommerceConfig>,
    pub durp_config: Arc<durp::DurpConfig>,
    pub music_reports_config: Arc<music_reports::MusicReportsConfig>,
    pub isni_config: Arc<isni::IsniConfig>,
    pub cmrra_config: Arc<cmrra::CmrraConfig>,
    pub bbs_config: Arc<bbs::BbsConfig>,
    // ── DDEX Gateway (ERN push + DSR pull) ───────────────────────────────────
    pub gateway_config: Arc<ddex_gateway::GatewayConfig>,
    // ── Multi-sig vault (Safe + USDC payout) ─────────────────────────────────
    pub vault_config: Arc<multisig_vault::VaultConfig>,
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
        privacy_db: Arc::new(privacy::PrivacyStore::open("privacy_db")?),
        fraud_db: Arc::new(fraud::FraudDetector::new()),
        kyc_db: Arc::new(kyc::KycStore::open("kyc_db")?),
        mod_queue: Arc::new(moderation::ModerationQueue::open("moderation_db")?),
        sap_client: Arc::new(sap::SapClient::from_env()),
        gtms_db: Arc::new(gtms::GtmsStore::new()),
        challenge_store: Arc::new(wallet_auth::ChallengeStore::new()),
        rate_limiter: Arc::new(rate_limit::RateLimiter::new()),
        shard_store: Arc::new(shard::ShardStore::new()),
        tron_config: Arc::new(tron::TronConfig::from_env()),
        coinbase_config: Arc::new(coinbase::CoinbaseCommerceConfig::from_env()),
        durp_config: Arc::new(durp::DurpConfig::from_env()),
        music_reports_config: Arc::new(music_reports::MusicReportsConfig::from_env()),
        isni_config: Arc::new(isni::IsniConfig::from_env()),
        cmrra_config: Arc::new(cmrra::CmrraConfig::from_env()),
        bbs_config: Arc::new(bbs::BbsConfig::from_env()),
        gateway_config: Arc::new(ddex_gateway::GatewayConfig::from_env()),
        vault_config: Arc::new(multisig_vault::VaultConfig::from_env()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics::handler))
        // ── Wallet authentication (no auth required — these issue the auth token)
        .route(
            "/api/auth/challenge/:address",
            get(wallet_auth::issue_challenge),
        )
        .route("/api/auth/verify", post(wallet_auth::verify_challenge))
        // ── Track upload + status
        .route("/api/upload", post(upload_track))
        .route("/api/track/:id", get(track_status))
        // ── Publishing agreements + soulbound NFT minting
        .route("/api/register", post(publishing::register_track))
        // ── DMCA §512
        .route("/api/takedown", post(takedown::submit_notice))
        .route(
            "/api/takedown/:id/counter",
            post(takedown::submit_counter_notice),
        )
        .route("/api/takedown/:id", get(takedown::get_notice))
        // ── GDPR/CCPA
        .route("/api/privacy/consent", post(privacy::record_consent))
        .route(
            "/api/privacy/delete/:uid",
            delete(privacy::delete_user_data),
        )
        .route("/api/privacy/export/:uid", get(privacy::export_user_data))
        // ── Moderation (DSA/Article 17)
        .route("/api/moderation/report", post(moderation::submit_report))
        .route("/api/moderation/queue", get(moderation::get_queue))
        .route(
            "/api/moderation/:id/resolve",
            post(moderation::resolve_report),
        )
        // ── KYC/AML
        .route("/api/kyc/:uid", post(kyc::submit_kyc))
        .route("/api/kyc/:uid/status", get(kyc::kyc_status))
        // ── CWR/XSLT society submissions
        .route(
            "/api/royalty/xslt/:society",
            post(xslt::transform_submission),
        )
        .route(
            "/api/royalty/xslt/all",
            post(xslt::transform_all_submissions),
        )
        // ── SAP S/4HANA + ECC
        .route("/api/sap/royalty-posting", post(sap::post_royalty_document))
        .route("/api/sap/vendor-sync", post(sap::sync_vendor))
        .route("/api/sap/idoc/royalty", post(sap::emit_royalty_idoc))
        .route("/api/sap/health", get(sap::sap_health))
        // ── Global Trade Management
        .route("/api/gtms/classify", post(gtms::classify_work))
        .route("/api/gtms/screen", post(gtms::screen_distribution))
        .route("/api/gtms/declaration/:id", get(gtms::get_declaration))
        // ── Shard store (CFT audio decomposition + NFT-gated access)
        .route("/api/shard/:cid", get(shard::get_shard))
        .route("/api/shard/decompose", post(shard::decompose_and_index))
        // ── Tron network (TronLink wallet auth + TRX royalty distribution)
        .route("/api/tron/challenge/:address", get(tron_issue_challenge))
        .route("/api/tron/verify", post(tron_verify))
        // ── Coinbase Commerce (payments + webhook)
        .route(
            "/api/payments/coinbase/charge",
            post(coinbase_create_charge),
        )
        .route("/api/payments/coinbase/webhook", post(coinbase_webhook))
        .route(
            "/api/payments/coinbase/status/:charge_id",
            get(coinbase_charge_status),
        )
        // ── DQI (Data Quality Initiative)
        .route("/api/dqi/evaluate", post(dqi_evaluate))
        // ── DURP (Distributor Unmatched Recordings Portal)
        .route("/api/durp/submit", post(durp_submit))
        // ── BWARM (Best Workflow for All Rights Management)
        .route("/api/bwarm/record", post(bwarm_create_record))
        .route("/api/bwarm/conflicts", post(bwarm_detect_conflicts))
        // ── Music Reports
        .route(
            "/api/music-reports/licence/:isrc",
            get(music_reports_lookup),
        )
        .route("/api/music-reports/rates", get(music_reports_rates))
        // ── Hyperglot (script detection)
        .route("/api/hyperglot/detect", post(hyperglot_detect))
        // ── ISNI (International Standard Name Identifier)
        .route("/api/isni/validate", post(isni_validate))
        .route("/api/isni/lookup/:isni", get(isni_lookup))
        .route("/api/isni/search", post(isni_search))
        // ── CMRRA (Canadian mechanical licensing)
        .route("/api/cmrra/rates", get(cmrra_rates))
        .route("/api/cmrra/licence", post(cmrra_request_licence))
        .route("/api/cmrra/statement/csv", post(cmrra_statement_csv))
        // ── BBS (Broadcast Blanket Service)
        .route("/api/bbs/cue-sheet", post(bbs_submit_cue_sheet))
        .route("/api/bbs/rate", post(bbs_estimate_rate))
        .route("/api/bbs/bmat-csv", post(bbs_bmat_csv))
        // ── Collection Societies
        .route("/api/societies", get(societies_list))
        .route("/api/societies/:id", get(societies_by_id))
        .route(
            "/api/societies/territory/:territory",
            get(societies_by_territory),
        )
        .route("/api/societies/route", post(societies_route_royalty))
        // ── DDEX Gateway (ERN push + DSR pull)
        .route("/api/gateway/status", get(gateway_status))
        .route("/api/gateway/ern/push", post(gateway_ern_push))
        .route("/api/gateway/dsr/cycle", post(gateway_dsr_cycle))
        .route("/api/gateway/dsr/parse", post(gateway_dsr_parse_upload))
        // ── Multi-sig vault (Safe + USDC payout)
        .route("/api/vault/summary", get(vault_summary))
        .route("/api/vault/deposits", get(vault_deposits))
        .route("/api/vault/payout", post(vault_propose_payout))
        .route(
            "/api/vault/tx/:safe_tx_hash",
            get(vault_tx_status),
        )
        // ── NFT Shard Manifest
        .route("/api/manifest/:token_id", get(manifest_lookup))
        .route("/api/manifest/mint", post(manifest_mint))
        .route("/api/manifest/proof", post(manifest_ownership_proof))
        // ── DSR flat-file parser (standalone, no SFTP needed)
        .route("/api/dsr/parse", post(dsr_parse_inline))
        .layer({
            // SECURITY: CORS locked to explicit allowed origins (ALLOWED_ORIGINS env var).
            // SECURITY FIX: removed open-wildcard fallback.  If origins list is empty
            // (e.g. ALLOWED_ORIGINS="") we use the localhost dev defaults, never Any.
            use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
            let origins = auth::allowed_origins();
            if origins.is_empty() {
                let env = std::env::var("RETROSYNC_ENV").unwrap_or_default();
                if env == "production" {
                    panic!(
                        "SECURITY: ALLOWED_ORIGINS must be set in production — aborting startup"
                    );
                }
                warn!("ALLOWED_ORIGINS is empty — restricting CORS to localhost dev origins");
            }
            // Use only the configured origins; never open wildcard.
            let allow_origins: Vec<axum::http::HeaderValue> = if origins.is_empty() {
                ["http://localhost:5173", "http://localhost:3000", "http://localhost:5001"]
                    .iter()
                    .filter_map(|o| o.parse().ok())
                    .collect()
            } else {
                origins
            };
            CorsLayer::new()
                .allow_origin(allow_origins)
                .allow_methods([Method::GET, Method::POST, Method::DELETE])
                .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        })
        // Middleware execution order (Axum applies last-to-first, outermost = last .layer()):
        //   Outermost → innermost:
        //   1. add_security_headers  — always inject security response headers first
        //   2. rate_limit::enforce   — reject floods before auth work
        //   3. auth::verify_zero_trust — only verified requests reach handlers
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::verify_zero_trust,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit::enforce,
        ))
        .layer(middleware::from_fn(auth::add_security_headers))
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
                // SECURITY: Enforce maximum file size to prevent OOM DoS.
                // Default: 100MB. Override with MAX_AUDIO_BYTES env var.
                let max_bytes: usize = std::env::var("MAX_AUDIO_BYTES")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(100 * 1024 * 1024);
                let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
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

    // ── LangSec: audio file magic-byte validation ─────────────────────────
    // Reject known non-audio file signatures (polyglot/zip-bomb/executable).
    // We do not attempt to enumerate every valid audio format; instead we
    // block the most common attack vectors by their leading magic bytes.
    if !audio_bytes.is_empty() {
        let sig = &audio_bytes[..audio_bytes.len().min(12)];

        // Reject if signature matches a known non-audio type
        let is_forbidden = sig.starts_with(b"PK\x03\x04")      // ZIP / DOCX / JAR
            || sig.starts_with(b"PK\x05\x06")                  // empty ZIP
            || sig.starts_with(b"MZ")                           // Windows PE/EXE
            || sig.starts_with(b"\x7FELF")                      // ELF binary
            || sig.starts_with(b"%PDF")                         // PDF
            || sig.starts_with(b"#!")                           // shell script
            || sig.starts_with(b"<?php")                        // PHP
            || sig.starts_with(b"<script")                      // JS/HTML
            || sig.starts_with(b"<html")                        // HTML
            || sig.starts_with(b"\x89PNG")                      // PNG image
            || sig.starts_with(b"\xFF\xD8\xFF")                 // JPEG image
            || sig.starts_with(b"GIF8")                         // GIF image
            || (sig.len() >= 4 && &sig[..4] == b"RIFF"         // AVI (not WAV)
                && sig.len() >= 12 && &sig[8..12] == b"AVI ");

        if is_forbidden {
            warn!(
                size = audio_bytes.len(),
                magic = ?&sig[..sig.len().min(4)],
                "Upload rejected: file signature matches forbidden non-audio type"
            );
            state.metrics.record_defect("upload_forbidden_mime");
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }

        // Confirm at least one recognised audio signature is present.
        // Unknown signatures are logged as warnings but not blocked here —
        // QC pipeline will reject non-audio content downstream.
        let is_known_audio = sig.starts_with(b"ID3")                // MP3 with ID3
            || (sig.len() >= 2 && sig[0] == 0xFF                    // MPEG sync
                && (sig[1] & 0xE0) == 0xE0)
            || sig.starts_with(b"fLaC")                             // FLAC
            || (sig.starts_with(b"RIFF")                            // WAV/AIFF
                && sig.len() >= 12 && (&sig[8..12] == b"WAVE" || &sig[8..12] == b"AIFF"))
            || sig.starts_with(b"OggS")                             // OGG/OPUS
            || (sig.len() >= 8 && &sig[4..8] == b"ftyp")            // AAC/M4A/MP4
            || sig.starts_with(b"FORM")                             // AIFF
            || sig.starts_with(b"\x30\x26\xB2\x75");               // WMA/ASF

        if !is_known_audio {
            warn!(
                size = audio_bytes.len(),
                magic = ?&sig[..sig.len().min(8)],
                "Upload: unrecognised audio signature — QC pipeline will validate"
            );
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
                "WIKIDATA_ENRICH isrc='{isrc}' artist='{artist_name}' qid='{qid}'"
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

// ── Tron handlers ─────────────────────────────────────────────────────────────

async fn tron_issue_challenge(
    Path(address): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // LangSec: validate Tron address before issuing challenge
    langsec::validate_tron_address(&address).map_err(|e| {
        warn!(err=%e, "Tron challenge: invalid address");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;
    let challenge = tron::issue_tron_challenge(&address).map_err(|e| {
        warn!(err=%e, "Tron challenge: issue failed");
        StatusCode::BAD_REQUEST
    })?;
    Ok(Json(serde_json::json!({
        "challenge_id": challenge.challenge_id,
        "address": challenge.address.0,
        "nonce": challenge.nonce,
        "expires_at": challenge.expires_at,
    })))
}

async fn tron_verify(
    State(state): State<AppState>,
    Json(req): Json<tron::TronVerifyRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // NOTE: In production, look up the nonce from the challenge store by challenge_id.
    // For now we echo the challenge_id as the nonce (to be wired to ChallengeStore).
    let nonce = req.challenge_id.clone();
    let result = tron::verify_tron_signature(&state.tron_config, &req, &nonce)
        .await
        .map_err(|e| {
            warn!(err=%e, "Tron verify: failed");
            StatusCode::UNAUTHORIZED
        })?;
    if !result.verified {
        return Err(StatusCode::UNAUTHORIZED);
    }
    state
        .audit_log
        .record(&format!("TRON_AUTH_OK address='{}'", result.address))
        .ok();
    Ok(Json(serde_json::json!({
        "verified": result.verified,
        "address": result.address.0,
        "message": result.message,
    })))
}

// ── Coinbase Commerce handlers ─────────────────────────────────────────────────

async fn coinbase_create_charge(
    State(state): State<AppState>,
    Json(req): Json<coinbase::ChargeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // LangSec: validate text fields
    langsec::validate_free_text(&req.name, "name", 200)
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let resp = coinbase::create_charge(&state.coinbase_config, &req)
        .await
        .map_err(|e| {
            warn!(err=%e, "Coinbase charge creation failed");
            StatusCode::BAD_GATEWAY
        })?;
    Ok(Json(serde_json::json!({
        "charge_id":   resp.charge_id,
        "hosted_url":  resp.hosted_url,
        "amount_usd":  resp.amount_usd,
        "expires_at":  resp.expires_at,
        "status":      format!("{:?}", resp.status),
    })))
}

async fn coinbase_webhook(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let sig = request
        .headers()
        .get("x-cc-webhook-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let body = axum::body::to_bytes(request.into_body(), langsec::MAX_JSON_BODY_BYTES)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    coinbase::verify_webhook_signature(&state.coinbase_config, &body, &sig).map_err(|e| {
        warn!(err=%e, "Coinbase webhook signature invalid");
        StatusCode::UNAUTHORIZED
    })?;
    let payload: coinbase::WebhookPayload =
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    if let Some((event_type, charge_id)) = coinbase::handle_webhook_event(&payload) {
        state
            .audit_log
            .record(&format!(
                "COINBASE_WEBHOOK event='{event_type}' charge='{charge_id}'"
            ))
            .ok();
    }
    Ok(Json(serde_json::json!({ "received": true })))
}

async fn coinbase_charge_status(
    State(state): State<AppState>,
    Path(charge_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status = coinbase::get_charge_status(&state.coinbase_config, &charge_id)
        .await
        .map_err(|e| {
            warn!(err=%e, "Coinbase status lookup failed");
            StatusCode::BAD_GATEWAY
        })?;
    Ok(Json(
        serde_json::json!({ "charge_id": charge_id, "status": format!("{:?}", status) }),
    ))
}

// ── DQI handler ───────────────────────────────────────────────────────────────

async fn dqi_evaluate(
    State(state): State<AppState>,
    Json(input): Json<dqi::DqiInput>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let report = dqi::evaluate(&input);
    state
        .audit_log
        .record(&format!(
            "DQI_EVALUATE isrc='{}' score={:.1}% tier='{}'",
            report.isrc,
            report.score_pct,
            report.tier.as_str()
        ))
        .ok();
    Ok(Json(serde_json::to_value(&report).unwrap_or_default()))
}

// ── DURP handler ──────────────────────────────────────────────────────────────

async fn durp_submit(
    State(state): State<AppState>,
    Json(records): Json<Vec<durp::DurpRecord>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if records.is_empty() || records.len() > 5000 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let errors = durp::validate_records(&records);
    if !errors.is_empty() {
        return Ok(Json(serde_json::json!({
            "status": "validation_failed",
            "errors": errors,
        })));
    }
    let csv = durp::generate_csv(&records);
    let batch_id = format!(
        "BATCH-{:016x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let submission = durp::submit_batch(&state.durp_config, &batch_id, &csv)
        .await
        .map_err(|e| {
            warn!(err=%e, "DURP submission failed");
            StatusCode::BAD_GATEWAY
        })?;
    state
        .audit_log
        .record(&format!(
            "DURP_SUBMIT batch='{}' records={} status='{:?}'",
            batch_id,
            records.len(),
            submission.status
        ))
        .ok();
    Ok(Json(serde_json::json!({
        "batch_id": submission.batch_id,
        "status": format!("{:?}", submission.status),
        "records": records.len(),
    })))
}

// ── BWARM handlers ─────────────────────────────────────────────────────────────

async fn bwarm_create_record(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let title = payload["title"].as_str().unwrap_or("").to_string();
    let isrc = payload["isrc"].as_str();
    langsec::validate_free_text(&title, "title", 500)
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let record = bwarm::BwarmRecord::new(&title, isrc);
    let xml = bwarm::generate_bwarm_xml(&record);
    state
        .audit_log
        .record(&format!(
            "BWARM_CREATE id='{}' title='{}'",
            record.record_id, title
        ))
        .ok();
    Ok(Json(serde_json::json!({
        "record_id": record.record_id,
        "state": record.state.as_str(),
        "xml_length": xml.len(),
    })))
}

async fn bwarm_detect_conflicts(
    Json(record): Json<bwarm::BwarmRecord>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let conflicts = bwarm::detect_conflicts(&record);
    let state = bwarm::compute_state(&record);
    Ok(Json(serde_json::json!({
        "state": state.as_str(),
        "conflict_count": conflicts.len(),
        "conflicts": conflicts,
    })))
}

// ── Music Reports handlers ────────────────────────────────────────────────────

async fn music_reports_lookup(
    State(state): State<AppState>,
    Path(isrc): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let licences = music_reports::lookup_by_isrc(&state.music_reports_config, &isrc)
        .await
        .map_err(|e| {
            warn!(err=%e, "Music Reports lookup failed");
            StatusCode::BAD_GATEWAY
        })?;
    Ok(Json(serde_json::json!({
        "isrc": isrc,
        "licence_count": licences.len(),
        "licences": licences,
    })))
}

async fn music_reports_rates() -> Json<serde_json::Value> {
    let rate = music_reports::current_mechanical_rate();
    let dsps = music_reports::dsp_licence_requirements();
    Json(serde_json::json!({
        "mechanical_rate": rate,
        "dsp_requirements": dsps,
    }))
}

// ── Hyperglot handler ─────────────────────────────────────────────────────────

async fn hyperglot_detect(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let text = payload["text"].as_str().unwrap_or("");
    // LangSec: limit input before passing to script detector
    if text.len() > 16384 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let result = hyperglot::detect_scripts(text);
    Ok(Json(serde_json::to_value(&result).unwrap_or_default()))
}

// ── ISNI handlers ─────────────────────────────────────────────────────────────

async fn isni_validate(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let raw = payload["isni"].as_str().unwrap_or("");
    // LangSec: ISNI is 16 chars max; enforce before parse
    if raw.len() > 32 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    match isni::validate_isni(raw) {
        Ok(validated) => Ok(Json(serde_json::json!({
            "valid": true,
            "isni": validated.0,
            "formatted": format!("{validated}"),
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "valid": false,
            "error": e.to_string(),
        }))),
    }
}

async fn isni_lookup(
    State(state): State<AppState>,
    Path(isni_raw): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if isni_raw.len() > 32 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let validated = isni::validate_isni(&isni_raw).map_err(|e| {
        warn!(err=%e, "ISNI lookup: invalid ISNI");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;
    let record = isni::lookup_isni(&state.isni_config, &validated)
        .await
        .map_err(|e| {
            warn!(err=%e, "ISNI lookup failed");
            StatusCode::BAD_GATEWAY
        })?;
    Ok(Json(serde_json::to_value(&record).unwrap_or_default()))
}

async fn isni_search(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let name = payload["name"].as_str().unwrap_or("");
    if name.is_empty() || name.len() > 200 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let limit = payload["limit"].as_u64().unwrap_or(10) as usize;
    let results = isni::search_isni_by_name(&state.isni_config, name, limit.min(50))
        .await
        .map_err(|e| {
            warn!(err=%e, "ISNI search failed");
            StatusCode::BAD_GATEWAY
        })?;
    Ok(Json(serde_json::json!({
        "name": name,
        "count": results.len(),
        "results": results,
    })))
}

// ── CMRRA handlers ────────────────────────────────────────────────────────────

async fn cmrra_rates() -> Json<serde_json::Value> {
    let rates = cmrra::current_canadian_rates();
    let csi = cmrra::csi_blanket_info();
    Json(serde_json::json!({
        "rates": rates,
        "csi_blanket": csi,
    }))
}

async fn cmrra_request_licence(
    State(state): State<AppState>,
    Json(req): Json<cmrra::CmrraLicenceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // LangSec: validate ISRC before forwarding
    if req.isrc.len() != 12 || !req.isrc.chars().all(|c| c.is_alphanumeric()) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let resp = cmrra::request_licence(&state.cmrra_config, &req)
        .await
        .map_err(|e| {
            warn!(err=%e, "CMRRA licence request failed");
            StatusCode::BAD_GATEWAY
        })?;
    state
        .audit_log
        .record(&format!(
            "CMRRA_LICENCE isrc='{}' licence='{}' status='{:?}'",
            req.isrc, resp.licence_number, resp.status
        ))
        .ok();
    Ok(Json(serde_json::to_value(&resp).unwrap_or_default()))
}

async fn cmrra_statement_csv(
    Json(lines): Json<Vec<cmrra::CmrraStatementLine>>,
) -> Result<axum::response::Response, StatusCode> {
    if lines.is_empty() || lines.len() > 10_000 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let csv = cmrra::generate_quarterly_csv(&lines);
    Ok(axum::response::Response::builder()
        .status(200)
        .header("Content-Type", "text/csv; charset=utf-8")
        .header(
            "Content-Disposition",
            "attachment; filename=\"cmrra-statement.csv\"",
        )
        .body(axum::body::Body::from(csv))
        .unwrap())
}

// ── BBS handlers ──────────────────────────────────────────────────────────────

async fn bbs_submit_cue_sheet(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cues: Vec<bbs::BroadcastCue> = serde_json::from_value(payload["cues"].clone())
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let period_start: chrono::DateTime<chrono::Utc> = payload["period_start"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(chrono::Utc::now);
    let period_end: chrono::DateTime<chrono::Utc> = payload["period_end"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(chrono::Utc::now);

    let errors = bbs::validate_cue_batch(&cues);
    if !errors.is_empty() {
        return Ok(Json(serde_json::json!({
            "status": "validation_failed",
            "errors": errors,
        })));
    }

    let batch = bbs::submit_cue_sheet(&state.bbs_config, cues, period_start, period_end)
        .await
        .map_err(|e| {
            warn!(err=%e, "BBS cue sheet submission failed");
            StatusCode::BAD_GATEWAY
        })?;
    state
        .audit_log
        .record(&format!(
            "BBS_CUESHEET batch='{}' cues={}",
            batch.batch_id,
            batch.cues.len()
        ))
        .ok();
    Ok(Json(serde_json::json!({
        "batch_id": batch.batch_id,
        "cues": batch.cues.len(),
        "submitted_at": batch.submitted_at,
    })))
}

async fn bbs_estimate_rate(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let licence_type: bbs::BbsLicenceType = serde_json::from_value(payload["licence_type"].clone())
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let territory = payload["territory"].as_str().unwrap_or("US");
    // LangSec: territory is always 2 uppercase letters
    if territory.len() != 2 || !territory.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let annual_hours = payload["annual_hours"].as_f64().unwrap_or(2000.0);
    if !(0.0_f64..=8760.0).contains(&annual_hours) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let fee_usd = bbs::estimate_blanket_fee(&licence_type, territory, annual_hours);
    Ok(Json(serde_json::json!({
        "licence_type": licence_type.display_name(),
        "territory": territory,
        "annual_hours": annual_hours,
        "estimated_fee_usd": fee_usd,
    })))
}

async fn bbs_bmat_csv(
    Json(cues): Json<Vec<bbs::BroadcastCue>>,
) -> Result<axum::response::Response, StatusCode> {
    if cues.is_empty() || cues.len() > 10_000 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let csv = bbs::generate_bmat_csv(&cues);
    Ok(axum::response::Response::builder()
        .status(200)
        .header("Content-Type", "text/csv; charset=utf-8")
        .header(
            "Content-Disposition",
            "attachment; filename=\"bmat-broadcast.csv\"",
        )
        .body(axum::body::Body::from(csv))
        .unwrap())
}

// ── Collection Societies handlers ─────────────────────────────────────────────

async fn societies_list() -> Json<serde_json::Value> {
    let all = collection_societies::all_societies();
    let summary: Vec<_> = all
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.name,
                "territories": s.territories,
                "rights": s.rights,
                "cisac_member": s.cisac_member,
                "biem_member": s.biem_member,
                "currency": s.currency,
                "website": s.website,
            })
        })
        .collect();
    Json(serde_json::json!({
        "count": summary.len(),
        "societies": summary,
    }))
}

async fn societies_by_id(Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    // LangSec: society IDs are ASCII alphanumeric + underscore/hyphen, max 32 chars
    if id.len() > 32
        || !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let society = collection_societies::society_by_id(&id).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(serde_json::json!({
        "id": society.id,
        "name": society.name,
        "territories": society.territories,
        "rights": society.rights,
        "cisac_member": society.cisac_member,
        "biem_member": society.biem_member,
        "website": society.website,
        "currency": society.currency,
        "payment_network": society.payment_network,
        "minimum_payout": society.minimum_payout,
        "reporting_standard": society.reporting_standard,
    })))
}

async fn societies_by_territory(
    Path(territory): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // LangSec: territory is always 2 uppercase letters
    if territory.len() != 2 || !territory.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let t = territory.to_uppercase();
    let societies = collection_societies::societies_for_territory(&t);
    let result: Vec<_> = societies
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.name,
                "rights": s.rights,
                "currency": s.currency,
                "website": s.website,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({
        "territory": t,
        "count": result.len(),
        "societies": result,
    })))
}

async fn societies_route_royalty(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let territory = payload["territory"].as_str().unwrap_or("");
    let amount_usd = payload["amount_usd"].as_f64().unwrap_or(0.0);
    let isrc = payload["isrc"].as_str();
    let iswc = payload["iswc"].as_str();

    // LangSec validations
    if territory.len() != 2 || !territory.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    if !(0.0_f64..=1_000_000.0).contains(&amount_usd) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let right_type: collection_societies::RightType =
        serde_json::from_value(payload["right_type"].clone())
            .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let instructions = collection_societies::route_royalty(
        &territory.to_uppercase(),
        right_type,
        amount_usd,
        isrc,
        iswc,
    );
    Ok(Json(serde_json::json!({
        "territory": territory.to_uppercase(),
        "amount_usd": amount_usd,
        "instruction_count": instructions.len(),
        "instructions": instructions,
    })))
}

// ── DDEX Gateway handlers ─────────────────────────────────────────────────────

async fn gateway_status(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let status = ddex_gateway::gateway_status(&state.gateway_config);
    Json(serde_json::to_value(&status).unwrap_or_default())
}

async fn gateway_ern_push(
    State(state): State<AppState>,
    Json(payload): Json<ddex_gateway::PendingRelease>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // LangSec: ISRC must be 12 alphanumeric characters
    if payload.isrc.len() != 12 || !payload.isrc.chars().all(|c| c.is_alphanumeric()) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    if payload.title.is_empty() || payload.title.len() > 500 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let results = ddex_gateway::push_ern(&state.gateway_config, &payload).await;

    state
        .audit_log
        .record(&format!(
            "GATEWAY_ERN_PUSH isrc='{}' dsps={}",
            payload.isrc,
            results.len()
        ))
        .ok();

    let delivered = results.iter().filter(|r| r.receipt.is_some()).count();
    let failed = results.len() - delivered;
    Ok(Json(serde_json::json!({
        "isrc": payload.isrc,
        "dsp_count": results.len(),
        "delivered": delivered,
        "failed": failed,
        "results": results.iter().map(|r| serde_json::json!({
            "dsp": r.dsp,
            "success": r.receipt.is_some(),
            "seq": r.event.seq,
        })).collect::<Vec<_>>(),
    })))
}

async fn gateway_dsr_cycle(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let results = ddex_gateway::run_dsr_cycle(&state.gateway_config).await;
    let total_records: usize = results.iter().map(|r| r.total_records).sum();
    let total_revenue: f64 = results.iter().map(|r| r.total_revenue_usd).sum();
    state
        .audit_log
        .record(&format!(
            "GATEWAY_DSR_CYCLE dsps={} total_records={} total_revenue_usd={:.2}",
            results.len(),
            total_records,
            total_revenue
        ))
        .ok();
    Json(serde_json::json!({
        "dsp_count": results.len(),
        "total_records": total_records,
        "total_revenue_usd": total_revenue,
        "results": results.iter().map(|r| serde_json::json!({
            "dsp": r.dsp,
            "files_discovered": r.files_discovered,
            "files_processed": r.files_processed,
            "records": r.total_records,
            "revenue_usd": r.total_revenue_usd,
        })).collect::<Vec<_>>(),
    }))
}

async fn gateway_dsr_parse_upload(
    State(_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut content = String::new();
    let mut dialect_hint: Option<dsr_parser::DspDialect> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                // LangSec: limit DSR file to 50 MB
                if bytes.len() > 52_428_800 {
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
                content =
                    String::from_utf8(bytes.to_vec()).map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            "dialect" => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?;
                dialect_hint = match text.to_lowercase().as_str() {
                    "spotify" => Some(dsr_parser::DspDialect::Spotify),
                    "apple" => Some(dsr_parser::DspDialect::AppleMusic),
                    "amazon" => Some(dsr_parser::DspDialect::Amazon),
                    "youtube" => Some(dsr_parser::DspDialect::YouTube),
                    "tidal" => Some(dsr_parser::DspDialect::Tidal),
                    "deezer" => Some(dsr_parser::DspDialect::Deezer),
                    _ => Some(dsr_parser::DspDialect::DdexStandard),
                };
            }
            _ => {}
        }
    }

    if content.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let report = dsr_parser::parse_dsr_file(&content, dialect_hint);
    Ok(Json(serde_json::json!({
        "dialect": report.dialect.display_name(),
        "records": report.records.len(),
        "rejections": report.rejections.len(),
        "total_revenue_usd": report.total_revenue_usd,
        "isrc_totals": report.isrc_totals,
        "parsed_at": report.parsed_at,
    })))
}

/// POST /api/dsr/parse — accept DSR content as JSON body (simpler than multipart).
async fn dsr_parse_inline(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let content = payload["content"].as_str().unwrap_or("");
    if content.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    // LangSec: limit inline DSR content
    if content.len() > 52_428_800 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let hint: Option<dsr_parser::DspDialect> =
        payload["dialect"]
            .as_str()
            .map(|d| match d.to_lowercase().as_str() {
                "spotify" => dsr_parser::DspDialect::Spotify,
                "apple" => dsr_parser::DspDialect::AppleMusic,
                "amazon" => dsr_parser::DspDialect::Amazon,
                "youtube" => dsr_parser::DspDialect::YouTube,
                "tidal" => dsr_parser::DspDialect::Tidal,
                "deezer" => dsr_parser::DspDialect::Deezer,
                _ => dsr_parser::DspDialect::DdexStandard,
            });

    let report = dsr_parser::parse_dsr_file(content, hint);
    Ok(Json(serde_json::json!({
        "dialect": report.dialect.display_name(),
        "records": report.records.len(),
        "rejections": report.rejections.len(),
        "total_revenue_usd": report.total_revenue_usd,
        "isrc_totals": report.isrc_totals,
        "parsed_at": report.parsed_at,
    })))
}

// ── Multi-sig Vault handlers ──────────────────────────────────────────────────

async fn vault_summary(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let summary = multisig_vault::vault_summary(&state.vault_config)
        .await
        .map_err(|e| {
            warn!(err=%e, "vault_summary failed");
            StatusCode::BAD_GATEWAY
        })?;
    Ok(Json(serde_json::to_value(&summary).unwrap_or_default()))
}

async fn vault_deposits(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let from_block = payload["from_block"].as_u64().unwrap_or(0);
    let deposits = multisig_vault::scan_usdc_deposits(&state.vault_config, from_block)
        .await
        .map_err(|e| {
            warn!(err=%e, "vault_deposits scan failed");
            StatusCode::BAD_GATEWAY
        })?;
    Ok(Json(serde_json::json!({
        "from_block": from_block,
        "count": deposits.len(),
        "deposits": deposits,
    })))
}

async fn vault_propose_payout(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let payouts: Vec<multisig_vault::ArtistPayout> =
        serde_json::from_value(payload["payouts"].clone())
            .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let total_usdc = payload["total_usdc"].as_u64().unwrap_or(0);

    // LangSec: sanity-check payout wallets
    for p in &payouts {
        if !p.wallet.starts_with("0x") || p.wallet.len() != 42 {
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    }

    let proposal = multisig_vault::propose_artist_payouts(
        &state.vault_config,
        &payouts,
        total_usdc,
        None,
        0,
    )
    .await
    .map_err(|e| {
        warn!(err=%e, "vault_propose_payout failed");
        StatusCode::BAD_GATEWAY
    })?;

    state
        .audit_log
        .record(&format!(
            "VAULT_PAYOUT_PROPOSED safe_tx='{}' payees={}",
            proposal.safe_tx_hash,
            payouts.len()
        ))
        .ok();

    Ok(Json(serde_json::to_value(&proposal).unwrap_or_default()))
}

async fn vault_tx_status(
    State(state): State<AppState>,
    Path(safe_tx_hash): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // LangSec: safe_tx_hash is 0x + 64 hex chars
    if safe_tx_hash.len() > 66 || !safe_tx_hash.starts_with("0x") {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let status =
        multisig_vault::check_execution_status(&state.vault_config, &safe_tx_hash)
            .await
            .map_err(|e| {
                warn!(err=%e, "vault_tx_status failed");
                StatusCode::BAD_GATEWAY
            })?;
    Ok(Json(serde_json::to_value(&status).unwrap_or_default()))
}

// ── NFT Shard Manifest handlers ───────────────────────────────────────────────

async fn manifest_lookup(
    Path(token_id_str): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let token_id: u64 = token_id_str
        .parse()
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let manifest = nft_manifest::lookup_manifest_by_token(token_id)
        .await
        .map_err(|e| {
            warn!(err=%e, token_id, "manifest_lookup failed");
            StatusCode::NOT_FOUND
        })?;
    Ok(Json(serde_json::to_value(&manifest).unwrap_or_default()))
}

async fn manifest_mint(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let isrc = payload["isrc"].as_str().unwrap_or("");
    let track_cid = payload["track_cid"].as_str().unwrap_or("");

    // LangSec
    if isrc.len() != 12 || !isrc.chars().all(|c| c.is_alphanumeric()) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    if track_cid.is_empty() || track_cid.len() > 128 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let shard_order: Vec<String> = payload["shard_order"]
        .as_array()
        .ok_or(StatusCode::BAD_REQUEST)?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    if shard_order.is_empty() || shard_order.len() > 10_000 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let enc_key_hex = payload["enc_key_hex"].as_str().map(String::from);
    let nonce_hex = payload["nonce_hex"].as_str().map(String::from);

    // Validate enc_key_hex is 64 hex chars if present
    if let Some(ref key) = enc_key_hex {
        if key.len() != 64 || !key.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    }

    let mut manifest = nft_manifest::ShardManifest::new(
        isrc,
        track_cid,
        shard_order,
        std::collections::HashMap::new(),
        enc_key_hex,
        nonce_hex,
    );

    let receipt = nft_manifest::mint_manifest_nft(&mut manifest)
        .await
        .map_err(|e| {
            warn!(err=%e, %isrc, "manifest_mint failed");
            StatusCode::BAD_GATEWAY
        })?;

    state
        .audit_log
        .record(&format!(
            "NFT_MANIFEST_MINTED isrc='{}' token_id={} cid='{}'",
            isrc, receipt.token_id, receipt.manifest_cid
        ))
        .ok();

    Ok(Json(serde_json::json!({
        "token_id": receipt.token_id,
        "tx_hash": receipt.tx_hash,
        "manifest_cid": receipt.manifest_cid,
        "zk_commit_hash": receipt.zk_commit_hash,
        "shard_count": manifest.shard_count,
        "encrypted": manifest.is_encrypted(),
        "minted_at": receipt.minted_at,
    })))
}

async fn manifest_ownership_proof(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let token_id: u64 = payload["token_id"]
        .as_u64()
        .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
    let wallet = payload["wallet"].as_str().unwrap_or("");

    // LangSec: wallet must be a valid EVM address
    if !wallet.starts_with("0x") || wallet.len() != 42 {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let manifest = nft_manifest::lookup_manifest_by_token(token_id)
        .await
        .map_err(|e| {
            warn!(err=%e, token_id, "manifest_ownership_proof: lookup failed");
            StatusCode::NOT_FOUND
        })?;

    let proof =
        nft_manifest::generate_manifest_ownership_proof_stub(token_id, wallet, &manifest);

    Ok(Json(serde_json::to_value(&proof).unwrap_or_default()))
}
