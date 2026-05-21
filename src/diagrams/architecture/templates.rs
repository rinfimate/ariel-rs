//! SVG template functions for the architecture renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::{esc, fmt};

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for an architecture diagram.
pub fn svg_root(max_w: f64, vb_x: f64, vb_y: f64, vb_w: f64, vb_h: f64) -> String {
    format!(
        "<svg id=\"mermaid-svg\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" font-family=\"Arial, sans-serif\" style=\"max-width: {}px;\" viewBox=\"{} {} {} {}\" role=\"graphics-document document\" aria-roledescription=\"architecture\">",
        max_w, vb_x, vb_y, vb_w, vb_h,
    )
}

// ---------------------------------------------------------------------------
// Group (container) rendering
// ---------------------------------------------------------------------------

/// Render the dashed container rectangle for a group.
pub fn group_rect(id: &str, x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        "<rect id=\"mermaid-svg-group-{id}\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"none\" stroke=\"hsl(240, 60%, 86.2745098039%)\" stroke-width=\"2\" stroke-dasharray=\"8\" class=\"node-bkg\"></rect>",
        id = id, x = x, y = y, w = w, h = h,
    )
}

/// Render the icon + label group for a group that has an icon.
pub fn group_icon_and_label(
    icon_x: f64,
    icon_y: f64,
    inner: &str,
    label_tx: f64,
    label_ty: f64,
    text: &str,
    text_color: &str,
) -> String {
    format!(
        "<g><g transform=\"translate({ix}, {iy})\"><g><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"30\" height=\"30\" viewBox=\"0 0 80 80\"><g>{inner}</g></svg></g></g><g dy=\"1em\" alignment-baseline=\"middle\" dominant-baseline=\"start\" text-anchor=\"start\" transform=\"translate({tx}, {ty})\"><g><rect class=\"background\" style=\"stroke: none\"></rect><text y=\"-10.1\" style=\"fill:{tc};\"><tspan class=\"text-outer-tspan row\" x=\"0\" y=\"-0.1em\" dy=\"1.1em\"><tspan font-style=\"normal\" class=\"text-inner-tspan\" font-weight=\"normal\">{text}</tspan></tspan></text></g></g></g>",
        ix = icon_x, iy = icon_y, inner = inner,
        tx = label_tx, ty = label_ty, text = text, tc = text_color,
    )
}

/// Render the plain text label for a group that has no icon.
pub fn group_plain_label(tx: f64, ty: f64, text: &str, text_color: &str) -> String {
    format!(
        "<g dy=\"1em\" alignment-baseline=\"middle\" dominant-baseline=\"start\" text-anchor=\"start\" transform=\"translate({tx}, {ty})\"><g><rect class=\"background\" style=\"stroke: none\"></rect><text y=\"-10.1\" style=\"fill:{tc};\"><tspan class=\"text-outer-tspan row\" x=\"0\" y=\"-0.1em\" dy=\"1.1em\"><tspan font-style=\"normal\" class=\"text-inner-tspan\" font-weight=\"normal\">{text}</tspan></tspan></text></g></g>",
        tx = tx, ty = ty, text = text, tc = text_color,
    )
}

// ---------------------------------------------------------------------------
// Service rendering
// ---------------------------------------------------------------------------

/// Render the opening `<g>` wrapper for an architecture service node.
pub fn service_group(id: &str, tx: f64, ty: f64) -> String {
    format!(
        "<g id=\"mermaid-svg-service-{id}\" class=\"architecture-service\" transform=\"translate({tx},{ty})\">",
        id = id, tx = tx, ty = ty,
    )
}

/// Render the label group below a service icon (in service-local coordinates).
pub fn service_label(text: &str, text_color: &str) -> String {
    format!(
        "<g dy=\"1em\" alignment-baseline=\"middle\" dominant-baseline=\"middle\" text-anchor=\"middle\" transform=\"translate(40, 80)\"><g><rect class=\"background\" style=\"stroke: none\"></rect><text y=\"-10.1\" style=\"fill:{text_color};\"><tspan class=\"text-outer-tspan row\" x=\"0\" y=\"-0.1em\" dy=\"1.1em\"><tspan font-style=\"normal\" class=\"text-inner-tspan\" font-weight=\"normal\">{text}</tspan></tspan></text></g></g>",
        text = text,
        text_color = text_color,
    )
}

/// Render an 80×80 service icon SVG wrapping the given inner art.
pub fn service_icon_svg(inner: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"80\" height=\"80\" viewBox=\"0 0 80 80\"><g>{inner}</g></svg>",
        inner = inner,
    )
}

// ---------------------------------------------------------------------------
// Edge rendering
// ---------------------------------------------------------------------------

/// Render an architecture edge `<path>` element.
pub fn edge_path(path: &str, lhs: &str, rhs: &str, line_color: &str) -> String {
    format!(
        "<path d=\"{path}\" stroke-width=\"3\" stroke=\"{line_color}\" fill=\"none\" class=\"edge\" id=\"mermaid-svg-L_{lhs}_{rhs}_0\"></path>",
        path = path,
        lhs = lhs,
        rhs = rhs,
        line_color = line_color,
    )
}

/// Render an arrowhead `<polygon>` for an architecture edge.
pub fn edge_arrow(pts: &str, t: &str, line_color: &str) -> String {
    format!(
        "<polygon points=\"{pts}\" transform=\"{t}\" fill=\"{line_color}\" class=\"arrow\"></polygon>",
        pts = pts,
        t = t,
        line_color = line_color,
    )
}

/// Render an edge label `<text>` element.
pub fn edge_label(x: f64, y: f64, text: &str, line_color: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"14\" fill=\"{line_color}\">{text}</text>",
        x = x,
        y = y,
        text = text,
        line_color = line_color,
    )
}
