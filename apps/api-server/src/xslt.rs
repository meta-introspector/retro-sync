//! XSLT transform layer for society-specific XML submission formats.
//!
//! Architecture:
//!   1. `WorkRegistration` structs → serialised to Retrosync canonical CWR-XML
//!      (namespace: https://retrosync.media/xml/cwr/1) via `to_canonical_xml()`.
//!   2. The canonical XML document is transformed by the appropriate `.xsl`
//!      stylesheet loaded from `XSLT_DIR` (default: `backend/xslt_transforms/`).
//!   3. The `xot` crate provides pure-Rust XSLT 1.0 + XPath 1.0 processing —
//!      no libxslt/libxml2 C dependency.
//!
//! HTTP API:
//!   POST /api/royalty/xslt/:society   — body: JSON WorkRegistration array
//!                                        returns: Content-Type: application/xml
//!   POST /api/royalty/xslt/all        — returns: ZIP of all society XMLs

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Writer as XmlWriter,
};
use std::io::Cursor;
use tracing::{info, warn};

use crate::royalty_reporting::{CollectionSociety, Publisher, WorkRegistration, Writer};
use crate::AppState;

// ── Canonical CWR-XML namespace ──────────────────────────────────────────────
const CWR_NS: &str = "https://retrosync.media/xml/cwr/1";

// ── Society routing ───────────────────────────────────────────────────────────

/// Map URL slug → (CollectionSociety, XSL filename)
fn resolve_society(slug: &str) -> Option<(CollectionSociety, &'static str)> {
    match slug {
        "apra" | "apra_amcos" => Some((CollectionSociety::ApraNz, "apra_amcos.xsl")),
        "gema" => Some((CollectionSociety::GemaDe, "gema.xsl")),
        "sacem" => Some((CollectionSociety::SacemFr, "sacem.xsl")),
        "prs" | "mcps" => Some((CollectionSociety::PrsUk, "prs.xsl")),
        "jasrac" => Some((CollectionSociety::JasracJp, "jasrac.xsl")),
        "socan" | "cmrra" => Some((CollectionSociety::Socan, "socan.xsl")),
        "samro" => Some((CollectionSociety::SamroZa, "samro.xsl")),
        "nordic" | "stim" | "tono" | "koda" | "teosto" | "stef" => {
            Some((CollectionSociety::StimSe, "nordic.xsl"))
        }
        _ => None,
    }
}

// ── Canonical XML serialiser ─────────────────────────────────────────────────

/// Serialise a slice of `WorkRegistration` into Retrosync canonical CWR-XML.
/// All XSLT stylesheets consume this intermediate representation.
#[zkperf_macros::zkperf]
pub fn to_canonical_xml(works: &[WorkRegistration]) -> anyhow::Result<String> {
    let mut buf = Vec::new();
    let mut writer = XmlWriter::new_with_indent(Cursor::new(&mut buf), b' ', 2);

    // <?xml version="1.0" encoding="UTF-8"?>
    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        None,
    )))?;

    // <cwr:WorkRegistrations xmlns:cwr="...">
    let mut root = BytesStart::new("cwr:WorkRegistrations");
    root.push_attribute(("xmlns:cwr", CWR_NS));
    writer.write_event(Event::Start(root))?;

    for work in works {
        write_work(&mut writer, work)?;
    }

    writer.write_event(Event::End(BytesEnd::new("cwr:WorkRegistrations")))?;
    Ok(String::from_utf8(buf)?)
}

fn text_elem(
    writer: &mut XmlWriter<Cursor<&mut Vec<u8>>>,
    tag: &str,
    value: &str,
) -> anyhow::Result<()> {
    writer.write_event(Event::Start(BytesStart::new(tag)))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}

fn write_work(
    writer: &mut XmlWriter<Cursor<&mut Vec<u8>>>,
    work: &WorkRegistration,
) -> anyhow::Result<()> {
    writer.write_event(Event::Start(BytesStart::new("cwr:Work")))?;

    text_elem(writer, "cwr:Iswc", work.iswc.as_deref().unwrap_or(""))?;
    text_elem(writer, "cwr:Title", &work.title)?;
    text_elem(writer, "cwr:Language", &work.language_code)?;
    text_elem(writer, "cwr:MusicArrangement", &work.music_arrangement)?;
    text_elem(writer, "cwr:VersionType", &work.version_type)?;
    text_elem(
        writer,
        "cwr:GrandRightsInd",
        if work.grand_rights_ind { "Y" } else { "N" },
    )?;
    text_elem(writer, "cwr:ExceptionalClause", &work.exceptional_clause)?;
    text_elem(
        writer,
        "cwr:OpusNumber",
        work.opus_number.as_deref().unwrap_or(""),
    )?;
    text_elem(
        writer,
        "cwr:CatalogueNumber",
        work.catalogue_number.as_deref().unwrap_or(""),
    )?;
    text_elem(writer, "cwr:PrimarySociety", work.society.cwr_code())?;

    // Writers
    writer.write_event(Event::Start(BytesStart::new("cwr:Writers")))?;
    for w in &work.writers {
        write_writer(writer, w)?;
    }
    writer.write_event(Event::End(BytesEnd::new("cwr:Writers")))?;

    // Publishers
    writer.write_event(Event::Start(BytesStart::new("cwr:Publishers")))?;
    for p in &work.publishers {
        write_publisher(writer, p)?;
    }
    writer.write_event(Event::End(BytesEnd::new("cwr:Publishers")))?;

    // AlternateTitles
    if !work.alternate_titles.is_empty() {
        writer.write_event(Event::Start(BytesStart::new("cwr:AlternateTitles")))?;
        for alt in &work.alternate_titles {
            writer.write_event(Event::Start(BytesStart::new("cwr:AlternateTitle")))?;
            text_elem(writer, "cwr:Title", &alt.title)?;
            text_elem(writer, "cwr:TitleType", alt.title_type.cwr_code())?;
            text_elem(
                writer,
                "cwr:Language",
                alt.language.as_deref().unwrap_or(""),
            )?;
            writer.write_event(Event::End(BytesEnd::new("cwr:AlternateTitle")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("cwr:AlternateTitles")))?;
    }

    // PerformingArtists
    if !work.performing_artists.is_empty() {
        writer.write_event(Event::Start(BytesStart::new("cwr:PerformingArtists")))?;
        for pa in &work.performing_artists {
            writer.write_event(Event::Start(BytesStart::new("cwr:PerformingArtist")))?;
            text_elem(writer, "cwr:LastName", &pa.last_name)?;
            text_elem(
                writer,
                "cwr:FirstName",
                pa.first_name.as_deref().unwrap_or(""),
            )?;
            text_elem(writer, "cwr:Isni", pa.isni.as_deref().unwrap_or(""))?;
            text_elem(writer, "cwr:IPI", pa.ipi.as_deref().unwrap_or(""))?;
            writer.write_event(Event::End(BytesEnd::new("cwr:PerformingArtist")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("cwr:PerformingArtists")))?;
    }

    // Recording
    if let Some(rec) = &work.recording {
        writer.write_event(Event::Start(BytesStart::new("cwr:Recording")))?;
        text_elem(writer, "cwr:Isrc", rec.isrc.as_deref().unwrap_or(""))?;
        text_elem(
            writer,
            "cwr:ReleaseTitle",
            rec.release_title.as_deref().unwrap_or(""),
        )?;
        text_elem(writer, "cwr:Label", rec.label.as_deref().unwrap_or(""))?;
        text_elem(
            writer,
            "cwr:ReleaseDate",
            rec.release_date.as_deref().unwrap_or(""),
        )?;
        text_elem(writer, "cwr:Format", rec.recording_format.cwr_code())?;
        text_elem(writer, "cwr:Technique", rec.recording_technique.cwr_code())?;
        text_elem(writer, "cwr:MediaType", rec.media_type.cwr_code())?;
        writer.write_event(Event::End(BytesEnd::new("cwr:Recording")))?;
    }

    // Territories
    writer.write_event(Event::Start(BytesStart::new("cwr:Territories")))?;
    for t in &work.territories {
        writer.write_event(Event::Start(BytesStart::new("cwr:Territory")))?;
        text_elem(writer, "cwr:TisCode", t.tis_code())?;
        writer.write_event(Event::End(BytesEnd::new("cwr:Territory")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("cwr:Territories")))?;

    writer.write_event(Event::End(BytesEnd::new("cwr:Work")))?;
    Ok(())
}

fn write_writer(writer: &mut XmlWriter<Cursor<&mut Vec<u8>>>, w: &Writer) -> anyhow::Result<()> {
    writer.write_event(Event::Start(BytesStart::new("cwr:Writer")))?;
    text_elem(writer, "cwr:LastName", &w.last_name)?;
    text_elem(writer, "cwr:FirstName", &w.first_name)?;
    text_elem(writer, "cwr:IpiCae", w.ipi_cae.as_deref().unwrap_or(""))?;
    text_elem(writer, "cwr:IpiBase", w.ipi_base.as_deref().unwrap_or(""))?;
    text_elem(writer, "cwr:Role", w.role.cwr_code())?;
    text_elem(writer, "cwr:SharePct", &format!("{:.4}", w.share_pct))?;
    text_elem(
        writer,
        "cwr:Society",
        w.society.as_ref().map(|s| s.cwr_code()).unwrap_or(""),
    )?;
    text_elem(
        writer,
        "cwr:Controlled",
        if w.controlled { "Y" } else { "N" },
    )?;
    writer.write_event(Event::End(BytesEnd::new("cwr:Writer")))?;
    Ok(())
}

fn write_publisher(
    writer: &mut XmlWriter<Cursor<&mut Vec<u8>>>,
    p: &Publisher,
) -> anyhow::Result<()> {
    writer.write_event(Event::Start(BytesStart::new("cwr:Publisher")))?;
    text_elem(writer, "cwr:Name", &p.name)?;
    text_elem(writer, "cwr:IpiCae", p.ipi_cae.as_deref().unwrap_or(""))?;
    text_elem(writer, "cwr:IpiBase", p.ipi_base.as_deref().unwrap_or(""))?;
    text_elem(writer, "cwr:PublisherType", p.publisher_type.cwr_code())?;
    text_elem(writer, "cwr:SharePct", &format!("{:.4}", p.share_pct))?;
    text_elem(
        writer,
        "cwr:Society",
        p.society.as_ref().map(|s| s.cwr_code()).unwrap_or(""),
    )?;
    writer.write_event(Event::End(BytesEnd::new("cwr:Publisher")))?;
    Ok(())
}

// ── XSLT processor ───────────────────────────────────────────────────────────

fn xslt_dir() -> std::path::PathBuf {
    std::env::var("XSLT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("backend/xslt_transforms"))
}

/// Load an XSL stylesheet and apply it to `xml_input`.
///
/// Currently validates both the source XML and the stylesheet via `xot`,
/// then returns the canonical XML as-is. Full XSLT 1.0 transform support
/// requires an XSLT engine (e.g. libxslt bindings) — tracked for future work.
#[zkperf_macros::zkperf]
pub fn apply_xslt(xml_input: &str, xsl_filename: &str) -> anyhow::Result<String> {
    let xsl_path = xslt_dir().join(xsl_filename);
    let xsl_src = std::fs::read_to_string(&xsl_path)
        .map_err(|e| anyhow::anyhow!("Cannot load stylesheet {}: {}", xsl_path.display(), e))?;

    // Validate both documents parse as well-formed XML
    let mut xot = xot::Xot::new();
    let source = xot.parse(xml_input)?;
    let _style = xot.parse(&xsl_src)?;

    // Serialize the validated source back out (identity transform)
    let output = xot.to_string(source)?;
    info!(stylesheet=%xsl_filename, "XSLT identity transform applied (full XSLT engine pending)");
    Ok(output)
}

// ── HTTP handlers ─────────────────────────────────────────────────────────────

/// POST /api/royalty/xslt/:society
/// Body: JSON array of WorkRegistration
/// Returns: application/xml transformed for the named society
#[zkperf_macros::zkperf]
pub async fn transform_submission(
    State(state): State<AppState>,
    Path(society_slug): Path<String>,
    Json(works): Json<Vec<WorkRegistration>>,
) -> Result<Response, StatusCode> {
    let (society, xsl_file) = resolve_society(&society_slug).ok_or_else(|| {
        warn!(slug=%society_slug, "Unknown society slug for XSLT transform");
        StatusCode::NOT_FOUND
    })?;

    let canonical = to_canonical_xml(&works).map_err(|e| {
        warn!(err=%e, "Failed to serialise canonical CWR-XML");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let output = apply_xslt(&canonical, xsl_file).map_err(|e| {
        warn!(err=%e, stylesheet=%xsl_file, "XSLT transform failed");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    state
        .audit_log
        .record(&format!(
            "XSLT_TRANSFORM society='{}' works={}",
            society.display_name(),
            works.len()
        ))
        .ok();

    Ok((
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        output,
    )
        .into_response())
}

/// POST /api/royalty/xslt/all
/// Body: JSON array of WorkRegistration
/// Returns: JSON map of society → XML string (all societies in one call)
#[zkperf_macros::zkperf]
pub async fn transform_all_submissions(
    State(state): State<AppState>,
    Json(works): Json<Vec<WorkRegistration>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let canonical = to_canonical_xml(&works).map_err(|e| {
        warn!(err=%e, "Failed to serialise canonical CWR-XML");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let slugs = [
        "apra", "gema", "sacem", "prs", "jasrac", "socan", "samro", "nordic",
    ];

    let mut results = serde_json::Map::new();
    for slug in slugs {
        let (_society, xsl_file) = match resolve_society(slug) {
            Some(v) => v,
            None => continue,
        };
        match apply_xslt(&canonical, xsl_file) {
            Ok(xml) => {
                results.insert(slug.to_string(), serde_json::Value::String(xml));
            }
            Err(e) => {
                warn!(slug=%slug, err=%e, "XSLT failed for society");
                results.insert(
                    slug.to_string(),
                    serde_json::Value::String(format!("ERROR: {e}")),
                );
            }
        }
    }

    state
        .audit_log
        .record(&format!(
            "XSLT_TRANSFORM_ALL works={} societies={}",
            works.len(),
            slugs.len()
        ))
        .ok();

    Ok(Json(serde_json::Value::Object(results)))
}