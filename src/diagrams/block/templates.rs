//! SVG template functions for the block diagram renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::{esc, fmt};

/// Format for pixel-based max-width (Mermaid uses fractional pixels).
pub fn fmt_px(v: f64) -> String {
    let s = format!("{:.6}", v);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for a block diagram.
pub fn svg_root(
    svg_id: &str,
    max_w: &str,
    vb_x: &str,
    vb_y: &str,
    vb_w: &str,
    vb_h: &str,
) -> String {
    format!(
        "<svg id=\"{}\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" style=\"max-width: {}px;\" viewBox=\"{} {} {} {}\" role=\"graphics-document document\" aria-roledescription=\"block\">",
        svg_id, max_w, vb_x, vb_y, vb_w, vb_h,
    )
}

/// Render the CSS style block for a block diagram (font, label, node defaults).
pub fn svg_style(
    svg_id: &str,
    fill: &str,
    stroke: &str,
    text_color: &str,
    line_color: &str,
) -> String {
    format!(
        "<style>\
#{id}{{font-family:Arial,sans-serif;font-size:16px;fill:{tc};}}\
#{id} svg{{font-family:Arial,sans-serif;font-size:16px;}}\
#{id} p{{margin:0;}}\
#{id} .label{{font-family:Arial,sans-serif;color:{tc};}}\
#{id} .cluster-label text{{fill:{tc};}}\
#{id} .cluster-label span,#{id} p{{color:{tc};}}\
#{id} .label text,#{id} span,#{id} p{{fill:{tc};color:{tc};}}\
#{id} .node rect,#{id} .node circle,#{id} .node ellipse,#{id} .node polygon,#{id} .node path{{fill:{fill};stroke:{stroke};stroke-width:1px;}}\
#{id} .flowchart-label text{{text-anchor:middle;}}\
#{id} .node .label{{text-align:center;}}\
#{id} .arrowheadPath{{fill:{lc};}}\
#{id} .edgePath .path{{stroke:{lc};stroke-width:2.0px;}}\
#{id} .flowchart-link{{stroke:{lc};fill:none;}}\
</style>",
        id = svg_id,
        fill = fill,
        stroke = stroke,
        tc = text_color,
        lc = line_color,
    )
}

// ---------------------------------------------------------------------------
// Node shapes
// ---------------------------------------------------------------------------

/// Render a square/default node background `<rect>`.
pub fn node_rect_square(x: &str, y: &str, w: &str, h: &str, style: &str) -> String {
    format!(
        "<rect class=\"basic label-container\" style=\"{style}\" rx=\"0\" ry=\"0\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\"></rect>",
        x = x, y = y, w = w, h = h, style = style,
    )
}

/// Render a rounded-rectangle node background `<rect>`.
pub fn node_rect_rounded(x: &str, y: &str, w: &str, h: &str, style: &str) -> String {
    format!(
        "<rect class=\"basic label-container\" style=\"{style}\" rx=\"5\" ry=\"5\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\"></rect>",
        x = x, y = y, w = w, h = h, style = style,
    )
}

/// Render a diamond node background `<polygon>`.
pub fn node_diamond(pts: &str, style: &str) -> String {
    format!(
        "<polygon points=\"{pts}\" class=\"basic label-container\" style=\"{style}\"></polygon>",
        pts = pts,
        style = style,
    )
}

/// Render a circle node background `<circle>`.
pub fn node_circle(r: &str, style: &str) -> String {
    format!(
        "<circle cx=\"0\" cy=\"0\" r=\"{r}\" class=\"basic label-container\" style=\"{style}\"></circle>",
        r = r, style = style,
    )
}

/// Render a cylinder node body `<rect>` (below ellipse caps).
pub fn node_cylinder_rect(x: &str, y: &str, w: &str, h: &str, fill: &str, stroke: &str) -> String {
    format!(
        "<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" style=\"fill:{fill};stroke:{stroke};stroke-width:1;\"></rect>",
        x = x, y = y, w = w, h = h, fill = fill, stroke = stroke,
    )
}

/// Render a cylinder node ellipse cap.
pub fn node_cylinder_ellipse(
    cx: &str,
    cy: &str,
    rx: &str,
    ry: &str,
    fill: &str,
    stroke: &str,
) -> String {
    format!(
        "<ellipse cx=\"{cx}\" cy=\"{cy}\" rx=\"{rx}\" ry=\"{ry}\" style=\"fill:{fill};stroke:{stroke};stroke-width:1;\"></ellipse>",
        cx = cx, cy = cy, rx = rx, ry = ry, fill = fill, stroke = stroke,
    )
}

/// Render a cluster (composite) background `<rect>` inside a node group.
pub fn cluster_composite_rect(x: &str, y: &str, w: &str, h: &str, style: &str) -> String {
    format!(
        r##"<rect class="basic cluster composite label-container" style="{style}" rx="0" ry="0" x="{x}" y="{y}" width="{w}" height="{h}"></rect>"##,
    )
}

/// Render a stadium-shape rounded `<rect>` for a stadium node.
pub fn node_rect_stadium(x: &str, y: &str, w: &str, h: &str, r: &str, style: &str) -> String {
    format!(
        r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" ry="{r}" class="label-container" style="{style}"></rect>"##,
    )
}

/// Render a cylinder node `<path>` with translate transform.
pub fn node_cylinder_path(style: &str, d: &str, tx: &str, ty: &str) -> String {
    format!(r##"<path style="{style}" d="{d}" transform="translate({tx},{ty})"></path>"##,)
}

/// Render a block-arrow `<polygon>` shape.
pub fn node_block_arrow(pts: &str, style: &str) -> String {
    format!(r##"<polygon points="{pts}" class="label-container" style="{style}"></polygon>"##,)
}

/// Render a hexagon node background `<polygon>`.
pub fn node_hexagon(pts: &str, style: &str) -> String {
    format!(
        "<polygon points=\"{pts}\" class=\"basic label-container\" style=\"{style}\"></polygon>",
        pts = pts,
        style = style,
    )
}

// ---------------------------------------------------------------------------
// Node outer group
// ---------------------------------------------------------------------------

/// Render the outer `<g>` wrapper for a block node.
pub fn node_group(svg_id: &str, id: &str, cx: &str, cy: &str) -> String {
    format!(
        "<g class=\"node default default flowchart-label\" id=\"{svg_id}-{id}\" transform=\"translate({cx}, {cy})\">",
        svg_id = svg_id, id = id, cx = cx, cy = cy,
    )
}

// ---------------------------------------------------------------------------
// Node label (foreignObject)
// ---------------------------------------------------------------------------

/// Render a label group + text matching Mermaid's tspan structure for htmlLabels:false.
/// The group is offset by -8.5 (-FONT_SIZE/1.88) and text y=-10.1 + tspan dy=1.1em
/// puts the baseline at +7.5 below the group center — same as Mermaid renders.
pub fn node_label_text(label: &str, text_color: &str) -> String {
    if label.is_empty() {
        return String::new();
    }
    format!(
        "<g class=\"label\" transform=\"translate(0,-8.5)\"><text text-anchor=\"middle\" y=\"-10.1\" font-family=\"Arial,sans-serif\" font-size=\"16\" fill=\"{color}\"><tspan x=\"0\" y=\"-0.1em\" dy=\"1.1em\"><tspan>{label}</tspan></tspan></text></g>",
        color = text_color,
        label = label,
    )
}

// ---------------------------------------------------------------------------
// Edge rendering
// ---------------------------------------------------------------------------

/// Render a block diagram edge `<path>`.
pub fn edge_path(
    path: &str,
    edge_id: &str,
    from_lc: &str,
    to_lc: &str,
    marker_end: &str,
    line_color: &str,
) -> String {
    format!(
        "<path d=\"{path}\" id=\"{edge_id}\" class=\" edge-thickness-normal edge-pattern-solid edge-thickness-normal edge-pattern-solid flowchart-link LS-{from_lc} LE-{to_lc}\" stroke=\"{line_color}\" fill=\"none\" stroke-width=\"1px\" stroke-dasharray=\"0\" marker-end=\"{marker_end}\"></path>",
        path = path, edge_id = edge_id, from_lc = from_lc, to_lc = to_lc, marker_end = marker_end, line_color = line_color,
    )
}

/// Render an edge label `<text>` element.
pub fn edge_label_text(mx: &str, my: &str, label: &str, text_color: &str, ff: &str) -> String {
    format!(
        "<text x=\"{mx}\" y=\"{my}\" text-anchor=\"middle\" font-size=\"12\" font-family=\"{ff}\" fill=\"{text_color}\">{label}</text>",
        mx = mx, my = my, label = label, text_color = text_color,
    )
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

/// Render the `pointEnd` filled-triangle marker for block diagrams.
pub fn marker_point_end(svg_id: &str, color: &str) -> String {
    format!(
        "<marker id=\"{svg_id}_block-pointEnd\" class=\"marker block\" viewBox=\"0 0 10 10\" refX=\"6\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"12\" markerHeight=\"12\" orient=\"auto\"><path d=\"M 0 0 L 10 5 L 0 10 z\" class=\"arrowMarkerPath\" style=\"fill:{color};stroke:{color};stroke-width:1;stroke-dasharray:1,0;\"></path></marker>",
        svg_id = svg_id, color = color,
    )
}

/// Render the `pointStart` filled-triangle marker for block diagrams.
pub fn marker_point_start(svg_id: &str, color: &str) -> String {
    format!(
        "<marker id=\"{svg_id}_block-pointStart\" class=\"marker block\" viewBox=\"0 0 10 10\" refX=\"4.5\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"12\" markerHeight=\"12\" orient=\"auto\"><path d=\"M 0 5 L 10 10 L 10 0 z\" class=\"arrowMarkerPath\" style=\"fill:{color};stroke:{color};stroke-width:1;stroke-dasharray:1,0;\"></path></marker>",
        svg_id = svg_id, color = color,
    )
}

/// Render the `circleEnd` open-circle marker for block diagrams.
pub fn marker_circle_end(svg_id: &str, color: &str) -> String {
    format!(
        "<marker id=\"{svg_id}_block-circleEnd\" class=\"marker block\" viewBox=\"0 0 10 10\" refX=\"11\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"11\" markerHeight=\"11\" orient=\"auto\"><circle cx=\"5\" cy=\"5\" r=\"5\" class=\"arrowMarkerPath\" style=\"fill:{color};stroke:{color};stroke-width:1;stroke-dasharray:1,0;\"></circle></marker>",
        svg_id = svg_id, color = color,
    )
}

/// Render the `circleStart` open-circle marker for block diagrams.
pub fn marker_circle_start(svg_id: &str, color: &str) -> String {
    format!(
        "<marker id=\"{svg_id}_block-circleStart\" class=\"marker block\" viewBox=\"0 0 10 10\" refX=\"-1\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"11\" markerHeight=\"11\" orient=\"auto\"><circle cx=\"5\" cy=\"5\" r=\"5\" class=\"arrowMarkerPath\" style=\"fill:{color};stroke:{color};stroke-width:1;stroke-dasharray:1,0;\"></circle></marker>",
        svg_id = svg_id, color = color,
    )
}

/// Render the `crossEnd` X-cross marker for block diagrams.
pub fn marker_cross_end(svg_id: &str, color: &str) -> String {
    format!(
        "<marker id=\"{svg_id}_block-crossEnd\" class=\"marker cross block\" viewBox=\"0 0 11 11\" refX=\"12\" refY=\"5.2\" markerUnits=\"userSpaceOnUse\" markerWidth=\"11\" markerHeight=\"11\" orient=\"auto\"><path d=\"M 1,1 l 9,9 M 10,1 l -9,9\" class=\"arrowMarkerPath\" style=\"fill:none;stroke:{color};stroke-width:2;stroke-dasharray:1,0;\"></path></marker>",
        svg_id = svg_id, color = color,
    )
}

/// Render the `crossStart` X-cross marker for block diagrams.
pub fn marker_cross_start(svg_id: &str, color: &str) -> String {
    format!(
        "<marker id=\"{svg_id}_block-crossStart\" class=\"marker cross block\" viewBox=\"0 0 11 11\" refX=\"-1\" refY=\"5.2\" markerUnits=\"userSpaceOnUse\" markerWidth=\"11\" markerHeight=\"11\" orient=\"auto\"><path d=\"M 1,1 l 9,9 M 10,1 l -9,9\" class=\"arrowMarkerPath\" style=\"fill:none;stroke:{color};stroke-width:2;stroke-dasharray:1,0;\"></path></marker>",
        svg_id = svg_id, color = color,
    )
}

/// Build all marker defs for a block diagram.
pub fn build_markers(svg_id: &str, color: &str) -> String {
    let mut m = String::new();
    m.push_str(&marker_point_end(svg_id, color));
    m.push_str(&marker_point_start(svg_id, color));
    m.push_str(&marker_circle_end(svg_id, color));
    m.push_str(&marker_circle_start(svg_id, color));
    m.push_str(&marker_cross_end(svg_id, color));
    m.push_str(&marker_cross_start(svg_id, color));
    m
}
