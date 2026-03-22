//! nft71_prove — Generate a Groth16 proof of the 71-shard collection.
//!
//! Pure Rust: reads real CBOR shards → MiMC Merkle tree → Groth16/BN254 proof.
//! No JS, no WASM, no snarkjs.
//!
//! Usage: cargo run -p fixtures --example nft71_prove --release

use std::path::Path;
use zk_circuits::nft71::*;

fn main() {
    let shard_dir = Path::new("fixtures/output/nft71");

    // Load 71 real shards and hash to field elements
    println!("=== NFT71 ZK Proof (pure Rust, Groth16/BN254) ===");
    print!("[1] loading shards...");
    let mut shard_hashes = [ark_bn254::Fr::from(0u64); SHARD_COUNT];
    for idx in 1..=71u64 {
        let path = shard_dir.join(format!("{:02}.cbor", idx));
        let data = std::fs::read(&path).unwrap_or_else(|_| panic!("missing shard {}", idx));
        shard_hashes[(idx - 1) as usize] = shard_to_field(&data);
    }
    println!(" {} shards loaded", SHARD_COUNT);

    // Build Merkle tree
    print!("[2] building Merkle tree (depth {TREE_DEPTH})...");
    let (root, levels) = build_merkle_tree(&shard_hashes);
    println!(" root = {:?}", root);

    // Extract proofs
    let mut merkle_siblings = [[ark_bn254::Fr::from(0u64); TREE_DEPTH]; SHARD_COUNT];
    let mut merkle_dirs = [[false; TREE_DEPTH]; SHARD_COUNT];
    for i in 0..SHARD_COUNT {
        let (s, d) = merkle_proof(&levels, i);
        merkle_siblings[i] = s;
        merkle_dirs[i] = d;
    }

    // Eigenspace: h.6 = 100% Earth
    let earth: u64 = 10000;
    let spoke: u64 = 0;
    let hub: u64 = 0;
    let eigen = eigenspace_commitment(earth, spoke, hub);
    println!("[3] eigenspace commitment = {:?}", eigen);

    let witness = NFT71Witness {
        shard_hashes,
        merkle_siblings,
        merkle_dirs,
        earth_pct: earth,
        spoke_pct: spoke,
        hub_pct: hub,
    };

    // Trusted setup
    println!("[4] running trusted setup...");
    let start = std::time::Instant::now();
    let (pk, vk) = setup();
    println!("    setup: {:.1}s", start.elapsed().as_secs_f64());

    // Prove
    println!("[5] generating proof...");
    let start = std::time::Instant::now();
    let (proof, public_inputs) = prove(&pk, witness);
    let prove_time = start.elapsed();
    println!("    proof: {:.1}s", prove_time.as_secs_f64());

    // Verify
    print!("[6] verifying...");
    let start = std::time::Instant::now();
    let valid = verify(&vk, &proof, &public_inputs);
    println!(" {} ({:.3}s)", if valid { "✓ VALID" } else { "✗ INVALID" }, start.elapsed().as_secs_f64());

    // Write proof artifacts
    let out = Path::new("circuits/build");
    std::fs::create_dir_all(out).unwrap();

    let proof_json = serde_json::json!({
        "circuit": "nft71",
        "scheme": "groth16",
        "curve": "bn254",
        "hash": "mimc",
        "shards": SHARD_COUNT,
        "tree_depth": TREE_DEPTH,
        "valid": valid,
        "prove_time_ms": prove_time.as_millis(),
        "public_inputs": {
            "merkle_root": format!("{:?}", public_inputs[0]),
            "eigenspace_commitment": format!("{:?}", public_inputs[1]),
            "shard_count": format!("{:?}", public_inputs[2]),
        },
        "proof": {
            "a": format!("{:?}", proof.a),
            "b": format!("{:?}", proof.b),
            "c": format!("{:?}", proof.c),
        },
    });
    std::fs::write(out.join("nft71_proof.json"), serde_json::to_string_pretty(&proof_json).unwrap()).unwrap();
    println!("\n→ proof written to {}", out.join("nft71_proof.json").display());
}
