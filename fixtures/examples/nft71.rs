//! nft71 — Encode ALL Hurrian Hymn h.6 data into 71 DA51 CBOR shards.
//!
//! Every shard contains real data: source text, LilyPond notation, MIDI bytes,
//! PDF bytes, WAV audio, witness JSON, references, and eigenspace analysis.
//!
//! Prime-indexed shards (20) are "generators" — SSP intervals + CFT structure.
//! Composite-indexed shards (51) are "derived" — content from factorization.
//!
//! Usage: cargo run -p fixtures --example nft71

use fixtures::hurrian_h6::{self, SSP, INTERVAL_NAMES};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

const PRIMES: [u64; 20] = [2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71];
fn is_prime(n: u64) -> bool { PRIMES.contains(&n) }

fn factorize(mut n: u64) -> String {
    let mut parts = Vec::new();
    for &p in &PRIMES {
        let mut exp = 0u32;
        while n % p == 0 { n /= p; exp += 1; }
        if exp > 0 {
            parts.push(if exp == 1 { format!("{p}") } else { format!("{p}^{e}", e=exp) });
        }
    }
    parts.join("·")
}

fn read_text(p: &str) -> String { std::fs::read_to_string(p).unwrap_or_default() }
fn read_b64(p: &str) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    std::fs::read(p).map(|b| STANDARD.encode(&b)).unwrap_or_default()
}
fn file_hash(p: &str) -> String {
    std::fs::read(p).map(|b| hex::encode(Sha256::digest(&b))).unwrap_or_default()
}

/// Build a DA51 CBOR shard from a JSON value.
fn da51_wrap(payload: &serde_json::Value) -> Vec<u8> {
    let json_bytes = serde_json::to_vec(payload).unwrap();
    let hash = Sha256::digest(&json_bytes);
    let mut cbor = vec![0xda, 0x51];
    cbor.extend_from_slice(&hash[..8]);
    let val: ciborium::Value = serde_json::from_slice(&json_bytes).unwrap();
    ciborium::into_writer(&val, &mut cbor).unwrap();
    cbor
}

fn main() {
    let base = Path::new("fixtures");
    let out_dir = base.join("output");
    let wit_dir = out_dir.join("witnesses");

    // Pre-compute eigenspace
    let eigen = hurrian_h6::embed_h6();
    let colophon = hurrian_h6::h6_colophon();
    let notation = hurrian_h6::h6_notation();

    // Load all real data
    let src_text = read_text("fixtures/data/hurrian_h6.txt");
    let ly_text = read_text("fixtures/lilypond/h6_west.ly");
    let refs_text = read_text("fixtures/data/references.txt");
    let yt_text = read_text("fixtures/data/youtube_sources.txt");

    let midi_b64 = read_b64("fixtures/output/h6_west.midi");
    let pdf_b64 = read_b64("fixtures/output/h6_west.pdf");
    let wav_b64 = read_b64("fixtures/output/h6_west.wav");

    let midi_hash = file_hash("fixtures/output/h6_west.midi");
    let pdf_hash = file_hash("fixtures/output/h6_west.pdf");
    let wav_hash = file_hash("fixtures/output/h6_west.wav");

    let w_source = read_text(&wit_dir.join("00_source.witness.json").to_string_lossy());
    let w_midi = read_text(&wit_dir.join("01_midi.witness.json").to_string_lossy());
    let w_pdf = read_text(&wit_dir.join("01_pdf.witness.json").to_string_lossy());
    let w_wav = read_text(&wit_dir.join("02_wav.witness.json").to_string_lossy());
    let w_commit = read_text(&wit_dir.join("99_commitment.witness.json").to_string_lossy());

    // Parse references into lines
    let ref_urls: Vec<&str> = refs_text.lines()
        .filter(|l| l.starts_with("http"))
        .collect();
    let yt_urls: Vec<&str> = yt_text.lines()
        .filter(|l| l.starts_with("http"))
        .collect();

    // Notation entries as JSON
    let notation_json: Vec<serde_json::Value> = notation.iter().map(|e| {
        serde_json::json!({"term": e.term, "count": e.count})
    }).collect();

    // Common header for every shard
    let header = |idx: u64, cat: &str, name: &str| -> serde_json::Value {
        serde_json::json!({
            "shard_index": idx,
            "of": 71,
            "prime": is_prime(idx),
            "factors": factorize(idx),
            "category": cat,
            "name": name,
            "ssp_member": SSP.contains(&idx),
            "collection": "Hurrian Hymn h.6 — 71-Shard NFT",
            "algebra": "Cl(15,0,0)",
            "pipeline": "retro-sync/nft71",
            "version": "0.3.0",
        })
    };

    // === DATA LAYERS — all content to be striped across 71 shards ===
    let witnesses = serde_json::json!([
        serde_json::from_str::<serde_json::Value>(&w_source).unwrap_or_default(),
        serde_json::from_str::<serde_json::Value>(&w_midi).unwrap_or_default(),
        serde_json::from_str::<serde_json::Value>(&w_pdf).unwrap_or_default(),
        serde_json::from_str::<serde_json::Value>(&w_wav).unwrap_or_default(),
        serde_json::from_str::<serde_json::Value>(&w_commit).unwrap_or_default(),
    ]);
    let eigenspace_json = serde_json::json!({
        "earth": eigen.earth_pct, "spoke": eigen.spoke_pct, "hub": eigen.hub_pct,
        "grade_energy": eigen.grade_energy.to_vec(),
        "fractran_state": format!("{}", eigen.fractran_state),
        "triplet_count": eigen.triplet_count,
    });
    let metadata_json = serde_json::json!({
        "tablet": "RS 15.30 + 15.49 + 17.387", "site": "Royal Palace, Ugarit (Ras Shamra, Syria)",
        "scribe": "Ammurabi", "tuning": "nīd qablim (nid qabli)", "strings": 9,
        "instrument": "sammûm", "deity": "Nikkal", "genre": "zaluzi",
        "date": "~1400 BC", "coordinates": "35.6°N 35.78°E",
    });
    let reconstructions_json = serde_json::json!([
        {"scholar": "M. L. West", "year": 1994, "approach": "dichords on descending diatonic scale", "notation": notation_json},
        {"scholar": "Anne D. Kilmer", "year": 1974, "approach": "first modern reconstruction, ascending scale"},
        {"scholar": "Marcelle Duchesne-Guillemin", "years": "1975, 1984", "approach": "melodic interpretation of interval names"},
        {"scholar": "Richard Dumbrill", "approach": "organological analysis, used by Peter Pringle"},
        {"scholar": "Raoul Gregory Vitale", "approach": "alternative interval reading"},
    ]);
    let refs_json: Vec<serde_json::Value> = ref_urls.iter().map(|u| serde_json::json!(u)).collect();
    let yt_json: Vec<serde_json::Value> = yt_urls.iter().map(|u| serde_json::json!(u)).collect();
    let pipeline_json = serde_json::json!({
        "sop": read_text("datasets/SOP-RETROSYNC-PUB-001.md"),
        "erdfa_cft": "e1=file(p2) → e2=para(p3) → e3=col(p5) → e4=line(p7) → e5=token(p11) → e6=byte(p13) → e7=emoji(p17) → e8=unicode(p19) → e9=bit(p23)",
        "boustrophedon": {"method": "Way of the Oxen", "rows": 3},
        "cl15": {"algebra": "Cl(15,0,0)", "generators": 15, "dimension": "2^15 = 32768"},
    });

    // Load PPM tile images (71 tiles, one per shard)
    let ppm_dir = out_dir.join("nft71_ppm");
    let mut ppm_mosaic = Vec::new();
    for i in 1..=71u64 {
        let path = ppm_dir.join(format!("{:02}.ppm", i));
        ppm_mosaic.extend(std::fs::read(&path).unwrap_or_default());
    }

    // Collect all layers as named byte blobs, then stripe across 71 shards
    let layers: Vec<(&str, Vec<u8>)> = vec![
        ("source",          src_text.as_bytes().to_vec()),
        ("lilypond",        ly_text.as_bytes().to_vec()),
        ("midi",            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &midi_b64).unwrap_or_default()),
        ("pdf",             base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &pdf_b64).unwrap_or_default()),
        ("wav",             base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &wav_b64).unwrap_or_default()),
        ("ppm_tiles",       ppm_mosaic),
        ("witnesses",       serde_json::to_vec(&witnesses).unwrap()),
        ("eigenspace",      serde_json::to_vec(&eigenspace_json).unwrap()),
        ("metadata",        serde_json::to_vec(&metadata_json).unwrap()),
        ("reconstructions", serde_json::to_vec(&reconstructions_json).unwrap()),
        ("references",      serde_json::to_vec(&serde_json::json!(refs_json)).unwrap()),
        ("youtube",         serde_json::to_vec(&serde_json::json!(yt_json)).unwrap()),
        ("pipeline",        serde_json::to_vec(&pipeline_json).unwrap()),
        ("colophon",        serde_json::to_vec(&serde_json::json!(colophon)).unwrap()),
    ];

    // SHA-256 of each complete layer for reconstruction verification
    let layer_hashes: Vec<(&str, String)> = layers.iter()
        .map(|(name, data)| (*name, hex::encode(Sha256::digest(data))))
        .collect();

    // Stripe: for each layer, split into 71 chunks (round-robin byte assignment)
    fn stripe(data: &[u8], n: usize) -> Vec<Vec<u8>> {
        let mut chunks: Vec<Vec<u8>> = (0..n).map(|_| Vec::new()).collect();
        for (i, &b) in data.iter().enumerate() {
            chunks[i % n].push(b);
        }
        chunks
    }

    let striped: Vec<(&str, Vec<Vec<u8>>)> = layers.iter()
        .map(|(name, data)| (*name, stripe(data, 71)))
        .collect();

    let mut shards: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let mut payloads: BTreeMap<u64, serde_json::Value> = BTreeMap::new();
    let mut manifest = Vec::new();

    for idx in 1..=71u64 {
        let i = (idx - 1) as usize;

        // Determine shard's primary identity
        let (cat, name) = if is_prime(idx) {
            let ssp_pos = SSP.iter().position(|&p| p == idx);
            ("generator", ssp_pos.map(|j| INTERVAL_NAMES[j]).unwrap_or("cft-prime"))
        } else {
            match idx {
                4|6 => ("source", "text"),
                8|9|10 => ("artifact", "binary"),
                12|14|15|16|18 => ("witness", "chain"),
                20..=25 => ("eigenspace", "cl15"),
                26..=35 => ("metadata", "tablet"),
                36..=42 => ("reconstruction", "scholarly"),
                44..=57 => ("reference", "url"),
                58..=65 => ("youtube", "private"),
                66..=70 => ("pipeline", "sop"),
                _ => ("reserved", "future"),
            }
        };

        let mut s = header(idx, cat, name);

        // Generator-specific: SSP interval data
        if is_prime(idx) {
            if let Some(j) = SSP.iter().position(|&p| p == idx) {
                s["interval"] = serde_json::json!({
                    "name": INTERVAL_NAMES[j], "ssp_index": j, "prime": SSP[j],
                    "string_pair": match j {
                        0 => "1-5", 1 => "2-6", 2 => "3-7", 3 => "4-1",
                        4 => "5-2", 5 => "6-3", 6 => "7-4", 7 => "7-5",
                        8 => "1-6", 9 => "2-7", 10 => "1-3", 11 => "2-4",
                        12 => "3-5", 13 => "4-6", _ => "colophon",
                    },
                    "type": if j < 7 { "primary" } else if j < 14 { "secondary" } else { "crown" },
                });
                let count: u32 = notation.iter()
                    .filter(|e| hurrian_h6::interval_to_ssp_index(&e.term) == Some(j))
                    .map(|e| e.count as u32).sum();
                s["occurrences_in_h6"] = serde_json::json!(count);
            }
        }

        // === INTERLEAVED DATA LAYERS — every shard carries a stripe of everything ===
        use base64::{Engine, engine::general_purpose::STANDARD};
        let mut data_layers = serde_json::Map::new();
        for (layer_name, chunks) in &striped {
            data_layers.insert(layer_name.to_string(), serde_json::json!({
                "chunk": i,
                "of": 71,
                "encoding": "base64",
                "data": STANDARD.encode(&chunks[i]),
                "total_bytes": layers.iter().find(|(n,_)| n == layer_name).unwrap().1.len(),
            }));
        }
        s["data_layers"] = serde_json::Value::Object(data_layers);

        // Layer hashes so any shard can verify reconstruction
        s["layer_hashes"] = serde_json::json!(
            layer_hashes.iter().map(|(n,h)| serde_json::json!({"layer": n, "sha256": h})).collect::<Vec<_>>()
        );

        // Artifact hashes for the original files
        s["artifact_hashes"] = serde_json::json!({
            "midi": midi_hash, "pdf": pdf_hash, "wav": wav_hash,
        });

        let payload = s;

        let cbor = da51_wrap(&payload);
        let hash = Sha256::digest(&serde_json::to_vec(&payload).unwrap());
        let cid = format!("bafk{}", hex::encode(&hash[..16]));
        let cat = payload["category"].as_str().unwrap_or("?");
        let name = payload["name"].as_str().unwrap_or("?");

        manifest.push(serde_json::json!({
            "index": idx,
            "cid": cid,
            "category": cat,
            "name": name,
            "prime": is_prime(idx),
            "factors": factorize(idx),
            "bytes": cbor.len(),
        }));

        shards.insert(idx, cbor);
        payloads.insert(idx, payload);
    }

    // Write shards (CBOR canonical + JSON for HF)
    let out = Path::new("fixtures/output/nft71");
    std::fs::create_dir_all(out).unwrap();
    let json_dir = out.join("json");
    std::fs::create_dir_all(&json_dir).unwrap();
    for (idx, cbor) in &shards {
        std::fs::write(out.join(format!("{:02}.cbor", idx)), cbor).unwrap();
        let json = serde_json::to_string_pretty(&payloads[idx]).unwrap();
        std::fs::write(json_dir.join(format!("{:02}.json", idx)), json).unwrap();
    }

    // Summary
    let total_bytes: usize = shards.values().map(|v| v.len()).sum();
    let generators = (1..=71u64).filter(|n| is_prime(*n)).count();

    let manifest_json = serde_json::to_string_pretty(&serde_json::json!({
        "title": "Hurrian Hymn h.6 — 71-Shard NFT Collection",
        "version": "0.3.0",
        "encoding": "interleaved — all data striped round-robin across 71 shards",
        "layers": layer_hashes.iter().map(|(n,h)| serde_json::json!({"name": n, "sha256": h})).collect::<Vec<_>>(),
        "shard_count": 71,
        "generators": generators,
        "derived": 71 - generators,
        "total_bytes": total_bytes,
        "algebra": "Cl(15,0,0)",
        "eigenspace": { "earth": eigen.earth_pct, "spoke": eigen.spoke_pct, "hub": eigen.hub_pct },
        "pipeline": "retro-sync/nft71",
        "shards": manifest,
    })).unwrap();
    std::fs::write(out.join("manifest.json"), &manifest_json).unwrap();

    println!("=== Hurrian Hymn h.6 — 71-Shard NFT Collection (real data) ===");
    println!("shards:     71 ({generators} generators + {} derived)", 71 - generators);
    println!("total:      {} bytes ({:.1} MB)", total_bytes, total_bytes as f64 / 1_048_576.0);
    println!("eigenspace: {:.0}% Earth / {:.0}% Spoke / {:.0}% Hub", eigen.earth_pct, eigen.spoke_pct, eigen.hub_pct);
    println!();
    for entry in &manifest {
        let marker = if entry["prime"].as_bool().unwrap() { "★" } else { "·" };
        let bytes = entry["bytes"].as_u64().unwrap();
        let size = if bytes > 1_000_000 { format!("{:.1}M", bytes as f64 / 1_048_576.0) }
                   else if bytes > 1000 { format!("{:.1}K", bytes as f64 / 1024.0) }
                   else { format!("{}B", bytes) };
        println!("{} {:>2} [{:<14}] {:<16} {:<30} {:>8}",
            marker, entry["index"], entry["factors"].as_str().unwrap(),
            entry["category"].as_str().unwrap(), entry["name"].as_str().unwrap(), size);
    }
    println!("\n→ {} shards written to {}", shards.len(), out.display());
}
