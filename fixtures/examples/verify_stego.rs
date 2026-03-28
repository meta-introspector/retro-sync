//! Verify stego round-trip: extract NFT7 from 71 PNGs, list all segments.

fn read_rgb(path: &std::path::Path) -> Vec<u8> {
    let f = std::fs::File::open(path).unwrap();
    let dec = png::Decoder::new(f);
    let mut reader = dec.read_info().unwrap();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    buf[..info.buffer_size()].to_vec()
}

fn main() {
    let png_dir = "fixtures/output/nft71_stego_png";
    let count = 71u64;

    println!("=== Hurrian Hymn h.6 ===");

    let mut chunks: Vec<Vec<u8>> = Vec::new();
    for idx in 1..=count {
        let path = std::path::Path::new(png_dir).join(format!("{:02}.png", idx));
        let rgb = read_rgb(&path);
        chunks.push(stego::extract(&rgb, stego::TILE_CAP));
    }

    let payload = stego::join_payload(&chunks);
    println!("tiles: {}  payload: {} B ({:.1} MB)", count, payload.len(), payload.len() as f64 / 1048576.0);

    match stego::nft7_decode(&payload) {
        Some(segments) => {
            println!("segments: {}", segments.len());
            for s in &segments {
                let magic = if s.data.len() >= 4 {
                    format!("{:02x}{:02x}{:02x}{:02x}", s.data[0], s.data[1], s.data[2], s.data[3])
                } else { "".into() };
                println!("  {:12} {:>10} B  {}", s.name, s.data.len(), magic);
            }
        }
        None => println!("ERROR: NFT7 decode failed"),
    }
}
