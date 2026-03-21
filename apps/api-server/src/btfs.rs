//! BTFS upload module — multipart POST to BTFS daemon /api/v0/add.
use shared::types::{BtfsCid, Isrc};
use tracing::{debug, info, instrument};

#[instrument(skip(audio_bytes), fields(bytes = audio_bytes.len()))]
pub async fn upload(audio_bytes: &[u8], title: &str, isrc: &Isrc) -> anyhow::Result<BtfsCid> {
    let api = std::env::var("BTFS_API_URL").unwrap_or_else(|_| "http://127.0.0.1:5001".into());
    let url = format!("{}/api/v0/add", api);
    let filename = format!("{}.bin", isrc.0.replace('/', "-"));
    let part = reqwest::multipart::Part::bytes(audio_bytes.to_vec())
        .file_name(filename)
        .mime_str("application/octet-stream")?;
    let form = reqwest::multipart::Form::new().part("file", part);
    debug!(url=%url, "Uploading to BTFS");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let resp = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("BTFS unreachable: {}", e))?;
    if !resp.status().is_success() {
        anyhow::bail!("BTFS /api/v0/add failed: {}", resp.status());
    }
    let body = resp.text().await?;
    let cid_str = body
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter_map(|v| v["Hash"].as_str().map(|s| s.to_string()))
        .last()
        .ok_or_else(|| anyhow::anyhow!("BTFS returned no CID"))?;
    let cid = shared::parsers::recognize_btfs_cid(&cid_str)
        .map_err(|e| anyhow::anyhow!("BTFS invalid CID: {}", e))?;
    info!(isrc=%isrc, cid=%cid.0, "Uploaded to BTFS");
    Ok(cid)
}

#[allow(dead_code)]
pub async fn pin(cid: &BtfsCid) -> anyhow::Result<()> {
    // SECURITY FIX: Pin errors are now propagated instead of silently ignored.
    // A failed pin means content is not guaranteed to persist on the BTFS network.
    let api = std::env::var("BTFS_API_URL").unwrap_or_else(|_| "http://127.0.0.1:5001".into());
    let url = format!("{}/api/v0/pin/add?arg={}", api, cid.0);
    let resp = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?
        .post(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("BTFS pin request failed for CID {}: {}", cid.0, e))?;

    if !resp.status().is_success() {
        anyhow::bail!("BTFS pin failed for CID {} — HTTP {}", cid.0, resp.status());
    }
    tracing::info!(cid=%cid.0, "BTFS content pinned successfully");
    Ok(())
}
