//! nft71_ppm_stego — Multi-layer steganographic embedding in 512×512 PPM tiles
//!
//! Stuffs the ENTIRE WAV (8.4MB) + MIDI + PDF + source + witnesses + cuneiform
//! into 71 PPM tiles using 6 bit-plane layers (R/G/B × bits 0,1).
//!
//! Capacity: 512×512 = 262144 px → 196,608 B/tile → 13.3 MB across 71 tiles
//! Payload:  WAV 8.4MB + PDF 69KB + MIDI 606B + text ~50KB = ~8.5MB  ✓ FITS
//!
//! Usage: cargo run -p fixtures --example nft71_ppm_stego

use sha2::{Digest, Sha256};
use std::path::Path;

const W: usize = 512;
const H: usize = 512;
const PIXELS: usize = W * H;
const PLANES: usize = 6; // R0 G0 B0 R1 G1 B1
const STEGO_CAP: usize = PIXELS * PLANES / 8; // 196,608 bytes per tile

struct Ppm { data: Vec<u8> }

impl Ppm {
    fn read(path: &Path) -> Self {
        let raw = std::fs::read(path).unwrap();
        let mut pos = 0;
        let mut nl = 0;
        for (i, &b) in raw.iter().enumerate() {
            if b == b'\n' { nl += 1; }
            if nl == 3 { pos = i + 1; break; }
        }
        Ppm { data: raw[pos..].to_vec() }
    }

    fn write(&self, path: &Path) {
        let mut out = format!("P6\n{W} {H}\n255\n").into_bytes();
        out.extend_from_slice(&self.data);
        std::fs::write(path, out).unwrap();
    }

    /// Embed a contiguous blob using all 6 bit planes as one stream.
    /// Bit order: for each pixel, R0 G0 B0 R1 G1 B1, then next pixel.
    fn embed_all(&mut self, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            if i >= STEGO_CAP { break; }
            for b in 0..8u8 {
                let bit_idx = i * 8 + b as usize;
                let px = bit_idx / PLANES;
                let plane = bit_idx % PLANES;
                if px >= PIXELS { return; }
                let channel = plane % 3;       // R=0 G=1 B=2
                let bit_pos = plane / 3;       // 0 or 1
                let idx = px * 3 + channel;
                let val = (byte >> b) & 1;
                self.data[idx] = (self.data[idx] & !(1 << bit_pos)) | (val << bit_pos);
            }
        }
    }

    /// Extract blob from all 6 bit planes.
    fn extract_all(&self, length: usize) -> Vec<u8> {
        (0..length.min(STEGO_CAP))
            .map(|i| {
                (0..8u8)
                    .map(|b| {
                        let bit_idx = i * 8 + b as usize;
                        let px = bit_idx / PLANES;
                        let plane = bit_idx % PLANES;
                        if px >= PIXELS { return 0; }
                        let channel = plane % 3;
                        let bit_pos = plane / 3;
                        let idx = px * 3 + channel;
                        ((self.data[idx] >> bit_pos) & 1) << b
                    })
                    .sum()
            })
            .collect()
    }
}

fn main() {
    let ppm_dir = Path::new("fixtures/output/nft71_ppm");
    let out_dir = Path::new("fixtures/output/nft71_stego_ppm");
    std::fs::create_dir_all(out_dir).unwrap();

    // Load all music data
    let wav = std::fs::read("fixtures/output/h6_west.wav").unwrap();
    let midi = std::fs::read("fixtures/output/h6_west.midi").unwrap();
    let pdf = std::fs::read("fixtures/output/h6_west.pdf").unwrap();
    let source = std::fs::read("fixtures/data/hurrian_h6.txt").unwrap();
    let ly = std::fs::read("fixtures/lilypond/h6_west.ly").unwrap();

    // Cuneiform UTF-8
    let cunei = "𒀸𒌑𒄴𒊑 𒄿𒊭𒅈𒌈 𒂊𒁍𒁍 𒉌𒀉𒃻 𒃻𒇷𒌈 𒆠𒁴𒈬 𒁉𒌈 𒊺𒊒 𒊭𒅖𒊭𒌈 𒊑𒁍𒌈 𒅖𒄣 𒋾𒌅𒅈𒃻 𒋾𒌅𒅈𒄿 𒊺𒅈𒁺 𒀀𒈬𒊏𒁉".as_bytes();

    // Witness chain
    let wit_dir = Path::new("fixtures/output/witnesses");
    let witnesses: Vec<u8> = ["00_source", "01_midi", "01_pdf", "02_wav", "99_commitment"]
        .iter()
        .flat_map(|n| std::fs::read(wit_dir.join(format!("{n}.witness.json"))).unwrap_or_default())
        .collect();

    // Build combined payload with length-prefixed segments
    // Format: [magic:4][segment_count:4][len0:4][data0...][len1:4][data1...]...
    let segments: Vec<(&str, &[u8])> = vec![
        ("wav",       &wav),
        ("midi",      &midi),
        ("pdf",       &pdf),
        ("source",    &source),
        ("lilypond",  &ly),
        ("cuneiform", cunei),
        ("witnesses", &witnesses),
    ];

    let mut payload = Vec::new();
    payload.extend_from_slice(b"NFT7");  // magic
    payload.extend_from_slice(&(segments.len() as u32).to_le_bytes());
    for (name, data) in &segments {
        let name_bytes = name.as_bytes();
        payload.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        payload.extend_from_slice(name_bytes);
        payload.extend_from_slice(&(data.len() as u32).to_le_bytes());
        payload.extend_from_slice(data);
    }

    let payload_hash = hex::encode(&Sha256::digest(&payload)[..8]);
    let total_cap = STEGO_CAP * 71;

    println!("=== Multi-Layer PPM Steganography — Full Music Encoding ===");
    println!("tiles:    71 × {W}×{H} PPM");
    println!("capacity: {STEGO_CAP} B/tile × 71 = {total_cap} B ({:.1} MB)", total_cap as f64 / 1048576.0);
    println!("payload:  {} B ({:.1} MB) [sha256:{payload_hash}]", payload.len(), payload.len() as f64 / 1048576.0);
    for (name, data) in &segments {
        println!("  {name:12} {:>10} B", data.len());
    }
    println!("fill:     {:.1}%", payload.len() as f64 / total_cap as f64 * 100.0);
    println!();

    if payload.len() > total_cap {
        eprintln!("ERROR: payload ({}) exceeds capacity ({})", payload.len(), total_cap);
        std::process::exit(1);
    }

    // Split payload across 71 tiles
    let chunk_size = STEGO_CAP;
    let mut verified = 0u32;

    for idx in 1..=71u64 {
        let padded = format!("{:02}", idx);
        let ppm_path = ppm_dir.join(format!("{padded}.ppm"));
        if !ppm_path.exists() { continue; }

        let mut ppm = Ppm::read(&ppm_path);
        let i = (idx - 1) as usize;
        let start = i * chunk_size;
        let end = (start + chunk_size).min(payload.len());

        // Pad chunk to full capacity
        let mut chunk = vec![0u8; chunk_size];
        if start < payload.len() {
            let len = end - start;
            chunk[..len].copy_from_slice(&payload[start..end]);
        }

        ppm.embed_all(&chunk);
        let out_path = out_dir.join(format!("{padded}.ppm"));
        ppm.write(&out_path);

        // Also write stripped PNG (no gamma — safe for browser Canvas stego)
        let png_dir = Path::new("fixtures/output/nft71_stego_png");
        std::fs::create_dir_all(png_dir).unwrap();
        stego::write_png(&png_dir.join(format!("{padded}.png")), &ppm.data, W as u32, H as u32);

        // Verify
        let verify = Ppm::read(&out_path);
        let extracted = verify.extract_all(chunk_size);
        let ok = extracted == chunk;
        if ok { verified += 1; }

        let used = if start < payload.len() { (end - start).min(chunk_size) } else { 0 };
        let marker = if fixtures::hurrian_h6::SSP.contains(&idx) { "★" } else { "·" };
        let hash = hex::encode(&Sha256::digest(&chunk)[..4]);
        println!("{marker} {padded} — {used:>6}B payload  [{hash}] {}", if ok { "✓" } else { "✗" });
    }

    println!();
    println!("=== Summary ===");
    println!("verified: {verified}/71");
    println!("payload:  {} B ({:.1} MB) stuffed into 71 tiles", payload.len(), payload.len() as f64 / 1048576.0);
    println!("segments: {}", segments.iter().map(|(n,_)| *n).collect::<Vec<_>>().join(", "));
    println!("hash:     {payload_hash}");
    println!("\n→ {verified} stego PPMs in {}", out_dir.display());
    println!("→ Extract: collect all 71, read 6 bit planes, concat, parse NFT7 header");
}
