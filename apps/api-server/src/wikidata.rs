//! Wikidata SPARQL enrichment — artist QID, MusicBrainz ID, label, genres.
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

const SPARQL: &str = "https://query.wikidata.org/sparql";
const UA: &str = "RetrosyncMediaGroup/1.0 (https://retrosync.media)";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WikidataArtist {
    pub qid: Option<String>,
    pub wikidata_url: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub label_name: Option<String>,
    pub label_qid: Option<String>,
    pub country: Option<String>,
    pub genres: Vec<String>,
    pub website: Option<String>,
    pub known_isrcs: Vec<String>,
}

#[derive(Deserialize)]
struct SparqlResp {
    results: SparqlResults,
}
#[derive(Deserialize)]
struct SparqlResults {
    bindings: Vec<serde_json::Value>,
}

pub async fn lookup_artist(name: &str) -> WikidataArtist {
    match lookup_inner(name).await {
        Ok(a) => a,
        Err(e) => {
            warn!(artist=%name, err=%e, "Wikidata failed");
            WikidataArtist::default()
        }
    }
}

async fn lookup_inner(name: &str) -> anyhow::Result<WikidataArtist> {
    let safe = name.replace('"', "\\\"");
    let query = format!(
        r#"
SELECT DISTINCT ?artist ?mbid ?label ?labelLabel ?country ?countryLabel ?genre ?genreLabel ?website ?isrc
WHERE {{
  ?artist rdfs:label "{safe}"@en .
  {{ ?artist wdt:P31/wdt:P279* wd:Q5 }} UNION {{ ?artist wdt:P31 wd:Q215380 }}
  OPTIONAL {{ ?artist wdt:P434 ?mbid }}
  OPTIONAL {{ ?artist wdt:P264 ?label }}
  OPTIONAL {{ ?artist wdt:P27  ?country }}
  OPTIONAL {{ ?artist wdt:P136 ?genre }}
  OPTIONAL {{ ?artist wdt:P856 ?website }}
  OPTIONAL {{ ?artist wdt:P1243 ?isrc }}
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en" }}
}} LIMIT 20"#
    );

    let client = reqwest::Client::builder()
        .user_agent(UA)
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let resp = client
        .get(SPARQL)
        .query(&[("query", &query), ("format", &"json".to_string())])
        .send()
        .await?
        .json::<SparqlResp>()
        .await?;

    let b = &resp.results.bindings;
    if b.is_empty() {
        return Ok(WikidataArtist::default());
    }

    let ext = |key: &str| -> Option<String> { b[0][key]["value"].as_str().map(|s| s.into()) };
    let qid = ext("artist")
        .as_ref()
        .and_then(|u| u.rsplit('/').next().map(|s| s.into()));
    let wikidata_url = qid
        .as_ref()
        .map(|q| format!("https://www.wikidata.org/wiki/{q}"));
    let mut genres = Vec::new();
    let mut known_isrcs = Vec::new();
    for row in b {
        if let Some(g) = row["genreLabel"]["value"].as_str() {
            let g = g.to_string();
            if !genres.contains(&g) {
                genres.push(g);
            }
        }
        if let Some(i) = row["isrc"]["value"].as_str() {
            let i = i.to_string();
            if !known_isrcs.contains(&i) {
                known_isrcs.push(i);
            }
        }
    }
    let a = WikidataArtist {
        qid,
        wikidata_url,
        musicbrainz_id: ext("mbid"),
        label_name: ext("labelLabel"),
        label_qid: ext("label").and_then(|u| u.rsplit('/').next().map(|s| s.into())),
        country: ext("countryLabel"),
        genres,
        website: ext("website"),
        known_isrcs,
    };
    info!(artist=%name, qid=?a.qid, "Wikidata enriched");
    Ok(a)
}

pub async fn isrc_exists(isrc: &str) -> bool {
    let query = format!(
        r#"ASK {{ ?item wdt:P1243 "{}" }}"#,
        isrc.replace('"', "\\\"")
    );
    #[derive(Deserialize)]
    struct AskResp {
        boolean: bool,
    }
    let client = reqwest::Client::builder()
        .user_agent(UA)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();
    match client
        .get(SPARQL)
        .query(&[("query", &query), ("format", &"json".to_string())])
        .send()
        .await
    {
        Ok(r) => r
            .json::<AskResp>()
            .await
            .map(|a| a.boolean)
            .unwrap_or(false),
        Err(_) => false,
    }
}
