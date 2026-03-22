//! DDEX ERN 4.1 registration with Master Pattern + Wikidata + creator attribution.
use serde::{Deserialize, Serialize};
use shared::master_pattern::{PatternFingerprint, RarityTier};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdexRegistration {
    pub isrc: String,
    pub iswc: Option<String>,
}

/// A single credited contributor for DDEX delivery (songwriter, publisher, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdexContributor {
    pub wallet_address: String,
    pub ipi_number: String,
    pub role: String,
    pub bps: u16,
}

/// Escape a string for safe embedding in XML content or attribute values.
/// Prevents XML injection from user-controlled inputs.
fn xml_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '&' => "&amp;".chars().collect::<Vec<_>>(),
            '<' => "&lt;".chars().collect(),
            '>' => "&gt;".chars().collect(),
            '"' => "&quot;".chars().collect(),
            '\'' => "&apos;".chars().collect(),
            c => vec![c],
        })
        .collect()
}

pub fn build_ern_xml_with_contributors(
    title: &str,
    isrc: &str,
    cid: &str,
    fp: &PatternFingerprint,
    wiki: &crate::wikidata::WikidataArtist,
    contributors: &[DdexContributor],
) -> String {
    // SECURITY: XML-escape all user-controlled inputs before embedding in XML
    let title = xml_escape(title);
    let isrc = xml_escape(isrc);
    let cid = xml_escape(cid);
    let wikidata_qid = xml_escape(wiki.qid.as_deref().unwrap_or(""));
    let wikidata_url = xml_escape(wiki.wikidata_url.as_deref().unwrap_or(""));
    let mbid = xml_escape(wiki.musicbrainz_id.as_deref().unwrap_or(""));
    let label_name = xml_escape(wiki.label_name.as_deref().unwrap_or(""));
    let country = xml_escape(wiki.country.as_deref().unwrap_or(""));
    let genres = xml_escape(&wiki.genres.join(", "));

    let tier = RarityTier::from_band(fp.band);

    // Build contributor XML block
    let contributor_xml: String = contributors
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let wallet = xml_escape(&c.wallet_address);
            let ipi = xml_escape(&c.ipi_number);
            let role = xml_escape(&c.role);
            let bps = c.bps;
            // DDEX ERN 4.1 ResourceContributor element with extended retrosync namespace
            format!(
                r#"      <ResourceContributor SequenceNumber="{seq}">
        <PartyName><FullName>{role}</FullName></PartyName>
        <PartyId>IPI:{ipi}</PartyId>
        <ResourceContributorRole>{role}</ResourceContributorRole>
        <rs:CreatorWallet>{wallet}</rs:CreatorWallet>
        <rs:RoyaltyBps>{bps}</rs:RoyaltyBps>
      </ResourceContributor>"#,
                seq = i + 1,
                role = role,
                ipi = ipi,
                wallet = wallet,
                bps = bps,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage
  xmlns:ern="http://ddex.net/xml/ern/41"
  xmlns:mp="http://retrosync.media/xml/master-pattern/1"
  xmlns:wd="http://retrosync.media/xml/wikidata/1"
  xmlns:rs="http://retrosync.media/xml/creator-attribution/1"
  MessageSchemaVersionId="ern/41" LanguageAndScriptCode="en">
  <MessageHeader>
    <MessageThreadId>retrosync-{isrc}</MessageThreadId>
    <MessageSender>
      <PartyId>PADPIDA2024RETROSYNC</PartyId>
      <PartyName><FullName>Retrosync Media Group</FullName></PartyName>
    </MessageSender>
    <MessageCreatedDateTime>{ts}</MessageCreatedDateTime>
  </MessageHeader>
  <ResourceList>
    <SoundRecording>
      <SoundRecordingType>MusicalWorkSoundRecording</SoundRecordingType>
      <SoundRecordingId><ISRC>{isrc}</ISRC></SoundRecordingId>
      <ReferenceTitle><TitleText>{title}</TitleText></ReferenceTitle>
      <ResourceContributorList>
{contributor_xml}
      </ResourceContributorList>
      <mp:MasterPattern>
        <mp:Band>{band}</mp:Band>
        <mp:BandName>{band_name}</mp:BandName>
        <mp:BandResidue>{residue}</mp:BandResidue>
        <mp:MappedPrime>{prime}</mp:MappedPrime>
        <mp:CyclePosition>{cycle}</mp:CyclePosition>
        <mp:DigitRoot>{dr}</mp:DigitRoot>
        <mp:ClosureVerified>{closure}</mp:ClosureVerified>
        <mp:BtfsCid>{cid}</mp:BtfsCid>
      </mp:MasterPattern>
      <wd:WikidataEnrichment>
        <wd:ArtistQID>{wikidata_qid}</wd:ArtistQID>
        <wd:WikidataURL>{wikidata_url}</wd:WikidataURL>
        <wd:MusicBrainzArtistID>{mbid}</wd:MusicBrainzArtistID>
        <wd:LabelName>{label_name}</wd:LabelName>
        <wd:CountryOfOrigin>{country}</wd:CountryOfOrigin>
        <wd:Genres>{genres}</wd:Genres>
      </wd:WikidataEnrichment>
    </SoundRecording>
  </ResourceList>
  <ReleaseList>
    <Release>
      <ReleaseId><ISRC>{isrc}</ISRC></ReleaseId>
      <ReleaseType>TrackRelease</ReleaseType>
      <ReleaseResourceReferenceList>
        <ReleaseResourceReference>A1</ReleaseResourceReference>
      </ReleaseResourceReferenceList>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#,
        isrc = isrc,
        title = title,
        cid = cid,
        contributor_xml = contributor_xml,
        band = fp.band,
        band_name = tier.as_str(),
        residue = fp.band_residue,
        prime = fp.mapped_prime,
        cycle = fp.cycle_position,
        dr = fp.digit_root,
        closure = fp.closure_verified,
        ts = chrono::Utc::now().to_rfc3339(),
        wikidata_qid = wikidata_qid,
        wikidata_url = wikidata_url,
        mbid = mbid,
        label_name = label_name,
        country = country,
        genres = genres,
    )
}

pub async fn register(
    title: &str,
    isrc: &shared::types::Isrc,
    cid: &shared::types::BtfsCid,
    fp: &PatternFingerprint,
    wiki: &crate::wikidata::WikidataArtist,
) -> anyhow::Result<DdexRegistration> {
    register_with_contributors(title, isrc, cid, fp, wiki, &[]).await
}

pub async fn register_with_contributors(
    title: &str,
    isrc: &shared::types::Isrc,
    cid: &shared::types::BtfsCid,
    fp: &PatternFingerprint,
    wiki: &crate::wikidata::WikidataArtist,
    contributors: &[DdexContributor],
) -> anyhow::Result<DdexRegistration> {
    let xml = build_ern_xml_with_contributors(title, &isrc.0, &cid.0, fp, wiki, contributors);
    let ddex_url =
        std::env::var("DDEX_SANDBOX_URL").unwrap_or_else(|_| "https://sandbox.ddex.net/ern".into());
    let api_key = std::env::var("DDEX_API_KEY").ok();

    info!(isrc=%isrc, band=%fp.band, contributors=%contributors.len(), "Submitting ERN 4.1 to DDEX");
    if std::env::var("DDEX_DEV_MODE").unwrap_or_default() == "1" {
        warn!("DDEX_DEV_MODE=1 — stub");
        return Ok(DdexRegistration {
            isrc: isrc.0.clone(),
            iswc: None,
        });
    }

    let mut client = reqwest::Client::new()
        .post(&ddex_url)
        .header("Content-Type", "application/xml");

    if let Some(key) = api_key {
        client = client.header("Authorization", format!("Bearer {key}"));
    }

    let resp = client.body(xml).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("DDEX failed: {}", resp.status());
    }
    Ok(DdexRegistration {
        isrc: isrc.0.clone(),
        iswc: None,
    })
}
