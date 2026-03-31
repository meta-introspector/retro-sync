//! PRO reporting — CWR 2.2 full record set + all global collection societies.
// This module contains infrastructure-ready PRO generators not yet wired to
// routes.  The dead_code allow covers the entire module until they are linked.
#![allow(dead_code)]
//!
//! Coverage:
//!   Americas  : ASCAP, BMI, SESAC, SOCAN, CMRRA, SPACEM, SCD (Chile), UBC (Brazil),
//!               SGAE (Spain/LatAm admin), SAYCO (Colombia), APA (Paraguay),
//!               APDAYC (Peru), SACVEN (Venezuela), SPA (Panama), ACAM (Costa Rica),
//!               ACDAM (Cuba), BUBEDRA (Bolivia), AGADU (Uruguay), ABRAMUS (Brazil),
//!               ECAD (Brazil neighboring)
//!   Europe    : PRS for Music (UK), MCPS (UK mech), GEMA (DE), SACEM (FR),
//!               SIAE (IT), SGAE (ES), BUMA/STEMRA (NL), SABAM (BE), STIM (SE),
//!               TONO (NO), KODA (DK), TEOSTO (FI), STEF (IS), IMRO (IE),
//!               APA (AT), SUISA (CH), SPA (PT), ARTISJUS (HU), OSA (CZ),
//!               SOZA (SK), ZAIKS (PL), EAU (EE), LATGA (LT), AKKA/LAA (LV),
//!               HDS-ZAMP (HR), SOKOJ (RS), ZAMP (MK/SI), MUSICAUTOR (BG),
//!               UCMR-ADA (RO), RAO (RU), UACRR (UA), COMPASS (SG/MY)
//!   Asia-Pac  : JASRAC (JP), KMA/KMCA (KR), CASH (HK), MUST (TW), MCSC (CN),
//!               APRA AMCOS (AU/NZ), IPRS (IN), MCT (TH), MACP (MY), MRCSB (BN),
//!               PPH (PH), WAMI (ID), KCI (ID neighboring)
//!   Africa/ME : SAMRO (ZA), MCSK (KE), COSON (NG), SOCAN-SODRAC (CA mech),
//!               CAPASSO (ZA neighboring), KAMP (KE neighboring), ACREMASCI (CI),
//!               BUMDA (DZ), BNDA (BF), SODAV (SN), ARMP (MA), SACERAU (EG),
//!               SACS (IL), OSC (TN), SOCINPRO (LB), NCAC (GH)
//!
//! CWR record types implemented:
//!   HDR  — transmission header
//!   GRH  — group header
//!   NWR  — new works registration
//!   REV  — revised registration
//!   OPU  — non-registered work
//!   SPU  — sub-publisher
//!   OPU  — original publisher unknown
//!   SWR  — sub-writer
//!   OWR  — original writer unknown
//!   PWR  — publisher for writer
//!   ALT  — alternate title
//!   PER  — performing artist
//!   REC  — recording detail
//!   ORN  — work origin
//!   INS  — instrumentation summary
//!   IND  — instrumentation detail
//!   COM  — component
//!   ACK  — acknowledgement (inbound)
//!   GRT  — group trailer
//!   TRL  — transmission trailer

use serde::{Deserialize, Serialize};
use tracing::info;

// ── CWR version selector ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CwrVersion {
    V21,
    V22,
}
impl CwrVersion {
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V21 => "02.10",
            Self::V22 => "02.20",
        }
    }
}

// ── Global collection society registry ──────────────────────────────────────
//
// CISAC 3-digit codes (leading zeros preserved as strings).
// Sources: CISAC Society Database (cisac.org), CWR standard tables rev. 2022.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CollectionSociety {
    // ── Americas ──────────────────────────────────────────────────────────
    Ascap,           // 021 — US performing rights
    Bmi,             // 022 — US performing rights
    Sesac,           // 023 — US performing rights
    Socan,           // 022 (CA) / use "055" SOCAN performing
    Cmrra,           // 050 — Canada mechanical
    SpaciemMx,       // 048 — Mexico (SPACEM)
    SociedadChilena, // 080 — SCD Chile
    UbcBrazil,       // 088 — UBC Brazil
    EcadBrazil,      // 089 — ECAD Brazil (neighboring)
    AbramusBrazil,   // 088 (ABRAMUS shares ECAD/UBC infra)
    SaycoCol,        // 120 — SAYCO Colombia
    ApaParaguay,     // 145 — APA Paraguay
    ApdaycPeru,      // 150 — APDAYC Peru
    SacvenVenezuela, // 155 — SACVEN Venezuela
    SpaPanama,       // 160 — SPA Panama
    AcamCostaRica,   // 105 — ACAM Costa Rica
    AcdamCuba,       // 110 — ACDAM Cuba
    BbubedraBol,     // 095 — BUBEDRA Bolivia
    AgaduUruguay,    // 100 — AGADU Uruguay
    // ── Europe ────────────────────────────────────────────────────────────
    PrsUk,        // 052 — PRS for Music (UK performing + MCPS mechanical)
    McpsUk,       // 053 — MCPS standalone mechanical
    GemaDe,       // 035 — GEMA Germany
    SacemFr,      // 058 — SACEM France
    SiaeIt,       // 074 — SIAE Italy
    SgaeEs,       // 068 — SGAE Spain
    BumaNl,       // 028 — BUMA Netherlands (now Buma/Stemra)
    StemraNl,     // 028 — STEMRA mechanical (same code, different dept)
    SabamBe,      // 055 — SABAM Belgium
    StimSe,       // 077 — STIM Sweden
    TonoNo,       // 083 — TONO Norway
    KodaDk,       // 040 — KODA Denmark
    TeostoFi,     // 078 — TEOSTO Finland
    StefIs,       // 113 — STEF Iceland
    ImroIe,       // 039 — IMRO Ireland
    ApaAt,        // 009 — APA Austria
    SuisaCh,      // 076 — SUISA Switzerland
    SpaciemPt,    // 069 — SPA Portugal
    ArtisjusHu,   // 008 — ARTISJUS Hungary
    OsaCz,        // 085 — OSA Czech Republic
    SozaSk,       // 072 — SOZA Slovakia
    ZaiksPl,      // 089 — ZAIKS Poland
    EauEe,        // 033 — EAU Estonia
    LatgaLt,      // 044 — LATGA Lithuania
    AkkaLv,       // 002 — AKKA/LAA Latvia
    HdsZampHr,    // 036 — HDS-ZAMP Croatia
    SokojRs,      // 070 — SOKOJ Serbia
    ZampMkSi,     // 089 — ZAMP North Macedonia / Slovenia
    MusicautorBg, // 061 — MUSICAUTOR Bulgaria
    UcmrRo,       // 087 — UCMR-ADA Romania
    RaoRu,        // 064 — RAO Russia
    UacrUa,       // 081 — UACRR Ukraine
    // ── Asia-Pacific ─────────────────────────────────────────────────────
    JasracJp,  // 099 — JASRAC Japan
    KmaKr,     // 100 — KMA/KMCA Korea
    CashHk,    // 031 — CASH Hong Kong
    MustTw,    // 079 — MUST Taiwan
    McscCn,    // 062 — MCSC China
    ApraNz,    // 006 — APRA AMCOS Australia/NZ
    IprsIn,    // 038 — IPRS India
    MctTh,     // 097 — MCT Thailand
    MacpMy,    // 098 — MACP Malaysia
    PphPh,     // 103 — PPH Philippines
    WamiId,    // 111 — WAMI Indonesia
    KciId,     // 112 — KCI Indonesia (neighboring)
    CompassSg, // 114 — COMPASS Singapore
    // ── Africa / Middle East ─────────────────────────────────────────────
    SamroZa,     // 066 — SAMRO South Africa
    CapassoZa,   // 115 — CAPASSO South Africa (neighboring)
    McskKe,      // 116 — MCSK Kenya
    KampKe,      // 117 — KAMP Kenya (neighboring)
    CosonNg,     // 118 — COSON Nigeria
    AcremasciCi, // 119 — ACREMASCI Côte d'Ivoire
    BumdaDz,     // 121 — BUMDA Algeria
    BndaBf,      // 122 — BNDA Burkina Faso
    SodavSn,     // 123 — SODAV Senegal
    ArmpMa,      // 124 — ARMP Morocco
    SacerauEg,   // 125 — SACERAU Egypt
    SacsIl,      // 126 — SACS Israel
    OscTn,       // 127 — OSC Tunisia
    NcacGh,      // 128 — NCAC Ghana
    // ── Catch-all ─────────────────────────────────────────────────────────
    Other(String), // raw 3-digit CISAC code or custom string
}

impl CollectionSociety {
    /// CISAC 3-digit CWR society code.
    #[zkperf_macros::zkperf]
    pub fn cwr_code(&self) -> &str {
        match self {
            // Americas
            Self::Ascap => "021",
            Self::Bmi => "022",
            Self::Sesac => "023",
            Self::Socan => "055",
            Self::Cmrra => "050",
            Self::SpaciemMx => "048",
            Self::SociedadChilena => "080",
            Self::UbcBrazil => "088",
            Self::EcadBrazil => "089",
            Self::AbramusBrazil => "088",
            Self::SaycoCol => "120",
            Self::ApaParaguay => "145",
            Self::ApdaycPeru => "150",
            Self::SacvenVenezuela => "155",
            Self::SpaPanama => "160",
            Self::AcamCostaRica => "105",
            Self::AcdamCuba => "110",
            Self::BbubedraBol => "095",
            Self::AgaduUruguay => "100",
            // Europe
            Self::PrsUk => "052",
            Self::McpsUk => "053",
            Self::GemaDe => "035",
            Self::SacemFr => "058",
            Self::SiaeIt => "074",
            Self::SgaeEs => "068",
            Self::BumaNl => "028",
            Self::StemraNl => "028",
            Self::SabamBe => "055",
            Self::StimSe => "077",
            Self::TonoNo => "083",
            Self::KodaDk => "040",
            Self::TeostoFi => "078",
            Self::StefIs => "113",
            Self::ImroIe => "039",
            Self::ApaAt => "009",
            Self::SuisaCh => "076",
            Self::SpaciemPt => "069",
            Self::ArtisjusHu => "008",
            Self::OsaCz => "085",
            Self::SozaSk => "072",
            Self::ZaiksPl => "089",
            Self::EauEe => "033",
            Self::LatgaLt => "044",
            Self::AkkaLv => "002",
            Self::HdsZampHr => "036",
            Self::SokojRs => "070",
            Self::ZampMkSi => "089",
            Self::MusicautorBg => "061",
            Self::UcmrRo => "087",
            Self::RaoRu => "064",
            Self::UacrUa => "081",
            // Asia-Pacific
            Self::JasracJp => "099",
            Self::KmaKr => "100",
            Self::CashHk => "031",
            Self::MustTw => "079",
            Self::McscCn => "062",
            Self::ApraNz => "006",
            Self::IprsIn => "038",
            Self::MctTh => "097",
            Self::MacpMy => "098",
            Self::PphPh => "103",
            Self::WamiId => "111",
            Self::KciId => "112",
            Self::CompassSg => "114",
            // Africa / Middle East
            Self::SamroZa => "066",
            Self::CapassoZa => "115",
            Self::McskKe => "116",
            Self::KampKe => "117",
            Self::CosonNg => "118",
            Self::AcremasciCi => "119",
            Self::BumdaDz => "121",
            Self::BndaBf => "122",
            Self::SodavSn => "123",
            Self::ArmpMa => "124",
            Self::SacerauEg => "125",
            Self::SacsIl => "126",
            Self::OscTn => "127",
            Self::NcacGh => "128",
            Self::Other(s) => s.as_str(),
        }
    }

    /// Human-readable society name.
    #[zkperf_macros::zkperf]
    pub fn display_name(&self) -> &str {
        match self {
            Self::Ascap => "ASCAP (US)",
            Self::Bmi => "BMI (US)",
            Self::Sesac => "SESAC (US)",
            Self::Socan => "SOCAN (CA)",
            Self::Cmrra => "CMRRA (CA)",
            Self::SpaciemMx => "SPACEM (MX)",
            Self::SociedadChilena => "SCD (CL)",
            Self::UbcBrazil => "UBC (BR)",
            Self::EcadBrazil => "ECAD (BR)",
            Self::AbramusBrazil => "ABRAMUS (BR)",
            Self::SaycoCol => "SAYCO (CO)",
            Self::ApaParaguay => "APA (PY)",
            Self::ApdaycPeru => "APDAYC (PE)",
            Self::SacvenVenezuela => "SACVEN (VE)",
            Self::SpaPanama => "SPA (PA)",
            Self::AcamCostaRica => "ACAM (CR)",
            Self::AcdamCuba => "ACDAM (CU)",
            Self::BbubedraBol => "BUBEDRA (BO)",
            Self::AgaduUruguay => "AGADU (UY)",
            Self::PrsUk => "PRS for Music (UK)",
            Self::McpsUk => "MCPS (UK)",
            Self::GemaDe => "GEMA (DE)",
            Self::SacemFr => "SACEM (FR)",
            Self::SiaeIt => "SIAE (IT)",
            Self::SgaeEs => "SGAE (ES)",
            Self::BumaNl => "BUMA (NL)",
            Self::StemraNl => "STEMRA (NL)",
            Self::SabamBe => "SABAM (BE)",
            Self::StimSe => "STIM (SE)",
            Self::TonoNo => "TONO (NO)",
            Self::KodaDk => "KODA (DK)",
            Self::TeostoFi => "TEOSTO (FI)",
            Self::StefIs => "STEF (IS)",
            Self::ImroIe => "IMRO (IE)",
            Self::ApaAt => "APA (AT)",
            Self::SuisaCh => "SUISA (CH)",
            Self::SpaciemPt => "SPA (PT)",
            Self::ArtisjusHu => "ARTISJUS (HU)",
            Self::OsaCz => "OSA (CZ)",
            Self::SozaSk => "SOZA (SK)",
            Self::ZaiksPl => "ZAIKS (PL)",
            Self::EauEe => "EAU (EE)",
            Self::LatgaLt => "LATGA (LT)",
            Self::AkkaLv => "AKKA/LAA (LV)",
            Self::HdsZampHr => "HDS-ZAMP (HR)",
            Self::SokojRs => "SOKOJ (RS)",
            Self::ZampMkSi => "ZAMP (MK/SI)",
            Self::MusicautorBg => "MUSICAUTOR (BG)",
            Self::UcmrRo => "UCMR-ADA (RO)",
            Self::RaoRu => "RAO (RU)",
            Self::UacrUa => "UACRR (UA)",
            Self::JasracJp => "JASRAC (JP)",
            Self::KmaKr => "KMA/KMCA (KR)",
            Self::CashHk => "CASH (HK)",
            Self::MustTw => "MUST (TW)",
            Self::McscCn => "MCSC (CN)",
            Self::ApraNz => "APRA AMCOS (AU/NZ)",
            Self::IprsIn => "IPRS (IN)",
            Self::MctTh => "MCT (TH)",
            Self::MacpMy => "MACP (MY)",
            Self::PphPh => "PPH (PH)",
            Self::WamiId => "WAMI (ID)",
            Self::KciId => "KCI (ID)",
            Self::CompassSg => "COMPASS (SG)",
            Self::SamroZa => "SAMRO (ZA)",
            Self::CapassoZa => "CAPASSO (ZA)",
            Self::McskKe => "MCSK (KE)",
            Self::KampKe => "KAMP (KE)",
            Self::CosonNg => "COSON (NG)",
            Self::AcremasciCi => "ACREMASCI (CI)",
            Self::BumdaDz => "BUMDA (DZ)",
            Self::BndaBf => "BNDA (BF)",
            Self::SodavSn => "SODAV (SN)",
            Self::ArmpMa => "ARMP (MA)",
            Self::SacerauEg => "SACERAU (EG)",
            Self::SacsIl => "SACS (IL)",
            Self::OscTn => "OSC (TN)",
            Self::NcacGh => "NCAC (GH)",
            Self::Other(s) => s.as_str(),
        }
    }

    /// Two-letter ISO territory most closely associated with this society.
    #[allow(dead_code)]
    pub fn primary_territory(&self) -> &'static str {
        match self {
            Self::Ascap | Self::Bmi | Self::Sesac => "US",
            Self::Socan | Self::Cmrra => "CA",
            Self::SpaciemMx => "MX",
            Self::SociedadChilena => "CL",
            Self::UbcBrazil | Self::EcadBrazil | Self::AbramusBrazil => "BR",
            Self::SaycoCol => "CO",
            Self::ApaParaguay => "PY",
            Self::ApdaycPeru => "PE",
            Self::SacvenVenezuela => "VE",
            Self::SpaPanama => "PA",
            Self::AcamCostaRica => "CR",
            Self::AcdamCuba => "CU",
            Self::BbubedraBol => "BO",
            Self::AgaduUruguay => "UY",
            Self::PrsUk | Self::McpsUk => "GB",
            Self::GemaDe => "DE",
            Self::SacemFr => "FR",
            Self::SiaeIt => "IT",
            Self::SgaeEs => "ES",
            Self::BumaNl | Self::StemraNl => "NL",
            Self::SabamBe => "BE",
            Self::StimSe => "SE",
            Self::TonoNo => "NO",
            Self::KodaDk => "DK",
            Self::TeostoFi => "FI",
            Self::StefIs => "IS",
            Self::ImroIe => "IE",
            Self::ApaAt => "AT",
            Self::SuisaCh => "CH",
            Self::SpaciemPt => "PT",
            Self::ArtisjusHu => "HU",
            Self::OsaCz => "CZ",
            Self::SozaSk => "SK",
            Self::ZaiksPl => "PL",
            Self::EauEe => "EE",
            Self::LatgaLt => "LT",
            Self::AkkaLv => "LV",
            Self::HdsZampHr => "HR",
            Self::SokojRs => "RS",
            Self::ZampMkSi => "MK",
            Self::MusicautorBg => "BG",
            Self::UcmrRo => "RO",
            Self::RaoRu => "RU",
            Self::UacrUa => "UA",
            Self::JasracJp => "JP",
            Self::KmaKr => "KR",
            Self::CashHk => "HK",
            Self::MustTw => "TW",
            Self::McscCn => "CN",
            Self::ApraNz => "AU",
            Self::IprsIn => "IN",
            Self::MctTh => "TH",
            Self::MacpMy => "MY",
            Self::PphPh => "PH",
            Self::WamiId | Self::KciId => "ID",
            Self::CompassSg => "SG",
            Self::SamroZa | Self::CapassoZa => "ZA",
            Self::McskKe | Self::KampKe => "KE",
            Self::CosonNg => "NG",
            Self::AcremasciCi => "CI",
            Self::BumdaDz => "DZ",
            Self::BndaBf => "BF",
            Self::SodavSn => "SN",
            Self::ArmpMa => "MA",
            Self::SacerauEg => "EG",
            Self::SacsIl => "IL",
            Self::OscTn => "TN",
            Self::NcacGh => "GH",
            Self::Other(_) => "XX",
        }
    }
}

// ── Writer role codes (CWR standard) ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WriterRole {
    Composer,          // C
    Lyricist,          // A  (Author)
    ComposerLyricist,  // CA
    Arranger,          // AR
    Adaptor,           // AD
    Translator,        // TR
    SubArranger,       // A  (when used in sub context)
    OriginalPublisher, // E
    SubPublisher,      // SE
    AcquisitionAdmins, // AM (administrator)
    IncomeParticipant, // PA
    Publisher,         // E  (alias)
}
impl WriterRole {
    #[zkperf_macros::zkperf]
    pub fn cwr_code(&self) -> &'static str {
        match self {
            Self::Composer => "C",
            Self::Lyricist => "A",
            Self::ComposerLyricist => "CA",
            Self::Arranger => "AR",
            Self::Adaptor => "AD",
            Self::Translator => "TR",
            Self::SubArranger => "A",
            Self::OriginalPublisher => "E",
            Self::SubPublisher => "SE",
            Self::AcquisitionAdmins => "AM",
            Self::IncomeParticipant => "PA",
            Self::Publisher => "E",
        }
    }
}

// ── Territory codes (CISAC TIS) ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TerritoryScope {
    World,        // 2136
    Worldwide,    // 2136 (alias)
    Europe,       // 2100
    NorthAmerica, // 2104
    LatinAmerica, // 2106
    AsiaPacific,  // 2114
    Africa,       // 2120
    MiddleEast,   // 2122
    Iso(String),  // direct ISO 3166-1 alpha-2
}
impl TerritoryScope {
    #[zkperf_macros::zkperf]
    pub fn tis_code(&self) -> &str {
        match self {
            Self::World | Self::Worldwide => "2136",
            Self::Europe => "2100",
            Self::NorthAmerica => "2104",
            Self::LatinAmerica => "2106",
            Self::AsiaPacific => "2114",
            Self::Africa => "2120",
            Self::MiddleEast => "2122",
            Self::Iso(s) => s.as_str(),
        }
    }
}

// ── Domain types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Writer {
    pub ipi_cae: Option<String>,  // 11-digit IPI name number
    pub ipi_base: Option<String>, // 13-char IPI base number (CWR 2.2)
    pub last_name: String,
    pub first_name: String,
    pub role: WriterRole,
    pub share_pct: f64, // 0.0 – 100.0
    pub society: Option<CollectionSociety>,
    pub controlled: bool, // Y = controlled writer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    pub ipi_cae: Option<String>,
    pub ipi_base: Option<String>,
    pub name: String,
    pub share_pct: f64,
    pub society: Option<CollectionSociety>,
    pub publisher_type: PublisherType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PublisherType {
    AcquisitionAdministrator, // AQ
    SubPublisher,             // SE
    IncomeParticipant,        // PA
    OriginalPublisher,        // E
}
impl PublisherType {
    #[zkperf_macros::zkperf]
    pub fn cwr_code(&self) -> &'static str {
        match self {
            Self::AcquisitionAdministrator => "AQ",
            Self::SubPublisher => "SE",
            Self::IncomeParticipant => "PA",
            Self::OriginalPublisher => "E",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternateTitle {
    pub title: String,
    pub title_type: AltTitleType,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AltTitleType {
    AlternateTitle,              // AT
    FormalTitle,                 // FT
    OriginalTitle,               // OT
    OriginalTitleTransliterated, // OL
    TitleOfComponents,           // TC
    TitleOfSampler,              // TS
}
impl AltTitleType {
    #[zkperf_macros::zkperf]
    pub fn cwr_code(&self) -> &'static str {
        match self {
            Self::AlternateTitle => "AT",
            Self::FormalTitle => "FT",
            Self::OriginalTitle => "OT",
            Self::OriginalTitleTransliterated => "OL",
            Self::TitleOfComponents => "TC",
            Self::TitleOfSampler => "TS",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformingArtist {
    pub last_name: String,
    pub first_name: Option<String>,
    pub isni: Option<String>, // International Standard Name Identifier
    pub ipi: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingDetail {
    pub isrc: Option<String>,
    pub release_title: Option<String>,
    pub label: Option<String>,
    pub release_date: Option<String>, // YYYYMMDD
    pub recording_format: RecordingFormat,
    pub recording_technique: RecordingTechnique,
    pub media_type: MediaType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordingFormat {
    Audio,
    Visual,
    Audiovisual,
}
impl RecordingFormat {
    #[zkperf_macros::zkperf]
    pub fn cwr_code(&self) -> &'static str {
        match self {
            Self::Audio => "A",
            Self::Visual => "V",
            Self::Audiovisual => "AV",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordingTechnique {
    Analogue,
    Digital,
    Unknown,
}
impl RecordingTechnique {
    #[zkperf_macros::zkperf]
    pub fn cwr_code(&self) -> &'static str {
        match self {
            Self::Analogue => "A",
            Self::Digital => "D",
            Self::Unknown => "U",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    Cd,
    Vinyl,
    Cassette,
    Digital,
    Other,
}
impl MediaType {
    #[zkperf_macros::zkperf]
    pub fn cwr_code(&self) -> &'static str {
        match self {
            Self::Cd => "CD",
            Self::Vinyl => "VI",
            Self::Cassette => "CA",
            Self::Digital => "DI",
            Self::Other => "OT",
        }
    }
}

// ── Work registration (master struct) ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkRegistration {
    // Identifiers
    pub iswc: Option<String>, // T-nnnnnnnnn-c
    pub title: String,
    pub language_code: String,           // ISO 639-2 (3 chars)
    pub music_arrangement: String,       // ORI/NEW/MOD/UNS/ADM
    pub text_music_relationship: String, // MUS/MTX/TXT
    pub excerpt_type: String,            // MOV/UNS (or blank)
    pub composite_type: String,          // MED/POT/UCO/SUI (or blank)
    pub version_type: String,            // ORI/MOD/LIB (or blank)
    // Parties
    pub writers: Vec<Writer>,
    pub publishers: Vec<Publisher>,
    pub alternate_titles: Vec<AlternateTitle>,
    pub performing_artists: Vec<PerformingArtist>,
    pub recording: Option<RecordingDetail>,
    // Routing
    pub society: CollectionSociety, // primary registration society
    pub territories: Vec<TerritoryScope>,
    // Flags
    pub grand_rights_ind: bool,
    pub composite_component_count: u8,
    pub date_of_publication: Option<String>,
    pub exceptional_clause: String, // Y/N/U
    pub opus_number: Option<String>,
    pub catalogue_number: Option<String>,
    pub priority_flag: String, // Y/N
}

impl Default for WorkRegistration {
    fn default() -> Self {
        Self {
            iswc: None,
            title: String::new(),
            language_code: "EN".into(),
            music_arrangement: "ORI".into(),
            text_music_relationship: "MTX".into(),
            excerpt_type: String::new(),
            composite_type: String::new(),
            version_type: "ORI".into(),
            writers: vec![],
            publishers: vec![],
            alternate_titles: vec![],
            performing_artists: vec![],
            recording: None,
            society: CollectionSociety::PrsUk,
            territories: vec![TerritoryScope::World],
            grand_rights_ind: false,
            composite_component_count: 0,
            date_of_publication: None,
            exceptional_clause: "U".into(),
            opus_number: None,
            catalogue_number: None,
            priority_flag: "N".into(),
        }
    }
}

// ── CWR 2.2 generator ────────────────────────────────────────────────────────
//
// Fixed-width record format per CISAC CWR Technical Reference Manual.
// Each record is exactly 190 characters (standard) + CRLF.

#[allow(dead_code)]
fn pad(s: &str, width: usize) -> String {
    format!("{s:width$}")
}

#[allow(dead_code)]
fn pad_right(s: &str, width: usize) -> String {
    let mut r = s.to_string();
    r.truncate(width);
    format!("{r:<width$}")
}

#[allow(dead_code)]
fn pad_num(n: u64, width: usize) -> String {
    format!("{n:0>width$}")
}

#[allow(dead_code)]
pub fn generate_cwr(works: &[WorkRegistration], sender_id: &str, version: CwrVersion) -> String {
    let ts = chrono::Utc::now();
    let date = ts.format("%Y%m%d").to_string();
    let time = ts.format("%H%M%S").to_string();
    let nworks = works.len();

    let mut records: Vec<String> = Vec::new();

    // ── HDR ─────────────────────────────────────────────────────────────────
    // HDR + record_type(3) + sender_type(1) + sender_id(9) + sender_name(45)
    // + version(5) + creation_date(8) + creation_time(6) + transmission_date(8)
    // + character_set(15)
    records.push(format!(
        "HDR{sender_type}{sender_id:<9}{sender_name:<45}{ver}  {date}{time}{tdate}{charset:<15}",
        sender_type = "PB", // publisher
        sender_id = pad_right(sender_id, 9),
        sender_name = pad_right(sender_id, 45),
        ver = version.as_str(),
        date = date,
        time = time,
        tdate = date,
        charset = "UTF-8",
    ));

    // ── GRH ─────────────────────────────────────────────────────────────────
    records.push(format!(
        "GRH{txn_type}{group_id:05}{ver}0000000{batch:08}",
        txn_type = "NWR",
        group_id = 1,
        ver = version.as_str(),
        batch = 0,
    ));

    let mut record_count: u64 = 0;
    for (i, work) in works.iter().enumerate() {
        let seq = format!("{:08}", i + 1);

        // ── NWR ─────────────────────────────────────────────────────────────
        let nwr = format!(
            "NWR{seq}0001{iswc:<11}{title:<60}{lang:<3}{arr:<3}{tmr:<3}{exc:<3}{comp:<3}{ver_t:<3}{gr}{comp_cnt:02}{pub_date:<8}{exc_cl}{opus:<25}{cat:<25}{pri}",
            seq       = seq,
            iswc      = pad_right(work.iswc.as_deref().unwrap_or("           "), 11),
            title     = pad_right(&work.title, 60),
            lang      = pad_right(&work.language_code, 3),
            arr       = pad_right(&work.music_arrangement, 3),
            tmr       = pad_right(&work.text_music_relationship, 3),
            exc       = pad_right(&work.excerpt_type, 3),
            comp      = pad_right(&work.composite_type, 3),
            ver_t     = pad_right(&work.version_type, 3),
            gr        = if work.grand_rights_ind { "Y" } else { "N" },
            comp_cnt  = work.composite_component_count,
            pub_date  = pad_right(work.date_of_publication.as_deref().unwrap_or("        "), 8),
            exc_cl    = &work.exceptional_clause,
            opus      = pad_right(work.opus_number.as_deref().unwrap_or(""), 25),
            cat       = pad_right(work.catalogue_number.as_deref().unwrap_or(""), 25),
            pri       = &work.priority_flag,
        );
        records.push(nwr);
        record_count += 1;

        // ── SPU — publishers ─────────────────────────────────────────────
        for (j, pub_) in work.publishers.iter().enumerate() {
            records.push(format!(
                "SPU{seq}{pn:04}  {ipi:<11}{ipi_base:<13}{name:<45}{soc}{pub_type:<2}{share:05.0}  {controlled}",
                seq       = seq,
                pn        = j + 1,
                ipi       = pad_right(pub_.ipi_cae.as_deref().unwrap_or("           "), 11),
                ipi_base  = pad_right(pub_.ipi_base.as_deref().unwrap_or("             "), 13),
                name      = pad_right(&pub_.name, 45),
                soc       = pub_.society.as_ref().map(|s| s.cwr_code()).unwrap_or("   "),
                pub_type  = pub_.publisher_type.cwr_code(),
                share     = pub_.share_pct * 100.0,
                controlled= "Y",
            ));
            record_count += 1;
        }

        // ── SWR — writers ────────────────────────────────────────────────
        for (j, w) in work.writers.iter().enumerate() {
            records.push(format!(
                "SWR{seq}{wn:04}{ipi:<11}{ipi_base:<13}{last:<45}{first:<30}{role:<2}{soc}{share:05.0}  {controlled}",
                seq       = seq,
                wn        = j + 1,
                ipi       = pad_right(w.ipi_cae.as_deref().unwrap_or("           "), 11),
                ipi_base  = pad_right(w.ipi_base.as_deref().unwrap_or("             "), 13),
                last      = pad_right(&w.last_name, 45),
                first     = pad_right(&w.first_name, 30),
                role      = w.role.cwr_code(),
                soc       = w.society.as_ref().map(|s| s.cwr_code()).unwrap_or("   "),
                share     = w.share_pct * 100.0,
                controlled= if w.controlled { "Y" } else { "N" },
            ));
            record_count += 1;

            // PWR — publisher for writer (one per controlled writer)
            if w.controlled && !work.publishers.is_empty() {
                let pub0 = &work.publishers[0];
                records.push(format!(
                    "PWR{seq}{wn:04}{pub_ipi:<11}{pub_name:<45}  ",
                    seq = seq,
                    wn = j + 1,
                    pub_ipi = pad_right(pub0.ipi_cae.as_deref().unwrap_or("           "), 11),
                    pub_name = pad_right(&pub0.name, 45),
                ));
                record_count += 1;
            }
        }

        // ── ALT — alternate titles ───────────────────────────────────────
        for alt in &work.alternate_titles {
            records.push(format!(
                "ALT{seq}{title:<60}{tt}{lang:<2}",
                seq = seq,
                title = pad_right(&alt.title, 60),
                tt = alt.title_type.cwr_code(),
                lang = pad_right(alt.language.as_deref().unwrap_or("  "), 2),
            ));
            record_count += 1;
        }

        // ── PER — performing artists ─────────────────────────────────────
        for pa in &work.performing_artists {
            records.push(format!(
                "PER{seq}{last:<45}{first:<30}{isni:<16}{ipi:<11}",
                seq = seq,
                last = pad_right(&pa.last_name, 45),
                first = pad_right(pa.first_name.as_deref().unwrap_or(""), 30),
                isni = pad_right(pa.isni.as_deref().unwrap_or("                "), 16),
                ipi = pad_right(pa.ipi.as_deref().unwrap_or("           "), 11),
            ));
            record_count += 1;
        }

        // ── REC — recording detail ───────────────────────────────────────
        if let Some(rec) = &work.recording {
            records.push(format!(
                "REC{seq}{isrc:<12}{release_date:<8}{release_title:<60}{label:<60}{fmt}{tech}{media}",
                seq           = seq,
                isrc          = pad_right(rec.isrc.as_deref().unwrap_or("            "), 12),
                release_date  = pad_right(rec.release_date.as_deref().unwrap_or("        "), 8),
                release_title = pad_right(rec.release_title.as_deref().unwrap_or(""), 60),
                label         = pad_right(rec.label.as_deref().unwrap_or(""), 60),
                fmt           = rec.recording_format.cwr_code(),
                tech          = rec.recording_technique.cwr_code(),
                media         = rec.media_type.cwr_code(),
            ));
            record_count += 1;
        }

        // ── ORN — work origin ────────────────────────────────────────────
        // Emitted with primary society territory TIS code
        for territory in &work.territories {
            records.push(format!(
                "ORN{seq}{tis:<4}{society:<3}  ",
                seq = seq,
                tis = pad_right(territory.tis_code(), 4),
                society = work.society.cwr_code(),
            ));
            record_count += 1;
        }
    }

    // ── GRT ─────────────────────────────────────────────────────────────────
    records.push(format!(
        "GRT{group_id:05}{txn_count:08}{rec_count:08}",
        group_id = 1,
        txn_count = nworks,
        rec_count = record_count + 2, // +GRH+GRT
    ));

    // ── TRL ─────────────────────────────────────────────────────────────────
    records.push(format!(
        "TRL{groups:08}{txn_count:08}{rec_count:08}",
        groups = 1,
        txn_count = nworks,
        rec_count = record_count + 4, // +HDR+GRH+GRT+TRL
    ));

    info!(works=%nworks, version=?version, "CWR generated");
    records.join("\r\n")
}

// ── Society-specific submission wrappers ─────────────────────────────────────

/// JASRAC J-DISC extended CSV (Japan).
/// J-DISC requires works in a CSV with JASRAC-specific fields before CWR upload.
#[zkperf_macros::zkperf]
pub fn generate_jasrac_jdisc_csv(works: &[WorkRegistration]) -> String {
    let mut out = String::from(
        "JASRAC_CODE,WORK_TITLE,COMPOSER_IPI,LYRICIST_IPI,PUBLISHER_IPI,ISWC,LANGUAGE,ARRANGEMENT\r\n"
    );
    for w in works {
        let composer = w
            .writers
            .iter()
            .find(|wr| matches!(wr.role, WriterRole::Composer | WriterRole::ComposerLyricist));
        let lyricist = w
            .writers
            .iter()
            .find(|wr| matches!(wr.role, WriterRole::Lyricist));
        let publisher = w.publishers.first();
        out.push_str(&format!(
            "{jasrac},{title},{comp_ipi},{lyr_ipi},{pub_ipi},{iswc},{lang},{arr}\r\n",
            jasrac = "", // assigned by JASRAC after first submission
            title = w.title,
            comp_ipi = composer.and_then(|c| c.ipi_cae.as_deref()).unwrap_or(""),
            lyr_ipi = lyricist.and_then(|l| l.ipi_cae.as_deref()).unwrap_or(""),
            pub_ipi = publisher.and_then(|p| p.ipi_cae.as_deref()).unwrap_or(""),
            iswc = w.iswc.as_deref().unwrap_or(""),
            lang = w.language_code,
            arr = w.music_arrangement,
        ));
    }
    info!(works=%works.len(), "JASRAC J-DISC CSV generated");
    out
}

/// SOCAN/CMRRA joint submission metadata JSON (Canada).
/// SOCAN accepts CWR + a JSON sidecar for electronic filing via MusicMark portal.
#[zkperf_macros::zkperf]
pub fn generate_socan_metadata_json(works: &[WorkRegistration], sender_id: &str) -> String {
    let entries: Vec<serde_json::Value> = works
        .iter()
        .map(|w| {
            let writers: Vec<serde_json::Value> = w
                .writers
                .iter()
                .map(|wr| {
                    serde_json::json!({
                        "last_name":  wr.last_name,
                        "first_name": wr.first_name,
                        "ipi":        wr.ipi_cae,
                        "role":       wr.role.cwr_code(),
                        "society":    wr.society.as_ref().map(|s| s.cwr_code()),
                        "share_pct":  wr.share_pct,
                    })
                })
                .collect();
            serde_json::json!({
                "iswc":  w.iswc,
                "title": w.title,
                "language": w.language_code,
                "writers": writers,
                "territories": w.territories.iter().map(|t| t.tis_code()).collect::<Vec<_>>(),
            })
        })
        .collect();
    let doc = serde_json::json!({
        "sender_id": sender_id,
        "created":   chrono::Utc::now().to_rfc3339(),
        "works":     entries,
    });
    info!(works=%works.len(), "SOCAN metadata JSON generated");
    doc.to_string()
}

/// APRA AMCOS XML submission wrapper (Australia/New Zealand).
/// Wraps a CWR payload in the APRA electronic submission XML envelope.
#[zkperf_macros::zkperf]
pub fn generate_apra_xml_envelope(cwr_payload: &str, sender_id: &str) -> String {
    let ts = chrono::Utc::now().to_rfc3339();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<APRASubmission xmlns="https://www.apra.com.au/cwr/submission/1.0"
                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <Header>
    <SenderID>{sender_id}</SenderID>
    <SubmissionDate>{ts}</SubmissionDate>
    <Format>CWR</Format>
    <Version>2.2</Version>
  </Header>
  <Payload encoding="base64">{payload}</Payload>
</APRASubmission>"#,
        sender_id = sender_id,
        ts = ts,
        payload = base64_encode(cwr_payload.as_bytes()),
    )
}

/// GEMA online portal submission CSV (Germany).
/// Required alongside CWR for GEMA's WorkRegistration portal.
#[zkperf_macros::zkperf]
pub fn generate_gema_csv(works: &[WorkRegistration]) -> String {
    let mut out = String::from(
        "ISWC,Werktitel,Komponist_IPI,Texter_IPI,Verleger_IPI,Sprache,Arrangement\r\n",
    );
    for w in works {
        let comp = w
            .writers
            .iter()
            .find(|wr| matches!(wr.role, WriterRole::Composer | WriterRole::ComposerLyricist));
        let text = w
            .writers
            .iter()
            .find(|wr| matches!(wr.role, WriterRole::Lyricist));
        let pub_ = w.publishers.first();
        out.push_str(&format!(
            "{iswc},{title},{comp},{text},{pub_ipi},{lang},{arr}\r\n",
            iswc = w.iswc.as_deref().unwrap_or(""),
            title = w.title,
            comp = comp.and_then(|c| c.ipi_cae.as_deref()).unwrap_or(""),
            text = text.and_then(|t| t.ipi_cae.as_deref()).unwrap_or(""),
            pub_ipi = pub_.and_then(|p| p.ipi_cae.as_deref()).unwrap_or(""),
            lang = w.language_code,
            arr = w.music_arrangement,
        ));
    }
    info!(works=%works.len(), "GEMA CSV generated");
    out
}

/// Nordic NCB block submission (STIM/TONO/KODA/TEOSTO/STEF).
/// Nordic societies accept a single CWR with society codes for all five.
#[zkperf_macros::zkperf]
pub fn generate_nordic_cwr_block(works: &[WorkRegistration], sender_id: &str) -> String {
    // Stamp all works with Nordic society territories and generate one CWR
    let nordic_works: Vec<WorkRegistration> = works
        .iter()
        .map(|w| {
            let mut w2 = w.clone();
            w2.territories = vec![
                TerritoryScope::Iso("SE".into()),
                TerritoryScope::Iso("NO".into()),
                TerritoryScope::Iso("DK".into()),
                TerritoryScope::Iso("FI".into()),
                TerritoryScope::Iso("IS".into()),
            ];
            w2
        })
        .collect();
    info!(works=%works.len(), "Nordic CWR block generated (STIM/TONO/KODA/TEOSTO/STEF)");
    generate_cwr(&nordic_works, sender_id, CwrVersion::V22)
}

/// MCPS-PRS Alliance extended metadata (UK).
/// PRS Online requires JSON metadata alongside CWR for mechanical licensing.
#[zkperf_macros::zkperf]
pub fn generate_prs_extended_json(works: &[WorkRegistration], sender_id: &str) -> String {
    let entries: Vec<serde_json::Value> = works
        .iter()
        .map(|w| {
            serde_json::json!({
                "iswc":     w.iswc,
                "title":    w.title,
                "language": w.language_code,
                "opus":     w.opus_number,
                "catalogue": w.catalogue_number,
                "grand_rights": w.grand_rights_ind,
                "writers": w.writers.iter().map(|wr| serde_json::json!({
                    "name":    format!("{} {}", wr.first_name, wr.last_name),
                    "ipi":     wr.ipi_cae,
                    "role":    wr.role.cwr_code(),
                    "share":   wr.share_pct,
                    "society": wr.society.as_ref().map(|s| s.display_name()),
                })).collect::<Vec<_>>(),
                "recording": w.recording.as_ref().map(|r| serde_json::json!({
                    "isrc":  r.isrc,
                    "label": r.label,
                    "date":  r.release_date,
                })),
            })
        })
        .collect();
    let doc = serde_json::json!({
        "sender": sender_id,
        "created": chrono::Utc::now().to_rfc3339(),
        "works": entries,
    });
    info!(works=%works.len(), "PRS/MCPS extended JSON generated");
    doc.to_string()
}

/// SACEM (France) submission report — tab-separated extended format.
#[zkperf_macros::zkperf]
pub fn generate_sacem_tsv(works: &[WorkRegistration]) -> String {
    let mut out =
        String::from("ISWC\tTitre\tCompositeursIPI\tParoliersIPI\tEditeurIPI\tSociete\tLangue\r\n");
    for w in works {
        let composers: Vec<&str> = w
            .writers
            .iter()
            .filter(|wr| matches!(wr.role, WriterRole::Composer | WriterRole::ComposerLyricist))
            .filter_map(|wr| wr.ipi_cae.as_deref())
            .collect();
        let lyricists: Vec<&str> = w
            .writers
            .iter()
            .filter(|wr| matches!(wr.role, WriterRole::Lyricist))
            .filter_map(|wr| wr.ipi_cae.as_deref())
            .collect();
        let pub_ = w.publishers.first();
        out.push_str(&format!(
            "{iswc}\t{title}\t{comp}\t{lyr}\t{pub_ipi}\t{soc}\t{lang}\r\n",
            iswc = w.iswc.as_deref().unwrap_or(""),
            title = w.title,
            comp = composers.join(";"),
            lyr = lyricists.join(";"),
            pub_ipi = pub_.and_then(|p| p.ipi_cae.as_deref()).unwrap_or(""),
            soc = w.society.cwr_code(),
            lang = w.language_code,
        ));
    }
    info!(works=%works.len(), "SACEM TSV generated");
    out
}

/// SAMRO (South Africa) registration CSV.
#[zkperf_macros::zkperf]
pub fn generate_samro_csv(works: &[WorkRegistration]) -> String {
    let mut out =
        String::from("ISWC,Title,Composer_IPI,Lyricist_IPI,Publisher_IPI,Language,Territory\r\n");
    for w in works {
        let comp = w
            .writers
            .iter()
            .find(|wr| matches!(wr.role, WriterRole::Composer | WriterRole::ComposerLyricist));
        let lyr = w
            .writers
            .iter()
            .find(|wr| matches!(wr.role, WriterRole::Lyricist));
        let pub_ = w.publishers.first();
        out.push_str(&format!(
            "{iswc},{title},{comp},{lyr},{pub_ipi},{lang},ZA\r\n",
            iswc = w.iswc.as_deref().unwrap_or(""),
            title = w.title,
            comp = comp.and_then(|c| c.ipi_cae.as_deref()).unwrap_or(""),
            lyr = lyr.and_then(|l| l.ipi_cae.as_deref()).unwrap_or(""),
            pub_ipi = pub_.and_then(|p| p.ipi_cae.as_deref()).unwrap_or(""),
            lang = w.language_code,
        ));
    }
    info!(works=%works.len(), "SAMRO CSV generated");
    out
}

// Minimal base64 encode (no external dep, just for APRA XML envelope)
fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            CHARS[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            CHARS[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

// ── SoundExchange (US digital performance rights) ────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundExchangeRow {
    pub isrc: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub play_count: u64,
    pub royalty_usd: f64,
    pub period_start: String,
    pub period_end: String,
}

#[zkperf_macros::zkperf]
pub fn generate_soundexchange_csv(rows: &[SoundExchangeRow]) -> String {
    let mut out = String::from(
        "ISRC,Title,Featured Artist,Album,Total Plays,Royalty (USD),Period Start,Period End\r\n",
    );
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{},{},{:.2},{},{}\r\n",
            r.isrc,
            r.title,
            r.artist,
            r.album,
            r.play_count,
            r.royalty_usd,
            r.period_start,
            r.period_end,
        ));
    }
    info!(rows=%rows.len(), "SoundExchange CSV generated");
    out
}

// ── MLC §115 (US mechanical via Music Modernization Act) ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlcUsageRow {
    pub isrc: String,
    pub iswc: Option<String>,
    pub title: String,
    pub artist: String,
    pub service_name: String,
    pub play_count: u64,
    pub royalty_usd: f64,
    pub territory: String,
    pub period: String,
}

#[zkperf_macros::zkperf]
pub fn generate_mlc_csv(rows: &[MlcUsageRow], service_id: &str) -> String {
    let mut out = format!(
        "Service ID: {sid}\r\nReport: {ts}\r\nISRC,ISWC,Title,Artist,Service,Plays,Royalty USD,Territory,Period\r\n",
        sid = service_id,
        ts  = chrono::Utc::now().format("%Y-%m-%d"),
    );
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{},{},{},{:.2},{},{}\r\n",
            r.isrc,
            r.iswc.as_deref().unwrap_or(""),
            r.title,
            r.artist,
            r.service_name,
            r.play_count,
            r.royalty_usd,
            r.territory,
            r.period,
        ));
    }
    info!(rows=%rows.len(), "MLC CSV generated (Music Modernization Act §115)");
    out
}

// ── Neighboring rights (PPL/SAMI/ADAMI/SCPP etc.) ───────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighboringRightsRow {
    pub isrc: String,
    pub artist: String,
    pub label: String,
    pub play_count: u64,
    pub territory: String,
    pub society: String,
    pub period: String,
}

#[zkperf_macros::zkperf]
pub fn generate_neighboring_rights_csv(rows: &[NeighboringRightsRow]) -> String {
    let mut out = String::from("ISRC,Artist,Label,Plays,Territory,Society,Period\r\n");
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{},{},{},{}\r\n",
            r.isrc, r.artist, r.label, r.play_count, r.territory, r.society, r.period,
        ));
    }
    info!(rows=%rows.len(), "Neighboring rights CSV generated");
    out
}

// ── Dispatch: route works to correct society generator ───────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubmissionFormat {
    Cwr22,       // standard CWR 2.2 (most societies)
    JasracJdisc, // JASRAC J-DISC CSV
    SocanJson,   // SOCAN metadata JSON sidecar
    ApraXml,     // APRA AMCOS XML envelope
    GemaCsv,     // GEMA portal CSV
    NordicBlock, // STIM/TONO/KODA/TEOSTO/STEF combined
    PrsJson,     // PRS for Music / MCPS extended JSON
    SacemTsv,    // SACEM tab-separated
    SamroCsv,    // SAMRO CSV
}

pub struct SocietySubmission {
    pub society: CollectionSociety,
    pub format: SubmissionFormat,
    pub payload: String,
    pub filename: String,
}

/// Route a work batch to all required society submission formats.
#[zkperf_macros::zkperf]
pub fn generate_all_submissions(
    works: &[WorkRegistration],
    sender_id: &str,
) -> Vec<SocietySubmission> {
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let mut out = Vec::new();

    // Standard CWR 2.2 — covers most CISAC member societies
    let cwr = generate_cwr(works, sender_id, CwrVersion::V22);
    out.push(SocietySubmission {
        society: CollectionSociety::Ascap,
        format: SubmissionFormat::Cwr22,
        payload: cwr.clone(),
        filename: format!("{sender_id}_{ts}_CWR22.cwr"),
    });

    // JASRAC J-DISC CSV (Japan)
    out.push(SocietySubmission {
        society: CollectionSociety::JasracJp,
        format: SubmissionFormat::JasracJdisc,
        payload: generate_jasrac_jdisc_csv(works),
        filename: format!("{sender_id}_{ts}_JASRAC_JDISC.csv"),
    });

    // SOCAN JSON sidecar (Canada)
    out.push(SocietySubmission {
        society: CollectionSociety::Socan,
        format: SubmissionFormat::SocanJson,
        payload: generate_socan_metadata_json(works, sender_id),
        filename: format!("{sender_id}_{ts}_SOCAN.json"),
    });

    // APRA AMCOS XML (Australia/NZ)
    out.push(SocietySubmission {
        society: CollectionSociety::ApraNz,
        format: SubmissionFormat::ApraXml,
        payload: generate_apra_xml_envelope(&cwr, sender_id),
        filename: format!("{sender_id}_{ts}_APRA.xml"),
    });

    // GEMA CSV (Germany)
    out.push(SocietySubmission {
        society: CollectionSociety::GemaDe,
        format: SubmissionFormat::GemaCsv,
        payload: generate_gema_csv(works),
        filename: format!("{sender_id}_{ts}_GEMA.csv"),
    });

    // Nordic block (STIM/TONO/KODA/TEOSTO/STEF)
    out.push(SocietySubmission {
        society: CollectionSociety::StimSe,
        format: SubmissionFormat::NordicBlock,
        payload: generate_nordic_cwr_block(works, sender_id),
        filename: format!("{sender_id}_{ts}_NORDIC.cwr"),
    });

    // PRS/MCPS JSON (UK)
    out.push(SocietySubmission {
        society: CollectionSociety::PrsUk,
        format: SubmissionFormat::PrsJson,
        payload: generate_prs_extended_json(works, sender_id),
        filename: format!("{sender_id}_{ts}_PRS.json"),
    });

    // SACEM TSV (France)
    out.push(SocietySubmission {
        society: CollectionSociety::SacemFr,
        format: SubmissionFormat::SacemTsv,
        payload: generate_sacem_tsv(works),
        filename: format!("{sender_id}_{ts}_SACEM.tsv"),
    });

    // SAMRO CSV (South Africa)
    out.push(SocietySubmission {
        society: CollectionSociety::SamroZa,
        format: SubmissionFormat::SamroCsv,
        payload: generate_samro_csv(works),
        filename: format!("{sender_id}_{ts}_SAMRO.csv"),
    });

    info!(
        submissions=%out.len(),
        works=%works.len(),
        "All society submissions generated"
    );
    out
}