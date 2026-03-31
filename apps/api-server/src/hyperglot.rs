#![allow(dead_code)] // Script detection module: full language validation API exposed
//! Hyperglot — Unicode script and language detection for multilingual metadata.
//!
//! Implements ISO 15924 script code detection using pure-Rust Unicode ranges.
//! Hyperglot (https://hyperglot.rosettatype.com) identifies languages from
//! writing systems; this module provides the same service without spawning
//! an external Python process.
//!
//! LangSec:
//!   All inputs are length-bounded (max 4096 codepoints) before scanning.
//!   Script detection is done via Unicode block ranges — no regex, no exec().
//!
//! Usage:
//!   let result = detect_scripts("Hello мир 日本語");
//!   // → [Latin (95%), Cyrillic (3%), CJK (2%)]
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// ISO 15924 script identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Script {
    Latin,
    Cyrillic,
    Arabic,
    Hebrew,
    Devanagari,
    Bengali,
    Gurmukhi,
    Gujarati,
    Tamil,
    Telugu,
    Kannada,
    Malayalam,
    Sinhala,
    Thai,
    Lao,
    Tibetan,
    Myanmar,
    Khmer,
    CjkUnified, // Han ideographs
    Hiragana,
    Katakana,
    Hangul,
    Greek,
    Georgian,
    Armenian,
    Ethiopic,
    Cherokee,
    Canadian, // Unified Canadian Aboriginal Syllabics
    Runic,
    Ogham,
    Common, // Digits, punctuation — script-neutral
    Unknown,
}

impl Script {
    /// ISO 15924 4-letter code.
    #[zkperf_macros::zkperf]
    pub fn iso_code(&self) -> &'static str {
        match self {
            Self::Latin => "Latn",
            Self::Cyrillic => "Cyrl",
            Self::Arabic => "Arab",
            Self::Hebrew => "Hebr",
            Self::Devanagari => "Deva",
            Self::Bengali => "Beng",
            Self::Gurmukhi => "Guru",
            Self::Gujarati => "Gujr",
            Self::Tamil => "Taml",
            Self::Telugu => "Telu",
            Self::Kannada => "Knda",
            Self::Malayalam => "Mlym",
            Self::Sinhala => "Sinh",
            Self::Thai => "Thai",
            Self::Lao => "Laoo",
            Self::Tibetan => "Tibt",
            Self::Myanmar => "Mymr",
            Self::Khmer => "Khmr",
            Self::CjkUnified => "Hani",
            Self::Hiragana => "Hira",
            Self::Katakana => "Kana",
            Self::Hangul => "Hang",
            Self::Greek => "Grek",
            Self::Georgian => "Geor",
            Self::Armenian => "Armn",
            Self::Ethiopic => "Ethi",
            Self::Cherokee => "Cher",
            Self::Canadian => "Cans",
            Self::Runic => "Runr",
            Self::Ogham => "Ogam",
            Self::Common => "Zyyy",
            Self::Unknown => "Zzzz",
        }
    }

    /// Human-readable English name for logging / metadata.
    #[zkperf_macros::zkperf]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Latin => "Latin",
            Self::Cyrillic => "Cyrillic",
            Self::Arabic => "Arabic",
            Self::Hebrew => "Hebrew",
            Self::Devanagari => "Devanagari",
            Self::Bengali => "Bengali",
            Self::Gurmukhi => "Gurmukhi",
            Self::Gujarati => "Gujarati",
            Self::Tamil => "Tamil",
            Self::Telugu => "Telugu",
            Self::Kannada => "Kannada",
            Self::Malayalam => "Malayalam",
            Self::Sinhala => "Sinhala",
            Self::Thai => "Thai",
            Self::Lao => "Lao",
            Self::Tibetan => "Tibetan",
            Self::Myanmar => "Myanmar",
            Self::Khmer => "Khmer",
            Self::CjkUnified => "CJK Unified Ideographs",
            Self::Hiragana => "Hiragana",
            Self::Katakana => "Katakana",
            Self::Hangul => "Hangul",
            Self::Greek => "Greek",
            Self::Georgian => "Georgian",
            Self::Armenian => "Armenian",
            Self::Ethiopic => "Ethiopic",
            Self::Cherokee => "Cherokee",
            Self::Canadian => "Canadian Aboriginal Syllabics",
            Self::Runic => "Runic",
            Self::Ogham => "Ogham",
            Self::Common => "Common (Neutral)",
            Self::Unknown => "Unknown",
        }
    }

    /// Writing direction.
    #[zkperf_macros::zkperf]
    pub fn is_rtl(&self) -> bool {
        matches!(self, Self::Arabic | Self::Hebrew)
    }
}

/// Map a Unicode codepoint to its ISO 15924 script using block ranges.
/// Source: Unicode 15.1 script assignment tables (chapter 4, Unicode standard).
fn codepoint_to_script(c: char) -> Script {
    let u = c as u32;
    match u {
        // Basic Latin (A-Z, a-z only) + Latin Extended
        // NOTE: 0x005B..=0x0060 (`[`, `\`, `]`, `^`, `_`, `` ` ``) are Common, not Latin.
        0x0041..=0x005A
        | 0x0061..=0x007A
        | 0x00C0..=0x024F
        | 0x0250..=0x02AF
        | 0x1D00..=0x1D7F
        | 0xFB00..=0xFB06 => Script::Latin,

        // Cyrillic
        0x0400..=0x04FF | 0x0500..=0x052F | 0x2DE0..=0x2DFF | 0xA640..=0xA69F => Script::Cyrillic,

        // Greek
        0x0370..=0x03FF | 0x1F00..=0x1FFF => Script::Greek,

        // Arabic
        0x0600..=0x06FF
        | 0x0750..=0x077F
        | 0xFB50..=0xFDFF
        | 0xFE70..=0xFEFF
        | 0x10E60..=0x10E7F => Script::Arabic,

        // Hebrew
        0x0590..=0x05FF | 0xFB1D..=0xFB4F => Script::Hebrew,

        // Devanagari (Hindi, Sanskrit, Marathi, Nepali…)
        0x0900..=0x097F | 0xA8E0..=0xA8FF => Script::Devanagari,

        // Bengali
        0x0980..=0x09FF => Script::Bengali,

        // Gurmukhi (Punjabi)
        0x0A00..=0x0A7F => Script::Gurmukhi,

        // Gujarati
        0x0A80..=0x0AFF => Script::Gujarati,

        // Tamil
        0x0B80..=0x0BFF => Script::Tamil,

        // Telugu
        0x0C00..=0x0C7F => Script::Telugu,

        // Kannada
        0x0C80..=0x0CFF => Script::Kannada,

        // Malayalam
        0x0D00..=0x0D7F => Script::Malayalam,

        // Sinhala
        0x0D80..=0x0DFF => Script::Sinhala,

        // Thai
        0x0E00..=0x0E7F => Script::Thai,

        // Lao
        0x0E80..=0x0EFF => Script::Lao,

        // Tibetan
        0x0F00..=0x0FFF => Script::Tibetan,

        // Myanmar
        0x1000..=0x109F | 0xA9E0..=0xA9FF | 0xAA60..=0xAA7F => Script::Myanmar,

        // Khmer
        0x1780..=0x17FF | 0x19E0..=0x19FF => Script::Khmer,

        // Georgian
        0x10A0..=0x10FF | 0x2D00..=0x2D2F => Script::Georgian,

        // Armenian
        0x0530..=0x058F | 0xFB13..=0xFB17 => Script::Armenian,

        // Ethiopic
        0x1200..=0x137F | 0x1380..=0x139F | 0x2D80..=0x2DDF | 0xAB01..=0xAB2F => Script::Ethiopic,

        // Hangul (Korean)
        0x1100..=0x11FF | 0x302E..=0x302F | 0x3131..=0x318F | 0xA960..=0xA97F | 0xAC00..=0xD7FF => {
            Script::Hangul
        }

        // Hiragana
        0x3041..=0x309F | 0x1B001..=0x1B0FF => Script::Hiragana,

        // Katakana
        0x30A0..=0x30FF | 0x31F0..=0x31FF | 0xFF66..=0xFF9F => Script::Katakana,

        // CJK Unified Ideographs (Han)
        0x4E00..=0x9FFF
        | 0x3400..=0x4DBF
        | 0x20000..=0x2A6DF
        | 0x2A700..=0x2CEAF
        | 0xF900..=0xFAFF => Script::CjkUnified,

        // Cherokee
        0x13A0..=0x13FF | 0xAB70..=0xABBF => Script::Cherokee,

        // Unified Canadian Aboriginal Syllabics
        0x1400..=0x167F | 0x18B0..=0x18FF => Script::Canadian,

        // Runic
        0x16A0..=0x16FF => Script::Runic,

        // Ogham
        0x1680..=0x169F => Script::Ogham,

        // Common: digits, punctuation, whitespace
        0x0021..=0x0040
        | 0x005B..=0x0060
        | 0x007B..=0x00BF
        | 0x2000..=0x206F
        | 0x2100..=0x214F
        | 0x3000..=0x303F
        | 0xFF01..=0xFF0F => Script::Common,

        _ => Script::Unknown,
    }
}

/// Script coverage result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptCoverage {
    pub script: Script,
    pub iso_code: String,
    pub display_name: String,
    pub codepoint_count: usize,
    pub coverage_pct: f32,
    pub is_rtl: bool,
}

/// Result of hyperglot analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperglotResult {
    /// All scripts found, sorted by coverage descending.
    pub scripts: Vec<ScriptCoverage>,
    /// Primary script (highest coverage, excluding Common/Unknown).
    pub primary_script: Option<String>,
    /// True if any RTL script detected.
    pub has_rtl: bool,
    /// True if multiple non-common scripts detected (multilingual text).
    pub is_multilingual: bool,
    /// Total analysed codepoints.
    pub total_codepoints: usize,
}

/// Maximum input length in codepoints (LangSec safety bound).
const MAX_INPUT_CODEPOINTS: usize = 4096;

/// Detect Unicode scripts in `text`.
///
/// Returns script coverage sorted by frequency descending.
/// Common (punctuation/digits) and Unknown codepoints are counted but not
/// included in the primary script selection.
#[instrument(skip(text))]
pub fn detect_scripts(text: &str) -> HyperglotResult {
    use std::collections::HashMap;

    // LangSec: hard cap on input size before any work is done
    let codepoints: Vec<char> = text.chars().take(MAX_INPUT_CODEPOINTS).collect();
    let total = codepoints.len();
    if total == 0 {
        return HyperglotResult {
            scripts: vec![],
            primary_script: None,
            has_rtl: false,
            is_multilingual: false,
            total_codepoints: 0,
        };
    }

    let mut counts: HashMap<Script, usize> = HashMap::new();
    for &c in &codepoints {
        *counts.entry(codepoint_to_script(c)).or_insert(0) += 1;
    }

    let mut scripts: Vec<ScriptCoverage> = counts
        .into_iter()
        .map(|(script, count)| {
            let pct = (count as f32 / total as f32) * 100.0;
            let iso = script.iso_code().to_string();
            let name = script.display_name().to_string();
            let rtl = script.is_rtl();
            ScriptCoverage {
                script,
                iso_code: iso,
                display_name: name,
                codepoint_count: count,
                coverage_pct: pct,
                is_rtl: rtl,
            }
        })
        .collect();

    // Sort by coverage descending
    scripts.sort_by(|a, b| b.codepoint_count.cmp(&a.codepoint_count));

    let has_rtl = scripts.iter().any(|s| s.is_rtl);

    // Primary = highest-coverage script excluding Common/Unknown
    let primary_script = scripts
        .iter()
        .find(|s| !matches!(s.script, Script::Common | Script::Unknown))
        .map(|s| s.iso_code.clone());

    // Multilingual = 2+ non-common/unknown scripts with ≥5% coverage each
    let significant: Vec<_> = scripts
        .iter()
        .filter(|s| !matches!(s.script, Script::Common | Script::Unknown) && s.coverage_pct >= 5.0)
        .collect();
    let is_multilingual = significant.len() >= 2;

    HyperglotResult {
        scripts,
        primary_script,
        has_rtl,
        is_multilingual,
        total_codepoints: total,
    }
}

/// Validate that a track title's script matches the declared language.
/// Returns `true` if the title is plausibly in the declared BCP-47 language.
#[zkperf_macros::zkperf]
pub fn validate_title_language(title: &str, bcp47_lang: &str) -> bool {
    let result = detect_scripts(title);
    let primary = match &result.primary_script {
        Some(s) => s.as_str(),
        None => return true, // empty / all-common → pass
    };
    // Map BCP-47 language prefixes to expected ISO 15924 script codes.
    // This is a best-effort check, not an RFC 5646 full lookup.
    let expected_script: &[&str] = match bcp47_lang.split('-').next().unwrap_or("") {
        "ja" => &["Hira", "Kana", "Hani"],
        "zh" => &["Hani"],
        "ko" => &["Hang"],
        "ar" => &["Arab"],
        "he" => &["Hebr"],
        "hi" | "mr" | "ne" | "sa" => &["Deva"],
        "ru" | "uk" | "bg" | "sr" | "mk" | "be" => &["Cyrl"],
        "ka" => &["Geor"],
        "hy" => &["Armn"],
        "th" => &["Thai"],
        "lo" => &["Laoo"],
        "my" => &["Mymr"],
        "km" => &["Khmr"],
        "am" | "ti" => &["Ethi"],
        _ => return true, // Latin or unknown → accept
    };
    expected_script.contains(&primary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latin_detection() {
        let r = detect_scripts("Hello World");
        assert_eq!(r.primary_script.as_deref(), Some("Latn"));
    }

    #[test]
    fn test_cyrillic_detection() {
        let r = detect_scripts("Привет мир");
        assert_eq!(r.primary_script.as_deref(), Some("Cyrl"));
    }

    #[test]
    fn test_arabic_detection() {
        let r = detect_scripts("مرحبا بالعالم");
        assert_eq!(r.primary_script.as_deref(), Some("Arab"));
        assert!(r.has_rtl);
    }

    #[test]
    fn test_multilingual() {
        let r = detect_scripts("Hello Привет مرحبا");
        assert!(r.is_multilingual);
    }

    #[test]
    fn test_cjk_detection() {
        let r = detect_scripts("日本語テスト");
        let codes: Vec<_> = r.scripts.iter().map(|s| s.iso_code.as_str()).collect();
        assert!(codes.contains(&"Hani") || codes.contains(&"Hira") || codes.contains(&"Kana"));
    }

    #[test]
    fn test_length_cap() {
        let long: String = "a".repeat(10000);
        let r = detect_scripts(&long);
        assert!(r.total_codepoints <= 4096);
    }
}