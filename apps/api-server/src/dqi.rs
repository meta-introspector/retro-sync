//! DQI — Data Quality Initiative.
//!
//! The Data Quality Initiative (DQI) is a joint DDEX / IFPI / RIAA / ARIA
//! programme that scores sound recording metadata quality and flags records
//! that fail to meet delivery standards required by DSPs, PROs, and the MLC.
//!
//! Reference: DDEX Data Quality Initiative v2.0 (2022)
//!            https://ddex.net/implementation/data-quality-initiative/
//!
//! Scoring model:
//!   Each field is scored 0 (absent/invalid) or 1 (present/valid).
//!   The total score is expressed as a percentage of the maximum possible score.
//!   DQI tiers:
//!     Gold   ≥ 90%  — all DSPs will accept; DDEX-ready
//!     Silver ≥ 70%  — accepted by most DSPs with caveats
//!     Bronze ≥ 50%  — accepted by some DSPs; PRO delivery may fail
//!     Below  < 50%  — reject at ingestion; require remediation
//!
//! LangSec: DQI scores are always server-computed — never trusted from client.
use crate::langsec;
use serde::{Deserialize, Serialize};
use tracing::info;

// ── DQI Field definitions ─────────────────────────────────────────────────────

/// A single DQI field and its score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DqiField {
    pub field_name: String,
    pub weight: u8, // 1–5 (5 = critical)
    pub score: u8,  // 0 or weight (present & valid = weight, else 0)
    pub present: bool,
    pub valid: bool,
    pub note: Option<String>,
}

/// DQI quality tier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum DqiTier {
    Gold,
    Silver,
    Bronze,
    BelowBronze,
}

impl DqiTier {
    #[zkperf_macros::zkperf]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gold => "Gold",
            Self::Silver => "Silver",
            Self::Bronze => "Bronze",
            Self::BelowBronze => "BelowBronze",
        }
    }
}

/// Full DQI report for a track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DqiReport {
    pub isrc: String,
    pub score_pct: f32,
    pub tier: DqiTier,
    pub max_score: u32,
    pub earned_score: u32,
    pub fields: Vec<DqiField>,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

// ── Metadata input for DQI evaluation ─────────────────────────────────────────

/// All metadata fields that DQI evaluates.
#[derive(Debug, Clone, Deserialize)]
pub struct DqiInput {
    // Required / critical (weight 5)
    pub isrc: Option<String>,
    pub title: Option<String>,
    pub primary_artist: Option<String>,
    pub label_name: Option<String>,
    // Core (weight 4)
    pub iswc: Option<String>,
    pub ipi_number: Option<String>,
    pub songwriter_name: Option<String>,
    pub publisher_name: Option<String>,
    pub release_date: Option<String>,
    pub territory: Option<String>,
    // Standard (weight 3)
    pub upc: Option<String>,
    pub bowi: Option<String>,
    pub wikidata_qid: Option<String>,
    pub genre: Option<String>,
    pub language: Option<String>,
    pub duration_secs: Option<u32>,
    // Enhanced (weight 2)
    pub featured_artists: Option<String>,
    pub catalogue_number: Option<String>,
    pub p_line: Option<String>, // ℗ line
    pub c_line: Option<String>, // © line
    pub original_release_date: Option<String>,
    // Supplementary (weight 1)
    pub bpm: Option<f32>,
    pub key_signature: Option<String>,
    pub explicit_content: Option<bool>,
    pub btfs_cid: Option<String>,
    pub musicbrainz_id: Option<String>,
}

// ── DQI evaluation engine ─────────────────────────────────────────────────────

/// Evaluate a track's metadata and return a DQI report.
#[zkperf_macros::zkperf]
pub fn evaluate(input: &DqiInput) -> DqiReport {
    let mut fields: Vec<DqiField> = Vec::new();
    let mut issues: Vec<String> = Vec::new();
    let mut recommendations: Vec<String> = Vec::new();

    // ── Critical fields (weight 5) ────────────────────────────────────────
    fields.push(eval_field_with_validator(
        "ISRC",
        5,
        &input.isrc,
        |v| shared::parsers::recognize_isrc(v).is_ok(),
        Some("ISRC is mandatory for DSP delivery and PRO registration"),
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_free_text_field(
        "Track Title",
        5,
        &input.title,
        500,
        Some("Title is required for all delivery channels"),
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_free_text_field(
        "Primary Artist",
        5,
        &input.primary_artist,
        500,
        Some("Primary artist required for artist-level royalty calculation"),
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_free_text_field(
        "Label Name",
        5,
        &input.label_name,
        500,
        Some("Label name required for publishing agreements"),
        &mut issues,
        &mut recommendations,
    ));

    // ── Core fields (weight 4) ────────────────────────────────────────────
    fields.push(eval_field_with_validator(
        "ISWC",
        4,
        &input.iswc,
        |v| {
            // ISWC: T-000.000.000-C (15 chars)
            v.len() == 15
                && v.starts_with("T-")
                && v.chars().filter(|c| c.is_ascii_digit()).count() == 10
        },
        Some("ISWC required for PRO registration (ASCAP, BMI, SOCAN, etc.)"),
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_field_with_validator(
        "IPI Number",
        4,
        &input.ipi_number,
        |v| v.len() == 11 && v.chars().all(|c| c.is_ascii_digit()),
        Some("IPI required for songwriter/publisher identification at PROs"),
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_free_text_field(
        "Songwriter Name",
        4,
        &input.songwriter_name,
        500,
        Some("Songwriter name required for CWR and PRO registration"),
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_free_text_field(
        "Publisher Name",
        4,
        &input.publisher_name,
        500,
        Some("Publisher name required for mechanical royalty distribution"),
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_date_field(
        "Release Date",
        4,
        &input.release_date,
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_field_with_validator(
        "Territory",
        4,
        &input.territory,
        |v| v == "Worldwide" || (v.len() == 2 && v.chars().all(|c| c.is_ascii_uppercase())),
        Some("Territory (ISO 3166-1 alpha-2 or 'Worldwide') required for licensing"),
        &mut issues,
        &mut recommendations,
    ));

    // ── Standard fields (weight 3) ────────────────────────────────────────
    fields.push(eval_field_with_validator(
        "UPC",
        3,
        &input.upc,
        |v| {
            let digits: String = v.chars().filter(|c| c.is_ascii_digit()).collect();
            digits.len() == 12 || digits.len() == 13
        },
        Some("UPC/EAN required for physical/digital release identification"),
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_field_with_validator(
        "BOWI",
        3,
        &input.bowi,
        |v| v.starts_with("bowi:") && v.len() == 41,
        Some("BOWI (Best Open Work Identifier) recommended for open metadata interoperability"),
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_field_with_validator(
        "Wikidata QID",
        3,
        &input.wikidata_qid,
        |v| v.starts_with('Q') && v[1..].chars().all(|c| c.is_ascii_digit()),
        Some("Wikidata QID links to artist's knowledge graph entry (improves DSP discoverability)"),
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_free_text_field(
        "Genre",
        3,
        &input.genre,
        100,
        None,
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_field_with_validator(
        "Language (BCP-47)",
        3,
        &input.language,
        |v| {
            v.len() >= 2
                && v.len() <= 35
                && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        },
        Some("BCP-47 language code improves metadata matching at PROs and DSPs"),
        &mut issues,
        &mut recommendations,
    ));

    fields.push(eval_field_with_validator(
        "Duration",
        3,
        &input.duration_secs.as_ref().map(|d| d.to_string()),
        |v| v.parse::<u32>().map(|d| d > 0 && d < 7200).unwrap_or(false),
        Some("Duration (seconds) required for DDEX ERN and DSP ingestion"),
        &mut issues,
        &mut recommendations,
    ));

    // ── Enhanced fields (weight 2) ────────────────────────────────────────
    fields.push(eval_optional_text(
        "Featured Artists",
        2,
        &input.featured_artists,
    ));
    fields.push(eval_optional_text(
        "Catalogue Number",
        2,
        &input.catalogue_number,
    ));
    fields.push(eval_optional_text("℗ Line", 2, &input.p_line));
    fields.push(eval_optional_text("© Line", 2, &input.c_line));
    fields.push(eval_date_field(
        "Original Release Date",
        2,
        &input.original_release_date,
        &mut issues,
        &mut recommendations,
    ));

    // ── Supplementary fields (weight 1) ──────────────────────────────────
    fields.push(eval_optional_text(
        "BPM",
        1,
        &input.bpm.as_ref().map(|b| b.to_string()),
    ));
    fields.push(eval_optional_text("Key Signature", 1, &input.key_signature));
    fields.push(eval_optional_text(
        "Explicit Flag",
        1,
        &input.explicit_content.as_ref().map(|b| b.to_string()),
    ));
    fields.push(eval_optional_text("BTFS CID", 1, &input.btfs_cid));
    fields.push(eval_optional_text(
        "MusicBrainz ID",
        1,
        &input.musicbrainz_id,
    ));

    // ── Scoring ───────────────────────────────────────────────────────────
    let max_score: u32 = fields.iter().map(|f| f.weight as u32).sum();
    let earned_score: u32 = fields.iter().map(|f| f.score as u32).sum();
    let score_pct = (earned_score as f32 / max_score as f32) * 100.0;

    let tier = match score_pct {
        p if p >= 90.0 => DqiTier::Gold,
        p if p >= 70.0 => DqiTier::Silver,
        p if p >= 50.0 => DqiTier::Bronze,
        _ => DqiTier::BelowBronze,
    };

    let isrc = input.isrc.clone().unwrap_or_else(|| "UNKNOWN".into());
    info!(isrc=%isrc, score_pct, tier=%tier.as_str(), "DQI evaluation");

    DqiReport {
        isrc,
        score_pct,
        tier,
        max_score,
        earned_score,
        fields,
        issues,
        recommendations,
    }
}

// ── Field evaluators ──────────────────────────────────────────────────────────

fn eval_field_with_validator<F>(
    name: &str,
    weight: u8,
    value: &Option<String>,
    validator: F,
    issue_text: Option<&str>,
    issues: &mut Vec<String>,
    recommendations: &mut Vec<String>,
) -> DqiField
where
    F: Fn(&str) -> bool,
{
    match value.as_deref() {
        None | Some("") => {
            if let Some(text) = issue_text {
                issues.push(format!("Missing: {name} — {text}"));
                recommendations.push(format!("Add {name} to improve DQI score"));
            }
            DqiField {
                field_name: name.to_string(),
                weight,
                score: 0,
                present: false,
                valid: false,
                note: issue_text.map(String::from),
            }
        }
        Some(v) if v.trim().is_empty() => {
            if let Some(text) = issue_text {
                issues.push(format!("Missing: {name} — {text}"));
                recommendations.push(format!("Add {name} to improve DQI score"));
            }
            DqiField {
                field_name: name.to_string(),
                weight,
                score: 0,
                present: false,
                valid: false,
                note: issue_text.map(String::from),
            }
        }
        Some(v) => {
            let valid = validator(v.trim());
            if !valid {
                issues.push(format!("Invalid: {name} — value '{v}' failed format check"));
            }
            DqiField {
                field_name: name.to_string(),
                weight,
                score: if valid { weight } else { 0 },
                present: true,
                valid,
                note: if valid {
                    None
                } else {
                    Some(format!("Value '{v}' is invalid"))
                },
            }
        }
    }
}

fn eval_free_text_field(
    name: &str,
    weight: u8,
    value: &Option<String>,
    max_len: usize,
    issue_text: Option<&str>,
    issues: &mut Vec<String>,
    recommendations: &mut Vec<String>,
) -> DqiField {
    eval_field_with_validator(
        name,
        weight,
        value,
        |v| !v.trim().is_empty() && langsec::validate_free_text(v, name, max_len).is_ok(),
        issue_text,
        issues,
        recommendations,
    )
}

fn eval_date_field(
    name: &str,
    weight: u8,
    value: &Option<String>,
    issues: &mut Vec<String>,
    recommendations: &mut Vec<String>,
) -> DqiField {
    eval_field_with_validator(
        name,
        weight,
        value,
        |v| {
            let parts: Vec<&str> = v.split('-').collect();
            parts.len() == 3
                && parts[0].len() == 4
                && parts[1].len() == 2
                && parts[2].len() == 2
                && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
        },
        None,
        issues,
        recommendations,
    )
}

fn eval_optional_text(name: &str, weight: u8, value: &Option<String>) -> DqiField {
    let present = value
        .as_ref()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    DqiField {
        field_name: name.to_string(),
        weight,
        score: if present { weight } else { 0 },
        present,
        valid: present,
        note: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gold_input() -> DqiInput {
        DqiInput {
            isrc: Some("US-S1Z-99-00001".into()),
            title: Some("Perfect Track".into()),
            primary_artist: Some("Perfect Artist".into()),
            label_name: Some("Perfect Label".into()),
            iswc: Some("T-000.000.001-C".into()),
            ipi_number: Some("00000000000".into()),
            songwriter_name: Some("Jane Songwriter".into()),
            publisher_name: Some("Perfect Publishing".into()),
            release_date: Some("2024-03-15".into()),
            territory: Some("Worldwide".into()),
            upc: Some("123456789012".into()),
            bowi: Some("bowi:12345678-1234-4234-b234-123456789012".into()),
            wikidata_qid: Some("Q123456".into()),
            genre: Some("Electronic".into()),
            language: Some("en".into()),
            duration_secs: Some(210),
            featured_artists: Some("Featured One".into()),
            catalogue_number: Some("CAT-001".into()),
            p_line: Some("℗ 2024 Perfect Label".into()),
            c_line: Some("© 2024 Perfect Publishing".into()),
            original_release_date: Some("2024-03-15".into()),
            bpm: Some(120.0),
            key_signature: Some("Am".into()),
            explicit_content: Some(false),
            btfs_cid: Some("QmTest".into()),
            musicbrainz_id: Some("mbid-test".into()),
        }
    }

    #[test]
    fn gold_tier_achieved() {
        let report = evaluate(&gold_input());
        assert_eq!(report.tier, DqiTier::Gold, "score: {}%", report.score_pct);
    }

    #[test]
    fn below_bronze_for_empty() {
        let report = evaluate(&DqiInput {
            isrc: None,
            title: None,
            primary_artist: None,
            label_name: None,
            iswc: None,
            ipi_number: None,
            songwriter_name: None,
            publisher_name: None,
            release_date: None,
            territory: None,
            upc: None,
            bowi: None,
            wikidata_qid: None,
            genre: None,
            language: None,
            duration_secs: None,
            featured_artists: None,
            catalogue_number: None,
            p_line: None,
            c_line: None,
            original_release_date: None,
            bpm: None,
            key_signature: None,
            explicit_content: None,
            btfs_cid: None,
            musicbrainz_id: None,
        });
        assert_eq!(report.tier, DqiTier::BelowBronze);
    }

    #[test]
    fn invalid_isrc_penalised() {
        let mut input = gold_input();
        input.isrc = Some("INVALID".into());
        let report = evaluate(&input);
        let isrc_field = report
            .fields
            .iter()
            .find(|f| f.field_name == "ISRC")
            .unwrap();
        assert!(!isrc_field.valid);
        assert_eq!(isrc_field.score, 0);
    }
}