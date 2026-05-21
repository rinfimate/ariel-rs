//! SVG template functions for the packet renderer.
//!
//! All colours are passed as parameters — no CSS injection, no hardcoded theme values.
//! CSS class names are kept to match Mermaid JS's DOM structure.
#![allow(dead_code)]

pub use crate::diagrams::util::{esc, fmt};

// ---------------------------------------------------------------------------
// SVG root — matches Mermaid's configureSvgSize(useMaxWidth=true) output
// ---------------------------------------------------------------------------

pub fn svg_root(svg_id: &str, w: &str, h: &str, max_w: &str, font_family: &str) -> String {
    format!(
        "<svg id=\"{svg_id}\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" viewBox=\"0 0 {w} {h}\" style=\"max-width: {max_w}px;\" font-family=\"{ff}\" role=\"graphics-document document\" aria-roledescription=\"packet\" aria-labelledby=\"chart-title-{svg_id}\"><title id=\"chart-title-{svg_id}\">Packet</title>",
        ff = font_family,
    )
}

pub fn empty_svg(svg_id: &str) -> String {
    format!(
        "<svg id=\"{svg_id}\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 50\"></svg>",
    )
}

// ---------------------------------------------------------------------------
// Block rect — `.packetBlock`
// ---------------------------------------------------------------------------

/// Field background rectangle.
/// Mermaid: `group.append("rect")…attr("class","packetBlock")`
pub fn block_rect(x: f64, y: f64, w: f64, h: f64, fill: &str, stroke: &str) -> String {
    format!(
        r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}" stroke="{stroke}" stroke-width="1" class="packetBlock"></rect>"#,
        x = fmt(x),
        y = fmt(y),
        w = fmt(w),
        h = fmt(h),
    )
}

// ---------------------------------------------------------------------------
// Label — `.packetLabel`
// ---------------------------------------------------------------------------

/// Field label centred inside the block.
/// Mermaid: `group.append("text").attr("class","packetLabel")…text(block.label)`
pub fn block_label(x: f64, y: f64, text: &str, color: &str) -> String {
    format!(
        r#"<text x="{x}" y="{y}" fill="{color}" font-size="12px" class="packetLabel" dominant-baseline="middle" text-anchor="middle">{text}</text>"#,
        x = fmt(x),
        y = fmt(y),
    )
}

// ---------------------------------------------------------------------------
// Bit number — `.packetByte start` / `.packetByte end`
// ---------------------------------------------------------------------------

/// Bit number label at the start or end of a block.
/// Mermaid: `group.append("text").attr("class","packetByte start/end")…text(bit)`
pub fn bit_label(
    x: f64,
    y: f64,
    bit: u32,
    class_suffix: &str,
    anchor: &str,
    color: &str,
) -> String {
    format!(
        r#"<text x="{x}" y="{y}" fill="{color}" font-size="10px" class="packetByte {class_suffix}" dominant-baseline="auto" text-anchor="{anchor}">{bit}</text>"#,
        x = fmt(x),
        y = fmt(y),
    )
}

// ---------------------------------------------------------------------------
// Title — `.packetTitle`
// ---------------------------------------------------------------------------

/// Diagram title.
/// Mermaid: `svg.append("text").text(title).attr("class","packetTitle")…`
pub fn diagram_title(x: f64, y: f64, text: &str, color: &str) -> String {
    format!(
        r#"<text x="{x}" y="{y}" fill="{color}" font-size="14px" dominant-baseline="middle" text-anchor="middle" class="packetTitle">{text}</text>"#,
        x = fmt(x),
        y = fmt(y),
    )
}
