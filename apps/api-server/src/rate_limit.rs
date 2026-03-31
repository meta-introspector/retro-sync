//! Per-IP sliding-window rate limiter as Axum middleware.
//!
//! Limits (per rolling 60-second window):
//!   /api/auth/*     → 10 req/min  (brute-force / challenge-grind protection)
//!   /api/upload     → 5  req/min  (large file upload rate-limit)
//!   everything else → 120 req/min (2 req/sec burst)
//!
//! IP resolution priority:
//!   1. X-Real-IP header (set by Replit / nginx proxy)
//!   2. first IP in X-Forwarded-For header
//!   3. "unknown" (all unknown clients share the general bucket)
//!
//! State is in-memory — counters reset on server restart (acceptable for
//! stateless sliding-window limits; persistent limits need Redis).
//!
//! Memory: each tracked IP costs ~72 bytes + 24 bytes × requests_in_window.
//! At 120 req/min/IP and 10,000 active IPs: ≈ 40 MB maximum.
//! Stale IPs are pruned when the map exceeds 50,000 entries.

use crate::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::{collections::HashMap, sync::Mutex, time::Instant};
use tracing::warn;

const WINDOW_SECS: u64 = 60;

/// Three-bucket limits (req per 60s)
const GENERAL_LIMIT: usize = 120;
const AUTH_LIMIT: usize = 10;
const UPLOAD_LIMIT: usize = 5;

/// Limit applied to requests whose source IP cannot be determined.
///
/// All such requests share the key "auth:unknown", "general:unknown", etc.
/// A much tighter limit than GENERAL_LIMIT prevents an attacker (or broken
/// proxy) from exhausting the shared bucket and causing collateral DoS for
/// other unresolvable clients.  Legitimate deployments should configure a
/// reverse proxy that sets X-Real-IP so this fallback is never hit.
const UNKNOWN_LIMIT_DIVISOR: usize = 10;

pub struct RateLimiter {
    /// Key: `"{path_bucket}:{client_ip}"` → sorted list of request instants
    windows: Mutex<HashMap<String, Vec<Instant>>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    #[zkperf_macros::zkperf]
    pub fn new() -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
        }
    }

    /// Returns `true` if the request is within the limit, `false` to reject.
    #[zkperf_macros::zkperf]
    pub fn check(&self, key: &str, limit: usize) -> bool {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(WINDOW_SECS);
        if let Ok(mut map) = self.windows.lock() {
            let times = map.entry(key.to_string()).or_default();
            // Prune entries older than the window
            times.retain(|&t| now.duration_since(t) < window);
            if times.len() >= limit {
                return false;
            }
            times.push(now);
            // Prune stale IPs to bound memory
            if map.len() > 50_000 {
                map.retain(|_, v| !v.is_empty());
            }
        }
        true
    }
}

/// Validate that a string is a well-formed IPv4 or IPv6 address.
/// Rejects empty strings, hostnames, and any header-injection payloads.
fn is_valid_ip(s: &str) -> bool {
    s.parse::<std::net::IpAddr>().is_ok()
}

/// Extract client IP from proxy headers, falling back to "unknown".
///
/// Header values are only trusted if they parse as a valid IP address.
/// This prevents an attacker from injecting arbitrary strings into the
/// rate-limit key by setting a crafted X-Forwarded-For or X-Real-IP header.
fn client_ip(request: &Request) -> String {
    // X-Real-IP (Nginx / Replit proxy)
    if let Some(v) = request.headers().get("x-real-ip") {
        if let Ok(s) = v.to_str() {
            let ip = s.trim();
            if is_valid_ip(ip) {
                return ip.to_string();
            }
            warn!(raw=%ip, "x-real-ip header is not a valid IP — ignoring");
        }
    }
    // X-Forwarded-For: client, proxy1, proxy2 — take the first (leftmost)
    if let Some(v) = request.headers().get("x-forwarded-for") {
        if let Ok(s) = v.to_str() {
            if let Some(ip) = s.split(',').next() {
                let ip = ip.trim();
                if is_valid_ip(ip) {
                    return ip.to_string();
                }
                warn!(raw=%ip, "x-forwarded-for first entry is not a valid IP — ignoring");
            }
        }
    }
    "unknown".to_string()
}

/// Classify a request path into a rate-limit bucket.
fn bucket(path: &str) -> (&'static str, usize) {
    if path.starts_with("/api/auth/") {
        ("auth", AUTH_LIMIT)
    } else if path == "/api/upload" {
        ("upload", UPLOAD_LIMIT)
    } else {
        ("general", GENERAL_LIMIT)
    }
}

/// Axum middleware: enforce per-IP rate limits.
#[zkperf_macros::zkperf]
pub async fn enforce(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Exempt health / metrics endpoints from rate limiting
    let path = request.uri().path().to_string();
    if path == "/health" || path == "/metrics" {
        return Ok(next.run(request).await);
    }

    let ip = client_ip(&request);
    let (bucket_name, base_limit) = bucket(&path);
    // Apply a tighter cap for requests with no resolvable IP (shared bucket).
    // This prevents a single unknown/misconfigured source from starving the
    // shared "unknown" key and causing collateral DoS for other clients.
    let limit = if ip == "unknown" {
        (base_limit / UNKNOWN_LIMIT_DIVISOR).max(1)
    } else {
        base_limit
    };
    let key = format!("{bucket_name}:{ip}");

    if !state.rate_limiter.check(&key, limit) {
        warn!(
            ip=%ip,
            path=%path,
            bucket=%bucket_name,
            limit=%limit,
            "Rate limit exceeded — 429"
        );
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}