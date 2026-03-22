//! Music shard module — CFT decomposition of audio metadata into DA51 CBOR shards.
//!
//! Scale tower for music (mirrors erdfa-publish text CFT):
//!   Track → Stem → Segment → Frame → Sample → Byte
//!
//! NFT holders get decryption keys for full-quality shards.
//! Public shards are degraded (truncated / lower bitrate).

use axum::{extract::{Path, State}, http::StatusCode, Json};
use erdfa_publish::{cft::Scale as TextScale, Component, Shard};
use shared::types::{BtfsCid, Isrc};
use tracing::{info, warn};

/// Audio-native CFT scales
#[derive(Clone, Copy, Debug)]
pub enum AudioScale {
    Track,   // whole release
    Stem,    // vocal / drums / bass / keys
    Segment, // verse / chorus / bridge
    Frame,   // ~23ms audio frame
    Sample,  // individual PCM sample
    Byte,    // raw bytes
}

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
}

/// Metadata for one audio shard
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AudioShardMeta {
    pub isrc: String,
    pub scale: String,
    pub index: usize,
    pub duration_ms: u64,
    pub quality: ShardQuality,
    pub cid: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ShardQuality {
    Full,             // lossless, NFT-gated
    Preview,          // 30s truncated
    Degraded { kbps: u16 }, // low bitrate public
    Steganographic,   // hidden in cover content
}

/// Decompose track metadata into erdfa shards at each audio scale.
pub fn decompose_track(isrc: &Isrc, stems: &[&str], segments: &[&str]) -> Vec<Shard> {
    let prefix = &isrc.0;
    let mut shards = Vec::new();

    // Track level
    shards.push(Shard::new(
        &format!("{}_track", prefix),
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
            &format!("{}_{}", prefix, stem),
            Component::KeyValue {
                pairs: vec![
                    ("isrc".into(), isrc.0.clone()),
                    ("scale".into(), AudioScale::Stem.tag().into()),
                    ("stem".into(), stem.to_string()),
                    ("index".into(), i.to_string()),
                    ("parent".into(), format!("{}_track", prefix)),
                ],
            },
        ));
    }

    // Segment level
    for (i, seg) in segments.iter().enumerate() {
        shards.push(Shard::new(
            &format!("{}_seg{}", prefix, i),
            Component::KeyValue {
                pairs: vec![
                    ("isrc".into(), isrc.0.clone()),
                    ("scale".into(), AudioScale::Segment.tag().into()),
                    ("label".into(), seg.to_string()),
                    ("index".into(), i.to_string()),
                    ("parent".into(), format!("{}_track", prefix)),
                ],
            },
        ));
    }

    shards
}

/// Serialize shards to DA51 CBOR bytes
pub fn shards_to_cbor(shards: &[Shard]) -> anyhow::Result<Vec<u8>> {
    let mut buf = Vec::new();
    ciborium::into_writer(shards, &mut buf)?;
    Ok(buf)
}

// ── HTTP handlers ─────────────────────────────────────────────────

use crate::AppState;

/// GET /api/shard/:cid
/// Returns shard data. Full quality requires NFT ownership proof.
pub async fn get_shard(
    State(state): State<AppState>,
    Path(cid): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let wallet = headers
        .get("x-wallet-address")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    // Look up shard
    let shard_data = state
        .shard_db
        .get(&cid)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check NFT ownership for full quality
    let has_access = match &wallet {
        Some(addr) => check_nft_ownership(addr, &cid).await,
        None => false,
    };

    if has_access {
        info!(cid=%cid, "Full-quality shard access granted");
        Ok(Json(serde_json::json!({
            "cid": cid,
            "quality": "full",
            "data": shard_data,
        })))
    } else {
        warn!(cid=%cid, "Degraded shard — no NFT ownership");
        Ok(Json(serde_json::json!({
            "cid": cid,
            "quality": "preview",
            "data": truncate_shard(&shard_data),
            "message": "Purchase NFT for full-quality access",
        })))
    }
}

/// Check if wallet owns the NFT granting access to this shard CID.
async fn check_nft_ownership(wallet: &str, _cid: &str) -> bool {
    if std::env::var("BTTC_DEV_MODE").unwrap_or_default() == "1" {
        return true;
    }
    // TODO: query MasterPattern.sol ownerOf() on BTTC
    let _ = wallet;
    false
}

fn truncate_shard(data: &serde_json::Value) -> serde_json::Value {
    // Return only metadata, strip audio payload
    match data.get("component") {
        Some(c) => serde_json::json!({ "component": c, "truncated": true }),
        None => serde_json::json!({ "truncated": true }),
    }
}
