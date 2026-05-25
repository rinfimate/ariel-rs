//! SVG template functions for the treeView renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::esc;

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG element for a treeView diagram.
///
/// `max_width` is the CSS `max-width` value (px), `vb_x/vb_y` are the viewBox
/// origin, and `vb_w/vb_h` are the viewBox dimensions.
#[allow(clippy::too_many_arguments)]
pub fn svg_root(
    svg_id: &str,
    max_width: f64,
    vb_x: f64,
    vb_y: f64,
    vb_w: f64,
    vb_h: f64,
    ff: &str,
) -> String {
    format!(
        concat!(
            r##"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" "##,
            r##"xmlns:xlink="http://www.w3.org/1999/xlink" "##,
            r##"viewBox="{vx} {vy} {vw} {vh}" "##,
            r##"style="max-width: {mw}px;" "##,
            r##"font-family="{ff}" font-size="16px" "##,
            r##"role="graphics-document document" "##,
            r##"aria-roledescription="treeView">"##,
        ),
        id = svg_id,
        mw = max_width,
        vx = vb_x,
        vy = vb_y,
        vw = vb_w,
        vh = vb_h,
        ff = ff,
    )
}

// ---------------------------------------------------------------------------
// Node text
// ---------------------------------------------------------------------------

/// Render a node label `<text>` element.
pub fn node_text(x: f64, y: f64, label: &str) -> String {
    format!(
        r##"<text dominant-baseline="middle" font-size="16px" fill="black" class="treeView-node-label" x="{x}" y="{y}">{label}</text>"##,
        x = x,
        y = y,
        label = label,
    )
}

// ---------------------------------------------------------------------------
// Connector lines
// ---------------------------------------------------------------------------

/// Render a horizontal connector `<line>` element.
pub fn h_line(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    format!(
        r##"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="black" stroke-width="1" class="treeView-node-line"></line>"##,
        x1 = x1,
        y1 = y1,
        x2 = x2,
        y2 = y2,
    )
}

/// Render a vertical connector `<line>` element.
pub fn v_line(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    format!(
        r##"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="black" stroke-width="1" class="treeView-node-line"></line>"##,
        x1 = x1,
        y1 = y1,
        x2 = x2,
        y2 = y2,
    )
}
