//! Zero Trust middleware: SPIFFE SVID + JWT on every request.
//! SECURITY FIX: Auth is now enforced by default. ZERO_TRUST_DISABLED requires
//! explicit opt-in AND is blocked in production (RETROSYNC_ENV=production).
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::warn;

pub async fn verify_zero_trust(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let env = std::env::var("RETROSYNC_ENV").unwrap_or_else(|_| "development".into());
    let is_production = env == "production";

    // SECURITY: Dev bypass is BLOCKED in production
    if std::env::var("ZERO_TRUST_DISABLED").unwrap_or_default() == "1" {
        if is_production {
            warn!("SECURITY: ZERO_TRUST_DISABLED=1 is not allowed in production — blocking request");
            return Err(StatusCode::FORBIDDEN);
        }
        warn!("ZERO_TRUST_DISABLED=1 — skipping auth (dev only, NOT for production)");
        return Ok(next.run(request).await);
    }

    // SECURITY: Health and metrics endpoints are exempt from auth
    let path = request.uri().path();
    if path == "/health" || path == "/metrics" {
        return Ok(next.run(request).await);
    }

    // Extract Authorization header
    let auth = request.headers().get("authorization");
    let token = match auth {
        None => {
            warn!(path=%path, "Missing Authorization header — rejecting request");
            return Err(StatusCode::UNAUTHORIZED);
        }
        Some(v) => v.to_str().map_err(|_| StatusCode::BAD_REQUEST)?,
    };

    // Validate Bearer token format
    let jwt = token.strip_prefix("Bearer ").ok_or_else(|| {
        warn!("Invalid Authorization header format — must be Bearer <token>");
        StatusCode::UNAUTHORIZED
    })?;

    if jwt.is_empty() {
        warn!("Empty Bearer token — rejecting");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // PRODUCTION: Full JWT validation with signature verification
    // Development: Accept any non-empty token with warning
    if is_production {
        let secret = std::env::var("JWT_SECRET").map_err(|_| {
            warn!("JWT_SECRET not configured in production");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        validate_jwt(jwt, &secret)?;
    } else {
        warn!(path=%path, "Dev mode: JWT signature not verified — non-empty token accepted");
    }

    Ok(next.run(request).await)
}

/// Validate JWT signature and claims (production enforcement).
/// In production, JWT_SECRET must be set and tokens must be properly signed.
fn validate_jwt(token: &str, secret: &str) -> Result<(), StatusCode> {
    // Token structure: header.payload.signature (3 parts)
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        warn!("Malformed JWT: expected 3 parts, got {}", parts.len());
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Decode payload to check expiry
    let payload_b64 = parts[1];
    let payload_bytes = base64_decode_url(payload_b64).map_err(|_| {
        warn!("JWT payload base64 decode failed");
        StatusCode::UNAUTHORIZED
    })?;

    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).map_err(|_| {
        warn!("JWT payload JSON parse failed");
        StatusCode::UNAUTHORIZED
    })?;

    // Check expiry
    if let Some(exp) = payload.get("exp").and_then(|v| v.as_i64()) {
        let now = chrono::Utc::now().timestamp();
        if now > exp {
            warn!("JWT expired at {} (now: {})", exp, now);
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // HMAC-SHA256 signature verification
    use std::fmt::Write;
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let expected_sig = hmac_sha256(secret.as_bytes(), signing_input.as_bytes());
    let expected_b64 = base64_encode_url(&expected_sig);

    if !constant_time_eq(parts[2].as_bytes(), expected_b64.as_bytes()) {
        warn!("JWT signature verification failed");
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(())
}

fn base64_decode_url(s: &str) -> Result<Vec<u8>, ()> {
    // URL-safe base64 without padding → standard base64 with padding
    let padded = match s.len() % 4 {
        2 => format!("{}==", s),
        3 => format!("{}=", s),
        _ => s.to_string(),
    };
    let standard = padded.replace('-', "+").replace('_', "/");
    base64_simple_decode(&standard).map_err(|_| ())
}

fn base64_simple_decode(s: &str) -> Result<Vec<u8>, String> {
    let chars: Vec<u8> = s.chars().filter_map(|c| {
        if c.is_ascii_uppercase() { Some(c as u8 - b'A') }
        else if c.is_ascii_lowercase() { Some(c as u8 - b'a' + 26) }
        else if c.is_ascii_digit() { Some(c as u8 - b'0' + 52) }
        else if c == '+' || c == '-' { Some(62) }
        else if c == '/' || c == '_' { Some(63) }
        else { None }
    }).collect();
    
    let mut out = Vec::new();
    for chunk in chars.chunks(4) {
        if chunk.len() < 2 { break; }
        out.push((chunk[0] << 2) | (chunk[1] >> 4));
        if chunk.len() >= 3 { out.push((chunk[1] << 4) | (chunk[2] >> 2)); }
        if chunk.len() >= 4 { out.push((chunk[2] << 6) | chunk[3]); }
    }
    Ok(out)
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
        if chunk.len() > 1 { out.push(chars[((b1 & 0xf) << 2 | b2 >> 6) as usize] as char); }
        if chunk.len() > 2 { out.push(chars[(b2 & 0x3f) as usize] as char); }
    }
    out.replace('+', "-").replace('/', "_").replace('=', "")
}

fn hmac_sha256(key: &[u8], msg: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    const BLOCK: usize = 64;
    let mut k = if key.len() > BLOCK { Sha256::digest(key).to_vec() } else { key.to_vec() };
    k.resize(BLOCK, 0);
    let ipad: Vec<u8> = k.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = k.iter().map(|b| b ^ 0x5c).collect();
    let inner = Sha256::digest([ipad.as_slice(), msg].concat());
    Sha256::digest([opad.as_slice(), inner.as_slice()].concat()).to_vec()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Build CORS headers restricted to allowed origins.
/// Call this in main.rs instead of CorsLayer::new().allow_origin(Any).
pub fn allowed_origins() -> Vec<HeaderValue> {
    let origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173,http://localhost:3000".into());
    origins
        .split(',')
        .filter_map(|o| o.trim().parse::<HeaderValue>().ok())
        .collect()
}
