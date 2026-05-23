//! SVG template functions for the timeline renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::esc;

// ---------------------------------------------------------------------------
// Arrowhead marker
// ---------------------------------------------------------------------------

/// Render the arrowhead `<marker>` and drop-shadow `<filter>` definitions.
pub fn arrowhead_marker(id: &str, line_color: &str, shadow_color: &str) -> String {
    format!(
        "<defs>\n  <marker id=\"{id}-arrowhead\" refX=\"5\" refY=\"2\" markerWidth=\"6\" markerHeight=\"4\" orient=\"auto\">\n    <path d=\"M 0,0 V 4 L6,2 Z\" fill=\"{line_color}\"></path>\n  </marker>\n  <filter id=\"{id}-dropshadow\" x=\"-20%\" y=\"-20%\" width=\"140%\" height=\"140%\">\n    <feDropShadow dx=\"1\" dy=\"2\" stdDeviation=\"2\" flood-color=\"{shadow_color}\"/>\n  </filter>\n</defs>",
        id = id,
        line_color = line_color,
        shadow_color = shadow_color,
    )
}

// ---------------------------------------------------------------------------
// Activity line
// ---------------------------------------------------------------------------

/// Render the horizontal activity line (timeline spine) with arrowhead.
pub fn activity_line(x1: f64, y: f64, x2: f64, diagram_id: &str, line_color: &str) -> String {
    format!(
        "<g class=\"lineWrapper\">\n  <line x1=\"{x1}\" y1=\"{y}\" x2=\"{x2}\" y2=\"{y}\" style=\"stroke:{line_color};stroke-width:4;\" marker-end=\"url(#{id}-arrowhead)\"></line>\n</g>",
        x1 = x1,
        y = y,
        x2 = x2,
        id = diagram_id,
        line_color = line_color,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element.
pub fn title_text(x: f64, title: &str, title_color: &str) -> String {
    format!(
        "<text x=\"{x}\" font-size=\"4ex\" font-weight=\"bold\" y=\"20\" fill=\"{title_color}\">{t}</text>",
        x = x,
        t = title,
        title_color = title_color,
    )
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper.
pub fn svg_root(id: &str, max_w: f64, vb_x: f64, vb_y: f64, vb_w: f64, vb_h: f64) -> String {
    format!(
        "<svg id=\"{id}\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" font-family=\"Arial, sans-serif\" style=\"max-width: {mw}px;\" viewBox=\"{vx} {vy} {vw} {vh}\" role=\"graphics-document document\" aria-roledescription=\"timeline\"><g></g><g></g>",
        id = id,
        mw = max_w,
        vx = vb_x,
        vy = vb_y,
        vw = vb_w,
        vh = vb_h,
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
pub fn connector_line(x: f64, y1: f64, y2: f64, diagram_id: &str, line_color: &str) -> String {
    format!(
        "<g class=\"lineWrapper\"><line x1=\"{x}\" y1=\"{y1}\" x2=\"{x}\" y2=\"{y2}\" style=\"stroke:{line_color};stroke-width:2;stroke-dasharray:5,5;\" marker-end=\"url(#{id}-arrowhead)\"></line></g>",
        x = x,
        y1 = y1,
        y2 = y2,
        id = diagram_id,
        line_color = line_color,
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
pub fn node_bg_path(id_val: usize, path_d: &str, fill: &str, shadow_filter: &str) -> String {
    format!(
        "    <path id=\"node-{id}\" class=\"node-bkg node-undefined\" fill=\"{fill}\" filter=\"{shadow_filter}\" d=\"{path}\"></path>\n",
        id = id_val,
        fill = fill,
        shadow_filter = shadow_filter,
        path = path_d,
    )
}

/// Render the bottom separator line for a timeline node.
pub fn node_separator_line(section_class: i64, height: f64, width: f64, stroke: &str) -> String {
    format!(
        "    <line class=\"node-line-{sc}\" x1=\"0\" y1=\"{h}\" x2=\"{w}\" y2=\"{h}\" stroke=\"{stroke}\" stroke-width=\"3\"></line>\n",
        sc = section_class,
        h = height,
        w = width,
        stroke = stroke,
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

/// Render a single `<tspan>` for a multi-line text element.
pub fn text_tspan(dy: &str, text: &str) -> String {
    format!(
        "<tspan x=\"0\" dy=\"{dy}\">{text}</tspan>",
        dy = dy,
        text = text,
    )
}

/// Render a `<text>` element wrapping pre-built `<tspan>` strings.
/// Used by `build_tspans` to produce the outer text element for timeline nodes.
/// `fill` sets the text color (e.g. `"#ffffff"` for white sections, `"black"` for light sections).
pub fn node_text_element(tspans: &str, fill: &str) -> String {
    format!(
        r##"<text dy="1em" alignment-baseline="middle" dominant-baseline="middle" text-anchor="middle" fill="{fill}">{tspans}</text>"##,
        tspans = tspans,
        fill = fill,
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
    _text_y: f64,
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
    // Match Mermaid reference: text group at (cx, 10), dy="1em" + tspan dy="1em"
    svg.push_str(&format!(
        "  <g transform=\"translate({tx}, 10)\"><text dy=\"1em\" alignment-baseline=\"middle\" dominant-baseline=\"middle\" text-anchor=\"middle\" style=\"fill:{fc};\" class=\"section-label\"><tspan x=\"0\" dy=\"1em\">{label}</tspan></text></g>\n",
        tx = text_x,
        fc = text_color,
        label = label,
    ));
    svg.push_str("</g>");
    svg
}
