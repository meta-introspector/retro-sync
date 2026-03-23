//! stego — 6-layer bit-plane steganography + NFT7 container
//!
//! Same code compiles native (tests, CLI) and WASM (browser viewer).
//! Bit layout: per pixel R0 G0 B0 R1 G1 B1, then next pixel.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// ── Constants ──────────────────────────────────────────────────────
pub const W: usize = 512;
pub const H: usize = 512;
pub const PIXELS: usize = W * H;
pub const PLANES: usize = 6;
pub const TILE_CAP: usize = PIXELS * PLANES / 8; // 196,608 B
pub const TILE_COUNT: usize = 71;
pub const TOTAL_CAP: usize = TILE_CAP * TILE_COUNT;
pub const PRIMES: &[u64] = &[2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71];

// ── Core bit-plane ops (work on raw RGB bytes, 3 bytes/pixel) ─────

/// Embed `data` into RGB pixel buffer using 6 bit planes.
pub fn embed(rgb: &mut [u8], data: &[u8]) {
    for (i, &byte) in data.iter().enumerate() {
        if i >= TILE_CAP { break; }
        for b in 0..8u8 {
            let bit_idx = i * 8 + b as usize;
            let px = bit_idx / PLANES;
            let plane = bit_idx % PLANES;
            if px >= PIXELS { return; }
            let ch = plane % 3;
            let bit_pos = plane / 3;
            let idx = px * 3 + ch;
            let val = (byte >> b) & 1;
            rgb[idx] = (rgb[idx] & !(1 << bit_pos)) | (val << bit_pos);
        }
    }
}

/// Extract `length` bytes from RGB pixel buffer.
pub fn extract(rgb: &[u8], length: usize) -> Vec<u8> {
    (0..length.min(TILE_CAP))
        .map(|i| {
            (0..8u8)
                .map(|b| {
                    let bit_idx = i * 8 + b as usize;
                    let px = bit_idx / PLANES;
                    let plane = bit_idx % PLANES;
                    if px >= PIXELS { return 0; }
                    let ch = plane % 3;
                    let bit_pos = plane / 3;
                    let idx = px * 3 + ch;
                    ((rgb[idx] >> bit_pos) & 1) << b
                })
                .sum()
        })
        .collect()
}

/// Extract from RGBA pixel buffer (Canvas getImageData format, 4 bytes/pixel).
pub fn extract_rgba(rgba: &[u8], length: usize) -> Vec<u8> {
    (0..length.min(TILE_CAP))
        .map(|i| {
            (0..8u8)
                .map(|b| {
                    let bit_idx = i * 8 + b as usize;
                    let px = bit_idx / PLANES;
                    let plane = bit_idx % PLANES;
                    if px >= PIXELS { return 0; }
                    let ch = plane % 3;
                    let bit_pos = plane / 3;
                    let idx = px * 4 + ch; // RGBA stride
                    ((rgba[idx] >> bit_pos) & 1) << b
                })
                .sum()
        })
        .collect()
}

// ── NFT7 container ────────────────────────────────────────────────

pub struct Segment {
    pub name: String,
    pub data: Vec<u8>,
}

/// Build NFT7 payload: magic + segment_count + [name_len + name + data_len + data]...
pub fn nft7_encode(segments: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"NFT7");
    out.extend_from_slice(&(segments.len() as u32).to_le_bytes());
    for (name, data) in segments {
        let nb = name.as_bytes();
        out.extend_from_slice(&(nb.len() as u32).to_le_bytes());
        out.extend_from_slice(nb);
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(data);
    }
    out
}

/// Parse NFT7 payload → Vec<Segment>. Returns None on bad magic.
pub fn nft7_decode(data: &[u8]) -> Option<Vec<Segment>> {
    if data.len() < 8 || &data[0..4] != b"NFT7" { return None; }
    let seg_count = u32::from_le_bytes(data[4..8].try_into().ok()?) as usize;
    let mut off = 8;
    let mut segs = Vec::with_capacity(seg_count);
    for _ in 0..seg_count {
        if off + 4 > data.len() { break; }
        let nl = u32::from_le_bytes(data[off..off+4].try_into().ok()?) as usize;
        off += 4;
        if off + nl + 4 > data.len() { break; }
        let name = String::from_utf8_lossy(&data[off..off+nl]).into_owned();
        off += nl;
        let dl = u32::from_le_bytes(data[off..off+4].try_into().ok()?) as usize;
        off += 4;
        if off + dl > data.len() { break; }
        segs.push(Segment { name, data: data[off..off+dl].to_vec() });
        off += dl;
    }
    Some(segs)
}

/// Split payload across N tiles, returning Vec of chunks (each TILE_CAP bytes, zero-padded).
pub fn split_payload(payload: &[u8], n: usize) -> Vec<Vec<u8>> {
    (0..n)
        .map(|i| {
            let start = i * TILE_CAP;
            let mut chunk = vec![0u8; TILE_CAP];
            if start < payload.len() {
                let end = (start + TILE_CAP).min(payload.len());
                chunk[..end - start].copy_from_slice(&payload[start..end]);
            }
            chunk
        })
        .collect()
}

/// Reassemble payload from tile chunks (concatenate, trim trailing zeros).
pub fn join_payload(chunks: &[Vec<u8>]) -> Vec<u8> {
    let mut out: Vec<u8> = chunks.iter().flat_map(|c| c.iter().copied()).collect();
    // Don't trim — NFT7 header tells us exact segment lengths
    out
}

// ── PNG output (no gamma, no ICC — safe for browser Canvas) ───────

#[cfg(feature = "png")]
pub fn write_png(path: &std::path::Path, rgb: &[u8], w: u32, h: u32) {
    let file = std::fs::File::create(path).unwrap();
    let ref mut bw = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(bw, w, h);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    // No set_srgb, no set_source_gamma — bare PNG, exact pixel values
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(rgb).unwrap();
}

// ── SVG → RGB rasterization (no gamma, no ICC) ───────────────────

/// Rasterize SVG to RGB pixel buffer at given dimensions.
#[cfg(feature = "svg")]
pub fn svg_to_rgb(svg_data: &[u8], w: u32, h: u32) -> Vec<u8> {
    let tree = resvg::usvg::Tree::from_data(svg_data, &Default::default()).unwrap();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).unwrap();
    let sx = w as f32 / tree.size().width();
    let sy = h as f32 / tree.size().height();
    let transform = resvg::tiny_skia::Transform::from_scale(sx, sy);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    // RGBA → RGB
    pixmap.data().chunks(4).flat_map(|c| [c[0], c[1], c[2]]).collect()
}

// ── WASM bindings ─────────────────────────────────────────────────

use sha2::{Digest, Sha256};

#[cfg(feature = "wasm")]
#[wasm_bindgen]
/// Decode one tile from RGBA pixel data. Returns raw stego bytes (TILE_CAP).
pub fn decode_tile(rgba: &[u8]) -> Vec<u8> {
    extract_rgba(rgba, TILE_CAP)
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
/// Reconstruct NFT7 payload from concatenated tile bytes.
/// Returns JSON with segments including sha256 hashes.
pub fn reconstruct_payload(all_bytes: &[u8]) -> String {
    match nft7_decode(all_bytes) {
        None => {
            let hex: String = all_bytes.iter().take(32)
                .map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
            format!("{{\"error\":\"bad magic\",\"first32\":\"{hex}\"}}")
        }
        Some(segs) => {
            let payload_hash = hex::encode(Sha256::digest(all_bytes));
            let items: Vec<String> = segs.iter().map(|s| {
                let hash = hex::encode(Sha256::digest(&s.data));
                format!("{{\"name\":\"{}\",\"size\":{},\"sha256\":\"{hash}\"}}", s.name, s.data.len())
            }).collect();
            format!("{{\"segments\":[{}],\"payload_sha256\":\"{payload_hash}\"}}",
                items.join(","))
        }
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
/// Extract a named segment from the payload. Returns empty vec if not found.
pub fn extract_segment(all_bytes: &[u8], name: &str) -> Vec<u8> {
    nft7_decode(all_bytes)
        .and_then(|segs| segs.into_iter().find(|s| s.name == name).map(|s| s.data))
        .unwrap_or_default()
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_single_tile() {
        let data = b"Hello, Hurrian Hymn h.6!";
        let mut rgb = vec![128u8; PIXELS * 3];
        embed(&mut rgb, data);
        let out = extract(&rgb, data.len());
        assert_eq!(&out, data);
    }

    #[test]
    fn round_trip_rgba_extract() {
        let data = b"RGBA test payload";
        let mut rgb = vec![128u8; PIXELS * 3];
        embed(&mut rgb, data);
        // Convert RGB → RGBA
        let mut rgba = vec![255u8; PIXELS * 4];
        for px in 0..PIXELS {
            rgba[px * 4]     = rgb[px * 3];
            rgba[px * 4 + 1] = rgb[px * 3 + 1];
            rgba[px * 4 + 2] = rgb[px * 3 + 2];
        }
        let out = extract_rgba(&rgba, data.len());
        assert_eq!(&out, data);
    }

    #[test]
    fn round_trip_nft7() {
        let segs: Vec<(&str, &[u8])> = vec![
            ("wav",    b"RIFF fake wav data here"),
            ("midi",   b"MThd midi"),
            ("source", b"Hurrian text"),
        ];
        let payload = nft7_encode(&segs);
        assert_eq!(&payload[..4], b"NFT7");
        let decoded = nft7_decode(&payload).unwrap();
        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[0].name, "wav");
        assert_eq!(decoded[0].data, b"RIFF fake wav data here");
        assert_eq!(decoded[2].name, "source");
        assert_eq!(decoded[2].data, b"Hurrian text");
    }

    #[test]
    fn round_trip_full_pipeline() {
        // Build a payload with known segments
        let wav = vec![0xABu8; 5000];
        let midi = vec![0xCDu8; 200];
        let src = b"cuneiform text here".to_vec();
        let segs: Vec<(&str, &[u8])> = vec![
            ("wav",    &wav),
            ("midi",   &midi),
            ("source", &src),
        ];
        let payload = nft7_encode(&segs);

        // Split across 3 tiles (enough for this small payload)
        let n = 3;
        let chunks = split_payload(&payload, n);
        assert_eq!(chunks.len(), n);
        assert_eq!(chunks[0].len(), TILE_CAP);

        // Embed each chunk into synthetic RGB tiles, then extract
        let mut extracted_chunks = Vec::new();
        for chunk in &chunks {
            let mut rgb = vec![128u8; PIXELS * 3];
            embed(&mut rgb, chunk);
            let out = extract(&rgb, TILE_CAP);
            assert_eq!(&out, chunk, "chunk round-trip failed");
            extracted_chunks.push(out);
        }

        // Reassemble and decode NFT7
        let reassembled = join_payload(&extracted_chunks);
        let decoded = nft7_decode(&reassembled).unwrap();
        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[0].name, "wav");
        assert_eq!(decoded[0].data, wav);
        assert_eq!(decoded[1].name, "midi");
        assert_eq!(decoded[1].data, midi);
        assert_eq!(decoded[2].name, "source");
        assert_eq!(decoded[2].data, src);
    }

    #[test]
    fn round_trip_full_pipeline_rgba() {
        // Same as above but extract via RGBA (simulates Canvas getImageData)
        let wav = vec![0x42u8; 8000];
        let segs: Vec<(&str, &[u8])> = vec![("wav", &wav)];
        let payload = nft7_encode(&segs);
        let chunks = split_payload(&payload, 2);

        let mut extracted_chunks = Vec::new();
        for chunk in &chunks {
            let mut rgb = vec![128u8; PIXELS * 3];
            embed(&mut rgb, chunk);
            // RGB → RGBA
            let mut rgba = vec![255u8; PIXELS * 4];
            for px in 0..PIXELS {
                rgba[px * 4]     = rgb[px * 3];
                rgba[px * 4 + 1] = rgb[px * 3 + 1];
                rgba[px * 4 + 2] = rgb[px * 3 + 2];
            }
            let out = extract_rgba(&rgba, TILE_CAP);
            assert_eq!(&out, chunk);
            extracted_chunks.push(out);
        }

        let reassembled = join_payload(&extracted_chunks);
        let decoded = nft7_decode(&reassembled).unwrap();
        assert_eq!(decoded[0].name, "wav");
        assert_eq!(decoded[0].data, wav);
    }
}
