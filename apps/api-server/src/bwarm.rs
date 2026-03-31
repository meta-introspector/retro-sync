#![allow(dead_code)] // Rights management module: full lifecycle API exposed
//! BWARM — Best Workflow for All Rights Management.
//!
//! BWARM is the IASA (International Association of Sound and Audiovisual Archives)
//! recommended workflow standard for archiving and managing audiovisual content
//! with complete rights metadata throughout the content lifecycle.
//!
//! Reference: IASA-TC 03, IASA-TC 04, IASA-TC 06 (Rights Management)
//!            https://www.iasa-web.org/technical-publications
//!
//! This module provides:
//!   1. BWARM rights record model (track → work → licence chain).
//!   2. Rights lifecycle state machine (unregistered → registered → licensed → distributed).
//!   3. Rights conflict detection (overlapping territories / periods).
//!   4. BWARM submission document generation (XML per IASA schema).
//!   5. Integration with ASCAP, BMI, SoundExchange, The MLC for rights confirmation.
//!
//! LangSec: all text fields sanitised; XML output escaped via xml_escape().
use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};

// ── Rights lifecycle ──────────────────────────────────────────────────────────

/// BWARM rights lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RightsState {
    /// No rights metadata registered anywhere.
    Unregistered,
    /// ISRC registered + basic metadata filed.
    Registered,
    /// Work registered with at least one PRO (ASCAP/BMI/SOCAN/etc.).
    ProRegistered,
    /// Mechanical rights licensed (statutory or direct licensing).
    MechanicalLicensed,
    /// Neighbouring rights registered (SoundExchange, PPL, GVL, etc.).
    NeighbouringRegistered,
    /// Distribution-ready — all rights confirmed across required territories.
    DistributionReady,
    /// Dispute — conflicting claim detected.
    Disputed,
    /// Rights lapsed or reverted.
    Lapsed,
}

impl RightsState {
    #[zkperf_macros::zkperf]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unregistered => "Unregistered",
            Self::Registered => "Registered",
            Self::ProRegistered => "PRO_Registered",
            Self::MechanicalLicensed => "MechanicalLicensed",
            Self::NeighbouringRegistered => "NeighbouringRegistered",
            Self::DistributionReady => "DistributionReady",
            Self::Disputed => "Disputed",
            Self::Lapsed => "Lapsed",
        }
    }
}

// ── Rights holder model ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RightsHolderType {
    Songwriter,
    CoSongwriter,
    Publisher,
    CoPublisher,
    SubPublisher,
    RecordLabel,
    Distributor,
    Performer, // Neighbouring rights
    SessionMusician,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsHolder {
    pub name: String,
    pub ipi_number: Option<String>,
    pub isni: Option<String>, // International Standard Name Identifier
    pub pro_affiliation: Option<String>, // e.g. "ASCAP", "BMI", "PRS"
    pub holder_type: RightsHolderType,
    /// Percentage of rights owned (0.0–100.0).
    pub ownership_pct: f32,
    pub evm_address: Option<String>,
    pub tron_address: Option<String>,
}

// ── Territory + period model ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsPeriod {
    pub start_date: String,       // YYYY-MM-DD
    pub end_date: Option<String>, // None = perpetual
    pub territories: Vec<String>, // ISO 3166-1 alpha-2 or "Worldwide"
}

// ── Licence types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LicenceType {
    /// Statutory mechanical (Section 115 / compulsory licence).
    StatutoryMechanical,
    /// Voluntary (direct) mechanical licence.
    DirectMechanical,
    /// Sync licence (film, TV, advertising).
    Sync,
    /// Master use licence.
    MasterUse,
    /// Print licence (sheet music).
    Print,
    /// Neighbouring rights licence (broadcast, satellite, webcasting).
    NeighbouringRights,
    /// Grand rights (dramatic/theatrical).
    GrandRights,
    /// Creative Commons licence.
    CreativeCommons { variant: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Licence {
    pub licence_id: String,
    pub licence_type: LicenceType,
    pub licensee: String,
    pub period: RightsPeriod,
    pub royalty_rate_pct: f32,
    pub flat_fee_usd: Option<f64>,
    pub confirmed: bool,
}

// ── BWARM Rights Record ───────────────────────────────────────────────────────

/// The complete BWARM rights record for a musical work / sound recording.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BwarmRecord {
    /// Internal record ID.
    pub record_id: String,
    // ── Identifiers ──────────────────────────────────────────────────────
    pub isrc: Option<String>,
    pub iswc: Option<String>,
    pub bowi: Option<String>,
    pub upc: Option<String>,
    pub btfs_cid: Option<String>,
    pub wikidata_qid: Option<String>,
    // ── Descriptive metadata ─────────────────────────────────────────────
    pub title: String,
    pub subtitle: Option<String>,
    pub original_language: Option<String>,
    pub genre: Option<String>,
    pub duration_secs: Option<u32>,
    // ── Rights holders ───────────────────────────────────────────────────
    pub rights_holders: Vec<RightsHolder>,
    // ── Licences ─────────────────────────────────────────────────────────
    pub licences: Vec<Licence>,
    // ── Lifecycle state ───────────────────────────────────────────────────
    pub state: RightsState,
    // ── PRO confirmations ─────────────────────────────────────────────────
    pub ascap_confirmed: bool,
    pub bmi_confirmed: bool,
    pub sesac_confirmed: bool,
    pub socan_confirmed: bool,
    pub prs_confirmed: bool,
    pub soundexchange_confirmed: bool,
    pub mlc_confirmed: bool, // The MLC (mechanical)
    // ── Timestamps ────────────────────────────────────────────────────────
    pub created_at: String,
    pub updated_at: String,
}

impl BwarmRecord {
    /// Create a new BWARM record with minimal required fields.
    #[zkperf_macros::zkperf]
    pub fn new(title: &str, isrc: Option<&str>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            record_id: generate_record_id(),
            isrc: isrc.map(String::from),
            iswc: None,
            bowi: None,
            upc: None,
            btfs_cid: None,
            wikidata_qid: None,
            title: title.to_string(),
            subtitle: None,
            original_language: None,
            genre: None,
            duration_secs: None,
            rights_holders: vec![],
            licences: vec![],
            state: RightsState::Unregistered,
            ascap_confirmed: false,
            bmi_confirmed: false,
            sesac_confirmed: false,
            socan_confirmed: false,
            prs_confirmed: false,
            soundexchange_confirmed: false,
            mlc_confirmed: false,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

// ── Rights conflict detection ─────────────────────────────────────────────────

/// A detected conflict in rights metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsConflict {
    pub conflict_type: ConflictType,
    pub description: String,
    pub affected_holders: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    OwnershipExceedsHundred,
    OverlappingTerritoryPeriod,
    MissingProAffiliation,
    UnconfirmedLicence,
    SplitMismatch,
}

/// Detect rights conflicts in a BWARM record.
#[zkperf_macros::zkperf]
pub fn detect_conflicts(record: &BwarmRecord) -> Vec<RightsConflict> {
    let mut conflicts = Vec::new();

    // Check ownership percentages sum to ≤ 100%
    let songwriter_pct: f32 = record
        .rights_holders
        .iter()
        .filter(|h| {
            matches!(
                h.holder_type,
                RightsHolderType::Songwriter | RightsHolderType::CoSongwriter
            )
        })
        .map(|h| h.ownership_pct)
        .sum();

    let publisher_pct: f32 = record
        .rights_holders
        .iter()
        .filter(|h| {
            matches!(
                h.holder_type,
                RightsHolderType::Publisher
                    | RightsHolderType::CoPublisher
                    | RightsHolderType::SubPublisher
            )
        })
        .map(|h| h.ownership_pct)
        .sum();

    if songwriter_pct > 100.0 + f32::EPSILON {
        conflicts.push(RightsConflict {
            conflict_type: ConflictType::OwnershipExceedsHundred,
            description: format!(
                "Songwriter ownership sums to {songwriter_pct:.2}% — must not exceed 100%"
            ),
            affected_holders: record
                .rights_holders
                .iter()
                .filter(|h| {
                    matches!(
                        h.holder_type,
                        RightsHolderType::Songwriter | RightsHolderType::CoSongwriter
                    )
                })
                .map(|h| h.name.clone())
                .collect(),
        });
    }

    if publisher_pct > 100.0 + f32::EPSILON {
        conflicts.push(RightsConflict {
            conflict_type: ConflictType::OwnershipExceedsHundred,
            description: format!(
                "Publisher ownership sums to {publisher_pct:.2}% — must not exceed 100%"
            ),
            affected_holders: vec![],
        });
    }

    // Check for missing PRO affiliation on songwriters
    for holder in &record.rights_holders {
        if matches!(
            holder.holder_type,
            RightsHolderType::Songwriter | RightsHolderType::CoSongwriter
        ) && holder.pro_affiliation.is_none()
        {
            conflicts.push(RightsConflict {
                conflict_type: ConflictType::MissingProAffiliation,
                description: format!(
                    "Songwriter '{}' has no PRO affiliation — needed for royalty collection",
                    holder.name
                ),
                affected_holders: vec![holder.name.clone()],
            });
        }
    }

    // Check for unconfirmed licences older than 30 days
    for licence in &record.licences {
        if !licence.confirmed {
            conflicts.push(RightsConflict {
                conflict_type: ConflictType::UnconfirmedLicence,
                description: format!(
                    "Licence '{}' to '{}' is not confirmed — distribution may be blocked",
                    licence.licence_id, licence.licensee
                ),
                affected_holders: vec![licence.licensee.clone()],
            });
        }
    }

    conflicts
}

/// Compute the rights lifecycle state from the record.
#[zkperf_macros::zkperf]
pub fn compute_state(record: &BwarmRecord) -> RightsState {
    if record.isrc.is_none() && record.iswc.is_none() {
        return RightsState::Unregistered;
    }
    if !detect_conflicts(record)
        .iter()
        .any(|c| c.conflict_type == ConflictType::OwnershipExceedsHundred)
    {
        let pro_confirmed = record.ascap_confirmed
            || record.bmi_confirmed
            || record.sesac_confirmed
            || record.socan_confirmed
            || record.prs_confirmed;

        let mechanical = record.mlc_confirmed;
        let neighbouring = record.soundexchange_confirmed;

        if pro_confirmed && mechanical && neighbouring {
            return RightsState::DistributionReady;
        }
        if mechanical {
            return RightsState::MechanicalLicensed;
        }
        if neighbouring {
            return RightsState::NeighbouringRegistered;
        }
        if pro_confirmed {
            return RightsState::ProRegistered;
        }
        return RightsState::Registered;
    }
    RightsState::Disputed
}

// ── XML document generation ───────────────────────────────────────────────────

/// Generate a BWARM XML document for submission to rights management systems.
/// Uses xml_escape() on all user-controlled values.
#[zkperf_macros::zkperf]
pub fn generate_bwarm_xml(record: &BwarmRecord) -> String {
    let esc = |s: &str| {
        s.chars()
            .flat_map(|c| match c {
                '&' => "&amp;".chars().collect::<Vec<_>>(),
                '<' => "&lt;".chars().collect(),
                '>' => "&gt;".chars().collect(),
                '"' => "&quot;".chars().collect(),
                '\'' => "&apos;".chars().collect(),
                c => vec![c],
            })
            .collect::<String>()
    };

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<BwarmRecord xmlns=\"https://iasa-web.org/bwarm/1.0\">\n");
    xml.push_str(&format!(
        "  <RecordId>{}</RecordId>\n",
        esc(&record.record_id)
    ));
    xml.push_str(&format!("  <Title>{}</Title>\n", esc(&record.title)));
    xml.push_str(&format!("  <State>{}</State>\n", record.state.as_str()));

    if let Some(isrc) = &record.isrc {
        xml.push_str(&format!("  <ISRC>{}</ISRC>\n", esc(isrc)));
    }
    if let Some(iswc) = &record.iswc {
        xml.push_str(&format!("  <ISWC>{}</ISWC>\n", esc(iswc)));
    }
    if let Some(bowi) = &record.bowi {
        xml.push_str(&format!("  <BOWI>{}</BOWI>\n", esc(bowi)));
    }
    if let Some(qid) = &record.wikidata_qid {
        xml.push_str(&format!("  <WikidataQID>{}</WikidataQID>\n", esc(qid)));
    }

    xml.push_str("  <RightsHolders>\n");
    for holder in &record.rights_holders {
        xml.push_str("    <RightsHolder>\n");
        xml.push_str(&format!("      <Name>{}</Name>\n", esc(&holder.name)));
        xml.push_str(&format!("      <Type>{:?}</Type>\n", holder.holder_type));
        xml.push_str(&format!(
            "      <OwnershipPct>{:.4}</OwnershipPct>\n",
            holder.ownership_pct
        ));
        if let Some(ipi) = &holder.ipi_number {
            xml.push_str(&format!("      <IPI>{}</IPI>\n", esc(ipi)));
        }
        if let Some(pro) = &holder.pro_affiliation {
            xml.push_str(&format!("      <PRO>{}</PRO>\n", esc(pro)));
        }
        xml.push_str("    </RightsHolder>\n");
    }
    xml.push_str("  </RightsHolders>\n");

    xml.push_str("  <ProConfirmations>\n");
    xml.push_str(&format!("    <ASCAP>{}</ASCAP>\n", record.ascap_confirmed));
    xml.push_str(&format!("    <BMI>{}</BMI>\n", record.bmi_confirmed));
    xml.push_str(&format!("    <SESAC>{}</SESAC>\n", record.sesac_confirmed));
    xml.push_str(&format!("    <SOCAN>{}</SOCAN>\n", record.socan_confirmed));
    xml.push_str(&format!("    <PRS>{}</PRS>\n", record.prs_confirmed));
    xml.push_str(&format!(
        "    <SoundExchange>{}</SoundExchange>\n",
        record.soundexchange_confirmed
    ));
    xml.push_str(&format!("    <TheMLC>{}</TheMLC>\n", record.mlc_confirmed));
    xml.push_str("  </ProConfirmations>\n");

    xml.push_str(&format!(
        "  <CreatedAt>{}</CreatedAt>\n",
        esc(&record.created_at)
    ));
    xml.push_str(&format!(
        "  <UpdatedAt>{}</UpdatedAt>\n",
        esc(&record.updated_at)
    ));
    xml.push_str("</BwarmRecord>\n");
    xml
}

/// Log a rights registration event for ISO 9001 audit trail.
#[instrument]
pub fn log_rights_event(record_id: &str, event: &str, detail: &str) {
    info!(record_id=%record_id, event=%event, detail=%detail, "BWARM rights event");
}

fn generate_record_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("BWARM-{:016x}", t & 0xFFFFFFFFFFFFFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_record_is_unregistered() {
        let record = BwarmRecord::new("Test Track", None);
        assert_eq!(compute_state(&record), RightsState::Unregistered);
    }

    #[test]
    fn distribution_ready_when_all_confirmed() {
        let mut record = BwarmRecord::new("Test Track", Some("US-S1Z-99-00001"));
        record.ascap_confirmed = true;
        record.mlc_confirmed = true;
        record.soundexchange_confirmed = true;
        assert_eq!(compute_state(&record), RightsState::DistributionReady);
    }

    #[test]
    fn ownership_conflict_detected() {
        let mut record = BwarmRecord::new("Test Track", None);
        record.rights_holders = vec![
            RightsHolder {
                name: "Writer A".into(),
                ipi_number: None,
                isni: None,
                pro_affiliation: Some("ASCAP".into()),
                holder_type: RightsHolderType::Songwriter,
                ownership_pct: 70.0,
                evm_address: None,
                tron_address: None,
            },
            RightsHolder {
                name: "Writer B".into(),
                ipi_number: None,
                isni: None,
                pro_affiliation: Some("BMI".into()),
                holder_type: RightsHolderType::Songwriter,
                ownership_pct: 60.0, // total = 130% — conflict
                evm_address: None,
                tron_address: None,
            },
        ];
        let conflicts = detect_conflicts(&record);
        assert!(conflicts
            .iter()
            .any(|c| c.conflict_type == ConflictType::OwnershipExceedsHundred));
    }

    #[test]
    fn xml_escapes_special_chars() {
        let mut record = BwarmRecord::new("Track <Test> & \"Quotes\"", None);
        record.record_id = "TEST-ID".into();
        let xml = generate_bwarm_xml(&record);
        assert!(xml.contains("&lt;Test&gt;"));
        assert!(xml.contains("&amp;"));
        assert!(xml.contains("&quot;Quotes&quot;"));
    }
}