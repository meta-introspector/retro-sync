//! btfs-keygen — Deterministic BTFS node keypair from Ledger hardware wallet.
//!
//! Derives a libp2p Ed25519 identity deterministically from a Ledger-signed
//! domain-separation message. The private key never leaves the derivation
//! process — only the final Ed25519 seed (derived via SHA-256) is used.
//!
//! Usage:
//!   cargo run --bin btfs-keygen -- --output ~/.btfs/ledger_identity
//!   cargo run --bin btfs-keygen -- --output ~/.btfs/ledger_identity --hd-path 1
//!
//! After generation:
//!   btfs config Identity.PrivKey $(cat ~/.btfs/ledger_identity/privkey.b64)
//!   btfs config Identity.PeerID  $(cat ~/.btfs/ledger_identity/peer_id.txt)
//!   systemctl --user restart btfs

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ed25519_dalek::{SigningKey, VerifyingKey};
use ethers::signers::{HDPath, Ledger, Signer};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

// ── Domain separation message ────────────────────────────────────────────────
// This exact string must never change — any change produces a different keypair.
// Version suffix allows future key rotation while keeping old keys derivable.
const DOMAIN_MSG: &[u8] = b"retrosync:btfs-node-identity:v1";

// ── libp2p protobuf key type constants ───────────────────────────────────────
// KeyType::Ed25519 = 1 in proto3
const LIBP2P_KEY_TYPE_ED25519: u8 = 1;

/// Encode an Ed25519 keypair in libp2p protobuf wire format.
/// Format: KeyType (varint) + key_data (bytes field)
/// This is what BTFS expects in Identity.PrivKey (base64-encoded).
fn encode_libp2p_privkey(signing_key: &SigningKey) -> Vec<u8> {
    // libp2p Ed25519 private key = 32-byte seed + 32-byte public key = 64 bytes
    let mut key_data = Vec::with_capacity(64);
    key_data.extend_from_slice(signing_key.as_bytes()); // 32-byte seed
    key_data.extend_from_slice(signing_key.verifying_key().as_bytes()); // 32-byte pubkey

    // Protobuf encoding:
    // field 1 (KeyType), wire type 0 (varint): tag = (1 << 3) | 0 = 0x08
    // field 2 (Data), wire type 2 (length-delimited): tag = (2 << 3) | 2 = 0x12
    let mut proto = vec![
        0x08,                    // field 1, varint
        LIBP2P_KEY_TYPE_ED25519, // Ed25519 = 1
        0x12,                    // field 2, length-delimited
        key_data.len() as u8,    // length
    ];
    proto.extend_from_slice(&key_data);
    proto
}

/// Encode Ed25519 public key in libp2p protobuf format.
fn encode_libp2p_pubkey(verifying_key: &VerifyingKey) -> Vec<u8> {
    let mut proto = vec![
        0x08,                    // field 1, varint
        LIBP2P_KEY_TYPE_ED25519, // Ed25519 = 1
        0x12,                    // field 2, length-delimited
        32u8,                    // public key length
    ];
    proto.extend_from_slice(verifying_key.as_bytes());
    proto
}

/// Derive peer ID from Ed25519 public key.
/// libp2p peer ID for Ed25519 keys = base58(multihash(identity, pubkey_proto))
/// For keys <= 42 bytes: identity multihash (no hashing, just prefixed)
/// Prefix: 0x00 (identity) + varint(length)
fn derive_peer_id(verifying_key: &VerifyingKey) -> String {
    let pubkey_proto = encode_libp2p_pubkey(verifying_key);
    // Identity multihash: 0x00 (identity code) + varint(len) + data
    let mut multihash = vec![
        0x00,                     // identity multihash code
        pubkey_proto.len() as u8, // length as varint
    ];
    multihash.extend_from_slice(&pubkey_proto);
    bs58::encode(multihash).into_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let output = args
        .windows(2)
        .find(|w| w[0] == "--output")
        .map(|w| PathBuf::from(&w[1]))
        .unwrap_or_else(|| PathBuf::from("btfs_ledger_identity"));

    let hd_index: u32 = args
        .windows(2)
        .find(|w| w[0] == "--hd-path")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(0);

    let chain_id: u64 = std::env::var("BTTC_CHAIN_ID")
        .unwrap_or_else(|_| "1029".into())
        .parse()
        .unwrap_or(1029);

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  Retrosync BTFS Node Keygen — Ledger Hardware Derivation ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("HD path:     m/44'/60'/0'/0/{hd_index}");
    println!("Chain ID:    {chain_id}");
    println!("Output dir:  {}", output.display());
    println!();
    println!("Make sure your Ledger is connected, unlocked, and the");
    println!("Ethereum app is open. Press Enter to continue...");
    let mut _line = String::new();
    std::io::stdin().read_line(&mut _line)?;

    // ── Connect to Ledger ────────────────────────────────────────────────────
    println!("[1/4] Connecting to Ledger...");
    let ledger = Ledger::new(HDPath::LedgerLive(hd_index as usize), chain_id)
        .await
        .map_err(|e| {
            anyhow!(
                "Cannot open Ledger: {e}. Device must be connected, unlocked, Ethereum app open."
            )
        })?;

    let eth_address = ledger.address();
    println!("      Ledger address: {eth_address:#x}");

    // ── Sign domain separation message ───────────────────────────────────────
    // We sign a fixed message so the same Ledger + HD path always produces
    // the same signature → same keypair. The Ledger will show this on screen.
    println!("[2/4] Signing domain message on Ledger...");
    println!(
        "      Message: \"{}\"",
        std::str::from_utf8(DOMAIN_MSG).unwrap()
    );
    println!("      Please approve the signing request on your Ledger device.");
    println!();

    let signature = ledger
        .sign_message(DOMAIN_MSG)
        .await
        .map_err(|e| anyhow!("Ledger signing failed: {e}"))?;

    // ── Derive Ed25519 seed ──────────────────────────────────────────────────
    // Stretch 65-byte signature (r + s + v) into 32-byte Ed25519 seed via SHA-256.
    // The signature is deterministic (same key + same message = same sig),
    // so this derivation is fully reproducible from the same Ledger.
    println!("[3/4] Deriving Ed25519 keypair from signature...");
    let mut sig_bytes = Vec::with_capacity(65);
    let mut r_bytes = [0u8; 32];
    let mut s_bytes = [0u8; 32];
    signature.r.to_big_endian(&mut r_bytes);
    signature.s.to_big_endian(&mut s_bytes);
    sig_bytes.extend_from_slice(&r_bytes);
    sig_bytes.extend_from_slice(&s_bytes);
    sig_bytes.push(signature.v as u8);

    // Domain-separate the seed derivation
    let mut seed_hasher = Sha256::new();
    seed_hasher.update(b"retrosync:btfs-ed25519-seed:v1");
    seed_hasher.update(&sig_bytes);
    let seed_bytes: [u8; 32] = seed_hasher.finalize().into();

    let signing_key = SigningKey::from_bytes(&seed_bytes);
    let verifying_key = signing_key.verifying_key();

    // ── Encode for BTFS ──────────────────────────────────────────────────────
    println!("[4/4] Encoding for BTFS config...");
    let privkey_proto = encode_libp2p_privkey(&signing_key);
    let privkey_b64 = B64.encode(&privkey_proto);
    let peer_id = derive_peer_id(&verifying_key);

    // ── Write output ─────────────────────────────────────────────────────────
    std::fs::create_dir_all(&output)?;

    // privkey.b64 — goes into btfs config Identity.PrivKey
    let privkey_path = output.join("privkey.b64");
    std::fs::write(&privkey_path, &privkey_b64)?;

    // peer_id.txt — goes into btfs config Identity.PeerID
    let peer_id_path = output.join("peer_id.txt");
    std::fs::write(&peer_id_path, &peer_id)?;

    // ledger_address.txt — the Ethereum/TRON address that owns this node
    let addr_path = output.join("ledger_address.txt");
    std::fs::write(&addr_path, format!("{eth_address:#x}"))?;

    // Summary JSON (no private material)
    let summary = serde_json::json!({
        "peer_id":        peer_id,
        "ledger_address": format!("{eth_address:#x}"),
        "hd_path":        format!("m/44'/60'/0'/0/{hd_index}"),
        "chain_id":       chain_id,
        "domain_msg":     std::str::from_utf8(DOMAIN_MSG).unwrap(),
        "derivation":     "SHA256(Ledger.sign_hash(SHA256(domain_msg))) -> Ed25519 seed",
        "note":           "Regenerate by running btfs-keygen with the same Ledger and HD path",
    });
    std::fs::write(
        output.join("summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    // Restrict permissions on privkey file
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&privkey_path, std::fs::Permissions::from_mode(0o600))?;
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  ✅  Keypair derived successfully                        ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("Peer ID:        {peer_id}");
    println!("Ledger address: {eth_address:#x}");
    println!();
    println!("NEXT STEPS — apply to BTFS config:");
    println!();
    println!("  systemctl --user stop btfs");
    println!(
        "  btfs config Identity.PrivKey $(cat {}/privkey.b64)",
        output.display()
    );
    println!(
        "  btfs config Identity.PeerID  $(cat {}/peer_id.txt)",
        output.display()
    );
    println!("  btfs config Identity.TronKey {eth_address:#x}");
    println!("  systemctl --user start btfs");
    println!();
    println!("To delete the on-disk key material after applying:");
    println!("  shred -uz {}/privkey.b64", output.display());
    println!();
    println!("To regenerate later, run this tool again with the same Ledger.");
    println!("The peer ID will be identical.");

    Ok(())
}
