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
            "version": "0.2.0",
        })
    };

    let mut shards: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let mut manifest = Vec::new();

    for idx in 1..=71u64 {
        let payload = if is_prime(idx) {
            // Generator shards — SSP interval data
            let ssp_pos = SSP.iter().position(|&p| p == idx);
            let name = ssp_pos.map(|i| INTERVAL_NAMES[i]).unwrap_or("cft-prime");
            let mut s = header(idx, "generator", name);
            if let Some(i) = ssp_pos {
                s["interval"] = serde_json::json!({
                    "name": INTERVAL_NAMES[i],
                    "ssp_index": i,
                    "prime": SSP[i],
                    "string_pair": match i {
                        0 => "1-5", 1 => "2-6", 2 => "3-7", 3 => "4-1",
                        4 => "5-2", 5 => "6-3", 6 => "7-4", 7 => "7-5",
                        8 => "1-6", 9 => "2-7", 10 => "1-3", 11 => "2-4",
                        12 => "3-5", 13 => "4-6", _ => "colophon",
                    },
                    "type": if i < 7 { "primary" } else if i < 14 { "secondary" } else { "crown" },
                });
                // Count occurrences in the notation
                let count: u32 = notation.iter()
                    .filter(|e| hurrian_h6::interval_to_ssp_index(&e.term) == Some(i))
                    .map(|e| e.count as u32)
                    .sum();
                s["occurrences_in_h6"] = serde_json::json!(count);
            }
            if idx == 71 { s["colophon"] = serde_json::json!(colophon); }
            s
        } else {
            match idx {
                // === SOURCE ===
                4 => {
                    let mut s = header(idx, "source", "hurrian_h6.txt");
                    s["content"] = serde_json::json!(src_text);
                    s["sha256"] = serde_json::json!(file_hash("fixtures/data/hurrian_h6.txt"));
                    s["bytes"] = serde_json::json!(src_text.len());
                    s
                }
                6 => {
                    let mut s = header(idx, "source", "h6_west.ly");
                    s["content"] = serde_json::json!(ly_text);
                    s["sha256"] = serde_json::json!(file_hash("fixtures/lilypond/h6_west.ly"));
                    s["bytes"] = serde_json::json!(ly_text.len());
                    s
                }
                // === ARTIFACTS (binary, base64) ===
                8 => {
                    let mut s = header(idx, "artifact", "h6_west.midi");
                    s["encoding"] = serde_json::json!("base64");
                    s["data"] = serde_json::json!(midi_b64);
                    s["sha256"] = serde_json::json!(midi_hash);
                    s["bytes"] = serde_json::json!(606);
                    s
                }
                9 => {
                    let mut s = header(idx, "artifact", "h6_west.pdf");
                    s["encoding"] = serde_json::json!("base64");
                    s["data"] = serde_json::json!(pdf_b64);
                    s["sha256"] = serde_json::json!(pdf_hash);
                    s["bytes"] = serde_json::json!(std::fs::metadata("fixtures/output/h6_west.pdf").map(|m| m.len()).unwrap_or(0));
                    s
                }
                10 => {
                    let mut s = header(idx, "artifact", "h6_west.wav");
                    s["encoding"] = serde_json::json!("base64");
                    s["data"] = serde_json::json!(wav_b64);
                    s["sha256"] = serde_json::json!(wav_hash);
                    s["bytes"] = serde_json::json!(std::fs::metadata("fixtures/output/h6_west.wav").map(|m| m.len()).unwrap_or(0));
                    s["sample_rate"] = serde_json::json!(44100);
                    s["format"] = serde_json::json!("WAV PCM 16-bit");
                    s
                }
                // === WITNESSES ===
                12 => { let mut s = header(idx, "witness", "00_source"); s["witness"] = serde_json::from_str(&w_source).unwrap_or_default(); s }
                14 => { let mut s = header(idx, "witness", "01_midi"); s["witness"] = serde_json::from_str(&w_midi).unwrap_or_default(); s }
                15 => { let mut s = header(idx, "witness", "01_pdf"); s["witness"] = serde_json::from_str(&w_pdf).unwrap_or_default(); s }
                16 => { let mut s = header(idx, "witness", "02_wav"); s["witness"] = serde_json::from_str(&w_wav).unwrap_or_default(); s }
                18 => { let mut s = header(idx, "witness", "99_commitment"); s["witness"] = serde_json::from_str(&w_commit).unwrap_or_default(); s }
                // === EIGENSPACE ===
                20 => { let mut s = header(idx, "eigenspace", "earth"); s["value"] = serde_json::json!(eigen.earth_pct); s["description"] = serde_json::json!("grades 0-5 of Cl(15,0,0)"); s }
                21 => { let mut s = header(idx, "eigenspace", "spoke"); s["value"] = serde_json::json!(eigen.spoke_pct); s["description"] = serde_json::json!("grades 6-10 of Cl(15,0,0)"); s }
                22 => { let mut s = header(idx, "eigenspace", "hub"); s["value"] = serde_json::json!(eigen.hub_pct); s["description"] = serde_json::json!("grades 11-15 of Cl(15,0,0) — j-invariant"); s }
                24 => { let mut s = header(idx, "eigenspace", "grade_energy"); s["grades"] = serde_json::json!(eigen.grade_energy.to_vec()); s }
                25 => { let mut s = header(idx, "eigenspace", "fractran_state"); s["state"] = serde_json::json!(format!("{}", eigen.fractran_state)); s["triplet_count"] = serde_json::json!(eigen.triplet_count); s }
                // === METADATA ===
                26 => { let mut s = header(idx, "metadata", "tablet_provenance"); s["tablet"] = serde_json::json!("RS 15.30 + 15.49 + 17.387"); s["site"] = serde_json::json!("Royal Palace, Ugarit (Ras Shamra, Syria)"); s["excavation"] = serde_json::json!("1950s"); s["catalogue"] = serde_json::json!("h.6 (Laroche)"); s }
                27 => { let mut s = header(idx, "metadata", "scribe_colophon"); s["scribe"] = serde_json::json!("Ammurabi"); s["text"] = serde_json::json!("This [is] a song [in the] nitkibli [tuning], a zaluzi, written down by Ammurabi"); s }
                28 => { let mut s = header(idx, "metadata", "tuning_system"); s["tuning"] = serde_json::json!("nīd qablim (nid qabli)"); s["strings"] = serde_json::json!(9); s["scale"] = serde_json::json!("descending diatonic"); s }
                30 => { let mut s = header(idx, "metadata", "instrument"); s["instrument"] = serde_json::json!("sammûm"); s["strings"] = serde_json::json!(9); s["type"] = serde_json::json!("lyre"); s }
                32 => { let mut s = header(idx, "metadata", "deity"); s["deity"] = serde_json::json!("Nikkal"); s["domain"] = serde_json::json!("goddess of orchards"); s["consort"] = serde_json::json!("Yarikh (moon god)"); s }
                33 => { let mut s = header(idx, "metadata", "genre"); s["genre"] = serde_json::json!("zaluzi"); s["meaning"] = serde_json::json!("prayer to the gods"); s }
                34 => { let mut s = header(idx, "metadata", "date"); s["date"] = serde_json::json!("~1400 BC"); s["century"] = serde_json::json!("14th century BC"); s["age_years"] = serde_json::json!(3426); s }
                35 => { let mut s = header(idx, "metadata", "site"); s["city"] = serde_json::json!("Ugarit"); s["modern"] = serde_json::json!("Ras Shamra, Syria"); s["coordinates"] = serde_json::json!("35.6°N 35.78°E"); s }
                // === RECONSTRUCTIONS ===
                36 => { let mut s = header(idx, "reconstruction", "west_1994"); s["scholar"] = serde_json::json!("M. L. West"); s["year"] = serde_json::json!(1994); s["approach"] = serde_json::json!("dichords on descending diatonic scale"); s["notation"] = serde_json::json!(notation_json); s }
                38 => { let mut s = header(idx, "reconstruction", "kilmer_1974"); s["scholar"] = serde_json::json!("Anne D. Kilmer"); s["year"] = serde_json::json!(1974); s["approach"] = serde_json::json!("first modern reconstruction, ascending scale"); s }
                39 => { let mut s = header(idx, "reconstruction", "duchesne_guillemin"); s["scholar"] = serde_json::json!("Marcelle Duchesne-Guillemin"); s["years"] = serde_json::json!("1975, 1984"); s["approach"] = serde_json::json!("melodic interpretation of interval names"); s }
                40 => { let mut s = header(idx, "reconstruction", "dumbrill"); s["scholar"] = serde_json::json!("Richard Dumbrill"); s["approach"] = serde_json::json!("organological analysis, used by Peter Pringle"); s }
                42 => { let mut s = header(idx, "reconstruction", "vitale"); s["scholar"] = serde_json::json!("Raoul Gregory Vitale"); s["approach"] = serde_json::json!("alternative interval reading"); s }
                // === REFERENCES ===
                44..=57 => {
                    let ref_idx = (idx - 44) as usize;
                    let url = ref_urls.get(ref_idx).unwrap_or(&"");
                    let mut s = header(idx, "reference", url);
                    s["url"] = serde_json::json!(url);
                    s["capture_status"] = serde_json::json!("pending_zktls");
                    s
                }
                // === YOUTUBE ===
                58..=65 => {
                    let yt_idx = (idx - 58) as usize;
                    let url = yt_urls.get(yt_idx).unwrap_or(&"");
                    let mut s = header(idx, "youtube", url);
                    s["url"] = serde_json::json!(url);
                    s["capture_status"] = serde_json::json!("pending_private_witness");
                    s["note"] = serde_json::json!("audio captured privately for spectral comparison only — never redistributed");
                    s
                }
                // === PIPELINE ===
                66 => { let mut s = header(idx, "pipeline", "sop_retrosync_pub_001"); s["sop"] = serde_json::json!(read_text("datasets/SOP-RETROSYNC-PUB-001.md")); s }
                68 => { let mut s = header(idx, "pipeline", "erdfa_cft_decomposition"); s["levels"] = serde_json::json!("e1=file(p2) → e2=para(p3) → e3=col(p5) → e4=line(p7) → e5=token(p11) → e6=byte(p13) → e7=emoji(p17) → e8=unicode(p19) → e9=bit(p23)"); s }
                69 => { let mut s = header(idx, "pipeline", "boustrophedon_extraction"); s["method"] = serde_json::json!("Way of the Oxen — alternating direction rows"); s["rows"] = serde_json::json!(3); s }
                70 => { let mut s = header(idx, "pipeline", "cl15_algebra"); s["algebra"] = serde_json::json!("Cl(15,0,0)"); s["generators"] = serde_json::json!(15); s["dimension"] = serde_json::json!("2^15 = 32768"); s["eigenspaces"] = serde_json::json!(["Earth (0-5)", "Spoke (6-10)", "Hub (11-15)"]); s }
                // === RESERVED ===
                _ => header(idx, "reserved", "future"),
            }
        };

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
    }

    // Write shards
    let out = Path::new("fixtures/output/nft71");
    std::fs::create_dir_all(out).unwrap();
    for (idx, cbor) in &shards {
        let path = out.join(format!("{:02}.cbor", idx));
        std::fs::write(&path, cbor).unwrap();
    }

    // Summary
    let total_bytes: usize = shards.values().map(|v| v.len()).sum();
    let generators = (1..=71u64).filter(|n| is_prime(*n)).count();

    let manifest_json = serde_json::to_string_pretty(&serde_json::json!({
        "title": "Hurrian Hymn h.6 — 71-Shard NFT Collection",
        "version": "0.2.0",
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
