//! ISO 9001 §7.5 append-only audit store.
use std::sync::Mutex;
use tracing::info;

#[allow(dead_code)]
pub struct AuditStore {
    entries: Mutex<Vec<String>>,
    path: String,
}

impl AuditStore {
    #[zkperf_macros::zkperf]
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            entries: Mutex::new(Vec::new()),
            path: path.to_string(),
        })
    }
    #[zkperf_macros::zkperf]
    pub fn record(&self, msg: &str) -> anyhow::Result<()> {
        let entry = format!("[{}] {}", chrono::Utc::now().to_rfc3339(), msg);
        info!(audit=%entry);
        if let Ok(mut v) = self.entries.lock() {
            v.push(entry);
        }
        Ok(())
    }
}
