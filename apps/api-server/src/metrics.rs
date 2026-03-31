//! Six Sigma Prometheus CTQ metrics.
use crate::AppState;
use axum::{extract::State, response::IntoResponse};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct CtqMetrics {
    pub uploads_total: AtomicU64,
    pub defects_total: AtomicU64,
    pub band_common: AtomicU64,
    pub band_rare: AtomicU64,
    pub band_legendary: AtomicU64,
    latency_sum_ms: AtomicU64,
    latency_count: AtomicU64,
}

impl Default for CtqMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl CtqMetrics {
    #[zkperf_macros::zkperf]
    pub fn new() -> Self {
        Self {
            uploads_total: AtomicU64::new(0),
            defects_total: AtomicU64::new(0),
            band_common: AtomicU64::new(0),
            band_rare: AtomicU64::new(0),
            band_legendary: AtomicU64::new(0),
            latency_sum_ms: AtomicU64::new(0),
            latency_count: AtomicU64::new(0),
        }
    }
    #[zkperf_macros::zkperf]
    pub fn record_defect(&self, _kind: &str) {
        self.defects_total.fetch_add(1, Ordering::Relaxed);
    }
    #[zkperf_macros::zkperf]
    pub fn record_band(&self, band: u8) {
        self.uploads_total.fetch_add(1, Ordering::Relaxed);
        match band {
            0 => self.band_common.fetch_add(1, Ordering::Relaxed),
            1 => self.band_rare.fetch_add(1, Ordering::Relaxed),
            _ => self.band_legendary.fetch_add(1, Ordering::Relaxed),
        };
    }
    #[zkperf_macros::zkperf]
    pub fn record_latency(&self, _name: &str, ms: f64) {
        self.latency_sum_ms.fetch_add(ms as u64, Ordering::Relaxed);
        self.latency_count.fetch_add(1, Ordering::Relaxed);
    }
    #[zkperf_macros::zkperf]
    pub fn band_distribution_in_control(&self) -> bool {
        let total = self.uploads_total.load(Ordering::Relaxed);
        if total < 30 {
            return true;
        }
        let common = self.band_common.load(Ordering::Relaxed) as f64 / total as f64;
        (common - 7.0 / 15.0).abs() <= 0.15
    }
    #[zkperf_macros::zkperf]
    pub fn metrics_text(&self) -> String {
        let up = self.uploads_total.load(Ordering::Relaxed);
        let de = self.defects_total.load(Ordering::Relaxed);
        let dpmo = if up > 0 { de * 1_000_000 / up } else { 0 };
        format!(
            "# HELP retrosync_uploads_total Total uploads\n\
             retrosync_uploads_total {up}\n\
             retrosync_defects_total {de}\n\
             retrosync_dpmo {dpmo}\n\
             retrosync_band_common {}\n\
             retrosync_band_rare {}\n\
             retrosync_band_legendary {}\n\
             retrosync_band_in_control {}\n",
            self.band_common.load(Ordering::Relaxed),
            self.band_rare.load(Ordering::Relaxed),
            self.band_legendary.load(Ordering::Relaxed),
            self.band_distribution_in_control() as u8,
        )
    }
}

#[zkperf_macros::zkperf]
pub async fn handler(State(state): State<AppState>) -> impl IntoResponse {
    state.metrics.metrics_text()
}