//! SVG template functions for the quadrant renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::{esc, fmt};

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

pub fn build_style(id: &str, ff: &str) -> String {
    use super::constants::*;
    format!(
        r#"#{id}{{font-family:{ff};font-size:16px;fill:#333;}}
#{id} .quadrant-point-label{{fill:{ptf};font-size:{plfs}px;}}
#{id} .quadrant-point{{fill:{pf};}}
#{id} .quadrant-title{{fill:{tf};font-size:{tfs}px;}}
#{id} .quadrant-xlabel{{fill:{xf};font-size:{xlfs}px;}}
#{id} .quadrant-ylabel{{fill:{yf};font-size:{ylfs}px;}}
"#,
        id = id,
        ff = ff,
        ptf = QUADRANT_POINT_TEXT_FILL,
        plfs = POINT_LABEL_FONT_SIZE,
        pf = QUADRANT_POINT_FILL,
        tf = QUADRANT_TITLE_FILL,
        tfs = TITLE_FONT_SIZE,
        xf = QUADRANT_X_AXIS_TEXT_FILL,
        xlfs = X_AXIS_LABEL_FONT_SIZE,
        yf = QUADRANT_Y_AXIS_TEXT_FILL,
        ylfs = Y_AXIS_LABEL_FONT_SIZE,
    )
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element for a quadrant chart.
pub fn svg_root(id: &str, w: &str, h: &str) -> String {
    format!(
        r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" style="max-width:{w}px;" role="graphics-document document" aria-roledescription="quadrantChart">"#,
    )
}

// ---------------------------------------------------------------------------
// Quadrant backgrounds
// ---------------------------------------------------------------------------

/// Render a quadrant background rect with its label text.
pub fn quadrant_group(x: &str, y: &str, w: &str, h: &str, fill: &str, text_svg: &str) -> String {
    format!(
        r#"<g class="quadrant"><rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}"></rect>{text_svg}</g>"#,
    )
}

// ---------------------------------------------------------------------------
// Border lines
// ---------------------------------------------------------------------------

/// Render a border `<line>` element.
pub fn border_line(x1: &str, y1: &str, x2: &str, y2: &str, sc: &str, sw: &str) -> String {
    format!(
        r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" style="stroke: {sc}; stroke-width: {sw};"></line>"#,
    )
}

// ---------------------------------------------------------------------------
// Data points
// ---------------------------------------------------------------------------

/// Render a data point group (circle + label).
pub fn data_point_group(
    cx: &str,
    cy: &str,
    r: &str,
    fill: &str,
    sc: &str,
    sw: &str,
    label_svg: &str,
) -> String {
    format!(
        r#"<g class="data-point"><circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" stroke="{sc}" stroke-width="{sw}"></circle>{label_svg}</g>"#,
    )
}

// ---------------------------------------------------------------------------
// Text elements
// ---------------------------------------------------------------------------

/// Render a text element with optional rotation transform.
pub fn text_el(fill: &str, fs: &str, db: &str, ta: &str, transform: &str, text: &str) -> String {
    format!(
        r#"<text x="0" y="0" fill="{fill}" font-size="{fs}" dominant-baseline="{db}" text-anchor="{ta}" transform="{transform}">{text}</text>"#,
    )
}

/// Render a label group wrapping a text element.
pub fn label_group(text_svg: &str) -> String {
    format!(r#"<g class="label">{text_svg}</g>"#)
}
