//! Generate Circom witness input for the NFT71 ZK proof.
//!
//! Reads the 71 real shards, builds a Poseidon Merkle tree (128 leaves),
//! and outputs the JSON witness for circom.
//!
//! Usage: cargo run -p fixtures --example nft71_witness

use sha2::{Digest, Sha256};
use std::path::Path;

/// Simplified Poseidon stand-in: we use SHA-256 truncated to 253 bits
/// (fits BN254 scalar field) as a placeholder. The circom circuit uses
/// real Poseidon — this just generates compatible field elements.
///
/// For production: use a proper Poseidon Rust implementation matching circomlib.
fn field_hash(data: &[u8]) -> String {
    let h = Sha256::digest(data);
    // Truncate to 253 bits (BN254 Fr) — clear top 3 bits of first byte
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&h);
    bytes[0] &= 0x1F; // clear top 3 bits
    // Convert to decimal string (circom input format)
    let mut val = num_bigint(bytes);
    val
}

/// Convert 32 bytes to decimal string.
fn num_bigint(bytes: [u8; 32]) -> String {
    let mut result = vec![0u8]; // start with 0
    for &byte in &bytes {
        // result = result * 256 + byte
        let mut carry = byte as u16;
        for digit in result.iter_mut().rev() {
            let v = (*digit as u16) * 256 + carry;
            *digit = (v % 10) as u8;
            carry = v / 10;
        }
        while carry > 0 {
            result.insert(0, (carry % 10) as u8);
            carry /= 10;
        }
    }
    if result.is_empty() { return "0".to_string(); }
    result.iter().map(|d| (b'0' + d) as char).collect()
}

/// Hash two field elements (Poseidon stand-in).
fn hash_pair(left: &str, right: &str) -> String {
    let mut data = Vec::new();
    data.extend_from_slice(left.as_bytes());
    data.push(0);
    data.extend_from_slice(right.as_bytes());
    field_hash(&data)
}

fn main() {
    let shard_dir = Path::new("fixtures/output/nft71");
    let depth = 7usize; // 2^7 = 128
    let n_leaves = 1 << depth;

    // Read 71 shard files and hash them
    let mut leaf_hashes: Vec<String> = Vec::new();
    for idx in 1..=71u64 {
        let path = shard_dir.join(format!("{:02}.cbor", idx));
        let data = std::fs::read(&path).unwrap_or_else(|_| panic!("missing shard {}", idx));
        leaf_hashes.push(field_hash(&data));
    }

    // Pad to 128 leaves with zeros
    let zero = "0".to_string();
    while leaf_hashes.len() < n_leaves {
        leaf_hashes.push(zero.clone());
    }

    // Build Merkle tree bottom-up
    let mut tree: Vec<Vec<String>> = vec![leaf_hashes.clone()];
    let mut current = leaf_hashes;
    for _ in 0..depth {
        let mut next = Vec::new();
        for i in (0..current.len()).step_by(2) {
            let left = &current[i];
            let right = if i + 1 < current.len() { &current[i + 1] } else { &zero };
            next.push(hash_pair(left, right));
        }
        tree.push(next.clone());
        current = next;
    }
    let root = &tree[depth][0];

    // Extract Merkle proofs for each of the 71 shards
    let mut path_elements: Vec<Vec<String>> = Vec::new();
    let mut path_indices: Vec<Vec<u8>> = Vec::new();

    for idx in 0..71usize {
        let mut elements = Vec::new();
        let mut indices = Vec::new();
        let mut pos = idx;
        for level in 0..depth {
            let sibling = if pos % 2 == 0 { pos + 1 } else { pos - 1 };
            let sibling_hash = if sibling < tree[level].len() {
                tree[level][sibling].clone()
            } else {
                zero.clone()
            };
            elements.push(sibling_hash);
            indices.push((pos % 2) as u8);
            pos /= 2;
        }
        path_elements.push(elements);
        path_indices.push(indices);
    }

    // Eigenspace: 100% Earth = 10000, 0% Spoke = 0, 0% Hub = 0
    let earth: u64 = 10000;
    let spoke: u64 = 0;
    let hub: u64 = 0;
    let eigen_commit = hash_pair(&hash_pair(&earth.to_string(), &spoke.to_string()), &hub.to_string());

    // Build witness JSON
    let shard_hashes_json: Vec<&str> = tree[0][..71].iter().map(|s| s.as_str()).collect();

    let witness = serde_json::json!({
        "merkleRoot": root,
        "eigenspaceCommitment": eigen_commit,
        "shardCount": "71",
        "shardHashes": shard_hashes_json,
        "merklePathElements": path_elements,
        "merklePathIndices": path_indices,
        "earthPct": earth.to_string(),
        "spokePct": spoke.to_string(),
        "hubPct": hub.to_string(),
    });

    let out_path = Path::new("circuits/build/input.json");
    std::fs::write(out_path, serde_json::to_string_pretty(&witness).unwrap()).unwrap();

    println!("=== NFT71 ZK Witness Generated ===");
    println!("shards:      71");
    println!("tree depth:  {depth}");
    println!("tree leaves: {n_leaves}");
    println!("merkle root: {}", root);
    println!("eigenspace:  {earth}/{spoke}/{hub} (Earth/Spoke/Hub ×100)");
    println!("eigen commit:{}", eigen_commit);
    println!("→ written to {}", out_path.display());
}
