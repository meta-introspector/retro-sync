//! nft71_svg — Generate 71 NFT tile SVGs for inspection before rasterization.
//!
//! Each tile: 512×512, cuneiform signs, interval names, notation, category colors.
//! SVGs can be opened in any browser for visual inspection.
//! Pipeline: SVG → PNG (resvg, no gamma) → stego embed
//!
//! Usage: cargo run -p fixtures --example nft71_svg

use std::path::Path;

const SZ: u32 = 512;

const CUNEIFORM: &[&str] = &[
    "𒀸𒌑𒄴𒊑", "𒄿𒊭𒅈𒌈", "𒂊𒁍𒁍", "𒉌𒀉𒃻", "𒃻𒇷𒌈",
    "𒆠𒁴𒈬", "𒁉𒌈", "𒊺𒊒", "𒊭𒅖𒊭𒌈", "𒊑𒁍𒌈",
    "𒅖𒄣", "𒋾𒌅𒅈𒃻", "𒋾𒌅𒅈𒄿", "𒊺𒅈𒁺", "𒀀𒈬𒊏𒁉",
];

const INTERVALS: &[&str] = &[
    "nīš tuḫrim", "išartum", "embūbum", "nīd qablim", "qablītum",
    "kitmum", "pītum", "šērum", "šalšatum", "rebûttum",
    "isqum", "titur qablītim", "titur išartim", "ṣerdum", "colophon",
];

const NOTATION_L1: &str = "qáb-li-te 3  ir-bu-te 1  qáb-li-te 3  ša-aḫ-ri 1  i-šar-te 10";
const NOTATION_L2: &str = "ti-ti-mi-šar-te 2  zi-ir-te 1  ša-aḫ-ri 2  ša-aš-ša-te 2  ir-bu-te 2";

const PRIMES: &[u64] = &[2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71];

fn is_prime(n: u64) -> bool { PRIMES.contains(&n) }

fn category(idx: u64) -> &'static str {
    if is_prime(idx) { return "generator"; }
    match idx {
        4|6 => "source",
        8|9|10 => "artifact",
        12|14|15|16|18 => "witness",
        20|21|22|24|25 => "eigenspace",
        26|27|28|30|32|33|34|35 => "metadata",
        36|38|39|40|42 => "reconstruction",
        44|45|46|48|49|50|51|52|54|55|56|57 => "reference",
        58|60|62|63|64|65 => "youtube",
        66|68|69|70 => "pipeline",
        _ => "reserved",
    }
}

fn cat_color(cat: &str) -> &'static str {
    match cat {
        "generator"      => "#50507e",
        "source"         => "#7d5b7e",
        "artifact"       => "#5b7e5b",
        "witness"        => "#7e7e5b",
        "eigenspace"     => "#5b7e7e",
        "metadata"       => "#7e5b5b",
        "reconstruction" => "#5b5b7e",
        "reference"      => "#7e7b5b",
        "youtube"        => "#7e5b7b",
        "pipeline"       => "#5b7b5b",
        _                => "#505068",
    }
}

fn tile_svg(idx: u64) -> String {
    let cat = category(idx);
    let bg = cat_color(cat);
    let border = if is_prime(idx) { "#ffd700" } else { "#666666" };
    let marker = if is_prime(idx) { "★" } else { "·" };
    let cunei = CUNEIFORM[((idx - 1) as usize) % CUNEIFORM.len()];
    let interval = INTERVALS[((idx - 1) as usize) % INTERVALS.len()];

    // Data-encoding visual elements: positions/colors carry information
    let hash = idx.wrapping_mul(2654435761); // knuth hash for pseudo-random
    let zig_h = 80 + (hash % 40) as u32;    // ziggurat height varies per tile
    let star_x = 40 + (hash % 60) as u32;   // star position encodes data
    let star_y = 30 + ((hash >> 8) % 40) as u32;
    let orb_r = 6 + (hash % 8) as u32;      // orbifold indicator radius
    let stripe_w = 3 + (hash % 4) as u32;   // border stripe width

    // Orbifold coords as visual data
    let o71 = idx % 71;
    let o59 = idx % 59;
    let o47 = idx % 47;

    // Generate ziggurat steps (Sumerian temple shape)
    let mut zig = String::new();
    let steps = 4 + (idx % 3) as u32;
    for s in 0..steps {
        let w = 200 - s * 30;
        let x = 256 - w / 2;
        let y = 340 + s * (zig_h / steps);
        let shade = 0x50 + (s * 0x10) as u8;
        zig.push_str(&format!(
            "  <rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{}\" fill=\"#{shade:02x}{:02x}{:02x}\" opacity=\"0.6\"/>\n",
            zig_h / steps, shade / 2, shade / 3
        ));
    }

    // Decorative border pattern (encodes data in spacing)
    let mut deco = String::new();
    for i in 0..16 {
        let x = 8 + i * 32 + (((hash >> (i % 8)) & 3) as u32);
        deco.push_str(&format!(
            "  <rect x=\"{x}\" y=\"6\" width=\"{stripe_w}\" height=\"8\" fill=\"{border}\" opacity=\"0.4\"/>\n"
        ));
        deco.push_str(&format!(
            "  <rect x=\"{x}\" y=\"498\" width=\"{stripe_w}\" height=\"8\" fill=\"{border}\" opacity=\"0.4\"/>\n"
        ));
    }

    // Star/rosette (Sumerian symbol, position encodes orbifold)
    let star_points: String = (0..8).map(|i| {
        let angle = std::f64::consts::PI * 2.0 * i as f64 / 8.0;
        let r = if i % 2 == 0 { 18.0 } else { 8.0 };
        let px = star_x as f64 + angle.cos() * r;
        let py = star_y as f64 + angle.sin() * r;
        format!("{:.1},{:.1}", px, py)
    }).collect::<Vec<_>>().join(" ");

    format!(r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{SZ}" height="{SZ}" viewBox="0 0 {SZ} {SZ}">
  <defs>
    <linearGradient id="bg{idx}" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{bg}"/>
      <stop offset="100%" stop-color="#606878"/>
    </linearGradient>
    <pattern id="grid{idx}" width="32" height="32" patternUnits="userSpaceOnUse">
      <rect width="32" height="32" fill="none"/>
      <rect x="0" y="0" width="16" height="16" fill="rgba(128,128,140,0.08)"/>
      <rect x="16" y="16" width="16" height="16" fill="rgba(128,128,140,0.08)"/>
    </pattern>
  </defs>
  <rect width="{SZ}" height="{SZ}" fill="url(#bg{idx})"/>
  <rect width="{SZ}" height="{SZ}" fill="url(#grid{idx})"/>
  <!-- border -->
  <rect x="0" y="0" width="{SZ}" height="4" fill="{border}"/>
  <rect x="0" y="0" width="4" height="{SZ}" fill="{border}"/>
  <rect x="0" y="{y_bot}" width="{SZ}" height="4" fill="{border}"/>
  <rect x="{x_rt}" y="0" width="4" height="{SZ}" fill="{border}"/>
  <!-- decorative border pattern (data in spacing) -->
{deco}
  <!-- star/rosette (position = orbifold coords) -->
  <polygon points="{star_points}" fill="#ffd700" opacity="0.5"/>
  <polygon points="{star_points2}" fill="#ffd700" opacity="0.3" transform="translate({sx2},0)"/>
  <!-- cuneiform panel -->
  <rect x="30" y="55" width="452" height="70" fill="#707888" rx="6"/>
  <text x="256" y="105" text-anchor="middle" fill="#ffd700" font-size="56" font-weight="bold">{cunei}</text>
  <!-- interval panel -->
  <rect x="50" y="135" width="412" height="38" fill="#606878" rx="4"/>
  <text x="256" y="162" text-anchor="middle" fill="#f0e8d0" font-family="monospace" font-size="22" font-weight="bold">{interval}</text>
  <!-- notation panel -->
  <rect x="20" y="185" width="472" height="50" fill="#586070" rx="3"/>
  <text x="256" y="207" text-anchor="middle" fill="#b0e8b0" font-family="monospace" font-size="11">{NOTATION_L1}</text>
  <text x="256" y="225" text-anchor="middle" fill="#b0e8b0" font-family="monospace" font-size="11">{NOTATION_L2}</text>
  <!-- orbifold panel -->
  <rect x="100" y="245" width="312" height="24" fill="#506068" rx="3"/>
  <text x="256" y="262" text-anchor="middle" fill="#a0a8c0" font-family="monospace" font-size="10">orbifold ({o71},{o59},{o47}) mod (71,59,47)</text>
  <!-- ziggurat -->
{zig}
  <!-- orbifold indicator circles -->
  <circle cx="460" cy="40" r="{orb_r}" fill="none" stroke="#78b6ff" stroke-width="2" opacity="0.7"/>
  <circle cx="460" cy="40" r="{orb_r2}" fill="none" stroke="#ff8868" stroke-width="1.5" opacity="0.5"/>
  <!-- shard info panel -->
  <rect x="30" y="425" width="452" height="75" fill="#586070" rx="5"/>
  <text x="256" y="448" text-anchor="middle" fill="#d0d8e0" font-family="monospace" font-size="16" font-weight="bold">{marker}{idx:02} {cat}</text>
  <text x="256" y="470" text-anchor="middle" fill="#90c0ff" font-family="monospace" font-size="12">Hurrian Hymn h.6 · Tablet RS 15.30 · ~1400 BC · Ugarit</text>
  <text x="256" y="490" text-anchor="middle" fill="#808898" font-family="monospace" font-size="9">DA51 CBOR · Groth16/BN254 · Cl(15,0,0) · 6-layer stego · shard {idx}/71</text>
</svg>"##,
        y_bot = SZ - 4,
        x_rt = SZ - 4,
        star_points2 = (0..8).map(|i| {
            let angle = std::f64::consts::PI * 2.0 * i as f64 / 8.0;
            let r = if i % 2 == 0 { 14.0 } else { 6.0 };
            format!("{:.1},{:.1}", star_x as f64 + angle.cos() * r, star_y as f64 + angle.sin() * r)
        }).collect::<Vec<_>>().join(" "),
        sx2 = SZ - star_x * 2,
        orb_r2 = orb_r + 4,
        prime = PRIMES[((idx - 1) as usize) % PRIMES.len()],
    )
}

fn main() {
    let out = Path::new("fixtures/output/nft71_svg");
    std::fs::create_dir_all(out).unwrap();

    println!("=== Generating 71 NFT tile SVGs ({SZ}×{SZ}) ===");
    for idx in 1..=71u64 {
        let svg = tile_svg(idx);
        let path = out.join(format!("{:02}.svg", idx));
        std::fs::write(&path, &svg).unwrap();
        let cat = category(idx);
        let marker = if is_prime(idx) { "★" } else { "·" };
        println!("{marker} {:02} [{cat}]", idx);
    }

    // Generate an HTML gallery for inspection
    let mut html = String::from(r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>NFT71 Tile Gallery</title>
<style>
body{background:#0a0a0a;color:#0f0;font-family:monospace;padding:1em}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:8px}
.tile{border:1px solid #333;text-align:center;font-size:11px;padding:4px}
.tile img{width:100%;display:block}
.tile.prime{border-color:#ffd700}
h1{color:#0ff}
</style></head><body>
<h1>𒀸𒌑𒄴𒊑 NFT71 Tile Gallery — Inspect Before Rasterization</h1>
<div class="grid">
"#);
    for idx in 1..=71u64 {
        let cls = if is_prime(idx) { "tile prime" } else { "tile" };
        let marker = if is_prime(idx) { "★" } else { "·" };
        html.push_str(&format!(
            "<div class=\"{cls}\"><img src=\"{:02}.svg\"><br>{marker}{:02} {}</div>\n",
            idx, idx, category(idx)
        ));
    }
    html.push_str("</div></body></html>");
    std::fs::write(out.join("gallery.html"), &html).unwrap();

    println!("\n→ 71 SVGs + gallery.html in {}", out.display());
    println!("→ Open gallery.html in browser to inspect");
}
