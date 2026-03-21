//! LUFS loudness + format QC. Target: -14±1 LUFS, stereo WAV/FLAC, 44.1–96kHz.
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

pub const TARGET_LUFS: f64 = -14.0;
pub const LUFS_TOLERANCE: f64 = 1.0;
pub const TRUE_PEAK_MAX: f64 = -1.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    Wav16,
    Wav24,
    Flac16,
    Flac24,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQcReport {
    pub passed: bool,
    pub format: AudioFormat,
    pub sample_rate_hz: u32,
    pub channels: u8,
    pub duration_secs: f64,
    pub integrated_lufs: Option<f64>,
    pub true_peak_dbfs: Option<f64>,
    pub lufs_ok: bool,
    pub format_ok: bool,
    pub channels_ok: bool,
    pub sample_rate_ok: bool,
    pub defects: Vec<String>,
}

pub fn detect_format(b: &[u8]) -> AudioFormat {
    if b.len() < 4 {
        return AudioFormat::Unknown("too short".into());
    }
    match &b[..4] {
        b"RIFF" => AudioFormat::Wav24,
        b"fLaC" => AudioFormat::Flac24,
        _ => AudioFormat::Unknown(format!("{:02x?}", &b[..4])),
    }
}

pub fn parse_wav_header(b: &[u8]) -> (u32, u8, u16) {
    if b.len() < 36 {
        return (44100, 2, 16);
    }
    let ch = u16::from_le_bytes([b[22], b[23]]) as u8;
    let sr = u32::from_le_bytes([b[24], b[25], b[26], b[27]]);
    let bd = u16::from_le_bytes([b[34], b[35]]);
    (sr, ch, bd)
}

pub fn run_qc(bytes: &[u8], lufs: Option<f64>, true_peak: Option<f64>) -> AudioQcReport {
    let fmt = detect_format(bytes);
    let (sr, ch, _) = parse_wav_header(bytes);
    let duration =
        (bytes.len().saturating_sub(44)) as f64 / (sr.max(1) as f64 * ch.max(1) as f64 * 3.0);
    let mut defects = Vec::new();
    let fmt_ok = matches!(
        fmt,
        AudioFormat::Wav16 | AudioFormat::Wav24 | AudioFormat::Flac16 | AudioFormat::Flac24
    );
    if !fmt_ok {
        defects.push("unsupported format".into());
    }
    let sr_ok = (44100..=96000).contains(&sr);
    if !sr_ok {
        defects.push(format!("sample rate {sr}Hz out of range"));
    }
    let ch_ok = ch == 2;
    if !ch_ok {
        defects.push(format!("{ch} channels — stereo required"));
    }
    let lufs_ok = match lufs {
        Some(l) => {
            let ok = (l - TARGET_LUFS).abs() <= LUFS_TOLERANCE;
            if !ok {
                defects.push(format!(
                    "{l:.1} LUFS — target {TARGET_LUFS:.1}±{LUFS_TOLERANCE:.1}"
                ));
            }
            ok
        }
        None => true,
    };
    let peak_ok = match true_peak {
        Some(p) => {
            let ok = p <= TRUE_PEAK_MAX;
            if !ok {
                defects.push(format!("true peak {p:.1} dBFS > {TRUE_PEAK_MAX:.1}"));
            }
            ok
        }
        None => true,
    };
    let passed = fmt_ok && sr_ok && ch_ok && lufs_ok && peak_ok;
    if !passed {
        warn!(defects=?defects, "Audio QC failed");
    } else {
        info!(sr=%sr, "Audio QC passed");
    }
    AudioQcReport {
        passed,
        format: fmt,
        sample_rate_hz: sr,
        channels: ch,
        duration_secs: duration,
        integrated_lufs: lufs,
        true_peak_dbfs: true_peak,
        lufs_ok,
        format_ok: fmt_ok,
        channels_ok: ch_ok,
        sample_rate_ok: sr_ok,
        defects,
    }
}
