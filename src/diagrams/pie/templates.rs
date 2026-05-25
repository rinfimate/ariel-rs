//! SVG template functions for the pie renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::{esc, fmt};

/// Format a value for showData display: integers show without decimal, floats show as-is.
pub fn fmt_value(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        // Strip trailing zeros
        let s = format!("{:.10}", v);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element for a pie diagram.
pub fn svg_root(id: &str, vbx: &str, vbw: &str, vbh: &str, mw: &str, ff: &str) -> String {
    format!(
        r##"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" font-family="{ff}" viewBox="{vbx} 0 {vbw} {vbh}" style="max-width: {mw}px;" role="graphics-document document" aria-roledescription="pie">"##,
    )
}

// ---------------------------------------------------------------------------
// Chart elements
// ---------------------------------------------------------------------------

/// Render the main content group translated to the pie centre.
pub fn main_group(tx: &str, ty: &str) -> String {
    format!("<g transform=\"translate({tx},{ty})\">")
}

/// Render the outer circle border of the pie.
pub fn outer_circle(r: &str, stroke: &str) -> String {
    format!(
        r##"<circle cx="0" cy="0" r="{r}" stroke="{stroke}" stroke-width="2" fill="none" class="pieOuterCircle"></circle>"##
    )
}

/// Render a pie slice `<path>`.
pub fn pie_slice(d: &str, color: &str, stroke: &str) -> String {
    format!(
        r##"<path d="{d}" fill="{color}" stroke="{stroke}" stroke-width="2" opacity="0.7" class="pieCircle"></path>"##
    )
}

/// Render a percentage label for a pie slice.
pub fn slice_label(cx: &str, cy: &str, pct: u64, text_color: &str) -> String {
    format!(
        r##"<text transform="translate({cx},{cy})" text-anchor="middle" font-size="17px" fill="{text_color}" class="slice">{pct}%</text>"##,
    )
}

/// Render the diagram title text.
pub fn title_text(y: &str, text: &str, text_color: &str) -> String {
    format!(
        r##"<text x="0" y="{y}" text-anchor="middle" font-size="25px" fill="{text_color}" class="pieTitleText">{text}</text>"##
    )
}

/// Render one legend item (coloured rect + label text).
pub fn legend_item(lx: &str, vert: &str, color: &str, text: &str, text_color: &str) -> String {
    format!(
        r##"<g class="legend" transform="translate({lx},{vert})"><rect width="18" height="18" style="fill: {color}; stroke: {color};"></rect><text x="22" y="14" fill="{text_color}" font-size="17px">{text}</text></g>"##,
    )
}
