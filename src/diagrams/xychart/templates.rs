//! SVG template functions for the xychart renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::fmt;

pub fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

pub fn build_style(id: &str, ff: &str) -> String {
    use super::constants::{AXIS_LABEL_FONT_SIZE, TITLE_COLOR, TITLE_FONT_SIZE};
    format!(
        concat!(
            "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}",
            "#{id} .main{{}}",
            "#{id} text{{font-family:{ff};}}",
            "#{id} .chart-title text{{font-size:{tfs}px;fill:{tc};text-anchor:middle;dominant-baseline:middle;}}",
            "#{id} .left-axis path,#{id} .bottom-axis path,#{id} .top-axis path{{fill:none;stroke:#333;}}",
            "#{id} .left-axis .label text,#{id} .bottom-axis .label text,#{id} .top-axis .label text{{fill:#333;font-size:{lfs}px;}}",
            "#{id} .plot rect{{opacity:0.85;}}",
            "#{id} .plot path{{fill:none;}}",
        ),
        id = id,
        ff = ff,
        tfs = TITLE_FONT_SIZE as i64,
        tc = TITLE_COLOR,
        lfs = AXIS_LABEL_FONT_SIZE as i64,
    )
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG root element for an xychart diagram.
pub fn svg_root(id: &str, width: i64, height: i64) -> String {
    format!(
        r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="max-width: {w}px;" viewBox="0 0 {w} {h}" role="graphics-document document" aria-roledescription="xychart">"#,
        id = id,
        w = width,
        h = height,
    )
}

// ---------------------------------------------------------------------------
// Background
// ---------------------------------------------------------------------------

/// Render the chart background `<rect>` inside the `.main` group.
pub fn main_group_with_bg(width: i64, height: i64, bg_color: &str) -> String {
    format!(
        r#"<g class="main"><rect width="{}" height="{}" class="background" fill="{}"></rect>"#,
        width, height, bg_color,
    )
}

// ---------------------------------------------------------------------------
// Element rendering
// ---------------------------------------------------------------------------

/// Render a single chart `<rect>` element.
pub fn chart_rect(
    x: &str,
    y: &str,
    w: &str,
    h: &str,
    fill: &str,
    stroke: &str,
    stroke_w: &str,
) -> String {
    format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="{}"></rect>"#,
        x, y, w, h, fill, stroke, stroke_w,
    )
}

/// Render a single chart `<text>` element with transform.
pub fn chart_text(
    fill: &str,
    font_size: &str,
    dominant_baseline: &str,
    text_anchor: &str,
    transform: &str,
    text: &str,
) -> String {
    format!(
        r#"<text x="0" y="0" fill="{}" font-size="{}" dominant-baseline="{}" text-anchor="{}" transform="{}">{}</text>"#,
        fill, font_size, dominant_baseline, text_anchor, transform, text,
    )
}

/// Render a single chart `<path>` element.
pub fn chart_path(path_d: &str, fill: &str, stroke: &str, stroke_w: &str) -> String {
    format!(
        r#"<path d="{}" fill="{}" stroke="{}" stroke-width="{}"></path>"#,
        path_d, fill, stroke, stroke_w,
    )
}

/// Render the opening `<g class="...">` for a group level.
pub fn group_open(class: &str) -> String {
    format!(r#"<g class="{}">"#, class)
}
