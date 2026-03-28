//! nft71_stego_svg — SVG → PNG stego pipeline (no ImageMagick, no gamma)
//!
//! Reads 71 SVGs, rasterizes to 512×512 RGB via resvg, embeds NFT7 payload
//! into 6 bit planes, writes stripped PNGs.
//!
//! Usage: cargo run -p fixtures --example nft71_stego_svg

use sha2::{Digest, Sha256};
use std::path::Path;

fn main() {
    let svg_dir = Path::new("fixtures/output/nft71_svg");
    let png_dir = Path::new("fixtures/output/nft71_stego_png");
    std::fs::create_dir_all(png_dir).unwrap();

    // Load all music data (graceful fallback for missing files)
    let load = |p: &str| -> Vec<u8> {
        match std::fs::read(p) {
            Ok(d) => { println!("  {p}: {} B", d.len()); d }
            Err(_) => { println!("  {p}: MISSING (skipped)"); vec![] }
        }
    };

    println!("=== Loading payload segments ===");
    let wav = load("fixtures/output/h6_west.wav");
    let midi_west = load("fixtures/output/h6_west.midi");
    let midi_01 = load("fixtures/output/yt_01.midi");
    let midi_04 = load("fixtures/output/yt_04.midi");
    let midi_06 = load("fixtures/output/yt_06.midi");
    let midi_07 = load("fixtures/output/yt_07.midi");
    let midi_08 = load("fixtures/output/yt_08.midi");
    let pdf = load("fixtures/output/h6_west.pdf");
    let source = load("fixtures/data/hurrian_h6.txt");
    let ly = load("fixtures/lilypond/h6_west.ly");
    let erdfa = load("fixtures/output/retro-sync.tar");
    let cunei = "𒀸𒌑𒄴𒊑 𒄿𒊭𒅈𒌈 𒂊𒁍𒁍 𒉌𒀉𒃻 𒃻𒇷𒌈 𒆠𒁴𒈬 𒁉𒌈 𒊺𒊒 𒊭𒅖𒊭𒌈 𒊑𒁍𒌈 𒅖𒄣 𒋾𒌅𒅈𒃻 𒋾𒌅𒅈𒄿 𒊺𒅈𒁺 𒀀𒈬𒊏𒁉".as_bytes().to_vec();

    let wit_dir = Path::new("fixtures/output/witnesses");
    let witnesses: Vec<u8> = ["00_source", "01_midi", "01_pdf", "02_wav", "99_commitment"]
        .iter()
        .flat_map(|n| std::fs::read(wit_dir.join(format!("{n}.witness.json"))).unwrap_or_default())
        .collect();

    // Build NFT7 payload
    let segments: Vec<(&str, &[u8])> = vec![
        ("wav",        &wav),
        ("midi_west",  &midi_west),
        ("midi_01",    &midi_01),
        ("midi_04",    &midi_04),
        ("midi_06",    &midi_06),
        ("midi_07",    &midi_07),
        ("midi_08",    &midi_08),
        ("pdf",        &pdf),
        ("source",     &source),
        ("lilypond",   &ly),
        ("cuneiform",  &cunei),
        ("witnesses",  &witnesses),
        ("erdfa",      &erdfa),
    ];
    let payload = stego::nft7_encode(&segments);
    let payload_hash = hex::encode(&Sha256::digest(&payload)[..8]);

    println!("\n=== SVG → PNG Steganography ===");
    println!("tiles:    71 × {}×{}", stego::W, stego::H);
    println!("capacity: {} B/tile × 71 = {} B", stego::TILE_CAP, stego::TOTAL_CAP);
    println!("payload:  {} B ({:.1} MB) [sha256:{payload_hash}]", payload.len(), payload.len() as f64 / 1048576.0);
    for (name, data) in &segments {
        if !data.is_empty() { println!("  {name:12} {:>10} B", data.len()); }
    }
    println!("fill:     {:.1}%\n", payload.len() as f64 / stego::TOTAL_CAP as f64 * 100.0);

    if payload.len() > stego::TOTAL_CAP {
        eprintln!("ERROR: payload exceeds capacity");
        std::process::exit(1);
    }

    let chunks = stego::split_payload(&payload, 71);
    let mut verified = 0u32;

    for idx in 1..=71u64 {
        let pad = format!("{:02}", idx);
        let svg_path = svg_dir.join(format!("{pad}.svg"));
        if !svg_path.exists() {
            println!("⚠ {pad}.svg missing, skipping");
            continue;
        }

        // Photo background → RGB (high-entropy carrier for invisible stego)
        let bg_path = Path::new("fixtures/output/nft71_bg").join(format!("{pad}.png"));
        let mut rgb = if bg_path.exists() {
            // Use photo background (Yazılıkaya relief crops)
            let f = std::fs::File::open(&bg_path).unwrap();
            let dec = png::Decoder::new(f);
            let mut reader = dec.read_info().unwrap();
            let mut buf = vec![0u8; reader.output_buffer_size()];
            let info = reader.next_frame(&mut buf).unwrap();
            let bg_rgb = buf[..info.buffer_size()].to_vec();

            // Composite: render SVG text overlay onto photo background
            let svg_data = std::fs::read(&svg_path).unwrap();
            let overlay = stego::svg_to_rgb(&svg_data, stego::W as u32, stego::H as u32);

            // Composite: SVG text panels opaque, photo shows through SVG background
            let mut composited = bg_rgb.clone();
            for px in 0..stego::PIXELS {
                let si = px * 3;
                if si + 2 >= overlay.len() || si + 2 >= composited.len() { break; }
                let or_ = overlay[si] as u16;
                let og = overlay[si+1] as u16;
                let ob = overlay[si+2] as u16;
                let brightness = (or_ + og + ob) / 3;
                
                if brightness > 100 {
                    // Bright pixel = text or panel → use SVG (opaque)
                    composited[si]   = overlay[si];
                    composited[si+1] = overlay[si+1];
                    composited[si+2] = overlay[si+2];
                } else if brightness > 60 {
                    // Mid-tone = SVG background color → blend 50/50
                    composited[si]   = ((or_ + composited[si] as u16) / 2) as u8;
                    composited[si+1] = ((og + composited[si+1] as u16) / 2) as u8;
                    composited[si+2] = ((ob + composited[si+2] as u16) / 2) as u8;
                }
                // Dark pixel = SVG empty area → keep photo (100%)
            }
            composited
        } else {
            // Fallback: rasterize SVG only
            let svg_data = std::fs::read(&svg_path).unwrap();
            stego::svg_to_rgb(&svg_data, stego::W as u32, stego::H as u32)
        };

        // Embed stego
        let chunk = &chunks[(idx - 1) as usize];
        stego::embed(&mut rgb, chunk);

        // Write PNG (no gamma)
        let png_path = png_dir.join(format!("{pad}.png"));
        stego::write_png(&png_path, &rgb, stego::W as u32, stego::H as u32);

        // Verify round-trip
        let extracted = stego::extract(&rgb, stego::TILE_CAP);
        let ok = extracted == *chunk;
        if ok { verified += 1; }

        let used = {
            let start = (idx as usize - 1) * stego::TILE_CAP;
            if start < payload.len() { (start + stego::TILE_CAP).min(payload.len()) - start } else { 0 }
        };
        let hash = hex::encode(&Sha256::digest(chunk)[..4]);
        let marker = if stego::PRIMES.contains(&idx) { "★" } else { "·" };
        println!("{marker} {pad} — {used:>6}B payload  [{hash}] {}", if ok { "✓" } else { "✗" });
    }

    println!("\n=== Summary ===");
    println!("verified: {verified}/71");
    println!("payload:  {} B ({:.1} MB)", payload.len(), payload.len() as f64 / 1048576.0);
    println!("hash:     {payload_hash}");
    println!("\n→ {verified} stego PNGs in {}", png_dir.display());
}
