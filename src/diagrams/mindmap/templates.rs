//! SVG template functions for the mindmap renderer.
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

/// Render the outer `<svg>` element for a mindmap diagram.
pub fn svg_root(id: &str, w: f64, h: f64) -> String {
    format!(
        "<svg id=\"{id}\" xmlns=\"http://www.w3.org/2000/svg\" width=\"100%\" viewBox=\"0 0 {w:.2} {h:.2}\" style=\"max-width:{w:.2}px;\" role=\"graphics-document document\" aria-roledescription=\"mindmap\">",
    )
}

/// Render an empty mindmap SVG placeholder.
pub fn empty_svg() -> &'static str {
    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\"></svg>"
}

// ---------------------------------------------------------------------------
// Node shapes
// ---------------------------------------------------------------------------

/// Render a circle node shape.
pub fn node_circle(cx: f64, cy: f64, r: f64, fill: &str) -> String {
    format!("<circle cx=\"{cx:.2}\" cy=\"{cy:.2}\" r=\"{r:.2}\" fill=\"{fill}\" stroke=\"none\"/>",)
}

/// Render a rectangular node shape with a bottom accent line.
#[allow(clippy::too_many_arguments)]
pub fn node_rect_with_line(
    cx: f64,
    _cy: f64,
    half_w: f64,
    _hh: f64,
    _rx: f64,
    _node_width: f64,
    fill: &str,
    line_color: &str,
    path_d: &str,
    line_y: f64,
) -> String {
    format!(
        "<path d=\"{path_d}\" fill=\"{fill}\" stroke=\"none\"/><line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{line_color}\" stroke-width=\"3\"/>",
        cx - half_w, line_y, cx + half_w, line_y,
    )
}

/// Render a mindmap edge (cubic Bézier path) between parent and child.
pub fn edge(px: f64, py: f64, mid_x: f64, cx: f64, cy: f64, color: &str) -> String {
    format!(
        "<path d=\"M{px:.2},{py:.2} C{mid_x:.2},{py:.2} {mid_x:.2},{cy:.2} {cx:.2},{cy:.2}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"1.5\" class=\"mindmap-edge\"/>",
    )
}

// ---------------------------------------------------------------------------
// Node group
// ---------------------------------------------------------------------------

/// Render the opening `<g>` tag for a mindmap node (section class + id).
pub fn node_group_open(section_class: &str, node_id: usize) -> String {
    format!(
        r##"<g class="mindmap-node {section_class}" id="node_{node_id}">"##,
        section_class = section_class,
        node_id = node_id,
    )
}

// ---------------------------------------------------------------------------
// Labels
// ---------------------------------------------------------------------------

/// Render a node text label.
pub fn node_label(cx: f64, cy: f64, ff: &str, font_size: f64, color: &str, text: &str) -> String {
    format!(
        "<text x=\"{cx:.2}\" y=\"{cy:.2}\" text-anchor=\"middle\" dominant-baseline=\"central\" font-family=\"{ff}\" font-size=\"{font_size:.0}px\" fill=\"{color}\" class=\"mindmap-node-label\">{text}</text>",
    )
}
