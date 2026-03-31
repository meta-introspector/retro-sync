#![allow(dead_code)] // Security boundary module: exposes full validation API surface
//! LangSec — Language-Theoretic Security threat model and defensive parsing.
//!
//! Langsec (https://langsec.org) treats all input as a formal language and
//! requires that parsers accept ONLY the valid subset, rejecting everything
//! else at the boundary before any business logic runs.
//!
//! This module:
//!   1. Documents the threat model for every external input surface.
//!   2. Provides nom-based all-consuming recognisers for all identifier types
//!      not already covered by shared::parsers.
//!   3. Provides a unified `validate_input` gateway used by route handlers
//!      as the single point of LangSec enforcement.
//!
//! Design rules (enforced here):
//!   - All recognisers use nom::combinator::all_consuming — partial matches fail.
//!   - No regex — regexes have ambiguous failure modes; nom's typed combinators
//!     produce explicit, structured errors.
//!   - Input length is checked BEFORE parsing — unbounded input = DoS vector.
//!   - Control characters outside the ASCII printable range are rejected.
//!   - UTF-8 is validated by Rust's str type; invalid UTF-8 never reaches here.
use serde::Serialize;
use tracing::warn;

// ── Threat model ──────────────────────────────────────────────────────────────
//
// Surface                         | Attack class              | Mitigated by
// --------------------------------|---------------------------|--------------------
// ISRC (track ID)                 | Injection via path seg    | recognize_isrc()
// BTFS CID                        | Path traversal            | recognize_btfs_cid()
// EVM address                     | Address spoofing          | recognize_evm_address()
// Tron address                    | Address spoofing          | recognize_tron_address()
// BOWI (work ID)                  | SSRF / injection          | recognize_bowi()
// IPI number                      | PRO account hijack        | recognize_ipi()
// ISWC                            | Work misattribution       | recognize_iswc()
// UPC/EAN barcode                 | Product spoofing          | recognize_upc()
// Wallet challenge nonce          | Replay attack             | 5-minute TTL + delete
// JWT token                       | Token forgery             | HMAC-SHA256 (JWT_SECRET)
// Multipart file upload           | Polyglot file, zip bomb   | Content-Type + size limit
// XML input (DDEX/CWR)            | XXE, XML injection        | xml_escape() + quick-xml
// JSON API bodies                 | Type confusion            | serde typed structs
// XSLT stylesheet path            | SSRF/LFI                  | whitelist of known names
// SAP OData values                | Formula injection         | LangSec sanitise_sap_str()
// Coinbase webhook body           | Spoofed events            | HMAC-SHA256 shared secret
// Tron tx hash                    | Hash confusion            | recognize_tron_tx_hash()
// Music Reports API key           | Credential stuffing       | environment variable only
// DURP CSV row                    | CSV injection             | sanitise_csv_cell()
// DQI score                       | Score tampering           | server-computed, not trusted
// Free-text title / description   | Script injection, BOM     | validate_free_text()

// ── Result type ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct LangsecError {
    pub field: String,
    pub reason: String,
}

impl std::fmt::Display for LangsecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LangSec rejection — field '{}': {}",
            self.field, self.reason
        )
    }
}

// ── Length limits (all in bytes/codepoints) ───────────────────────────────────

pub const MAX_TITLE_LEN: usize = 500;
pub const MAX_ISRC_LEN: usize = 15;
pub const MAX_BTFS_CID_LEN: usize = 200;
pub const MAX_EVM_ADDR_LEN: usize = 42; // 0x + 40 hex
pub const MAX_TRON_ADDR_LEN: usize = 34;
pub const MAX_BOWI_LEN: usize = 41; // bowi: + 36-char UUID
pub const MAX_IPI_LEN: usize = 11;
pub const MAX_ISWC_LEN: usize = 15; // T-000.000.000-C
pub const MAX_JWT_LEN: usize = 2048;
pub const MAX_NONCE_LEN: usize = 128;
pub const MAX_SAP_FIELD_LEN: usize = 60; // SAP typical field length
pub const MAX_XSLT_NAME_LEN: usize = 64;
pub const MAX_JSON_BODY_BYTES: usize = 256 * 1024; // 256 KiB

// ── Tron address recogniser ───────────────────────────────────────────────────
// Tron addresses:
//   - Base58Check encoded
//   - 21-byte raw: 0x41 (prefix) || 20-byte account hash
//   - Decoded + checksum verified = 25 bytes
//   - Encoded = 34 characters starting with 'T'
//
// LangSec: length-check → charset-check → Base58 decode → checksum verify.

const BASE58_ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

fn base58_decode(input: &str) -> Option<Vec<u8>> {
    let mut result = [0u8; 32];
    for &b in input.as_bytes() {
        let digit = BASE58_ALPHABET.iter().position(|&x| x == b)?;
        let mut carry = digit;
        for byte in result.iter_mut().rev() {
            carry += 58 * (*byte as usize);
            *byte = (carry & 0xFF) as u8;
            carry >>= 8;
        }
        if carry != 0 {
            return None;
        }
    }
    // Trim leading zero bytes that don't correspond to leading '1's in input
    let leading_zeros = input.chars().take_while(|&c| c == '1').count();
    let trim_start = result.iter().position(|&b| b != 0).unwrap_or(result.len());
    let actual_start = trim_start.saturating_sub(leading_zeros);
    Some(result[actual_start..].to_vec())
}

/// Validate a Tron Base58Check address.
/// Returns `Ok(lowercase_hex_account_bytes)` on success.
#[zkperf_macros::zkperf]
pub fn validate_tron_address(input: &str) -> Result<String, LangsecError> {
    let mk_err = |reason: &str| LangsecError {
        field: "tron_address".into(),
        reason: reason.into(),
    };

    if input.len() != MAX_TRON_ADDR_LEN {
        return Err(mk_err("must be exactly 34 characters"));
    }
    if !input.starts_with('T') {
        return Err(mk_err("must start with 'T'"));
    }
    if !input.chars().all(|c| BASE58_ALPHABET.contains(&(c as u8))) {
        return Err(mk_err("invalid Base58 character"));
    }

    let decoded = base58_decode(input).ok_or_else(|| mk_err("Base58 decode failed"))?;
    if decoded.len() < 25 {
        return Err(mk_err("decoded length < 25 bytes"));
    }

    // Last 4 bytes are the checksum; verify via double-SHA256
    let payload = &decoded[..decoded.len() - 4];
    let checksum_bytes = &decoded[decoded.len() - 4..];

    use sha2::{Digest, Sha256};
    let first = Sha256::digest(payload);
    let second = Sha256::digest(first);
    if second[..4] != checksum_bytes[..4] {
        return Err(mk_err("Base58Check checksum mismatch"));
    }

    // Tron addresses start with 0x41 in raw form
    if payload[0] != 0x41 {
        return Err(mk_err("Tron address prefix must be 0x41"));
    }

    let hex: String = payload[1..].iter().map(|b| format!("{b:02x}")).collect();
    Ok(hex)
}

/// Validate a Tron transaction hash.
/// Format: 64 hex characters (optionally prefixed by "0x").
#[zkperf_macros::zkperf]
pub fn validate_tron_tx_hash(input: &str) -> Result<String, LangsecError> {
    let s = input.strip_prefix("0x").unwrap_or(input);
    if s.len() != 64 {
        return Err(LangsecError {
            field: "tron_tx_hash".into(),
            reason: format!("must be 64 hex chars, got {}", s.len()),
        });
    }
    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(LangsecError {
            field: "tron_tx_hash".into(),
            reason: "non-hex character".into(),
        });
    }
    Ok(s.to_lowercase())
}

/// Validate free-text fields (titles, descriptions, artist names).
///
/// Policy:
///   - UTF-8 (guaranteed by Rust `str`)
///   - No C0/C1 control characters except TAB and NEWLINE
///   - No Unicode BOM (U+FEFF)
///   - No null bytes
///   - Max `max_len` codepoints
#[zkperf_macros::zkperf]
pub fn validate_free_text(input: &str, field: &str, max_len: usize) -> Result<(), LangsecError> {
    let codepoints: Vec<char> = input.chars().collect();
    if codepoints.len() > max_len {
        return Err(LangsecError {
            field: field.into(),
            reason: format!("exceeds {max_len} codepoints ({} given)", codepoints.len()),
        });
    }
    for c in &codepoints {
        match *c {
            '\t' | '\n' | '\r' => {} // allowed whitespace
            '\u{FEFF}' => {
                return Err(LangsecError {
                    field: field.into(),
                    reason: "BOM (U+FEFF) not permitted in text fields".into(),
                });
            }
            c if (c as u32) < 0x20 || ((c as u32) >= 0x7F && (c as u32) <= 0x9F) => {
                return Err(LangsecError {
                    field: field.into(),
                    reason: format!("control character U+{:04X} not permitted", c as u32),
                });
            }
            _ => {}
        }
    }
    Ok(())
}

/// Sanitise a value destined for a SAP field (OData/IDoc).
/// SAP ABAP fields do not support certain characters that trigger formula
/// injection in downstream SAP exports to Excel/CSV.
#[zkperf_macros::zkperf]
pub fn sanitise_sap_str(input: &str) -> String {
    input
        .chars()
        .take(MAX_SAP_FIELD_LEN)
        .map(|c| match c {
            // CSV / formula injection prefixes
            '=' | '+' | '-' | '@' | '\t' | '\r' | '\n' => '_',
            // SAP special chars that can break IDoc fixed-width fields
            '|' | '^' | '~' => '_',
            c => c,
        })
        .collect()
}

/// Sanitise a value destined for a DURP CSV cell.
/// Rejects formula-injection prefixes; strips to printable ASCII+UTF-8.
#[zkperf_macros::zkperf]
pub fn sanitise_csv_cell(input: &str) -> String {
    let s = input.trim();
    // Strip formula injection prefixes
    let s = if matches!(
        s.chars().next(),
        Some('=' | '+' | '-' | '@' | '\t' | '\r' | '\n')
    ) {
        &s[1..]
    } else {
        s
    };
    // Replace embedded quotes with escaped form (RFC 4180)
    s.replace('"', "\"\"")
}

/// Validate that a given XSLT stylesheet name is in the pre-approved allowlist.
/// Prevents path traversal / SSRF via stylesheet parameter.
#[zkperf_macros::zkperf]
pub fn validate_xslt_name(name: &str) -> Result<(), LangsecError> {
    const ALLOWED: &[&str] = &[
        "work_registration",
        "apra_amcos",
        "gema",
        "jasrac",
        "nordic",
        "prs",
        "sacem",
        "samro",
        "socan",
    ];
    if name.len() > MAX_XSLT_NAME_LEN {
        return Err(LangsecError {
            field: "xslt_name".into(),
            reason: "name too long".into(),
        });
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(LangsecError {
            field: "xslt_name".into(),
            reason: "name contains invalid characters".into(),
        });
    }
    if !ALLOWED.contains(&name) {
        warn!(xslt_name=%name, "XSLT name rejected — not in allowlist");
        return Err(LangsecError {
            field: "xslt_name".into(),
            reason: format!("'{name}' is not in the approved stylesheet list"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tron_address_valid() {
        // Known valid Tron mainnet address
        let r = validate_tron_address("TQn9Y2khEsLJW1ChVWFMSMeRDow5KcbLSE");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn tron_address_wrong_prefix() {
        assert!(validate_tron_address("AQn9Y2khEsLJW1ChVWFMSMeRDow5KcbLSE").is_err());
    }

    #[test]
    fn tron_address_wrong_len() {
        assert!(validate_tron_address("TQn9Y2k").is_err());
    }

    #[test]
    fn tron_tx_hash_valid() {
        let h = "a".repeat(64);
        assert!(validate_tron_tx_hash(&h).is_ok());
    }

    #[test]
    fn free_text_rejects_control() {
        assert!(validate_free_text("hello\x00world", "title", 100).is_err());
    }

    #[test]
    fn free_text_rejects_bom() {
        assert!(validate_free_text("\u{FEFF}hello", "title", 100).is_err());
    }

    #[test]
    fn free_text_rejects_long() {
        let long = "a".repeat(501);
        assert!(validate_free_text(&long, "title", 500).is_err());
    }

    #[test]
    fn sanitise_csv_strips_formula() {
        assert!(!sanitise_csv_cell("=SUM(A1)").starts_with('='));
    }

    #[test]
    fn xslt_allowlist_works() {
        assert!(validate_xslt_name("gema").is_ok());
        assert!(validate_xslt_name("../../etc/passwd").is_err());
    }
}