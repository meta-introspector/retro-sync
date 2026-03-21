//! Mirror uploads: Internet Archive + BBS (both non-blocking).
use shared::master_pattern::RarityTier;
use tracing::{info, instrument, warn};

#[instrument]
pub async fn push_all(
    cid: &shared::types::BtfsCid,
    isrc: &str,
    title: &str,
    band: u8,
) -> anyhow::Result<()> {
    let (ia, bbs) = tokio::join!(
        push_internet_archive(cid, isrc, title, band),
        push_bbs(cid, isrc, title, band),
    );
    if let Err(e) = ia {
        warn!(err=%e, "IA mirror failed");
    }
    if let Err(e) = bbs {
        warn!(err=%e, "BBS mirror failed");
    }
    Ok(())
}

async fn push_internet_archive(
    cid: &shared::types::BtfsCid,
    isrc: &str,
    title: &str,
    band: u8,
) -> anyhow::Result<()> {
    let access = std::env::var("ARCHIVE_ACCESS_KEY").unwrap_or_default();
    if access.is_empty() {
        warn!("ARCHIVE_ACCESS_KEY not set — skipping IA");
        return Ok(());
    }
    let tier = RarityTier::from_band(band);
    let identifier = format!("retrosync-{}", isrc.replace('/', "-").to_lowercase());
    let url = format!(
        "https://s3.us.archive.org/{}/{}.meta.json",
        identifier, identifier
    );
    let meta = serde_json::json!({ "title": title, "isrc": isrc, "btfs_cid": cid.0,
                                          "band": band, "rarity": tier.as_str() });
    let secret = std::env::var("ARCHIVE_SECRET_KEY").unwrap_or_default();
    let resp = reqwest::Client::new()
        .put(&url)
        .header("Authorization", format!("LOW {}:{}", access, secret))
        .header("x-archive-auto-make-bucket", "1")
        .header("x-archive-meta-title", title)
        .header("x-archive-meta-mediatype", "audio")
        .header("Content-Type", "application/json")
        .body(meta.to_string())
        .send()
        .await?;
    if resp.status().is_success() {
        info!(isrc=%isrc, "Mirrored to IA");
    }
    Ok(())
}

async fn push_bbs(
    cid: &shared::types::BtfsCid,
    isrc: &str,
    title: &str,
    band: u8,
) -> anyhow::Result<()> {
    let url = std::env::var("MIRROR_BBS_URL").unwrap_or_default();
    if url.is_empty() {
        return Ok(());
    }
    let tier = RarityTier::from_band(band);
    let payload = serde_json::json!({
        "type": "track_announce", "isrc": isrc, "title": title,
        "btfs_cid": cid.0, "band": band, "rarity": tier.as_str(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?
        .post(&url)
        .json(&payload)
        .send()
        .await?;
    info!(isrc=%isrc, "Announced to BBS");
    Ok(())
}
