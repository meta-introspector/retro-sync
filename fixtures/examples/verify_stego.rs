//! Verify stego round-trip: extract NFT7 from project tiles, list all segments.

fn read_rgb(path: &std::path::Path) -> Vec<u8> {
    let f = std::fs::File::open(path).unwrap();
    let dec = png::Decoder::new(f);
    let mut reader = dec.read_info().unwrap();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    buf[..info.buffer_size()].to_vec()
}

fn main() {
    let platform: toml::Value = toml::from_str(
        &std::fs::read_to_string("retro-sync.toml").expect("retro-sync.toml not found")
    ).unwrap();

    let projects_dir = platform["platform"]["projects_dir"].as_str().unwrap();
    let project_name = std::env::args().nth(1)
        .unwrap_or_else(|| platform["platform"]["default"].as_str().unwrap().to_string());

    let project_path = format!("{}/{}/project.toml", projects_dir, project_name);
    let project: toml::Value = toml::from_str(
        &std::fs::read_to_string(&project_path).unwrap_or_else(|_| panic!("{} not found", project_path))
    ).unwrap();

    let png_dir = project["paths"]["stego_png"].as_str().unwrap();
    let count = project["tiles"]["count"].as_integer().unwrap() as u64;
    let pattern = project["tiles"]["pattern"].as_str().unwrap();
    let title = project["project"]["title"].as_str().unwrap_or(&project_name);

    println!("=== {} ===", title);

    let mut chunks: Vec<Vec<u8>> = Vec::new();
    for idx in 1..=count {
        let fname = pattern.replacen("{:02}", &format!("{:02}", idx), 1);
        let path = std::path::Path::new(png_dir).join(&fname);
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
