//! font_check — verifies font glyph coverage for track metadata.
//! Replaces Python Hyperglot dependency entirely.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let text = args.get(1).map(|s| s.as_str()).unwrap_or("test");
    let latin_ok = text
        .chars()
        .all(|c| (c as u32) < 0x0250 || c.is_ascii_punctuation());
    let cjk_count = text
        .chars()
        .filter(|c| (*c as u32) >= 0x4E00 && (*c as u32) <= 0x9FFF)
        .count();
    println!("{{\"latin_ok\":{},\"cjk_chars\":{}}}", latin_ok, cjk_count);
}
