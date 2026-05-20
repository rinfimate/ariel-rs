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

// ---------------------------------------------------------------------------
// Node shapes
// ---------------------------------------------------------------------------

/// Render a square/default node background `<rect>`.
pub fn node_rect_square(x: &str, y: &str, w: &str, h: &str) -> String {
    format!(
        "<rect class=\"basic label-container\" style=\"\" rx=\"0\" ry=\"0\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\"></rect>",
        x = x, y = y, w = w, h = h,
    )
}

/// Render a rounded-rectangle node background `<rect>`.
pub fn node_rect_rounded(x: &str, y: &str, w: &str, h: &str) -> String {
    format!(
        "<rect class=\"basic label-container\" style=\"\" rx=\"8\" ry=\"8\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\"></rect>",
        x = x, y = y, w = w, h = h,
    )
}

/// Render a diamond node background `<polygon>`.
pub fn node_diamond(pts: &str) -> String {
    format!(
        "<polygon points=\"{pts}\" class=\"basic label-container\" style=\"\"></polygon>",
        pts = pts,
    )
}

/// Render a circle node background `<circle>`.
pub fn node_circle(r: &str) -> String {
    format!(
        "<circle cx=\"0\" cy=\"0\" r=\"{r}\" class=\"basic label-container\" style=\"\"></circle>",
        r = r,
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

/// Render a hexagon node background `<polygon>`.
pub fn node_hexagon(pts: &str) -> String {
    format!(
        "<polygon points=\"{pts}\" class=\"basic label-container\" style=\"\"></polygon>",
        pts = pts,
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

/// Render the node label group using `<foreignObject>`.
pub fn node_label_fo(tx: &str, ty: &str, fw: &str, fo_h: &str, label: &str) -> String {
    format!(
        "<g class=\"label\" style=\"\" transform=\"translate({tx}, {ty})\"><rect></rect><foreignObject width=\"{fw}\" height=\"{fo_h}\"><div xmlns=\"http://www.w3.org/1999/xhtml\" style=\"display: table-cell; white-space: nowrap; line-height: 1.5;\"><span class=\"nodeLabel \"><p>{label}</p></span></div></foreignObject></g>",
        tx = tx, ty = ty, fw = fw, fo_h = fo_h, label = label,
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
) -> String {
    format!(
        "<path d=\"{path}\" id=\"{edge_id}\" class=\" edge-thickness-normal edge-pattern-solid edge-thickness-normal edge-pattern-solid flowchart-link LS-{from_lc} LE-{to_lc}\" marker-end=\"{marker_end}\"></path>",
        path = path, edge_id = edge_id, from_lc = from_lc, to_lc = to_lc, marker_end = marker_end,
    )
}

/// Render an edge label `<text>` element.
pub fn edge_label_text(mx: &str, my: &str, label: &str) -> String {
    format!(
        "<text x=\"{mx}\" y=\"{my}\" text-anchor=\"middle\" font-size=\"12\" font-family=\"Arial, sans-serif\" fill=\"#333\">{label}</text>",
        mx = mx, my = my, label = label,
    )
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

/// Render the `pointEnd` filled-triangle marker for block diagrams.
pub fn marker_point_end(svg_id: &str) -> String {
    format!(
        "<marker id=\"{svg_id}_block-pointEnd\" class=\"marker block\" viewBox=\"0 0 10 10\" refX=\"6\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"12\" markerHeight=\"12\" orient=\"auto\"><path d=\"M 0 0 L 10 5 L 0 10 z\" class=\"arrowMarkerPath\" style=\"stroke-width: 1; stroke-dasharray: 1, 0;\"></path></marker>",
        svg_id = svg_id,
    )
}

/// Render the `pointStart` filled-triangle marker for block diagrams.
pub fn marker_point_start(svg_id: &str) -> String {
    format!(
        "<marker id=\"{svg_id}_block-pointStart\" class=\"marker block\" viewBox=\"0 0 10 10\" refX=\"4.5\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"12\" markerHeight=\"12\" orient=\"auto\"><path d=\"M 0 5 L 10 10 L 10 0 z\" class=\"arrowMarkerPath\" style=\"stroke-width: 1; stroke-dasharray: 1, 0;\"></path></marker>",
        svg_id = svg_id,
    )
}

/// Render the `circleEnd` open-circle marker for block diagrams.
pub fn marker_circle_end(svg_id: &str) -> String {
    format!(
        "<marker id=\"{svg_id}_block-circleEnd\" class=\"marker block\" viewBox=\"0 0 10 10\" refX=\"11\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"11\" markerHeight=\"11\" orient=\"auto\"><circle cx=\"5\" cy=\"5\" r=\"5\" class=\"arrowMarkerPath\" style=\"stroke-width: 1; stroke-dasharray: 1, 0;\"></circle></marker>",
        svg_id = svg_id,
    )
}

/// Render the `circleStart` open-circle marker for block diagrams.
pub fn marker_circle_start(svg_id: &str) -> String {
    format!(
        "<marker id=\"{svg_id}_block-circleStart\" class=\"marker block\" viewBox=\"0 0 10 10\" refX=\"-1\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"11\" markerHeight=\"11\" orient=\"auto\"><circle cx=\"5\" cy=\"5\" r=\"5\" class=\"arrowMarkerPath\" style=\"stroke-width: 1; stroke-dasharray: 1, 0;\"></circle></marker>",
        svg_id = svg_id,
    )
}

/// Render the `crossEnd` X-cross marker for block diagrams.
pub fn marker_cross_end(svg_id: &str) -> String {
    format!(
        "<marker id=\"{svg_id}_block-crossEnd\" class=\"marker cross block\" viewBox=\"0 0 11 11\" refX=\"12\" refY=\"5.2\" markerUnits=\"userSpaceOnUse\" markerWidth=\"11\" markerHeight=\"11\" orient=\"auto\"><path d=\"M 1,1 l 9,9 M 10,1 l -9,9\" class=\"arrowMarkerPath\" style=\"stroke-width: 2; stroke-dasharray: 1, 0;\"></path></marker>",
        svg_id = svg_id,
    )
}

/// Render the `crossStart` X-cross marker for block diagrams.
pub fn marker_cross_start(svg_id: &str) -> String {
    format!(
        "<marker id=\"{svg_id}_block-crossStart\" class=\"marker cross block\" viewBox=\"0 0 11 11\" refX=\"-1\" refY=\"5.2\" markerUnits=\"userSpaceOnUse\" markerWidth=\"11\" markerHeight=\"11\" orient=\"auto\"><path d=\"M 1,1 l 9,9 M 10,1 l -9,9\" class=\"arrowMarkerPath\" style=\"stroke-width: 2; stroke-dasharray: 1, 0;\"></path></marker>",
        svg_id = svg_id,
    )
}

/// Build all marker defs for a block diagram.
pub fn build_markers(svg_id: &str) -> String {
    let mut m = String::new();
    m.push_str(&marker_point_end(svg_id));
    m.push_str(&marker_point_start(svg_id));
    m.push_str(&marker_circle_end(svg_id));
    m.push_str(&marker_circle_start(svg_id));
    m.push_str(&marker_cross_end(svg_id));
    m.push_str(&marker_cross_start(svg_id));
    m
}

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

/// Build the CSS style block (matches Mermaid's block diagram style exactly).
pub fn build_style(id: &str, ff: &str) -> String {
    let mut c = String::new();
    c.push_str(&format!(
        "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}"
    ));
    c.push_str("@keyframes edge-animation-frame{from{stroke-dashoffset:0;}}");
    c.push_str("@keyframes dash{to{stroke-dashoffset:0;}}");
    c.push_str(&format!("#{id} .edge-animation-slow{{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 50s linear infinite;stroke-linecap:round;}}"));
    c.push_str(&format!("#{id} .edge-animation-fast{{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 20s linear infinite;stroke-linecap:round;}}"));
    c.push_str(&format!("#{id} .error-icon{{fill:#552222;}}"));
    c.push_str(&format!(
        "#{id} .error-text{{fill:#552222;stroke:#552222;}}"
    ));
    c.push_str(&format!(
        "#{id} .edge-thickness-normal{{stroke-width:1px;}}"
    ));
    c.push_str(&format!(
        "#{id} .edge-thickness-thick{{stroke-width:3.5px;}}"
    ));
    c.push_str(&format!("#{id} .edge-pattern-solid{{stroke-dasharray:0;}}"));
    c.push_str(&format!(
        "#{id} .edge-thickness-invisible{{stroke-width:0;fill:none;}}"
    ));
    c.push_str(&format!(
        "#{id} .edge-pattern-dashed{{stroke-dasharray:3;}}"
    ));
    c.push_str(&format!(
        "#{id} .edge-pattern-dotted{{stroke-dasharray:2;}}"
    ));
    c.push_str(&format!("#{id} .marker{{fill:#333333;stroke:#333333;}}"));
    c.push_str(&format!("#{id} .marker.cross{{stroke:#333333;}}"));
    c.push_str(&format!("#{id} svg{{font-family:{ff};font-size:16px;}}"));
    c.push_str(&format!("#{id} p{{margin:0;}}"));
    c.push_str(&format!("#{id} .label{{font-family:{ff};color:#333;}}"));
    c.push_str(&format!("#{id} .cluster-label text{{fill:#333;}}"));
    c.push_str(&format!("#{id} .cluster-label span,#{id} p{{color:#333;}}"));
    c.push_str(&format!(
        "#{id} .label text,#{id} span,#{id} p{{fill:#333;color:#333;}}"
    ));
    c.push_str(&format!("#{id} .node rect,#{id} .node circle,#{id} .node ellipse,#{id} .node polygon,#{id} .node path{{fill:#ECECFF;stroke:#9370DB;stroke-width:1px;}}"));
    c.push_str(&format!(
        "#{id} .flowchart-label text{{text-anchor:middle;}}"
    ));
    c.push_str(&format!("#{id} .node .label{{text-align:center;}}"));
    c.push_str(&format!("#{id} .node.clickable{{cursor:pointer;}}"));
    c.push_str(&format!("#{id} .arrowheadPath{{fill:#333333;}}"));
    c.push_str(&format!(
        "#{id} .edgePath .path{{stroke:#333333;stroke-width:2.0px;}}"
    ));
    c.push_str(&format!(
        "#{id} .flowchart-link{{stroke:#333333;fill:none;}}"
    ));
    c.push_str(&format!(
        "#{id} .edgeLabel{{background-color:rgba(232,232,232, 0.8);text-align:center;}}"
    ));
    c.push_str(&format!(
        "#{id} .edgeLabel p{{margin:0;padding:0;display:inline;}}"
    ));
    c.push_str(&format!("#{id} .edgeLabel rect{{opacity:0.5;background-color:rgba(232,232,232, 0.8);fill:rgba(232,232,232, 0.8);}}"));
    c.push_str(&format!(
        "#{id} .labelBkg{{background-color:rgba(232,232,232, 0.8);}}"
    ));
    c.push_str(&format!("#{id} .node .cluster{{fill:rgba(255, 255, 222, 0.5);stroke:rgba(170, 170, 51, 0.2);box-shadow:rgba(50, 50, 93, 0.25) 0px 13px 27px -5px,rgba(0, 0, 0, 0.3) 0px 8px 16px -8px;stroke-width:1px;}}"));
    c.push_str(&format!("#{id} .cluster text{{fill:#333;}}"));
    c.push_str(&format!("#{id} .cluster span,#{id} p{{color:#333;}}"));
    c.push_str(&format!("#{id} div.mermaidTooltip{{position:absolute;text-align:center;max-width:200px;padding:2px;font-family:{ff};font-size:12px;background:hsl(80, 100%, 96.2745098039%);border:1px solid #aaaa33;border-radius:2px;pointer-events:none;z-index:100;}}"));
    c.push_str(&format!(
        "#{id} .flowchartTitleText{{text-anchor:middle;font-size:18px;fill:#333;}}"
    ));
    c.push_str(&format!("#{id} .label-icon{{display:inline-block;height:1em;overflow:visible;vertical-align:-0.125em;}}"));
    c.push_str(&format!(
        "#{id} .node .label-icon path{{fill:currentColor;stroke:revert;stroke-width:revert;}}"
    ));
    c.push_str(&format!("#{id} .node .neo-node{{stroke:#9370DB;}}"));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node rect,#{id} [data-look=\"neo\"].cluster rect,#{id} [data-look=\"neo\"].node polygon{{stroke:#9370DB;filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node path{{stroke:#9370DB;stroke-width:1px;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node .outer-path{{filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node .neo-line path{{stroke:#9370DB;filter:none;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node circle{{stroke:#9370DB;filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node circle .state-start{{fill:#000000;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].icon-shape .icon{{fill:#9370DB;filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!("#{id} [data-look=\"neo\"].icon-shape .icon-neo path{{stroke:#9370DB;filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!("#{id} :root{{--mermaid-font-family:{ff};}}"));
    c
}
