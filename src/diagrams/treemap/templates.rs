//! SVG template functions for the treemap renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::esc;

pub fn fmt_f(v: f64) -> String {
    if v == v.floor() && v.is_finite() {
        format!("{}", v as i64)
    } else {
        // Up to 4 significant decimal digits, strip trailing zeros
        let s = format!("{:.4}", v);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

pub fn fmt_i(v: f64) -> i64 {
    v.round() as i64
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper element for an empty treemap.
pub(crate) fn svg_root_empty(max_w: i64, vb_x: i64, vb_y: i64, vb_w: i64, vb_h: i64) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100%" style="max-width: {max_w}px;" viewBox="{vb_x} {vb_y} {vb_w} {vb_h}" role="graphics-document" class="flowchart"></svg>"##,
    )
}

/// Render the outer SVG wrapper element for a non-empty treemap.
pub(crate) fn svg_root(max_w: i64, vb_x: i64, vb_y: i64, vb_w: i64, vb_h: i64) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" font-family="Arial, sans-serif" width="100%" style="max-width: {max_w}px;" viewBox="{vb_x} {vb_y} {vb_w} {vb_h}" role="graphics-document" class="flowchart">"##,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element.
pub(crate) fn title_text(cx: &str, ty: &str, text_color: &str, text: &str) -> String {
    format!(
        r##"<text x="{cx}" y="{ty}" class="treemapTitle" text-anchor="middle" dominant-baseline="middle" fill="{text_color}" font-size="14px">{text}</text>"##,
    )
}

// ---------------------------------------------------------------------------
// Container group
// ---------------------------------------------------------------------------

/// Render the opening `<g>` of the treemap container group.
pub(crate) fn container_group_open(ty: &str) -> String {
    format!(r##"<g transform="translate(0, {ty})" class="treemapContainer">"##)
}

// ---------------------------------------------------------------------------
// Section (branch node)
// ---------------------------------------------------------------------------

/// Render the opening `<g>` of a treemap section group.
pub(crate) fn section_group_open(x: &str, y: &str) -> String {
    format!(r##"<g class="treemapSection" transform="translate({x},{y})">"##)
}

/// Render the section header background `<rect>`.
pub(crate) fn section_header_rect(w: &str, h: &str, display: &str) -> String {
    format!(
        r##"<rect width="{w}" height="{h}" class="treemapSectionHeader" fill="none" fill-opacity="0.6" stroke-width="0.6" style="{display}"></rect>"##,
    )
}

/// Render a `<clipPath>` for the section header text.
pub(crate) fn section_clip_path(idx: usize, clip_w: &str, clip_h: &str) -> String {
    format!(
        r##"<clipPath id="clip-section-mermaid-svg-0-{idx}"><rect width="{clip_w}" height="{clip_h}"></rect></clipPath>"##,
    )
}

/// Render the full-section background `<rect>`.
pub(crate) fn section_rect(
    w: &str,
    h: &str,
    idx: usize,
    fill: &str,
    stroke: &str,
    display: &str,
) -> String {
    format!(
        r##"<rect width="{w}" height="{h}" class="treemapSection section{idx}" fill="{fill}" fill-opacity="0.6" stroke="{stroke}" stroke-width="2" stroke-opacity="0.4" style="{display}"></rect>"##,
    )
}

/// X offset (px) for section label text inside its header.
const SECTION_LABEL_X: &str = "6";

/// Render the section label `<text>`.
pub(crate) fn section_label_text(header_mid_y: &str, style: &str, text: &str) -> String {
    format!(
        r##"<text class="treemapSectionLabel" x="{SECTION_LABEL_X}" y="{header_mid_y}" dominant-baseline="middle" font-weight="bold" style="{style}">{text}</text>"##,
    )
}

/// Render the section value `<text>`.
pub(crate) fn section_value_text(
    val_x: &str,
    header_mid_y: &str,
    style: &str,
    text: &str,
) -> String {
    format!(
        r##"<text class="treemapSectionValue" x="{val_x}" y="{header_mid_y}" text-anchor="end" dominant-baseline="middle" font-style="italic" style="{style}">{text}</text>"##,
    )
}

// ---------------------------------------------------------------------------
// Leaf node
// ---------------------------------------------------------------------------

/// Render the opening `<g>` of a treemap leaf group.
pub(crate) fn leaf_group_open(idx: usize, x: &str, y: &str) -> String {
    format!(
        r##"<g class="treemapNode treemapLeafGroup leaf{idx}x" transform="translate({x},{y})">"##,
    )
}

/// Render the leaf background `<rect>`.
pub(crate) fn leaf_rect(w: &str, h: &str, fill: &str) -> String {
    format!(
        r##"<rect width="{w}" height="{h}" class="treemapLeaf" fill="{fill}" style="" fill-opacity="0.3" stroke="{fill}" stroke-width="3"></rect>"##,
    )
}

/// Render the leaf `<clipPath>`.
pub(crate) fn leaf_clip_path(idx: usize, clip_w: &str, clip_h: &str) -> String {
    format!(
        r##"<clipPath id="clip-mermaid-svg-0-{idx}"><rect width="{clip_w}" height="{clip_h}"></rect></clipPath>"##,
    )
}

/// Render the leaf label `<text>`.
pub(crate) fn leaf_label_text(
    cx: &str,
    cy: &str,
    font_size: u32,
    color: &str,
    idx: usize,
    text: &str,
) -> String {
    format!(
        r##"<text class="treemapLabel" x="{cx}" y="{cy}" style="text-anchor: middle; dominant-baseline: middle; font-size: {font_size}px;fill:{color};" clip-path="url(#clip-mermaid-svg-0-{idx})">{text}</text>"##,
    )
}

/// Render the leaf value `<text>`.
pub(crate) fn leaf_value_text(
    cx: &str,
    value_y: &str,
    font_size: u32,
    color: &str,
    idx: usize,
    text: &str,
) -> String {
    format!(
        r##"<text class="treemapValue" x="{cx}" y="{value_y}" style="text-anchor: middle; dominant-baseline: hanging; font-size: {font_size}px; fill: {color};" clip-path="url(#clip-mermaid-svg-0-{idx})">{text}</text>"##,
    )
}
