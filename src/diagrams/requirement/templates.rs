//! SVG template functions for the requirement renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element for a requirement diagram.
pub fn svg_root(sid: &str, gw: f64, gh: f64) -> String {
    format!(
        "<svg id=\"{sid}\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" class=\"requirementDiagram\" style=\"max-width:{gw:.3}px;\" viewBox=\"0 0 {gw:.3} {gh:.3}\">",
    )
}

/// Render an empty requirement SVG placeholder.
pub fn empty_svg(sid: &str) -> String {
    format!(
        r##"<svg id="{sid}" xmlns="http://www.w3.org/2000/svg" width="100%" viewBox="0 0 100 50"></svg>"##,
    )
}

// ---------------------------------------------------------------------------
// Markers (arrow-end and contains-start)
// ---------------------------------------------------------------------------

/// Render the open-chevron arrow-end marker definition.
pub fn marker_arrow_end(sid: &str, rel_color: &str) -> String {
    format!(
        "<defs><marker id=\"{sid}_req_arrow_end\" refX=\"20\" refY=\"10\" markerWidth=\"20\" markerHeight=\"20\" markerUnits=\"userSpaceOnUse\" orient=\"auto\"><path d=\"M0,0 L20,10 M20,10 L0,20\" stroke=\"{rel_color}\" fill=\"none\" stroke-width=\"1\"/></marker></defs>",
    )
}

/// Render the circle-crosshair contains-start marker definition.
pub fn marker_contains_start(sid: &str, rel_color: &str) -> String {
    format!(
        "<defs><marker id=\"{sid}_req_contains_start\" refX=\"0\" refY=\"10\" markerWidth=\"20\" markerHeight=\"20\" markerUnits=\"userSpaceOnUse\" orient=\"auto\"><g><circle cx=\"10\" cy=\"10\" r=\"9\" stroke=\"{rel_color}\" stroke-width=\"1\" fill=\"none\"/><line x1=\"1\" x2=\"19\" y1=\"10\" y2=\"10\" stroke=\"{rel_color}\" stroke-width=\"1\"/><line y1=\"1\" y2=\"19\" x1=\"10\" x2=\"10\" stroke=\"{rel_color}\" stroke-width=\"1\"/></g></marker></defs>",
    )
}

// ---------------------------------------------------------------------------
// Node boxes
// ---------------------------------------------------------------------------

/// Render the outer `<g>` wrapper for a requirement/element node.
pub fn node_group_open(cx: f64, cy: f64) -> String {
    format!(
        "<g class=\"node default\" data-look=\"classic\" transform=\"translate({cx:.3},{cy:.3})\">",
    )
}

/// Render the filled background path for a node box.
pub fn node_box_path(l: f64, t: f64, r: f64, b: f64, stroke: &str, fill: &str) -> String {
    format!(
        "<g class=\"basic label-container outer-path\"><path d=\"M{l:.3} {t:.3} L{r:.3} {t:.3} L{r:.3} {b:.3} L{l:.3} {b:.3} Z\" stroke=\"{stroke}\" stroke-width=\"1.3\" fill=\"{fill}\"/></g>",
    )
}

/// Render the horizontal divider line separating header from body.
pub fn node_divider(l: f64, r: f64, y: f64, stroke: &str) -> String {
    format!(
        "<g class=\"divider\"><path d=\"M{l:.3} {y:.3} L{r:.3} {y:.3}\" stroke=\"{stroke}\" stroke-width=\"1.3\" fill=\"none\"/></g>",
    )
}

/// Render a centered label as a plain SVG `<text>` element.
pub fn label_text(cx: f64, cy: f64, fs: f64, fill: &str, text: &str) -> String {
    format!(
        "<text x=\"{cx:.3}\" y=\"{cy:.3}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"{fs}\" fill=\"{fill}\">{text}</text>",
    )
}

/// Render a bold centered label as a plain SVG `<text>` element (for node names).
pub fn label_text_bold(cx: f64, cy: f64, fs: f64, fill: &str, text: &str) -> String {
    format!(
        "<text x=\"{cx:.3}\" y=\"{cy:.3}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"{fs}\" font-weight=\"bold\" fill=\"{fill}\">{text}</text>",
    )
}

/// Render a left-aligned body item label as a plain SVG `<text>` element.
pub fn label_text_body(x: f64, cy: f64, fs: f64, fill: &str, text: &str) -> String {
    format!(
        "<text x=\"{x:.3}\" y=\"{cy:.3}\" dominant-baseline=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"{fs}\" fill=\"{fill}\">{text}</text>",
    )
}

// ---------------------------------------------------------------------------
// Edges (relationships)
// ---------------------------------------------------------------------------

/// Render a relationship edge path.
pub fn relation_path(
    d: &str,
    rel_color: &str,
    dash: &str,
    marker_start: &str,
    marker_end: &str,
) -> String {
    format!(
        "<path d=\"{d}\" class=\"relationshipLine\" stroke=\"{rel_color}\" stroke-width=\"1\" fill=\"none\" stroke-dasharray=\"{dash}\"{marker_start}{marker_end}/>",
    )
}

/// Render an edge label as a plain SVG `<text>` element.
pub fn edge_label_text(
    mx: f64,
    my: f64,
    lw: f64,
    fs: f64,
    fill: &str,
    bg: &str,
    text: &str,
) -> String {
    let ox = -(lw / 2.0);
    format!(
        "<g class=\"edgeLabel\" transform=\"translate({mx:.3},{my:.3})\"><rect x=\"{ox:.3}\" y=\"-12\" width=\"{lw:.3}\" height=\"24\" fill=\"{bg}\" stroke=\"none\"/><text x=\"0\" y=\"0\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"{fs}\" fill=\"{fill}\">{text}</text></g>",
    )
}

/// Render the `marker-end` attribute string for a relationship edge.
pub fn marker_end_attr(sid: &str) -> String {
    format!(" marker-end=\"url(#{sid}_req_arrow_end)\"")
}

/// Render the `marker-start` attribute string for a contains edge.
pub fn marker_start_attr(sid: &str) -> String {
    format!(" marker-start=\"url(#{sid}_req_contains_start)\"")
}
