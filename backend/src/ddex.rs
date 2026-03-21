//! DDEX ERN 4.1 registration with Master Pattern + Wikidata namespaces.
use serde::{Deserialize, Serialize};
use shared::master_pattern::{PatternFingerprint, RarityTier};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdexRegistration {
    pub isrc: String,
    pub iswc: Option<String>,
}

pub fn build_ern_xml(
    title: &str,
    isrc: &str,
    cid: &str,
    fp: &PatternFingerprint,
    wiki: &crate::wikidata::WikidataArtist,
) -> String {
    let tier = RarityTier::from_band(fp.band);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage
  xmlns:ern="http://ddex.net/xml/ern/41"
  xmlns:mp="http://retrosync.media/xml/master-pattern/1"
  xmlns:wd="http://retrosync.media/xml/wikidata/1"
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
        band = fp.band,
        band_name = tier.as_str(),
        residue = fp.band_residue,
        prime = fp.mapped_prime,
        cycle = fp.cycle_position,
        dr = fp.digit_root,
        closure = fp.closure_verified,
        ts = chrono::Utc::now().to_rfc3339(),
        wikidata_qid = wiki.qid.as_deref().unwrap_or(""),
        wikidata_url = wiki.wikidata_url.as_deref().unwrap_or(""),
        mbid = wiki.musicbrainz_id.as_deref().unwrap_or(""),
        label_name = wiki.label_name.as_deref().unwrap_or(""),
        country = wiki.country.as_deref().unwrap_or(""),
        genres = wiki.genres.join(", "),
    )
}

pub async fn register(
    title: &str,
    isrc: &shared::types::Isrc,
    cid: &shared::types::BtfsCid,
    fp: &PatternFingerprint,
    wiki: &crate::wikidata::WikidataArtist,
) -> anyhow::Result<DdexRegistration> {
    let xml = build_ern_xml(title, &isrc.0, &cid.0, fp, wiki);
    let ddex_url =
        std::env::var("DDEX_SANDBOX_URL").unwrap_or_else(|_| "https://sandbox.ddex.net/ern".into());
    info!(isrc=%isrc, band=%fp.band, "Submitting ERN 4.1 to DDEX");
    if std::env::var("DDEX_DEV_MODE").unwrap_or_default() == "1" {
        warn!("DDEX_DEV_MODE=1 — stub");
        return Ok(DdexRegistration {
            isrc: isrc.0.clone(),
            iswc: None,
        });
    }
    let resp = reqwest::Client::new()
        .post(&ddex_url)
        .header("Content-Type", "application/xml")
        .body(xml)
        .send()
        .await?;
    if !resp.status().is_success() {
        anyhow::bail!("DDEX failed: {}", resp.status());
    }
    Ok(DdexRegistration {
        isrc: isrc.0.clone(),
        iswc: None,
    })
}
