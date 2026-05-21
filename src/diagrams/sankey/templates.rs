//! SVG template functions for the sankey renderer.
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

/// Render the outer SVG wrapper for an empty (no-nodes) sankey diagram.
pub fn svg_empty(id: &str, width: f64, height: f64) -> String {
    format!(
        r##"<svg id="{id}" xmlns="http://www.w3.org/2000/svg" width="100%" viewBox="0 0 {w} {h}" style="max-width:{w}px;"></svg>"##,
        id = id,
        w = width,
        h = height
    )
}

/// Render the outer SVG root element for a populated sankey diagram.
pub fn svg_root(id: &str, width: f64, height: f64) -> String {
    format!(
        r##"<svg id="{id}" xmlns="http://www.w3.org/2000/svg" width="100%" viewBox="0 0 {w} {h}" style="max-width:{w}px;" role="graphics-document document" aria-roledescription="sankey">"##,
        id = id,
        w = width,
        h = height,
    )
}

// ---------------------------------------------------------------------------
// Node rendering
// ---------------------------------------------------------------------------

/// Render the opening `<g>` wrapper for a single sankey node.
pub fn node_group(i: usize, x0: f64, y0: f64) -> String {
    format!(
        r##"<g class="node" id="node-{i}" transform="translate({x0:.2},{y0:.2})" x="{x0:.2}" y="{y0:.2}">"##,
        i = i,
        x0 = x0,
        y0 = y0,
    )
}

/// Render the filled rectangle inside a sankey node group.
pub fn node_rect(height: f64, width: f64, color: &str) -> String {
    format!(
        r##"<rect height="{h:.2}" width="{w:.2}" fill="{color}" shape-rendering="crispEdges"/>"##,
        h = height,
        w = width,
        color = color,
    )
}

// ---------------------------------------------------------------------------
// Label rendering
// ---------------------------------------------------------------------------

/// Render a node label `<text>` element.
pub fn node_label_text(
    lx: f64,
    ly: f64,
    dy: &str,
    anchor: &str,
    font_family: &str,
    text_color: &str,
    content: &str,
) -> String {
    format!(
        "<text x=\"{lx:.2}\" y=\"{ly:.2}\" dy=\"{dy}\" text-anchor=\"{anchor}\" font-family=\"{ff}\" font-size=\"14px\" fill=\"{tc}\">{content}</text>",
        lx = lx,
        ly = ly,
        dy = dy,
        anchor = anchor,
        ff = font_family,
        tc = text_color,
        content = content,
    )
}

// ---------------------------------------------------------------------------
// Gradient defs
// ---------------------------------------------------------------------------

/// Render a linear gradient `<linearGradient>` definition for a link.
pub fn linear_gradient(li: usize, x1: f64, x2: f64, src_color: &str, tgt_color: &str) -> String {
    format!(
        r##"<linearGradient id="lg-{li}" gradientUnits="userSpaceOnUse" x1="{x1:.2}" x2="{x2:.2}"><stop offset="0%" stop-color="{sc}"/><stop offset="100%" stop-color="{tc}"/></linearGradient>"##,
        li = li,
        x1 = x1,
        x2 = x2,
        sc = src_color,
        tc = tgt_color,
    )
}

// ---------------------------------------------------------------------------
// Link rendering
// ---------------------------------------------------------------------------

/// Render a single sankey link `<path>` wrapped in a mix-blend-mode group.
pub fn link_path(path_d: &str, stroke: &str, stroke_width: f64) -> String {
    format!(
        r##"<g class="link" style="mix-blend-mode:multiply;"><path d="{d}" fill="none" stroke="{stroke}" stroke-width="{w:.2}" stroke-opacity="0.5"/></g>"##,
        d = path_d,
        stroke = stroke,
        w = stroke_width,
    )
}

/// Render the opening `<g>` tag for the node-labels group (includes font-size attribute).
pub fn node_labels_group_open(font_size: &str) -> String {
    format!(
        r##"<g class="node-labels" font-size="{font_size}">"##,
        font_size = font_size,
    )
}
