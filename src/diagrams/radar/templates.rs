//! SVG template functions for the radar renderer.
//!
//! Faithful port of Mermaid JS styles.ts and renderer.ts output structure.
//! Uses CSS classes (radarGraticule, radarAxisLine, radarAxisLabel, etc.)
//! to match the reference SVG exactly.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::{esc, fmt};

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element.
/// Mermaid JS sets width="100%" with max-width style and a square viewBox.
pub fn svg_root(total_w: &str, total_h: &str, ff: &str) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" font-family="{ff}" style="max-width: {total_w}px;" viewBox="0 0 {total_w} {total_h}" role="graphics-document document" aria-roledescription="radar">"##,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element.
/// Placed at (x=0, y=-(height/2 + marginTop)) relative to the translated group.
pub fn title_text(y: &str, text: &str, color: &str, font_size: &str) -> String {
    format!(
        r##"<text class="radarTitle" x="0" y="{y}" dominant-baseline="hanging" text-anchor="middle" font-size="{font_size}px" fill="{color}">{text}</text>"##,
    )
}

// ---------------------------------------------------------------------------
// Graticule (grid rings)
// ---------------------------------------------------------------------------

/// Render a circular graticule ring (centred at origin via g transform).
pub fn graticule_circle(r: &str, color: &str, opacity: &str, stroke_width: &str) -> String {
    format!(
        r##"<circle r="{r}" class="radarGraticule" fill="{color}" fill-opacity="{opacity}" stroke="{color}" stroke-width="{stroke_width}"></circle>"##,
    )
}

/// Render a polygon graticule ring.
pub fn graticule_polygon(pts: &str, color: &str, opacity: &str, stroke_width: &str) -> String {
    format!(
        r##"<polygon points="{pts}" class="radarGraticule" fill="{color}" fill-opacity="{opacity}" stroke="{color}" stroke-width="{stroke_width}"></polygon>"##,
    )
}

// ---------------------------------------------------------------------------
// Axes (spokes)
// ---------------------------------------------------------------------------

/// Render a radial spoke line from origin to (x2, y2).
pub fn axis_line(x2: &str, y2: &str, color: &str) -> String {
    format!(
        r##"<line x1="0" y1="0" x2="{x2}" y2="{y2}" class="radarAxisLine" stroke="{color}" stroke-width="2"></line>"##,
    )
}

/// Render an axis label text element.
pub fn axis_label(x: &str, y: &str, text: &str, color: &str, font_size: &str) -> String {
    format!(
        r##"<text x="{x}" y="{y}" class="radarAxisLabel" dominant-baseline="middle" text-anchor="middle" font-size="{font_size}px" fill="{color}">{text}</text>"##,
    )
}

// ---------------------------------------------------------------------------
// Data curves
// ---------------------------------------------------------------------------

/// Render a data curve `<path>` using inline colour.
pub fn curve_path(
    d: &str,
    index: usize,
    color: &str,
    fill_opacity: f64,
    stroke_width: f64,
) -> String {
    format!(
        r##"<path d="{d}" class="radarCurve-{index}" fill="{color}" fill-opacity="{fill_opacity}" stroke="{color}" stroke-width="{stroke_width}"></path>"##,
    )
}

/// Render a data polygon using inline colour.
pub fn curve_polygon(
    pts: &str,
    index: usize,
    color: &str,
    fill_opacity: f64,
    stroke_width: f64,
) -> String {
    format!(
        r##"<polygon points="{pts}" class="radarCurve-{index}" fill="{color}" fill-opacity="{fill_opacity}" stroke="{color}" stroke-width="{stroke_width}"></polygon>"##,
    )
}

// ---------------------------------------------------------------------------
// Legend
// ---------------------------------------------------------------------------

/// Render the legend group wrapper with translate.
pub fn legend_group_open(tx: &str, ty: &str) -> String {
    format!(r##"<g transform="translate({tx}, {ty})">"##,)
}

/// Render a legend colour swatch rect with inline colour.
pub fn legend_rect(index: usize, color: &str, fill_opacity: f64) -> String {
    format!(
        r##"<rect width="12" height="12" class="radarLegendBox-{index}" fill="{color}" fill-opacity="{fill_opacity}" stroke="{color}"></rect>"##,
    )
}

/// Render a legend label text.
pub fn legend_label(text: &str, font_size: &str, text_color: &str) -> String {
    format!(
        r##"<text x="16" y="0" class="radarLegendText" text-anchor="start" font-size="{font_size}px" dominant-baseline="hanging" fill="{text_color}">{text}</text>"##,
    )
}

/// Render the centered drawing group wrapper `<g><g transform="translate(cx, cy)">`.
pub fn centered_group_open(cx: &str, cy: &str) -> String {
    format!(
        r##"<g><g transform="translate({cx}, {cy})">"##,
        cx = cx,
        cy = cy,
    )
}
