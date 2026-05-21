//! SVG template functions for the Ishikawa (fishbone) diagram renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::{esc, fmt};

// ---------------------------------------------------------------------------
// Arrow marker
// ---------------------------------------------------------------------------

/// Render the arrowhead `<marker>` definition for Ishikawa branches.
pub fn arrowhead_marker(mid: &str, color: &str) -> String {
    format!(
        r##"<defs><marker id="{mid}" viewBox="0 0 10 10" refX="0" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M 10 0 L 0 5 L 10 10 Z" fill="{color}" class="ishikawa-arrow"/></marker></defs>"##,
        mid = mid,
        color = color,
    )
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for an Ishikawa diagram.
#[allow(clippy::too_many_arguments)]
pub fn svg_root(
    vw: f64,
    vh: f64,
    mw: f64,
    _style: &str,
    title_part: &str,
    tx: f64,
    ty: f64,
    content: &str,
) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" font-family="Arial, sans-serif" viewBox="0 0 {vw:.5} {vh:.5}" width="100%" style="max-width:{mw:.0}px">{title_part}<g class="ishikawa" transform="translate({tx:.5},{ty:.5})">{content}</g></svg>"##,
        vw = vw,
        vh = vh,
        mw = mw,
        title_part = title_part,
        tx = tx,
        ty = ty,
        content = content,
    )
}

// ---------------------------------------------------------------------------
// Spine
// ---------------------------------------------------------------------------

/// Render the main spine `<line>`.
pub fn spine_line(x1: f64, y: f64, color: &str) -> String {
    format!(
        r##"<line class="ishikawa-spine" stroke="{color}" stroke-width="2" fill="none" x1="{:.5}" y1="{:.5}" x2="0" y2="{:.5}"/>"##,
        x1,
        y,
        y,
        color = color,
    )
}

// ---------------------------------------------------------------------------
// Branches
// ---------------------------------------------------------------------------

/// Render a main branch `<line>` (diagonal, with marker-start arrowhead).
pub fn branch_line(x1: f64, y1: f64, x2: f64, y2: f64, marker_url: &str, color: &str) -> String {
    format!(
        r##"<line class="ishikawa-branch" stroke="{color}" stroke-width="2" fill="none" x1="{:.5}" y1="{:.5}" x2="{:.5}" y2="{:.5}" marker-start="{}"/>"##,
        x1,
        y1,
        x2,
        y2,
        marker_url,
        color = color,
    )
}

/// Render a sub-branch `<line>` (with marker-start arrowhead).
pub fn sub_branch_line(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    marker_url: &str,
    color: &str,
) -> String {
    format!(
        r##"<line class="ishikawa-sub-branch" stroke="{color}" stroke-width="1" fill="none" x1="{:.5}" y1="{:.5}" x2="{:.5}" y2="{:.5}" marker-start="{}"/>"##,
        x1,
        y1,
        x2,
        y2,
        marker_url,
        color = color,
    )
}

// ---------------------------------------------------------------------------
// Labels
// ---------------------------------------------------------------------------

/// Render a horizontal sub-branch label `<text>` (text-anchor: end).
pub fn sub_label_horizontal(x: f64, y: f64, font_size: f64, text: &str, color: &str) -> String {
    format!(
        r##"<text class="ishikawa-label align" fill="{color}" text-anchor="end" dominant-baseline="middle" x="{:.2}" y="{:.2}" font-size="{}">{}</text>"##,
        x,
        y,
        font_size,
        text,
        color = color,
    )
}

/// Render a diagonal sub-branch label `<text>` (text-anchor: end).
pub fn sub_label_diagonal(x: f64, y: f64, font_size: f64, text: &str, color: &str) -> String {
    format!(
        r##"<text class="ishikawa-label" fill="{color}" text-anchor="end" x="{:.2}" y="{:.2}" font-size="{}">{}</text>"##,
        x,
        y,
        font_size,
        text,
        color = color,
    )
}

/// Render a cause label box `<rect>`.
pub fn cause_label_rect(x: f64, y: f64, w: f64, h: f64, fill: &str, stroke: &str) -> String {
    format!(
        r##"<rect class="ishikawa-label-box" fill="{fill}" stroke="{stroke}" stroke-width="2" x="{:.5}" y="{:.5}" width="{:.5}" height="{:.5}"/>"##,
        x,
        y,
        w,
        h,
        fill = fill,
        stroke = stroke,
    )
}

/// Render a cause label `<text>` (text-anchor: middle).
pub fn cause_label_text(x: f64, y: f64, font_size: f64, text: &str, color: &str) -> String {
    format!(
        r##"<text class="ishikawa-label cause" fill="{color}" text-anchor="middle" dominant-baseline="middle" x="{:.2}" y="{:.2}" font-size="{}">{}</text>"##,
        x,
        y,
        font_size,
        text,
        color = color,
    )
}

// ---------------------------------------------------------------------------
// Head (fish head)
// ---------------------------------------------------------------------------

/// Render the fish head group containing the kite path and label text.
#[allow(clippy::too_many_arguments)]
pub fn head_group(
    spine_y: f64,
    head_path: &str,
    label_x: f64,
    label_y: f64,
    font_size: f64,
    label: &str,
    fill: &str,
    stroke: &str,
    text_color: &str,
) -> String {
    format!(
        r##"<g class="ishikawa-head-group" transform="translate(0,{:.1})"><path class="ishikawa-head" fill="{fill}" stroke="{stroke}" stroke-width="2" d="{}"/><text class="ishikawa-head-label" fill="{text_color}" font-weight="600" text-anchor="middle" dominant-baseline="middle" x="{:.1}" y="{:.1}" font-size="{}">{}</text></g>"##,
        spine_y,
        head_path,
        label_x,
        label_y,
        font_size,
        label,
        fill = fill,
        stroke = stroke,
        text_color = text_color,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the Ishikawa diagram title `<text>`.
pub fn title_text(cx: f64, text: &str, color: &str) -> String {
    format!(
        r##"<text class="ishikawa-title" fill="{color}" x="{:.1}" y="20" text-anchor="middle" font-size="16" font-weight="bold">{}</text>"##,
        cx,
        text,
        color = color,
    )
}

// ---------------------------------------------------------------------------
// Empty diagram fallback
// ---------------------------------------------------------------------------

/// Render the empty Ishikawa fallback SVG.
pub fn empty_svg() -> &'static str {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50"><text x="10" y="30" font-size="14">Empty Ishikawa</text></svg>"##
}
