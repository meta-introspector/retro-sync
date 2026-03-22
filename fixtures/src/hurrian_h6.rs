//! Hurrian Hymn h.6 — oldest surviving notated music (~1400 BC, Ugarit)
//!
//! Encodes the Babylonian interval notation through the Shem HaMephorash
//! SSP boustrophedon pipeline into Cl(15,0,0) and produces a DA51 CBOR shard.
//!
//! The 14 Babylonian interval terms map to the 15 supersingular primes.
//! The 15th prime (71) is the "crown" — the colophon/provenance dimension.
//!
//! Source: Dietrich & Loretz 1975, tablet RS 15.30 + 15.49 + 17.387
//! Tuning: nid qabli (nīd qablim)
//! Scribe: Ammurabi

use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// The 15 supersingular primes — generators of Cl(15,0,0).
pub const SSP: [u64; 15] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71];

/// Human-readable names for the 15 SSP slots (14 intervals + colophon).
pub const INTERVAL_NAMES: [&str; 15] = [
    "nis_tuhrim", "isartum", "embubum", "nid_qablim", "qablitum",
    "kitmum", "pitum", "serum", "salsatum", "rebuttum",
    "isqum", "titur_qablitim", "titur_isartim", "serdum", "colophon",
];

/// Babylonian interval terms → SSP index.
/// 7 primary (tuning names, fifths/fourths) + 7 secondary (thirds/sixths) = 14.
/// Index 14 (p=71) reserved for the colophon/provenance "crown" dimension.
///
/// String pairs from the Babylonian theoretical text (UET VI/3 899):
///   Primary (tuning names):
///     0: nīš tuḫrim   (1–5, fifth)   → p2
///     1: išartum       (2–6, fifth)   → p3
///     2: embūbum       (3–7, fifth)   → p5
///     3: nīd qablim   (4–1, fourth)  → p7   ← h.6 tuning
///     4: qablītum     (5–2, fourth)  → p11
///     5: kitmum        (6–3, fourth)  → p13
///     6: pītum         (7–4, fourth)  → p17
///   Secondary:
///     7: šērum         (7–5, third)   → p19
///     8: šalšatum      (1–6, third)   → p23
///     9: rebûttum      (2–7, sixth)   → p29
///    10: isqum         (1–3, third)   → p31
///    11: titur qablītim(2–4, third)   → p41
///    12: titur išartim (3–5, third)   → p47
///    13: ṣerdum        (4–6, third)   → p59
///    14: [colophon]    (provenance)   → p71
pub fn interval_to_ssp_index(term: &str) -> Option<usize> {
    match term {
        "nis_tuhrim" | "nish" => Some(0),
        "isartu" | "isharte" | "isarte" => Some(1),
        "embubum" | "embubu" => Some(2),
        "nid_qablim" | "qablite" => Some(3),
        "qablitum" | "qabli" => Some(4),
        "kitmum" | "kitmu" => Some(5),
        "pitum" | "pitu" => Some(6),
        "serum" | "sahri" => Some(7),
        "salsatum" | "sassate" => Some(8),
        "rebutum" | "irbute" => Some(9),
        "isqum" => Some(10),
        "titur_qablitim" | "titimisarte" | "titimisharte" => Some(11),
        "titur_isartim" => Some(12),
        "serdum" | "zirte" => Some(13),
        "colophon" | "ustamari" => Some(14),
        _ => None,
    }
}

/// A single notation entry: interval name + repetition count.
#[derive(Clone, Debug)]
pub struct NotationEntry {
    pub term: String,
    pub count: u8,
}

/// First two lines of h.6 notation (Dietrich & Loretz 1975 transcription).
///
/// Line 1: qáb-li-te 3 ir-bu-te 1 qáb-li-te 3 ša-aḫ-ri 1 i-šar-te 10 ušta-ma-a-ri
/// Line 2: ti-ti-mi-šar-te 2 zi-ir-te 1 ša-aḫ-ri 2 ša-aš-ša-te 2 ir-bu-te 2
pub fn h6_notation() -> Vec<NotationEntry> {
    vec![
        NotationEntry { term: "qablite".into(), count: 3 },
        NotationEntry { term: "irbute".into(), count: 1 },
        NotationEntry { term: "qablite".into(), count: 3 },
        NotationEntry { term: "sahri".into(), count: 1 },
        NotationEntry { term: "isarte".into(), count: 10 },
        NotationEntry { term: "ustamari".into(), count: 1 },
        // Line 2
        NotationEntry { term: "titimisarte".into(), count: 2 },
        NotationEntry { term: "zirte".into(), count: 1 },
        NotationEntry { term: "sahri".into(), count: 2 },
        NotationEntry { term: "sassate".into(), count: 2 },
        NotationEntry { term: "irbute".into(), count: 2 },
    ]
}

/// Colophon metadata — the 1400 BC equivalent of DDEX/ISRC.
pub fn h6_colophon() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("tuning".into(), "nid_qablim".into());
    m.insert("genre".into(), "zaluzi".into());
    m.insert("scribe".into(), "Ammurabi".into());
    m.insert("deity".into(), "Nikkal".into());
    m.insert("instrument".into(), "sammum_9string".into());
    m.insert("tablet".into(), "RS_15.30+15.49+17.387".into());
    m.insert("catalogue".into(), "h.6".into());
    m.insert("site".into(), "Ugarit".into());
    m.insert("date_approx".into(), "-1400".into());
    m
}

/// Embed a notation sequence into Cl(15,0,0) via SSP boustrophedon.
///
/// Each interval term maps to an SSP generator eᵢ.
/// The count is the coefficient.
/// Boustrophedon: arrange entries in 3 rows, read columns alternating direction.
pub fn embed_h6() -> EigenspaceResult {
    let notation = h6_notation();

    // Expand: each entry becomes `count` copies of its SSP index
    let expanded: Vec<usize> = notation
        .iter()
        .filter_map(|e| {
            interval_to_ssp_index(&e.term).map(|idx| vec![idx; e.count as usize])
        })
        .flatten()
        .collect();

    // Arrange into 3 rows for boustrophedon (pad with colophon=14 if needed)
    let row_len = (expanded.len() + 2) / 3;
    let mut rows = vec![vec![14usize; row_len]; 3];
    for (i, &val) in expanded.iter().enumerate() {
        rows[i / row_len][i % row_len] = val;
    }

    // Boustrophedon extraction: row0[i], row1[n-1-i], row2[i]
    let n = row_len;
    let triplets: Vec<[usize; 3]> = (0..n)
        .map(|i| [rows[0][i], rows[1][n - 1 - i], rows[2][i]])
        .collect();

    // Grade accumulator: for each triplet, the geometric product of 3 basis
    // vectors eₐ·eᵦ·eᵧ has grade = number of distinct indices.
    let mut grade_energy = [0.0f64; 16];
    for tri in &triplets {
        let mut blade: u16 = 0;
        for &idx in tri {
            blade ^= 1u16 << idx;
        }
        let grade = blade.count_ones() as usize;
        grade_energy[grade] += 1.0;
    }

    // Eigenspace decomposition: Earth (0-5), Spoke (6-10), Hub (11-15)
    let earth: f64 = grade_energy[..6].iter().sum();
    let spoke: f64 = grade_energy[6..11].iter().sum();
    let hub: f64 = grade_energy[11..].iter().sum();
    let total = earth + spoke + hub;

    let (ep, sp, hp) = if total > 0.0 {
        (earth / total * 100.0, spoke / total * 100.0, hub / total * 100.0)
    } else {
        (0.0, 0.0, 0.0)
    };

    // FRACTRAN state: product of primes raised to grade counts
    let fractran_state: u128 = grade_energy
        .iter()
        .enumerate()
        .filter(|(_, &e)| e > 0.0)
        .fold(1u128, |acc, (i, &e)| {
            let p = if i < 15 { SSP[i] } else { 71 } as u128;
            acc.saturating_mul(p.saturating_pow(e as u32))
        });

    EigenspaceResult {
        triplet_count: triplets.len(),
        grade_energy,
        earth_pct: ep,
        spoke_pct: sp,
        hub_pct: hp,
        fractran_state,
    }
}

#[derive(Debug)]
pub struct EigenspaceResult {
    pub triplet_count: usize,
    pub grade_energy: [f64; 16],
    pub earth_pct: f64,
    pub spoke_pct: f64,
    pub hub_pct: f64,
    pub fractran_state: u128,
}

/// Produce a DA51 CBOR shard from the h.6 analysis.
pub fn h6_shard_cbor() -> Vec<u8> {
    let result = embed_h6();
    let colophon = h6_colophon();

    let shard = serde_json::json!({
        "type": "da51-shard",
        "version": "0.1.0",
        "source": {
            "title": "Hurrian Hymn h.6 — Hymn to Nikkal",
            "date": "-1400",
            "site": "Ugarit (Ras Shamra, Syria)",
            "tablet": "RS 15.30 + 15.49 + 17.387",
            "catalogue": "h.6 (Laroche)",
            "scribe": colophon.get("scribe").unwrap(),
            "tuning": colophon.get("tuning").unwrap(),
            "genre": colophon.get("genre").unwrap(),
            "instrument": colophon.get("instrument").unwrap(),
            "deity": colophon.get("deity").unwrap(),
        },
        "algebra": "Cl(15,0,0)",
        "pipeline": "shem-hamephorash-ssp-boustrophedon",
        "eigenspace": {
            "earth_pct": result.earth_pct,
            "spoke_pct": result.spoke_pct,
            "hub_pct": result.hub_pct,
            "triplet_count": result.triplet_count,
        },
        "fractran_state": format!("{}", result.fractran_state),
        "grade_energy": result.grade_energy.to_vec(),
        "colophon": colophon,
    });

    let json_bytes = serde_json::to_vec(&shard).unwrap_or_default();

    // DA51 CBOR envelope
    let mut cbor = Vec::new();
    // Magic: 0xda51 prefix
    cbor.extend_from_slice(&[0xda, 0x51]);
    // SHA-256 of payload (first 8 bytes as CID stub)
    let hash = Sha256::digest(&json_bytes);
    cbor.extend_from_slice(&hash[..8]);
    // CBOR-wrapped JSON payload
    let _ = ciborium::into_writer(&serde_json::from_slice::<ciborium::Value>(&json_bytes)
        .unwrap_or(ciborium::Value::Null), &mut cbor);
    cbor
}
