//! Music shard module — CFT decomposition of audio metadata into DA51 CBOR shards.
//!
//! Scale tower for music (mirrors erdfa-publish text CFT):
//!   Track → Stem → Segment → Frame → Sample → Byte
//!
//! Shards are semantic representations of track structure encoded as DA51-tagged
//! CBOR bytes using the erdfa-publish library.
//!
//! NFT holders get full-quality shards; public access returns 30-second previews.
//!
//! Pre-generated source shards (Emacs Lisp / Fractran VM reflections of each
//! Rust module) live in `shards/` at the repo root and can be served directly
//! via GET /api/shard/:cid once indexed at startup or via POST /api/shard/index.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use erdfa_publish::{cft::Scale as TextScale, Component, Shard, ShardSet};
use shared::types::Isrc;
use std::{collections::HashMap, sync::RwLock};
use tracing::{info, warn};

// ── Audio CFT scale tower ──────────────────────────────────────────────────

/// Audio-native CFT scales — mirrors the text CFT in erdfa-publish.
/// All six variants are part of the public tower API even if only Track/Stem/Segment
/// are emitted by the current decompose_track() implementation.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum AudioScale {
    Track,   // whole release
    Stem,    // vocal / drums / bass / keys
    Segment, // verse / chorus / bridge
    Frame,   // ~23 ms audio frame
    Sample,  // individual PCM sample
    Byte,    // raw bytes
}

#[allow(dead_code)]
impl AudioScale {
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Track => "cft.track",
            Self::Stem => "cft.stem",
            Self::Segment => "cft.segment",
            Self::Frame => "cft.frame",
            Self::Sample => "cft.sample",
            Self::Byte => "cft.byte",
        }
    }
    pub fn depth(&self) -> u8 {
        match self {
            Self::Track => 0,
            Self::Stem => 1,
            Self::Segment => 2,
            Self::Frame => 3,
            Self::Sample => 4,
            Self::Byte => 5,
        }
    }
    /// Corresponding text-domain scale for cross-tower morphisms.
    pub fn text_analogue(&self) -> TextScale {
        match self {
            Self::Track => TextScale::Post,
            Self::Stem => TextScale::Paragraph,
            Self::Segment => TextScale::Line,
            Self::Frame => TextScale::Token,
            Self::Sample => TextScale::Emoji,
            Self::Byte => TextScale::Byte,
        }
    }
}

// ── Shard quality tiers ────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ShardQuality {
    Full,                   // lossless, NFT-gated
    Preview,                // 30-second truncated
    Degraded { kbps: u16 }, // low-bitrate public stream
    Steganographic,         // hidden in cover content
}

// ── In-memory shard store ──────────────────────────────────────────────────

/// Lightweight in-process shard index (cid → JSON metadata).
/// Populated at startup by indexing pre-built shards from disk or via upload.
pub struct ShardStore(pub RwLock<HashMap<String, serde_json::Value>>);

impl ShardStore {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }

    pub fn insert(&self, cid: &str, data: serde_json::Value) {
        self.0.write().unwrap().insert(cid.to_string(), data);
    }

    pub fn get(&self, cid: &str) -> Option<serde_json::Value> {
        self.0.read().unwrap().get(cid).cloned()
    }
}

impl Default for ShardStore {
    fn default() -> Self {
        Self::new()
    }
}

// ── CFT decomposition ──────────────────────────────────────────────────────

/// Decompose track metadata into erdfa-publish `Shard`s at each audio scale.
///
/// Returns shards for:
///   - one Track-level shard
///   - one Stem shard per stem label
///   - one Segment shard per segment label
pub fn decompose_track(isrc: &Isrc, stems: &[&str], segments: &[&str]) -> Vec<Shard> {
    let prefix = &isrc.0;
    let mut shards = Vec::new();

    // Track level
    shards.push(Shard::new(
        format!("{prefix}_track"),
        Component::KeyValue {
            pairs: vec![
                ("isrc".into(), isrc.0.clone()),
                ("scale".into(), AudioScale::Track.tag().into()),
                ("stems".into(), stems.len().to_string()),
                ("segments".into(), segments.len().to_string()),
            ],
        },
    ));

    // Stem level
    for (i, stem) in stems.iter().enumerate() {
        shards.push(Shard::new(
            format!("{prefix}_{stem}"),
            Component::KeyValue {
                pairs: vec![
                    ("isrc".into(), isrc.0.clone()),
                    ("scale".into(), AudioScale::Stem.tag().into()),
                    ("stem".into(), stem.to_string()),
                    ("index".into(), i.to_string()),
                    ("parent".into(), format!("{prefix}_track")),
                ],
            },
        ));
    }

    // Segment level
    for (i, seg) in segments.iter().enumerate() {
        shards.push(Shard::new(
            format!("{prefix}_seg{i}"),
            Component::KeyValue {
                pairs: vec![
                    ("isrc".into(), isrc.0.clone()),
                    ("scale".into(), AudioScale::Segment.tag().into()),
                    ("label".into(), seg.to_string()),
                    ("index".into(), i.to_string()),
                    ("parent".into(), format!("{prefix}_track")),
                ],
            },
        ));
    }

    shards
}

/// Build a `ShardSet` manifest and serialise all shards to a DA51-tagged CBOR tar archive.
///
/// Each shard is encoded individually with `Shard::to_cbor()` (DA51 tag) and
/// collected using `ShardSet::to_tar()`.  Intended for batch export of track shards.
#[allow(dead_code)]
pub fn shards_to_tar(name: &str, shards: &[Shard]) -> anyhow::Result<Vec<u8>> {
    let mut set = ShardSet::new(name);
    for s in shards {
        set.add(s);
    }
    let mut buf = Vec::new();
    set.to_tar(shards, &mut buf)?;
    Ok(buf)
}

// ── HTTP handlers ──────────────────────────────────────────────────────────

use crate::AppState;

/// `GET /api/shard/:cid`
///
/// Returns shard JSON. Wallet header `x-wallet-address` is checked for NFT
/// ownership; holders receive `quality: "full"`, everyone else gets a truncated
/// preview with a purchase prompt.
pub async fn get_shard(
    State(state): State<AppState>,
    Path(cid): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let wallet = headers
        .get("x-wallet-address")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let shard_data = state.shard_store.get(&cid).ok_or(StatusCode::NOT_FOUND)?;

    let has_access = match &wallet {
        Some(addr) => check_nft_ownership(addr, &cid).await,
        None => false,
    };

    if has_access {
        info!(cid = %cid, "Full-quality shard access granted");
        Ok(Json(serde_json::json!({
            "cid":     cid,
            "quality": "full",
            "data":    shard_data,
        })))
    } else {
        warn!(cid = %cid, "Degraded shard served — no NFT ownership proven");
        Ok(Json(serde_json::json!({
            "cid":     cid,
            "quality": "preview",
            "data":    truncate_shard(&shard_data),
            "message": "Purchase the SoulboundNFT for full-quality access",
        })))
    }
}

/// `POST /api/shard/decompose`
///
/// Accepts `{ "isrc": "...", "stems": [...], "segments": [...] }`, runs
/// `decompose_track`, stores shards in the in-process index, and returns
/// the shard CID list.
pub async fn decompose_and_index(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let isrc_str = body
        .get("isrc")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let isrc =
        shared::parsers::recognize_isrc(isrc_str).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let stems: Vec<&str> = body
        .get("stems")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let segments: Vec<&str> = body
        .get("segments")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let shards = decompose_track(&isrc, &stems, &segments);

    for shard in &shards {
        let json = serde_json::to_value(shard).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        state.shard_store.insert(&shard.cid, json);
        info!(id = %shard.id, cid = %shard.cid, "Shard indexed");
    }

    Ok(Json(serde_json::json!({
        "isrc":   isrc_str,
        "shards": shards.len(),
        "cids":   shard_cid_list(&shards),
    })))
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Query MasterPattern.sol `ownerOf()` on BTTC to verify NFT ownership.
/// In dev mode (`BTTC_DEV_MODE=1`) always returns `true`.
async fn check_nft_ownership(wallet: &str, _cid: &str) -> bool {
    if std::env::var("BTTC_DEV_MODE").unwrap_or_default() == "1" {
        return true;
    }
    // TODO: ethers call to MasterPattern.sol ownerOf() on BTTC mainnet
    let _ = wallet;
    false
}

fn truncate_shard(data: &serde_json::Value) -> serde_json::Value {
    match data.get("component") {
        Some(c) => serde_json::json!({ "component": c, "truncated": true }),
        None => serde_json::json!({ "truncated": true }),
    }
}

fn shard_cid_list(shards: &[Shard]) -> Vec<String> {
    shards.iter().map(|s| s.cid.clone()).collect()
}
