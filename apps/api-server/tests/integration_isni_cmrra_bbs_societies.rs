// Integration tests for ISNI, CMRRA, BBS, and Collection Societies modules.
// Run with: cargo test -p backend --test integration_isni_cmrra_bbs_societies
#![allow(dead_code)]

use backend::bbs::{self, BbsLicenceType, BroadcastCue};
use backend::cmrra::{self, CmrraStatementLine, CmrraUseType};
use backend::collection_societies::{self, RightType};
use backend::isni::{self, normalise_isni, validate_isni};

// ── ISNI ──────────────────────────────────────────────────────────────────────

#[test]
fn isni_valid_all_digits() {
    let result = validate_isni("0000000121500908");
    assert!(result.is_ok(), "valid ISNI must parse OK: {result:?}");
}

#[test]
fn isni_valid_with_spaces() {
    let result = validate_isni("0000 0001 2150 0908");
    assert!(result.is_ok());
}

#[test]
fn isni_valid_with_prefix() {
    let result = validate_isni("ISNI 0000000121500908");
    assert!(result.is_ok());
}

#[test]
fn isni_invalid_length_short() {
    let err = validate_isni("123").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("length") || msg.contains("3"),
        "error should mention length, got: {msg}"
    );
}

#[test]
fn isni_invalid_check_digit() {
    // Flip last digit so check digit fails MOD 11-2
    let err = validate_isni("0000000121500901").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("check") || msg.contains("digit") || msg.contains("invalid"),
        "error should mention check digit, got: {msg}"
    );
}

#[test]
fn isni_normalise_strips_prefix_and_spaces() {
    let norm = normalise_isni("ISNI 0000 0001 2150 0908");
    assert_eq!(norm, "0000000121500908");
}

#[test]
fn isni_normalise_strips_hyphens() {
    let norm = normalise_isni("0000-0001-2150-0908");
    assert_eq!(norm, "0000000121500908");
}

#[test]
fn isni_display_formatted() {
    let isni = validate_isni("0000000121500908").unwrap();
    let formatted = format!("{isni}");
    assert!(
        formatted.contains(' ') || formatted.contains("ISNI"),
        "formatted ISNI should have spaces or prefix: {formatted}"
    );
}

// ── CMRRA ─────────────────────────────────────────────────────────────────────

#[test]
fn cmrra_rates_physical_positive() {
    let rates = cmrra::current_canadian_rates();
    assert!(
        rates.physical_per_unit_cad_cents > 0.0,
        "physical rate (Tariff 22.A) must be positive, got {}",
        rates.physical_per_unit_cad_cents
    );
}

#[test]
fn cmrra_rates_download_positive() {
    let rates = cmrra::current_canadian_rates();
    assert!(
        rates.download_per_track_cad_cents > 0.0,
        "download rate (Tariff 22.D) must be positive, got {}",
        rates.download_per_track_cad_cents
    );
}

#[test]
fn cmrra_rates_streaming_positive() {
    let rates = cmrra::current_canadian_rates();
    assert!(
        rates.streaming_per_stream_cad_cents > 0.0,
        "streaming rate (Tariff 22.G) must be positive, got {}",
        rates.streaming_per_stream_cad_cents
    );
}

#[test]
fn cmrra_rates_board_reference_nonempty() {
    let rates = cmrra::current_canadian_rates();
    assert!(
        !rates.board_reference.is_empty(),
        "board_reference should reference Copyright Board order"
    );
}

#[test]
fn cmrra_csi_blanket_territories_nonempty() {
    let info = cmrra::csi_blanket_info();
    assert!(
        !info.territories.is_empty(),
        "CSI blanket must cover at least one territory"
    );
}

#[test]
fn cmrra_csi_blanket_minimum_positive() {
    let info = cmrra::csi_blanket_info();
    assert!(
        info.annual_minimum_cad > 0.0,
        "CSI blanket annual minimum must be positive, got {}",
        info.annual_minimum_cad
    );
}

#[test]
fn cmrra_generate_statement_csv_has_header() {
    let lines = vec![CmrraStatementLine {
        isrc: "CAXXX2300001".into(),
        title: "Test Track".into(),
        units: 1000,
        rate_cad_cents: 10.2,
        royalty_cad: 102.0,
        use_type: "PermanentDownload".into(),
        period: "2024Q1".into(),
    }];
    let csv = cmrra::generate_quarterly_csv(&lines);
    assert!(
        csv.contains("ISRC") || csv.to_uppercase().contains("ISRC"),
        "CSV must have an ISRC column header"
    );
    assert!(
        csv.contains("CAXXX2300001"),
        "CSV must contain the ISRC value"
    );
}

#[test]
fn cmrra_use_type_tariff_refs_nonempty() {
    let types = [
        CmrraUseType::PhysicalRecording,
        CmrraUseType::PermanentDownload,
        CmrraUseType::InteractiveStreaming,
        CmrraUseType::LimitedDownload,
        CmrraUseType::Ringtone,
        CmrraUseType::PrivateCopying,
    ];
    for t in &types {
        let r = t.tariff_ref();
        assert!(!r.is_empty(), "tariff_ref for {t:?} must not be empty");
    }
}

// ── BBS ───────────────────────────────────────────────────────────────────────

fn sample_cue() -> BroadcastCue {
    BroadcastCue {
        isrc: "GBAYE0601498".into(),
        iswc: Some("T-070.234.057-8".into()),
        title: "Let It Be".into(),
        artist: "The Beatles".into(),
        station_id: "BBC-RADIO-2".into(),
        territory: "GB".into(),
        played_at: chrono::Utc::now(),
        duration_secs: 243,
        use_type: BbsLicenceType::RadioBroadcast,
        featured: true,
    }
}

#[test]
fn bbs_validate_good_cue_returns_no_errors() {
    let cues = vec![sample_cue()];
    let errors = bbs::validate_cue_batch(&cues);
    assert!(
        errors.is_empty(),
        "valid cue should have no errors: {errors:?}"
    );
}

#[test]
fn bbs_validate_empty_batch_returns_error() {
    let errors = bbs::validate_cue_batch(&[]);
    assert!(
        !errors.is_empty(),
        "empty batch should return a validation error"
    );
}

#[test]
fn bbs_validate_duration_too_long() {
    let mut cue = sample_cue();
    cue.duration_secs = 7201;
    let errors = bbs::validate_cue_batch(&[cue]);
    assert!(
        errors
            .iter()
            .any(|e| e.field.contains("duration") || e.reason.contains("7200")),
        "should flag excessive duration: {errors:?}"
    );
}

#[test]
fn bbs_validate_bad_isrc_flagged() {
    let mut cue = sample_cue();
    cue.isrc = "NOT-AN-ISRC!!".into();
    let errors = bbs::validate_cue_batch(&[cue]);
    assert!(
        errors
            .iter()
            .any(|e| e.field.to_lowercase().contains("isrc")),
        "invalid ISRC should be flagged: {errors:?}"
    );
}

#[test]
fn bbs_validate_bad_territory_flagged() {
    let mut cue = sample_cue();
    cue.territory = "XYZ".into();
    let errors = bbs::validate_cue_batch(&[cue]);
    assert!(
        errors
            .iter()
            .any(|e| e.field.to_lowercase().contains("territory")),
        "invalid territory should be flagged: {errors:?}"
    );
}

#[test]
fn bbs_estimate_blanket_fee_positive() {
    let fee = bbs::estimate_blanket_fee(&BbsLicenceType::RadioBroadcast, "US", 2000.0);
    assert!(fee > 0.0, "estimated fee must be positive, got {fee}");
}

#[test]
fn bbs_estimate_blanket_fee_zero_hours_uses_clamp_floor() {
    let fee = bbs::estimate_blanket_fee(&BbsLicenceType::RadioBroadcast, "US", 0.0);
    assert!(
        fee > 0.0,
        "fee with 0 hours should use clamp floor and remain positive"
    );
}

#[test]
fn bbs_bmat_csv_contains_isrc() {
    let cues = vec![sample_cue()];
    let csv = bbs::generate_bmat_csv(&cues);
    assert!(
        csv.contains("GBAYE0601498"),
        "BMAT CSV must contain the cue ISRC"
    );
}

#[test]
fn bbs_licence_type_display_names_nonempty() {
    let types = [
        BbsLicenceType::BackgroundMusic,
        BbsLicenceType::RadioBroadcast,
        BbsLicenceType::TvBroadcast,
        BbsLicenceType::OnlineRadio,
        BbsLicenceType::Podcast,
        BbsLicenceType::Sync,
        BbsLicenceType::Cinema,
    ];
    for t in &types {
        let name = t.display_name();
        assert!(!name.is_empty(), "display_name for {t:?} must not be empty");
    }
}

// ── Collection Societies ──────────────────────────────────────────────────────

#[test]
fn societies_registry_has_minimum_count() {
    let all = collection_societies::all_societies();
    assert!(
        all.len() >= 50,
        "registry should have at least 50 societies, found {}",
        all.len()
    );
}

#[test]
fn societies_registry_contains_major_orgs() {
    let all = collection_societies::all_societies();
    let ids: Vec<&str> = all.iter().map(|s| s.id).collect();
    for major in &[
        "ASCAP", "BMI", "SESAC", "SOCAN", "PRS", "GEMA", "SACEM", "JASRAC",
    ] {
        assert!(ids.contains(major), "registry must contain {major}");
    }
}

#[test]
fn societies_by_id_ascap_found() {
    let s = collection_societies::society_by_id("ASCAP");
    assert!(s.is_some(), "ASCAP must be findable by ID");
    let s = s.unwrap();
    assert!(
        s.territories.contains(&"US"),
        "ASCAP should cover US territory"
    );
}

#[test]
fn societies_by_id_unknown_returns_none() {
    let s = collection_societies::society_by_id("TOTALLY_UNKNOWN_XYZ");
    assert!(s.is_none(), "unknown ID should return None");
}

#[test]
fn societies_for_territory_us_nonempty() {
    let list = collection_societies::societies_for_territory("US");
    assert!(
        !list.is_empty(),
        "US should have at least one collection society"
    );
}

#[test]
fn societies_for_territory_ca_has_socan() {
    let list = collection_societies::societies_for_territory("CA");
    let ids: Vec<&str> = list.iter().map(|s| s.id).collect();
    assert!(
        ids.contains(&"SOCAN"),
        "Canada should include SOCAN, found: {ids:?}"
    );
}

#[test]
fn societies_route_royalty_us_performance_nonempty() {
    let instructions = collection_societies::route_royalty(
        "US",
        RightType::Performance,
        100.0,
        Some("GBAYE0601498"),
        None,
    );
    assert!(
        !instructions.is_empty(),
        "should produce routing instructions for US performance"
    );
}

#[test]
fn societies_route_royalty_unknown_territory_does_not_panic() {
    let _instructions =
        collection_societies::route_royalty("ZZ", RightType::Mechanical, 50.0, None, None);
}
