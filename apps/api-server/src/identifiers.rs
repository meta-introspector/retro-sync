//! Backend identifier validators: BOWI, UPC/EAN, IPI/CAE, ISWC.
//!
//! BOWI (Best Open Work Identifier) — https://bowi.org
//! Free, open, persistent URI for musical compositions.
//! Wikidata property P10836. Format: bowi:{uuid4}
//!
//! Minting policy:
//!   1. Check Wikidata P10836 for existing BOWI (via wikidata::lookup_artist)
//!   2. Found → use it (de-duplication preserved across PROs and DSPs)
//!   3. Not found → mint a new UUID4; artist registers at bowi.org
pub use shared::identifiers::recognize_bowi;
pub use shared::types::Bowi;

#[allow(dead_code)]
/// Mint a fresh BOWI for a work with no existing registration.
/// Returns a valid bowi:{uuid4} — artist should then register at https://bowi.org/register
#[zkperf_macros::zkperf]
pub fn mint_bowi() -> Bowi {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let a = t.subsec_nanos();
    let b = t.as_secs();
    let c = a.wrapping_mul(0x9e3779b9).wrapping_add(b as u32);
    let d = b.wrapping_mul(0x6c62272e);
    let variant = [b'8', b'9', b'a', b'b'][((c >> 6) & 0x3) as usize] as char;
    Bowi(format!(
        "bowi:{:08x}-{:04x}-4{:03x}-{}{:03x}-{:012x}",
        a,
        (c >> 16) & 0xffff,
        c & 0xfff,
        variant,
        (c >> 2) & 0xfff,
        d & 0xffffffffffff,
    ))
}

#[allow(dead_code)]
/// Resolve BOWI from Wikidata enrichment or mint a new one.
/// Returns (bowi, is_existing): is_existing=true means Wikidata had P10836.
#[zkperf_macros::zkperf]
pub async fn resolve_or_mint_bowi(wiki_bowi: Option<&str>) -> (Bowi, bool) {
    if let Some(b) = wiki_bowi {
        if let Ok(parsed) = recognize_bowi(b) {
            return (parsed, true);
        }
    }
    (mint_bowi(), false)
}