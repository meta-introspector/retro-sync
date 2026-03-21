//! Streaming fraud detection — velocity checks + play ratio analysis.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RiskLevel {
    Clean,
    Suspicious,
    HighRisk,
    Confirmed,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayEvent {
    pub track_isrc: String,
    pub user_id: String,
    pub ip_hash: String,
    pub device_id: String,
    pub country_code: String,
    pub play_duration_secs: f64,
    pub track_duration_secs: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FraudAnalysis {
    pub risk_level: RiskLevel,
    pub signals: Vec<String>,
    pub action: FraudAction,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum FraudAction {
    Allow,
    Flag,
    Throttle,
    Block,
    Suspend,
}

struct Window {
    count: u64,
    start: std::time::Instant,
}

pub struct FraudDetector {
    ip_vel: Mutex<HashMap<String, Window>>,
    usr_vel: Mutex<HashMap<String, Window>>,
    /// SECURITY FIX: Changed from Vec<String> (O(n) scan) to HashSet<String> (O(1) lookup).
    /// Prevents DoS via blocked-list inflation attack.
    blocked: Mutex<HashSet<String>>,
}

impl FraudDetector {
    pub fn new() -> Self {
        Self {
            ip_vel: Mutex::new(HashMap::new()),
            usr_vel: Mutex::new(HashMap::new()),
            blocked: Mutex::new(HashSet::new()),
        }
    }
    pub fn analyse(&self, e: &PlayEvent) -> FraudAnalysis {
        let mut signals = Vec::new();
        let mut risk = RiskLevel::Clean;
        let ratio = e.play_duration_secs / e.track_duration_secs.max(1.0);
        if ratio < 0.05 {
            signals.push(format!("play ratio {:.2} — bot skip", ratio));
            risk = RiskLevel::Suspicious;
        }
        let ip_c = self.inc(&self.ip_vel, &e.ip_hash);
        if ip_c > 200 {
            signals.push(format!("IP velocity {} — click farm", ip_c));
            risk = RiskLevel::HighRisk;
        } else if ip_c > 50 {
            signals.push(format!("IP velocity {} — suspicious", ip_c));
            if risk < RiskLevel::Suspicious {
                risk = RiskLevel::Suspicious;
            }
        }
        let usr_c = self.inc(&self.usr_vel, &e.user_id);
        if usr_c > 100 {
            signals.push(format!("user velocity {} — bot", usr_c));
            risk = RiskLevel::HighRisk;
        }
        if self.is_blocked(&e.track_isrc) {
            signals.push("ISRC blocklisted".into());
            risk = RiskLevel::Confirmed;
        }
        if risk >= RiskLevel::Suspicious {
            warn!(isrc=%e.track_isrc, risk=?risk, "Fraud signal");
        }
        let action = match risk {
            RiskLevel::Clean => FraudAction::Allow,
            RiskLevel::Suspicious => FraudAction::Flag,
            RiskLevel::HighRisk => FraudAction::Block,
            RiskLevel::Confirmed => FraudAction::Suspend,
        };
        FraudAnalysis {
            risk_level: risk,
            signals,
            action,
        }
    }
    fn inc(&self, m: &Mutex<HashMap<String, Window>>, k: &str) -> u64 {
        if let Ok(mut map) = m.lock() {
            let now = std::time::Instant::now();
            let e = map.entry(k.to_string()).or_insert(Window {
                count: 0,
                start: now,
            });
            if now.duration_since(e.start).as_secs() > 3600 {
                e.count = 0;
                e.start = now;
            }
            e.count += 1;
            e.count
        } else {
            0
        }
    }
    pub fn block_isrc(&self, isrc: &str) {
        if let Ok(mut s) = self.blocked.lock() {
            s.insert(isrc.to_string());
        }
    }
    pub fn is_blocked(&self, isrc: &str) -> bool {
        self.blocked
            .lock()
            .map(|s| s.contains(isrc))
            .unwrap_or(false)
    }
}
