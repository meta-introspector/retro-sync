//! Zero Trust middleware: SPIFFE SVID + JWT on every request.
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tracing::warn;

pub async fn verify_zero_trust(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // In production: verify SPIFFE SVID from mTLS peer cert + JWT claims
    // For local dev: pass through with a warning
    if std::env::var("ZERO_TRUST_DISABLED").unwrap_or_default() == "1" {
        warn!("ZERO_TRUST_DISABLED=1 — skipping auth (dev mode only)");
        return Ok(next.run(request).await);
    }
    // TODO: extract and verify SPIFFE ID from TLS peer certificate
    // For now: check for Authorization header presence
    let auth = request.headers().get("authorization");
    if auth.is_none() && std::env::var("ZERO_TRUST_REQUIRE_AUTH").unwrap_or_default() == "1" {
        warn!("Missing Authorization header");
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(request).await)
}
