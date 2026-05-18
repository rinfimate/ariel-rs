//! SVG template functions for the timeline renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Arrowhead marker
// ---------------------------------------------------------------------------

/// Render the arrowhead `<marker>` definition used by the timeline activity line.
pub fn arrowhead_marker(id: &str) -> String {
    format!(
        "<defs>\n  <marker id=\"{id}-arrowhead\" refX=\"5\" refY=\"2\" markerWidth=\"6\" markerHeight=\"4\" orient=\"auto\">\n    <path d=\"M 0,0 V 4 L6,2 Z\"></path>\n  </marker>\n</defs>",
        id = id,
    )
}

// ---------------------------------------------------------------------------
// Activity line
// ---------------------------------------------------------------------------

/// Render the horizontal activity line (timeline spine) with arrowhead.
pub fn activity_line(x1: f64, y: f64, x2: f64, diagram_id: &str) -> String {
    format!(
        "<g class=\"lineWrapper\">\n  <line x1=\"{x1}\" y1=\"{y}\" x2=\"{x2}\" y2=\"{y}\" style=\"stroke:black;stroke-width:4;\" marker-end=\"url(#{id}-arrowhead)\"></line>\n</g>",
        x1 = x1,
        y = y,
        x2 = x2,
        id = diagram_id,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element.
pub fn title_text(x: f64, title: &str) -> String {
    format!(
        "<text x=\"{x}\" font-size=\"4ex\" font-weight=\"bold\" y=\"20\">{t}</text>",
        x = x,
        t = title,
    )
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper including embedded style block.
pub fn svg_root(
    id: &str,
    max_w: f64,
    vb_x: f64,
    vb_y: f64,
    vb_w: f64,
    vb_h: f64,
    style: &str,
) -> String {
    format!(
        "<svg id=\"{id}\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" style=\"max-width: {mw}px;\" viewBox=\"{vx} {vy} {vw} {vh}\" role=\"graphics-document document\" aria-roledescription=\"timeline\"><style>{style}</style><g></g><g></g>",
        id = id,
        mw = max_w,
        vx = vb_x,
        vy = vb_y,
        vw = vb_w,
        vh = vb_h,
        style = style,
    )
}

// ---------------------------------------------------------------------------
// Task / event wrappers
// ---------------------------------------------------------------------------

/// Render the wrapper `<g>` that positions a task node.
pub fn task_wrapper(x: f64, y: f64, svg: &str) -> String {
    format!(
        "<g class=\"taskWrapper\" transform=\"translate({x}, {y})\">{svg}</g>",
        x = x,
        y = y,
        svg = svg,
    )
}

/// Render the wrapper `<g>` that positions an event node.
pub fn event_wrapper(x: f64, y: f64, svg: &str) -> String {
    format!(
        "<g class=\"eventWrapper\" transform=\"translate({x}, {y})\">{svg}</g>",
        x = x,
        y = y,
        svg = svg,
    )
}

/// Render the connector (dashed vertical) line between a task and its events.
pub fn connector_line(x: f64, y1: f64, y2: f64, diagram_id: &str) -> String {
    format!(
        "<g class=\"lineWrapper\"><line x1=\"{x}\" y1=\"{y1}\" x2=\"{x}\" y2=\"{y2}\" style=\"stroke:black;stroke-width:2;stroke-dasharray:5,5;\" marker-end=\"url(#{id}-arrowhead)\"></line></g>",
        x = x,
        y1 = y1,
        y2 = y2,
        id = diagram_id,
    )
}

// ---------------------------------------------------------------------------
// Node elements
// ---------------------------------------------------------------------------

/// Render the opening `<g>` for a timeline node at a given section class.
pub fn node_group_open(section_class: i64) -> String {
    format!(
        "<g class=\"timeline-node section-{sc}\">\n",
        sc = section_class,
    )
}

/// Render the background path for a timeline node.
pub fn node_bg_path(id_val: usize, path_d: &str) -> String {
    format!(
        "    <path id=\"node-{id}\" class=\"node-bkg node-undefined\" d=\"{path}\"></path>\n",
        id = id_val,
        path = path_d,
    )
}

/// Render the bottom separator line for a timeline node.
pub fn node_separator_line(section_class: i64, height: f64, width: f64) -> String {
    format!(
        "    <line class=\"node-line-{sc}\" x1=\"0\" y1=\"{h}\" x2=\"{w}\" y2=\"{h}\"></line>\n",
        sc = section_class,
        h = height,
        w = width,
    )
}

/// Render the text content `<g>` for a timeline node.
pub fn node_text_group(tx: f64, ty: f64, tspans: &str) -> String {
    format!(
        "  <g transform=\"translate({tx}, {ty})\">{tspans}</g>\n",
        tx = tx,
        ty = ty,
        tspans = tspans,
    )
}

// ---------------------------------------------------------------------------
// Section box
// ---------------------------------------------------------------------------

/// Render the section header `<g>` with path, line and label.
#[allow(clippy::too_many_arguments)]
pub fn section_box(
    x: f64,
    y: f64,
    path: &str,
    color: &str,
    line_class: i64,
    height: f64,
    width: f64,
    line_color: &str,
    text_x: f64,
    text_y: f64,
    text_color: &str,
    label: &str,
) -> String {
    let mut svg = String::new();
    svg.push_str(&format!(
        "<g transform=\"translate({x}, {y})\">\n",
        x = x,
        y = y,
    ));
    svg.push_str(&format!(
        "  <path d=\"{path}\" style=\"fill:{color};stroke:{color};stroke-width:2px;\"/>\n",
        path = path,
        color = color,
    ));
    svg.push_str(&format!(
        "  <line class=\"node-line-{cls}\" x1=\"0\" y1=\"{y}\" x2=\"{w}\" y2=\"{y}\" style=\"stroke:{lc};stroke-width:3;\"/>\n",
        cls = line_class,
        y = height,
        w = width,
        lc = line_color,
    ));
    svg.push_str(&format!(
        "  <text x=\"{tx}\" y=\"{ty}\" class=\"section-label\" style=\"fill:{fc};\" text-anchor=\"middle\" dominant-baseline=\"middle\">{label}</text>\n",
        tx = text_x,
        ty = text_y,
        fc = text_color,
        label = label,
    ));
    svg.push_str("</g>");
    svg
}
