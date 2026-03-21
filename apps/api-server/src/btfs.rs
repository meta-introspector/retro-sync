//! BTFS upload module — multipart POST to BTFS daemon /api/v0/add.
//!
//! SECURITY:
//! - Set BTFS_API_KEY env var to authenticate to your BTFS node.
//!   Every request carries `X-API-Key: {BTFS_API_KEY}` header.
//! - Set BTFS_API_URL to a private internal URL; never expose port 5001 publicly.
//! - The pin() function now propagates errors — a failed pin is treated as
//!   a data loss condition and must be investigated.
use shared::types::{BtfsCid, Isrc};
use tracing::{debug, info, instrument};

/// Build a reqwest client with a 120-second timeout and the BTFS API key header.
fn btfs_client() -> anyhow::Result<(reqwest::Client, Option<String>)> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let api_key = std::env::var("BTFS_API_KEY").ok();
    Ok((client, api_key))
}

/// Attach BTFS API key to a request builder if BTFS_API_KEY is set.
fn with_api_key(builder: reqwest::RequestBuilder, api_key: Option<&str>) -> reqwest::RequestBuilder {
    match api_key {
        Some(key) => builder.header("X-API-Key", key),
        None => builder,
    }
}

#[instrument(skip(audio_bytes), fields(bytes = audio_bytes.len()))]
pub async fn upload(audio_bytes: &[u8], title: &str, isrc: &Isrc) -> anyhow::Result<BtfsCid> {
    let api = std::env::var("BTFS_API_URL").unwrap_or_else(|_| "http://127.0.0.1:5001".into());
    let url = format!("{}/api/v0/add", api);
    let filename = format!("{}.bin", isrc.0.replace('/', "-"));

    let (client, api_key) = btfs_client()?;

    let part = reqwest::multipart::Part::bytes(audio_bytes.to_vec())
        .file_name(filename)
        .mime_str("application/octet-stream")?;
    let form = reqwest::multipart::Form::new().part("file", part);

    debug!(url=%url, has_api_key=%api_key.is_some(), "Uploading to BTFS");

    let req = with_api_key(client.post(&url), api_key.as_deref()).multipart(form);
    let resp = req
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("BTFS unreachable at {}: {}", url, e))?;

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
    // SECURITY: Pin errors propagated — a failed pin means content is not
    // guaranteed to persist on the BTFS network. Do not silently ignore.
    let api = std::env::var("BTFS_API_URL").unwrap_or_else(|_| "http://127.0.0.1:5001".into());
    let url = format!("{}/api/v0/pin/add?arg={}", api, cid.0);

    let (client, api_key) = btfs_client()?;
    let req = with_api_key(client.post(&url), api_key.as_deref());

    let resp = req
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("BTFS pin request failed for CID {}: {}", cid.0, e))?;

    if !resp.status().is_success() {
        anyhow::bail!("BTFS pin failed for CID {} — HTTP {}", cid.0, resp.status());
    }

    info!(cid=%cid.0, "BTFS content pinned successfully");
    Ok(())
}
