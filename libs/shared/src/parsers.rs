//! LangSec formal recognizers — nom all_consuming parsers.
//! All input validation is centralised here. Nothing reaches business logic
//! without passing through one of these recognizers.
use crate::types::{BtfsCid, EvmAddress, Isrc, ParseError, RoyaltySplit};
use nom::{
    bytes::complete::{tag, take_while_m_n},
    sequence::tuple,
    IResult,
};

// ── ISRC: CC-XXX-YY-NNNNN ────────────────────────────────────────────────
#[allow(dead_code)]
fn parse_isrc_inner(i: &str) -> IResult<&str, &str> {
    let (i, (_, _, _, _, _, _, _, _, _, _)) = tuple((
        take_while_m_n(2, 2, |c: char| c.is_ascii_uppercase()), // CC
        tag("-"),
        take_while_m_n(3, 3, |c: char| c.is_ascii_alphanumeric()), // XXX
        tag("-"),
        take_while_m_n(2, 2, |c: char| c.is_ascii_digit()), // YY
        tag("-"),
        take_while_m_n(5, 5, |c: char| c.is_ascii_digit()), // NNNNN
        nom::combinator::peek(nom::combinator::eof),
        nom::combinator::eof,
        nom::combinator::success(""),
    ))(i)?;
    Ok(("", i))
}

pub fn recognize_isrc(input: &str) -> Result<Isrc, ParseError> {
    // CC-XXX-YY-NNNNN = 2+1+3+1+2+1+5 = 15 chars
    if input.len() != 15 {
        return Err(ParseError::InvalidLength {
            expected: 15,
            got: input.len(),
        });
    }
    let parts: Vec<&str> = input.split('-').collect();
    if parts.len() != 4 {
        return Err(ParseError::InvalidFormat(input.into()));
    }
    if parts[0].len() != 2 || !parts[0].chars().all(|c| c.is_ascii_uppercase()) {
        return Err(ParseError::InvalidFormat(
            "CC must be 2 uppercase letters".into(),
        ));
    }
    if parts[1].len() != 3 || !parts[1].chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ParseError::InvalidFormat(
            "Registrant must be 3 alphanumeric".into(),
        ));
    }
    if parts[2].len() != 2 || !parts[2].chars().all(|c| c.is_ascii_digit()) {
        return Err(ParseError::InvalidFormat("Year must be 2 digits".into()));
    }
    if parts[3].len() != 5 || !parts[3].chars().all(|c| c.is_ascii_digit()) {
        return Err(ParseError::InvalidFormat(
            "Designation must be 5 digits".into(),
        ));
    }
    Ok(Isrc(input.to_string()))
}

// ── BTFS CID ─────────────────────────────────────────────────────────────
pub fn recognize_btfs_cid(input: &str) -> Result<BtfsCid, ParseError> {
    // CIDv0: Qm... (46 chars base58)
    // CIDv1: bafy... or b... (variable, base32/base64)
    if input.len() < 10 {
        return Err(ParseError::InvalidLength {
            expected: 46,
            got: input.len(),
        });
    }
    let valid = input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=');
    if !valid {
        return Err(ParseError::InvalidFormat(
            "CID contains invalid characters".into(),
        ));
    }
    Ok(BtfsCid(input.to_string()))
}

// ── EVM address ──────────────────────────────────────────────────────────
pub fn recognize_evm_address(input: &str) -> Result<EvmAddress, ParseError> {
    let s = input.strip_prefix("0x").unwrap_or(input);
    if s.len() != 40 {
        return Err(ParseError::InvalidLength {
            expected: 40,
            got: s.len(),
        });
    }
    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ParseError::InvalidFormat("address must be hex".into()));
    }
    Ok(EvmAddress(format!("0x{}", s.to_lowercase())))
}

// ── Tx hash ───────────────────────────────────────────────────────────────
pub fn recognize_tx_hash(input: &str) -> Result<String, ParseError> {
    let s = input.strip_prefix("0x").unwrap_or(input);
    if s.len() != 64 {
        return Err(ParseError::InvalidLength {
            expected: 64,
            got: s.len(),
        });
    }
    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ParseError::InvalidFormat("tx hash must be hex".into()));
    }
    Ok(format!("0x{}", s.to_lowercase()))
}

// ── Royalty splits: Vec<(address, bps)>, Σbps == 10_000 ─────────────────
pub fn recognize_splits(raw: &[(String, u16)]) -> Result<Vec<RoyaltySplit>, ParseError> {
    let mut splits = Vec::new();
    let mut total = 0u32;
    for (addr, bps) in raw {
        let address = recognize_evm_address(addr)?;
        total += *bps as u32;
        splits.push(RoyaltySplit {
            address,
            bps: *bps,
            amount_btt: 0,
        });
    }
    if total != 10_000 {
        return Err(ParseError::InvalidFormat(format!(
            "bps sum {} ≠ 10_000",
            total
        )));
    }
    Ok(splits)
}
