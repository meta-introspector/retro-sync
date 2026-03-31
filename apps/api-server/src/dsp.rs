//! DSP delivery spec validation — Spotify, Apple Music, Amazon, YouTube, TikTok, Tidal.
use crate::audio_qc::AudioQcReport;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Dsp {
    Spotify,
    AppleMusic,
    AmazonMusic,
    YouTubeMusic,
    TikTok,
    Tidal,
}

impl Dsp {
    #[zkperf_macros::zkperf]
    pub fn all() -> &'static [Dsp] {
        &[
            Dsp::Spotify,
            Dsp::AppleMusic,
            Dsp::AmazonMusic,
            Dsp::YouTubeMusic,
            Dsp::TikTok,
            Dsp::Tidal,
        ]
    }
    #[zkperf_macros::zkperf]
    pub fn name(&self) -> &'static str {
        match self {
            Dsp::Spotify => "Spotify",
            Dsp::AppleMusic => "Apple Music",
            Dsp::AmazonMusic => "Amazon Music",
            Dsp::YouTubeMusic => "YouTube Music",
            Dsp::TikTok => "TikTok Music",
            Dsp::Tidal => "Tidal",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DspSpec {
    pub dsp: Dsp,
    pub lufs_target: f64,
    pub lufs_tol: f64,
    pub true_peak_max: f64,
    pub sample_rates: Vec<u32>,
    pub stereo: bool,
    pub isrc_req: bool,
    pub upc_req: bool,
    pub cover_art_min_px: u32,
}

impl DspSpec {
    #[zkperf_macros::zkperf]
    pub fn for_dsp(d: &Dsp) -> Self {
        match d {
            Dsp::Spotify => Self {
                dsp: Dsp::Spotify,
                lufs_target: -14.0,
                lufs_tol: 1.0,
                true_peak_max: -1.0,
                sample_rates: vec![44100, 48000],
                stereo: true,
                isrc_req: true,
                upc_req: true,
                cover_art_min_px: 3000,
            },
            Dsp::AppleMusic => Self {
                dsp: Dsp::AppleMusic,
                lufs_target: -16.0,
                lufs_tol: 1.0,
                true_peak_max: -1.0,
                sample_rates: vec![44100, 48000, 96000],
                stereo: true,
                isrc_req: true,
                upc_req: true,
                cover_art_min_px: 3000,
            },
            Dsp::AmazonMusic => Self {
                dsp: Dsp::AmazonMusic,
                lufs_target: -14.0,
                lufs_tol: 1.0,
                true_peak_max: -2.0,
                sample_rates: vec![44100, 48000],
                stereo: true,
                isrc_req: true,
                upc_req: true,
                cover_art_min_px: 3000,
            },
            Dsp::YouTubeMusic => Self {
                dsp: Dsp::YouTubeMusic,
                lufs_target: -14.0,
                lufs_tol: 2.0,
                true_peak_max: -1.0,
                sample_rates: vec![44100, 48000],
                stereo: false,
                isrc_req: true,
                upc_req: false,
                cover_art_min_px: 1400,
            },
            Dsp::TikTok => Self {
                dsp: Dsp::TikTok,
                lufs_target: -14.0,
                lufs_tol: 2.0,
                true_peak_max: -1.0,
                sample_rates: vec![44100, 48000],
                stereo: false,
                isrc_req: true,
                upc_req: false,
                cover_art_min_px: 1400,
            },
            Dsp::Tidal => Self {
                dsp: Dsp::Tidal,
                lufs_target: -14.0,
                lufs_tol: 1.0,
                true_peak_max: -1.0,
                sample_rates: vec![44100, 48000, 96000],
                stereo: true,
                isrc_req: true,
                upc_req: true,
                cover_art_min_px: 3000,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspValidationResult {
    pub dsp: String,
    pub passed: bool,
    pub defects: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TrackMeta {
    pub isrc: Option<String>,
    pub upc: Option<String>,
    pub explicit: bool,
    pub territory_rights: bool,
    pub contributor_meta: bool,
    pub cover_art_px: Option<u32>,
}

#[zkperf_macros::zkperf]
pub fn validate_all(qc: &AudioQcReport, meta: &TrackMeta) -> Vec<DspValidationResult> {
    Dsp::all()
        .iter()
        .map(|d| validate_for(d, qc, meta))
        .collect()
}

#[zkperf_macros::zkperf]
pub fn validate_for(dsp: &Dsp, qc: &AudioQcReport, meta: &TrackMeta) -> DspValidationResult {
    let spec = DspSpec::for_dsp(dsp);
    let mut def = Vec::new();
    if !qc.format_ok {
        def.push("unsupported format".into());
    }
    if !qc.channels_ok && spec.stereo {
        def.push("stereo required".into());
    }
    if !qc.sample_rate_ok {
        def.push(format!("{}Hz not accepted", qc.sample_rate_hz));
    }
    if let Some(l) = qc.integrated_lufs {
        if (l - spec.lufs_target).abs() > spec.lufs_tol {
            def.push(format!(
                "{:.1} LUFS (need {:.1}±{:.1})",
                l, spec.lufs_target, spec.lufs_tol
            ));
        }
    }
    if spec.isrc_req && meta.isrc.is_none() {
        def.push("ISRC required".into());
    }
    if spec.upc_req && meta.upc.is_none() {
        def.push("UPC required".into());
    }
    if let Some(px) = meta.cover_art_px {
        if px < spec.cover_art_min_px {
            def.push(format!(
                "cover art {}px — need {}px",
                px, spec.cover_art_min_px
            ));
        }
    } else {
        def.push(format!(
            "cover art missing — {} needs {}px",
            spec.dsp.name(),
            spec.cover_art_min_px
        ));
    }
    DspValidationResult {
        dsp: spec.dsp.name().into(),
        passed: def.is_empty(),
        defects: def,
    }
}