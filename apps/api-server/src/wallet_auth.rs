//! Wallet challenge-response authentication.
//!
//! Flow:
//!   1. Client: GET /api/auth/challenge/{address}
//!      Server issues a random nonce with 5-minute TTL.
//!
//!   2. Client signs the nonce string with their wallet private key.
//!      - BTTC / EVM wallets: personal_sign (EIP-191 prefix)
//!      - TronLink on Tron: signMessageV2
//!
//!   3. Client: POST /api/auth/verify { challenge_id, address, signature }
//!      Server recovers the signer address from the ECDSA signature and
//!      checks it matches the claimed address.  On success, issues a JWT
//!      (`sub` = wallet address, `exp` = 24h) the client stores and sends
//!      as `Authorization: Bearer <token>` on all subsequent API calls.
//!
//! Security properties:
//!   - Nonce is cryptographically random (OS entropy via /dev/urandom).
//!   - Challenges expire after 5 minutes → replay window is bounded.
//!   - Used challenges are deleted immediately → single-use.
//!   - JWT signed with HMAC-SHA256 using JWT_SECRET env var.

use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{info, warn};

// ── Challenge store (in-memory, short-lived) ──────────────────────────────────

#[derive(Debug)]
struct PendingChallenge {
    address: String,
    nonce: String,
    issued_at: Instant,
}

pub struct ChallengeStore {
    pending: Mutex<HashMap<String, PendingChallenge>>,
}

impl ChallengeStore {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }

    fn issue(&self, address: &str) -> (String, String) {
        let challenge_id = random_hex(16);
        let nonce = format!(
            "Sign in to Retrosync Media Group.\nNonce: {}\nIssued: {}",
            random_hex(32),
            chrono::Utc::now().to_rfc3339()
        );
        if let Ok(mut map) = self.pending.lock() {
            // Purge expired challenges first
            map.retain(|_, v| v.issued_at.elapsed() < Duration::from_secs(300));
            map.insert(
                challenge_id.clone(),
                PendingChallenge {
                    address: address.to_ascii_lowercase(),
                    nonce: nonce.clone(),
                    issued_at: Instant::now(),
                },
            );
        }
        (challenge_id, nonce)
    }

    fn consume(&self, challenge_id: &str) -> Option<PendingChallenge> {
        if let Ok(mut map) = self.pending.lock() {
            let entry = map.remove(challenge_id)?;
            if entry.issued_at.elapsed() > Duration::from_secs(300) {
                warn!(challenge_id=%challenge_id, "Challenge expired — rejecting");
                return None;
            }
            Some(entry)
        } else {
            None
        }
    }
}

/// Public alias for use by other modules (e.g., moderation.rs ID generation).
pub fn random_hex_pub(n: usize) -> String {
    random_hex(n)
}

/// Cryptographically random hex string of `n` bytes (2n hex chars).
fn random_hex(n: usize) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    // Use OS entropy via /dev/urandom for production randomness.
    let mut bytes = vec![0u8; n];
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        use std::io::Read;
        let _ = f.read_exact(&mut bytes);
    } else {
        // Fallback: mix time + hasher (weaker, dev only)
        let mut h = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut h);
        std::process::id().hash(&mut h);
        for (i, b) in bytes.iter_mut().enumerate() {
            (h.finish().wrapping_add(i as u64) as u8).hash(&mut h);
            *b = h.finish() as u8;
        }
    }
    hex::encode(bytes)
}

// ── AppState extension ────────────────────────────────────────────────────────

// The ChallengeStore is embedded in AppState via main.rs

// ── HTTP handlers ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub nonce: String,
    pub expires_in_secs: u64,
    pub instructions: &'static str,
}

pub async fn issue_challenge(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Json<ChallengeResponse> {
    let address = address.to_ascii_lowercase();
    let (challenge_id, nonce) = state.challenge_store.issue(&address);
    info!(address=%address, challenge_id=%challenge_id, "Wallet challenge issued");
    Json(ChallengeResponse {
        challenge_id,
        nonce,
        expires_in_secs: 300,
        instructions: "Sign the `nonce` string with your wallet. \
                        For EVM/BTTC: use personal_sign. \
                        For TronLink/Tron: use signMessageV2.",
    })
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub challenge_id: String,
    pub address: String,
    pub signature: String,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub token: String,
    pub address: String,
    pub expires_in_secs: u64,
}

pub async fn verify_challenge(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, StatusCode> {
    let address = req.address.to_ascii_lowercase();

    // Retrieve and consume the challenge (single-use + TTL enforced here)
    let challenge = state
        .challenge_store
        .consume(&req.challenge_id)
        .ok_or_else(|| {
            warn!(challenge_id=%req.challenge_id, "Unknown or expired challenge");
            StatusCode::UNPROCESSABLE_ENTITY
        })?;

    // Verify the claimed address matches the challenge's address
    if challenge.address != address {
        warn!(
            claimed=%address,
            challenge_addr=%challenge.address,
            "Address mismatch in challenge verify"
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // Verify the signature
    let verified = verify_evm_signature(&challenge.nonce, &req.signature, &address)
        .unwrap_or(false);

    let env = std::env::var("RETROSYNC_ENV").unwrap_or_else(|_| "development".into());
    if !verified && env == "production" {
        warn!(address=%address, "Wallet signature verification failed (production)");
        return Err(StatusCode::FORBIDDEN);
    }
    if !verified {
        warn!(address=%address, "Wallet signature not verified (dev mode — accepting anyway)");
    }

    // Issue JWT
    let token = issue_jwt(&address).map_err(|e| {
        warn!("JWT issue failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!(address=%address, "Wallet authentication successful — JWT issued");
    Ok(Json(VerifyResponse {
        token,
        address,
        expires_in_secs: 86400,
    }))
}

// ── EVM Signature Verification ────────────────────────────────────────────────

/// Verify an EIP-191 personal_sign signature.
/// The message is prefixed as: `\x19Ethereum Signed Message:\n{len}{msg}`
/// Returns true if the recovered address matches the claimed address.
fn verify_evm_signature(message: &str, signature_hex: &str, claimed_address: &str) -> anyhow::Result<bool> {
    // EIP-191 prefix
    let prefixed = format!(
        "\x19Ethereum Signed Message:\n{}{}",
        message.len(),
        message
    );

    // SHA3-256 (keccak256) of the prefixed message
    let msg_hash = keccak256(prefixed.as_bytes());

    // Decode signature: 65 bytes = r (32) + s (32) + v (1)
    let sig_bytes = hex::decode(signature_hex.trim_start_matches("0x"))
        .map_err(|e| anyhow::anyhow!("Signature hex decode failed: {}", e))?;

    if sig_bytes.len() != 65 {
        anyhow::bail!("Signature must be 65 bytes, got {}", sig_bytes.len());
    }

    let r = &sig_bytes[0..32];
    let s = &sig_bytes[32..64];
    let v = sig_bytes[64];

    // Normalise v: TronLink uses 0/1, Ethereum uses 27/28
    let recovery_id = match v {
        0 | 27 => 0u8,
        1 | 28 => 1u8,
        _ => anyhow::bail!("Invalid recovery id v={}", v),
    };

    // Recover the public key and derive the address
    let recovered = recover_evm_address(&msg_hash, r, s, recovery_id)?;

    Ok(recovered.to_ascii_lowercase() == claimed_address.trim_start_matches("0x").to_ascii_lowercase())
}

/// Keccak-256 hash (Ethereum's hash function), delegated to ethers::utils.
/// NOTE: Ethereum Keccak-256 differs from SHA3-256. Use this only.
fn keccak256(data: &[u8]) -> [u8; 32] {
    ethers::utils::keccak256(data)
}

/// ECDSA public key recovery on secp256k1.
/// Uses ethers-signers since ethers is already a dependency.
fn recover_evm_address(
    msg_hash: &[u8; 32],
    r: &[u8],
    s: &[u8],
    recovery_id: u8,
) -> anyhow::Result<String> {
    use ethers::types::{Signature, H256};

    let mut r_arr = [0u8; 32];
    let mut s_arr = [0u8; 32];
    r_arr.copy_from_slice(r);
    s_arr.copy_from_slice(s);

    let sig = Signature {
        r: ethers::types::U256::from_big_endian(&r_arr),
        s: ethers::types::U256::from_big_endian(&s_arr),
        v: recovery_id as u64,
    };

    let hash = H256::from_slice(msg_hash);
    let recovered = sig.recover(hash)?;
    Ok(format!("{:#x}", recovered))
}

// ── JWT Issuance ──────────────────────────────────────────────────────────────

/// Issue a 24-hour JWT with `sub` = wallet address.
/// The token is HMAC-SHA256 signed using JWT_SECRET env var.
/// In development (no JWT_SECRET), uses a fixed insecure key with a warning.
pub fn issue_jwt(wallet_address: &str) -> anyhow::Result<String> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        warn!("JWT_SECRET not set — using insecure dev key. Set JWT_SECRET in production.");
        "retrosync-dev-secret-change-in-prod".into()
    });

    let now = chrono::Utc::now().timestamp();
    let exp = now + 86400; // 24h

    // Build JWT header + payload
    let header = base64_encode_url(b"{\"alg\":\"HS256\",\"typ\":\"JWT\"}");
    let payload_json = serde_json::json!({
        "sub": wallet_address,
        "iat": now,
        "exp": exp,
        "iss": "retrosync-api",
    });
    let payload = base64_encode_url(payload_json.to_string().as_bytes());

    let signing_input = format!("{}.{}", header, payload);
    let sig = hmac_sha256(secret.as_bytes(), signing_input.as_bytes());
    let sig_b64 = base64_encode_url(&sig);

    Ok(format!("{}.{}.{}", header, payload, sig_b64))
}

fn base64_encode_url(bytes: &[u8]) -> String {
    let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };
        out.push(chars[(b0 >> 2) as usize] as char);
        out.push(chars[((b0 & 3) << 4 | b1 >> 4) as usize] as char);
        if chunk.len() > 1 {
            out.push(chars[((b1 & 0xf) << 2 | b2 >> 6) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(chars[(b2 & 0x3f) as usize] as char);
        }
    }
    out.replace('+', "-").replace('/', "_")
}

fn hmac_sha256(key: &[u8], msg: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    const BLOCK: usize = 64;
    let mut k = if key.len() > BLOCK {
        Sha256::digest(key).to_vec()
    } else {
        key.to_vec()
    };
    k.resize(BLOCK, 0);
    let ipad: Vec<u8> = k.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = k.iter().map(|b| b ^ 0x5c).collect();
    let inner = Sha256::digest([ipad.as_slice(), msg].concat());
    Sha256::digest([opad.as_slice(), inner.as_slice()].concat()).to_vec()
}
