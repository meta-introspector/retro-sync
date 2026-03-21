//! Alphabet Analysis — Parse A/B methods for title resonance.
//!
//! Parse A: direct letter value sum (A=1..Z=26)
//! Parse B: expanded digit sum of letter values (S=19 → 1+9=10)
//!
//! "RETROSYNC": Parse A=137→dr=2, Parse B=56→dr=2 → Band 2 (Legendary) ✓
use crate::master_pattern::{band_from_digit_root, digit_root};
use serde::{Deserialize, Serialize};

pub fn letter_value(c: char) -> u64 {
    if c.is_ascii_alphabetic() {
        (c.to_ascii_uppercase() as u64) - 64
    } else {
        0
    }
}

pub fn parse_a(text: &str) -> u64 {
    text.chars().map(letter_value).sum()
}

pub fn parse_b(text: &str) -> u64 {
    text.chars()
        .map(|c| {
            let v = letter_value(c);
            v.to_string()
                .chars()
                .filter_map(|d| d.to_digit(10))
                .map(|d| d as u64)
                .sum::<u64>()
        })
        .sum()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphabetAnalysis {
    pub text: String,
    pub parse_a_sum: u64,
    pub parse_a_dr: u64,
    pub parse_b_sum: u64,
    pub parse_b_dr: u64,
    pub band: u8,
    pub resonant_band: u8,
}

pub fn analyse(text: &str) -> AlphabetAnalysis {
    let a_sum = parse_a(text);
    let b_sum = parse_b(text);
    let a_dr = digit_root(a_sum);
    let b_dr = digit_root(b_sum);
    let band = band_from_digit_root(a_dr);
    AlphabetAnalysis {
        text: text.to_string(),
        parse_a_sum: a_sum,
        parse_a_dr: a_dr,
        parse_b_sum: b_sum,
        parse_b_dr: b_dr,
        band,
        resonant_band: band_from_digit_root(b_dr),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceReport {
    pub artist: AlphabetAnalysis,
    pub title: AlphabetAnalysis,
    pub track_band: u8,
    pub title_resonant: bool,
    pub artist_resonant: bool,
    pub full_resonance: bool,
}

pub fn resonance_report(artist: &str, title: &str, track_band: u8) -> ResonanceReport {
    let a = analyse(artist);
    let t = analyse(title);
    let title_resonant = t.band == track_band;
    let artist_resonant = a.band == track_band;
    ResonanceReport {
        artist: a,
        title: t,
        track_band,
        title_resonant,
        artist_resonant,
        full_resonance: title_resonant && artist_resonant,
    }
}

pub fn analyse_with_resonance(text: &str, _track_band: u8) -> AlphabetAnalysis {
    analyse(text)
}
