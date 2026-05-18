//! SVG template functions for the pie renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element for a pie diagram.
pub fn svg_root(id: &str, vbx: &str, vbw: &str, vbh: &str, mw: &str) -> String {
    format!(
        r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="{vbx} 0 {vbw} {vbh}" style="max-width: {mw}px;" role="graphics-document document" aria-roledescription="pie">"#,
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
pub fn outer_circle(r: &str) -> String {
    format!(r#"<circle cx="0" cy="0" r="{r}" class="pieOuterCircle"></circle>"#)
}

/// Render a pie slice `<path>`.
pub fn pie_slice(d: &str, color: &str) -> String {
    format!(r#"<path d="{d}" fill="{color}" class="pieCircle"></path>"#)
}

/// Render a percentage label for a pie slice.
pub fn slice_label(cx: &str, cy: &str, pct: u64) -> String {
    format!(
        r#"<text transform="translate({cx},{cy})" class="slice" style="text-anchor: middle;">{pct}%</text>"#,
    )
}

/// Render the diagram title text.
pub fn title_text(y: &str, text: &str) -> String {
    format!(r#"<text x="0" y="{y}" class="pieTitleText">{text}</text>"#)
}

/// Render one legend item (coloured rect + label text).
pub fn legend_item(lx: &str, vert: &str, color: &str, text: &str) -> String {
    format!(
        r#"<g class="legend" transform="translate({lx},{vert})"><rect width="18" height="18" style="fill: {color}; stroke: {color};"></rect><text x="22" y="14">{text}</text></g>"#,
    )
}
