use fixtures::hurrian_h6::{embed_h6, h6_shard_cbor};
use fixtures::witness::witness_h6;

fn main() {
    let r = embed_h6();
    println!("=== Hurrian Hymn h.6 — Cl(15) Eigenspace ===");
    println!("Triplets: {}", r.triplet_count);
    println!("Earth: {:.1}%  Spoke: {:.1}%  Hub: {:.1}%", r.earth_pct, r.spoke_pct, r.hub_pct);
    println!("FRACTRAN state: {}", r.fractran_state);

    let shard = h6_shard_cbor();
    println!("\nDA51 CBOR shard: {} bytes", shard.len());
    println!("Magic: 0x{:02x}{:02x}", shard[0], shard[1]);

    let w = witness_h6();
    println!("\n=== Witness Chain ===");
    println!("Commitment: {}", w.commitment);
    println!("{}", serde_json::to_string_pretty(&w).unwrap());
}
