//! SVG template functions for the wardley renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::esc;

/// Format a float for SVG output — up to 10 significant digits but no trailing zeros.
pub fn fmt_f(v: f64) -> String {
    // Round to 10 decimal places to avoid floating point noise, strip trailing zeros
    let s = format!("{:.10}", v);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    if s.is_empty() || s == "-" {
        return "0".to_string();
    }
    s.to_string()
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element.
pub fn svg_root(width: f64, height: f64) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" viewBox=\"0 0 {width} {height}\" width=\"100%\" style=\"max-width: {width}px;\">",
    )
}

// ---------------------------------------------------------------------------
// Defs / markers
// ---------------------------------------------------------------------------

/// Render the `<defs>` block containing all arrow markers.
pub fn defs_block(svg_id: &str) -> String {
    format!(
        "<defs>\
<marker id=\"arrow-{svg_id}\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"6\" markerHeight=\"6\" orient=\"auto-start-reverse\">\
<path d=\"M 0 0 L 10 5 L 0 10 z\" fill=\"#dc3545\" stroke=\"none\"></path></marker>\
<marker id=\"link-arrow-end-{svg_id}\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"5\" markerHeight=\"5\" orient=\"auto\">\
<path d=\"M 0 0 L 10 5 L 0 10 z\" fill=\"#333333\" stroke=\"none\"></path></marker>\
<marker id=\"link-arrow-start-{svg_id}\" viewBox=\"0 0 10 10\" refX=\"1\" refY=\"5\" markerWidth=\"5\" markerHeight=\"5\" orient=\"auto\">\
<path d=\"M 10 0 L 0 5 L 10 10 z\" fill=\"#333333\" stroke=\"none\"></path></marker>\
</defs>",
    )
}

// ---------------------------------------------------------------------------
// Background
// ---------------------------------------------------------------------------

/// Render the background `<rect>` filling the entire canvas.
pub fn background_rect(width: f64, height: f64) -> String {
    format!(
        "<rect class=\"wardley-background\" width=\"{width}\" height=\"{height}\" fill=\"white\"></rect>",
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element.
pub fn title_text(x: f64, y: f64, font_size: f64, text: &str) -> String {
    format!(
        "<text class=\"wardley-title\" x=\"{x}\" y=\"{y}\" fill=\"#131300\" font-size=\"{font_size}\" font-weight=\"bold\" text-anchor=\"middle\" dominant-baseline=\"middle\">{text}</text>",
    )
}

// ---------------------------------------------------------------------------
// Axes
// ---------------------------------------------------------------------------

/// Render a single axis `<line>` (horizontal or vertical).
pub fn axis_line(x1: f64, x2: f64, y1: f64, y2: f64) -> String {
    format!(
        "<line x1=\"{x1}\" x2=\"{x2}\" y1=\"{y1}\" y2=\"{y2}\" stroke=\"#333333\" stroke-width=\"1\"></line>",
    )
}

/// Render the X-axis "Evolution" label.
pub fn axis_label_x(x: f64, y: f64, font_size: f64) -> String {
    format!(
        "<text class=\"wardley-axis-label wardley-axis-label-x\" x=\"{x}\" y=\"{y}\" fill=\"#131300\" font-size=\"{font_size}\" font-weight=\"bold\" text-anchor=\"middle\">Evolution</text>",
    )
}

/// Render the Y-axis "Visibility" label (rotated).
pub fn axis_label_y(x: f64, y: f64, font_size: f64) -> String {
    format!(
        "<text class=\"wardley-axis-label wardley-axis-label-y\" x=\"{x}\" y=\"{y}\" fill=\"#131300\" font-size=\"{font_size}\" font-weight=\"bold\" text-anchor=\"middle\" transform=\"rotate(-90 {x} {y})\">Visibility</text>",
    )
}

// ---------------------------------------------------------------------------
// Evolution stages
// ---------------------------------------------------------------------------

/// Render a vertical dashed stage boundary `<line>`.
pub fn stage_line(x: f64, y_top: f64, y_bottom: f64) -> String {
    format!(
        "<line x1=\"{x}\" x2=\"{x}\" y1=\"{y_top}\" y2=\"{y_bottom}\" stroke=\"#000\" stroke-width=\"1\" stroke-dasharray=\"5 5\" opacity=\"0.8\"></line>",
    )
}

/// Render a stage name `<text>` label below the X axis.
pub fn stage_label(x: f64, y: f64, font_size: f64, text: &str) -> String {
    format!(
        "<text class=\"wardley-stage-label\" x=\"{x}\" y=\"{y}\" fill=\"#131300\" font-size=\"{font_size}\" text-anchor=\"middle\">{text}</text>",
    )
}

// ---------------------------------------------------------------------------
// Links
// ---------------------------------------------------------------------------

/// Render a link `<line>` between two components (solid or dashed).
pub fn link_line(x1: &str, y1: &str, x2: &str, y2: &str, dash_attr: &str) -> String {
    format!(
        "<line class=\"wardley-link\" x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"#333333\" stroke-width=\"1\"{dash_attr}></line>",
    )
}

/// Render a link label `<text>` element at the midpoint of the link.
pub fn link_label(x: &str, y: &str, font_size: f64, text: &str) -> String {
    format!(
        "<text class=\"wardley-link-label\" x=\"{x}\" y=\"{y}\" fill=\"#131300\" font-size=\"{font_size}\" text-anchor=\"middle\" dominant-baseline=\"middle\">{text}</text>",
    )
}

// ---------------------------------------------------------------------------
// Trend (evolve) arrows
// ---------------------------------------------------------------------------

/// Render an evolve trend `<line>` with a red arrowhead.
pub fn trend_arrow(x1: &str, y1: &str, x2: &str, y2: &str, svg_id: &str) -> String {
    format!(
        "<line class=\"wardley-trend\" x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"#dc3545\" stroke-width=\"1\" stroke-dasharray=\"4 4\" marker-end=\"url(#arrow-{svg_id})\"></line>",
    )
}

// ---------------------------------------------------------------------------
// Node group
// ---------------------------------------------------------------------------

/// Render the opening `<g>` tag for a node group.
pub fn node_group_open(class_suffix: &str) -> String {
    format!("<g class=\"wardley-node wardley-node--{class_suffix}\">")
}

// ---------------------------------------------------------------------------
// Anchor nodes
// ---------------------------------------------------------------------------

/// Render an anchor label `<text>` (bold, centered, no circle).
pub fn anchor_label(x: &str, y: &str, font_size: f64, text: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" class=\"wardley-node-label\" fill=\"#000\" font-size=\"{font_size}\" font-weight=\"bold\" text-anchor=\"middle\" dominant-baseline=\"middle\">{text}</text>",
    )
}

// ---------------------------------------------------------------------------
// Note nodes
// ---------------------------------------------------------------------------

/// Render a note `<text>` element.
pub fn note_text(x: &str, y: &str, text: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" text-anchor=\"start\" font-size=\"11\" fill=\"#131300\" font-weight=\"bold\">{text}</text>",
    )
}

// ---------------------------------------------------------------------------
// Component nodes
// ---------------------------------------------------------------------------

/// Render a sourcing overlay circle (outsource, buy, or build).
pub fn sourcing_overlay_circle(
    class_name: &str,
    cx: &str,
    cy: &str,
    r: f64,
    fill: &str,
    stroke: &str,
) -> String {
    format!(
        "<circle class=\"{class_name}\" cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"1\"></circle>",
    )
}

/// Render the main component circle (white fill, grey stroke).
pub fn component_circle(cx: &str, cy: &str, r: f64) -> String {
    format!(
        "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"white\" stroke=\"#333333\" stroke-width=\"1\"></circle>",
    )
}

/// Render the inertia vertical line to the right of a component node.
pub fn inertia_line(x: &str, y1: &str, y2: &str) -> String {
    format!(
        "<line class=\"wardley-inertia\" x1=\"{x}\" y1=\"{y1}\" x2=\"{x}\" y2=\"{y2}\" stroke=\"#333333\" stroke-width=\"6\"></line>",
    )
}

/// Render a component text label.
pub fn component_label(x: &str, y: &str, font_size: f64, text: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" class=\"wardley-node-label\" fill=\"#131300\" font-size=\"{font_size}\" font-weight=\"normal\" text-anchor=\"start\" dominant-baseline=\"auto\">{text}</text>",
    )
}

// ---------------------------------------------------------------------------
// Annotations
// ---------------------------------------------------------------------------

/// Render a single annotation (numbered circle + bold text).
pub fn annotation(cx: &str, cy: &str, number: u32) -> String {
    format!(
        "<g class=\"wardley-annotation\"><circle cx=\"{cx}\" cy=\"{cy}\" r=\"10\" fill=\"white\" stroke=\"#333333\" stroke-width=\"1.5\"></circle><text x=\"{cx}\" y=\"{cy}\" text-anchor=\"middle\" dominant-baseline=\"central\" font-size=\"10\" fill=\"#131300\" font-weight=\"bold\">{number}</text></g>",
    )
}
