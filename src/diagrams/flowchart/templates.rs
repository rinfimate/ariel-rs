//! SVG template functions for the flowchart renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper element.
pub fn svg_root(id: &str, max_w: f64, vb_w: f64, vb_h: f64) -> String {
    format!(
        r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" class="flowchart" style="max-width: {max_w}px;" viewBox="0 0 {vb_w} {vb_h}" role="graphics-document document" aria-roledescription="flowchart-v2">"#,
    )
}

/// Render a drop-shadow `<defs><filter>` element (standard size: 130% × 130%).
pub fn drop_shadow_filter(id: &str) -> String {
    format!(
        "<defs><filter id=\"{id}-drop-shadow\" height=\"130%\" width=\"130%\"><feDropShadow dx=\"4\" dy=\"4\" stdDeviation=\"0\" flood-opacity=\"0.06\" flood-color=\"#000000\"></feDropShadow></filter></defs>",
    )
}

/// Render a drop-shadow `<defs><filter>` element (small size: 150% × 150%).
pub fn drop_shadow_filter_small(id: &str) -> String {
    format!(
        "<defs><filter id=\"{id}-drop-shadow-small\" height=\"150%\" width=\"150%\"><feDropShadow dx=\"2\" dy=\"2\" stdDeviation=\"0\" flood-opacity=\"0.06\" flood-color=\"#000000\"></feDropShadow></filter></defs>",
    )
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

/// Render all flowchart arrow-head `<marker>` definitions for the given SVG id.
pub fn all_markers(id: &str) -> String {
    let mut m = String::new();
    m.push_str(&marker_point_end(id));
    m.push_str(&marker_point_start(id));
    m.push_str(&marker_point_end_margin(id));
    m.push_str(&marker_point_start_margin(id));
    m.push_str(&marker_circle_end(id));
    m.push_str(&marker_circle_start(id));
    m.push_str(&marker_circle_end_margin(id));
    m.push_str(&marker_circle_start_margin(id));
    m.push_str(&marker_cross_end(id));
    m.push_str(&marker_cross_start(id));
    m.push_str(&marker_cross_end_margin(id));
    m.push_str(&marker_cross_start_margin(id));
    m
}

/// Render the `pointEnd` filled-triangle marker.
pub fn marker_point_end(id: &str) -> String {
    format!(
        r#"<marker id="{id}_flowchart-v2-pointEnd" class="marker flowchart-v2" viewBox="0 0 10 10" refX="5" refY="5" markerUnits="userSpaceOnUse" markerWidth="8" markerHeight="8" orient="auto"><path d="M 0 0 L 10 5 L 0 10 z" class="arrowMarkerPath" style="stroke-width: 1; stroke-dasharray: 1, 0;"></path></marker>"#
    )
}

/// Render the `pointStart` filled-triangle marker (reverse direction).
pub fn marker_point_start(id: &str) -> String {
    format!(
        r#"<marker id="{id}_flowchart-v2-pointStart" class="marker flowchart-v2" viewBox="0 0 10 10" refX="4.5" refY="5" markerUnits="userSpaceOnUse" markerWidth="8" markerHeight="8" orient="auto"><path d="M 0 5 L 10 10 L 10 0 z" class="arrowMarkerPath" style="stroke-width: 1; stroke-dasharray: 1, 0;"></path></marker>"#
    )
}

/// Render the `pointEnd-margin` filled-triangle marker (wider viewBox).
pub fn marker_point_end_margin(id: &str) -> String {
    format!(
        r#"<marker id="{id}_flowchart-v2-pointEnd-margin" class="marker flowchart-v2" viewBox="0 0 11.5 14" refX="11.5" refY="7" markerUnits="userSpaceOnUse" markerWidth="10.5" markerHeight="14" orient="auto"><path d="M 0 0 L 11.5 7 L 0 14 z" class="arrowMarkerPath" style="stroke-width: 0; stroke-dasharray: 1, 0;"></path></marker>"#
    )
}

/// Render the `pointStart-margin` filled-triangle marker (polygon, wider viewBox).
pub fn marker_point_start_margin(id: &str) -> String {
    format!(
        r#"<marker id="{id}_flowchart-v2-pointStart-margin" class="marker flowchart-v2" viewBox="0 0 11.5 14" refX="1" refY="7" markerUnits="userSpaceOnUse" markerWidth="11.5" markerHeight="14" orient="auto"><polygon points="0,7 11.5,14 11.5,0" class="arrowMarkerPath" style="stroke-width: 0; stroke-dasharray: 1, 0;"></polygon></marker>"#
    )
}

/// Render the `circleEnd` open-circle marker.
pub fn marker_circle_end(id: &str) -> String {
    format!(
        r#"<marker id="{id}_flowchart-v2-circleEnd" class="marker flowchart-v2" viewBox="0 0 10 10" refX="11" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><circle cx="5" cy="5" r="5" class="arrowMarkerPath" style="stroke-width: 1; stroke-dasharray: 1, 0;"></circle></marker>"#
    )
}

/// Render the `circleStart` open-circle marker (reverse direction).
pub fn marker_circle_start(id: &str) -> String {
    format!(
        r#"<marker id="{id}_flowchart-v2-circleStart" class="marker flowchart-v2" viewBox="0 0 10 10" refX="-1" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><circle cx="5" cy="5" r="5" class="arrowMarkerPath" style="stroke-width: 1; stroke-dasharray: 1, 0;"></circle></marker>"#
    )
}

/// Render the `circleEnd-margin` open-circle marker (wider).
pub fn marker_circle_end_margin(id: &str) -> String {
    format!(
        r#"<marker id="{id}_flowchart-v2-circleEnd-margin" class="marker flowchart-v2" viewBox="0 0 10 10" refY="5" refX="12.25" markerUnits="userSpaceOnUse" markerWidth="14" markerHeight="14" orient="auto"><circle cx="5" cy="5" r="5" class="arrowMarkerPath" style="stroke-width: 0; stroke-dasharray: 1, 0;"></circle></marker>"#
    )
}

/// Render the `circleStart-margin` open-circle marker (wider, reverse direction).
pub fn marker_circle_start_margin(id: &str) -> String {
    format!(
        r#"<marker id="{id}_flowchart-v2-circleStart-margin" class="marker flowchart-v2" viewBox="0 0 10 10" refX="-2" refY="5" markerUnits="userSpaceOnUse" markerWidth="14" markerHeight="14" orient="auto"><circle cx="5" cy="5" r="5" class="arrowMarkerPath" style="stroke-width: 0; stroke-dasharray: 1, 0;"></circle></marker>"#
    )
}

/// Render the `crossEnd` X-cross marker.
pub fn marker_cross_end(id: &str) -> String {
    format!(
        r#"<marker id="{id}_flowchart-v2-crossEnd" class="marker cross flowchart-v2" viewBox="0 0 11 11" refX="12" refY="5.2" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><path d="M 1,1 l 9,9 M 10,1 l -9,9" class="arrowMarkerPath" style="stroke-width: 2; stroke-dasharray: 1, 0;"></path></marker>"#
    )
}

/// Render the `crossStart` X-cross marker (reverse direction).
pub fn marker_cross_start(id: &str) -> String {
    format!(
        r#"<marker id="{id}_flowchart-v2-crossStart" class="marker cross flowchart-v2" viewBox="0 0 11 11" refX="-1" refY="5.2" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto"><path d="M 1,1 l 9,9 M 10,1 l -9,9" class="arrowMarkerPath" style="stroke-width: 2; stroke-dasharray: 1, 0;"></path></marker>"#
    )
}

/// Render the `crossEnd-margin` X-cross marker (wider viewBox).
pub fn marker_cross_end_margin(id: &str) -> String {
    format!(
        r#"<marker id="{id}_flowchart-v2-crossEnd-margin" class="marker cross flowchart-v2" viewBox="0 0 15 15" refX="17.7" refY="7.5" markerUnits="userSpaceOnUse" markerWidth="12" markerHeight="12" orient="auto"><path d="M 1,1 L 14,14 M 1,14 L 14,1" class="arrowMarkerPath" style="stroke-width: 2.5;"></path></marker>"#
    )
}

/// Render the `crossStart-margin` X-cross marker (wider viewBox, reverse direction).
pub fn marker_cross_start_margin(id: &str) -> String {
    format!(
        r#"<marker id="{id}_flowchart-v2-crossStart-margin" class="marker cross flowchart-v2" viewBox="0 0 15 15" refX="-3.5" refY="7.5" markerUnits="userSpaceOnUse" markerWidth="12" markerHeight="12" orient="auto"><path d="M 1,1 L 14,14 M 1,14 L 14,1" class="arrowMarkerPath" style="stroke-width: 2.5; stroke-dasharray: 1, 0;"></path></marker>"#
    )
}

// ---------------------------------------------------------------------------
// Edge rendering
// ---------------------------------------------------------------------------

/// Render a `marker-end` attribute referencing the `pointEnd` arrowhead marker.
pub fn marker_end_point(svg_id: &str) -> String {
    format!(r#" marker-end="url(#{svg_id}_flowchart-v2-pointEnd)""#)
}

/// Render a `marker-end` attribute referencing the `crossEnd` X-cross marker.
pub fn marker_end_cross(svg_id: &str) -> String {
    format!(r#" marker-end="url(#{svg_id}_flowchart-v2-crossEnd)""#)
}

/// Render a `marker-end` attribute referencing the `circleEnd` open-circle marker.
pub fn marker_end_circle(svg_id: &str) -> String {
    format!(r#" marker-end="url(#{svg_id}_flowchart-v2-circleEnd)""#)
}

/// Render an edge `<path>` element.
pub fn edge_path(path_d: &str, edge_id: &str, classes: &str, marker_end: &str) -> String {
    format!(
        r#"<path d="{path_d}" id="{edge_id}" class="{classes}" style=";" data-edge="true" data-et="edge" data-id="{edge_id}" data-look="classic"{marker_end}></path>"#,
    )
}

/// Render an edge label using `<foreignObject>` HTML (matches Mermaid reference).
#[allow(clippy::too_many_arguments)]
pub fn edge_label_fo(
    mx: &str,
    my: &str,
    edge_id: &str,
    ox: &str,
    fo_width: &str,
    fo_height: f64,
    label_y_offset: i32,
    text: &str,
) -> String {
    format!(
        r#"<g class="edgeLabel" transform="translate({mx}, {my})"><g class="label" data-id="{edge_id}" transform="translate({ox}, {label_y_offset})"><foreignObject width="{fo_width}" height="{fo_height}"><div xmlns="http://www.w3.org/1999/xhtml" class="labelBkg" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;"><span class="edgeLabel "><p>{text}</p></span></div></foreignObject></g></g>"#,
    )
}

/// Render an edge label using plain SVG `<text>` (fallback when foreignObject is disabled).
#[allow(clippy::too_many_arguments)]
pub fn edge_label_text(
    mx: &str,
    my: &str,
    ox: &str,
    fo_width: &str,
    fo_height: f64,
    label_y_offset: i32,
    text_label_y: i32,
    font_family: &str,
    font_size: f64,
    text: &str,
) -> String {
    format!(
        r##"<g class="edgeLabel" transform="translate({mx}, {my})"><rect x="{ox}" y="{label_y_offset}" width="{fo_width}" height="{fo_height}" fill="rgba(232,232,232,0.8)" stroke="none"></rect><text x="0" y="{text_label_y}" text-anchor="middle" font-family="{font_family}" font-size="{font_size}" fill="#333">{text}</text></g>"##,
    )
}

/// Render an empty edge label placeholder (no visible text).
pub fn edge_label_empty(edge_id: &str) -> String {
    format!(
        r#"<g class="edgeLabel"><g class="label" data-id="{edge_id}" transform="translate(0, 0)"><foreignObject width="0" height="0"><div xmlns="http://www.w3.org/1999/xhtml" class="labelBkg" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;"><span class="edgeLabel "></span></div></foreignObject></g></g>"#,
    )
}

// ---------------------------------------------------------------------------
// Cluster (subgraph) rendering
// ---------------------------------------------------------------------------

/// Render the outer `<g class="cluster ...">` wrapper including its rect and label.
pub fn cluster_group(
    svg_id: &str,
    sg_id: &str,
    x: &str,
    y: &str,
    w: &str,
    h: &str,
    label_html: &str,
) -> String {
    format!(
        r#"<g class="cluster " id="{svg_id}-{sg_id}" data-look="classic"><rect style="" x="{x}" y="{y}" width="{w}" height="{h}"></rect>{label_html}</g>"#,
    )
}

/// Render a cluster label using `<foreignObject>` HTML.
pub fn cluster_label_fo(
    lx: &str,
    ly: &str,
    fo_width: &str,
    fo_height: f64,
    label_text: &str,
) -> String {
    format!(
        r#"<g class="cluster-label " transform="translate({lx}, {ly})"><foreignObject width="{fo_width}" height="{fo_height}"><div xmlns="http://www.w3.org/1999/xhtml" style="display: table-cell; white-space: nowrap; line-height: 1.5;"><span class="nodeLabel "><p>{label_text}</p></span></div></foreignObject></g>"#,
    )
}

/// Render a cluster label using a plain SVG `<text>` element.
pub fn cluster_label_text(
    cx: &str,
    text_y: &str,
    font_family: &str,
    font_size: f64,
    label_text: &str,
) -> String {
    format!(
        r##"<text x="{cx}" y="{text_y}" text-anchor="middle" font-family="{font_family}" font-size="{font_size}" fill="#333">{label_text}</text>"##,
    )
}

/// Render a `<g class="root" transform="translate(ox, oy)">` for an internal subgraph.
pub fn subgraph_root_group(ox: &str, oy: &str) -> String {
    format!(r#"<g class="root" transform="translate({ox}, {oy})">"#)
}

// ---------------------------------------------------------------------------
// Node rendering
// ---------------------------------------------------------------------------

/// Render the outer `<g class="node ...">` wrapper for a leaf node.
pub fn node_group(dom_id: &str, cx: &str, cy: &str) -> String {
    format!(
        r#"<g class="node default  " id="{dom_id}" data-look="classic" transform="translate({cx}, {cy})">"#,
    )
}

/// Render a rectangular node background (`<rect>`).
pub fn node_rect(x: &str, w: &str, style: &str) -> String {
    format!(
        r#"<rect class="basic label-container" style="{style}" x="{x}" y="-27" width="{w}" height="54"></rect>"#,
    )
}

/// Render a rounded-rectangle node background (`<rect rx ry>`).
pub fn node_rounded_rect(x: &str, w: &str, style: &str) -> String {
    format!(
        r#"<rect class="basic label-container" style="{style}" rx="5" ry="5" x="{x}" y="-27" width="{w}" height="54"></rect>"#,
    )
}

/// Render a Stadium (pill-shaped) node background (`<rect>` with large rx/ry).
pub fn node_stadium_rect(rx: &str, x: &str, y: &str, w: &str, h: &str, style: &str) -> String {
    format!(
        r#"<rect class="basic label-container" style="{style}" rx="{rx}" ry="{rx}" x="{x}" y="{y}" width="{w}" height="{h}"></rect>"#,
    )
}

/// Render a Diamond node background (`<polygon>`).
pub fn node_diamond(
    hw: &str,
    dim_w: &str,
    neg_hh: &str,
    neg_h: &str,
    tx: &str,
    ty: &str,
    style: &str,
) -> String {
    format!(
        r#"<polygon points="{hw},0 {dim_w},{neg_hh} {hw},{neg_h} 0,{neg_hh}" class="label-container" style="{style}" transform="translate({tx},{ty})"></polygon>"#,
    )
}

/// Render a Circle node background (`<circle>`).
pub fn node_circle(r: &str, style: &str) -> String {
    format!(r#"<circle class="label-container" style="{style}" cx="0" cy="0" r="{r}"></circle>"#,)
}

/// Render an Asymmetric (flag/chevron) node background (`<polygon>`).
pub fn node_asymmetric(points: &str, style: &str) -> String {
    format!(r#"<polygon points="{points}" class="label-container" style="{style}"></polygon>"#,)
}

/// Render a Hexagon node background (`<polygon>`).
pub fn node_hexagon(points: &str, style: &str) -> String {
    format!(r#"<polygon points="{points}" class="label-container" style="{style}"></polygon>"#,)
}

/// Render the main body `<path>` for a Cylinder (database) node.
pub fn node_cylinder_body(path_d: &str, style: &str) -> String {
    format!(r#"<path class="basic label-container" d="{path_d}" style="{style}"></path>"#,)
}

/// Render the top-ellipse arc `<path>` for a Cylinder node (visible top rim).
pub fn node_cylinder_top(path_d: &str, stroke: &str) -> String {
    format!(r#"<path d="{path_d}" fill="none" stroke="{stroke}" style=""></path>"#,)
}

/// Render the main `<rect>` background for a Subroutine node.
pub fn node_subroutine_rect(x: &str, y: &str, w: &str, h: &str, style: &str) -> String {
    format!(
        r#"<rect class="basic label-container" style="{style}" x="{x}" y="{y}" width="{w}" height="{h}"></rect>"#,
    )
}

/// Render one of the two decorative vertical lines on a Subroutine node.
pub fn node_subroutine_line(x: &str, y1: &str, y2: &str, stroke: &str) -> String {
    format!(
        r#"<line x1="{x}" y1="{y1}" x2="{x}" y2="{y2}" style="stroke:{stroke};stroke-width:1px;"></line>"#,
    )
}

/// Render a node label using `<foreignObject>` HTML (matches Mermaid reference).
#[allow(clippy::too_many_arguments)]
pub fn node_label_fo(
    label_color_style: &str,
    tx: &str,
    ty: i32,
    fo_width: &str,
    fo_height: f64,
    div_color_style: &str,
    span_style_attr: &str,
    label_text: &str,
) -> String {
    format!(
        r#"<g class="label" style="{label_color_style}" transform="translate({tx}, {ty})"><rect></rect><foreignObject width="{fo_width}" height="{fo_height}"><div xmlns="http://www.w3.org/1999/xhtml" style="{div_color_style}display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;"><span class="nodeLabel "{span_style_attr}><p{span_style_attr}>{label_text}</p></span></div></foreignObject></g>"#,
    )
}

/// Render a node label using a plain SVG `<text>` element.
pub fn node_label_text(
    label_color_style: &str,
    text_label_y: i32,
    font_family: &str,
    font_size: f64,
    text_fill: &str,
    label_text: &str,
) -> String {
    format!(
        r#"<g class="label" style="{label_color_style}" transform="translate(0, 0)"><text x="0" y="{text_label_y}" text-anchor="middle" font-family="{font_family}" font-size="{font_size}" fill="{text_fill}">{label_text}</text></g>"#,
    )
}
