//! SVG template functions for the state diagram renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Drop-shadow filter defs
// ---------------------------------------------------------------------------

/// Render the standard (130% × 130%) drop-shadow `<defs><filter>` element.
pub fn drop_shadow_filter(id: &str) -> String {
    format!(
        "<defs><filter id=\"{id}-drop-shadow\" height=\"130%\" width=\"130%\"><feDropShadow dx=\"4\" dy=\"4\" stdDeviation=\"0\" flood-opacity=\"0.06\" flood-color=\"#000000\"></feDropShadow></filter></defs>",
        id = id,
    )
}

/// Render the small (150% × 150%) drop-shadow `<defs><filter>` element.
pub fn drop_shadow_filter_small(id: &str) -> String {
    format!(
        "<defs><filter id=\"{id}-drop-shadow-small\" height=\"150%\" width=\"150%\"><feDropShadow dx=\"2\" dy=\"2\" stdDeviation=\"0\" flood-opacity=\"0.06\" flood-color=\"#000000\"></feDropShadow></filter></defs>",
        id = id,
    )
}

// ---------------------------------------------------------------------------
// Edge rendering
// ---------------------------------------------------------------------------

/// Render a state transition `<path>` edge.
pub fn edge_path(path_d: &str, edge_id: &str, extra_class: &str, marker: &str) -> String {
    format!(
        "<path d=\"{}\" id=\"{}\" class=\" edge-thickness-normal edge-pattern-solid transition{}\" style=\"fill:none;;;fill:none\" data-edge=\"true\" data-et=\"edge\" data-id=\"{}\" data-look=\"classic\"{}></path>",
        path_d, edge_id, extra_class, edge_id, marker
    )
}

/// Render a `marker-end` attribute for the stateDiagram barbEnd arrowhead.
pub fn marker_end_barb(svg_id: &str) -> String {
    format!(" marker-end=\"url(#{}_stateDiagram-barbEnd)\"", svg_id)
}

// ---------------------------------------------------------------------------
// Edge label rendering
// ---------------------------------------------------------------------------

/// Render a state edge label using `<foreignObject>` HTML.
pub fn edge_label_fo(edge_id: &str, x: &str, y: &str, width: &str, label: &str) -> String {
    format!(
        "<g class=\"edgeLabel\"><g class=\"label\" data-id=\"{}\" transform=\"translate({}, {})\"><foreignObject width=\"{}\" height=\"24\"><div xmlns=\"http://www.w3.org/1999/xhtml\" class=\"labelBkg\" style=\"display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;\"><span class=\"edgeLabel \">{}</span></div></foreignObject></g></g>",
        edge_id, x, y, width, label
    )
}

/// Render a state edge label using plain SVG `<text>`.
pub fn edge_label_text(x: &str, y: &str, font_size: &str, label: &str) -> String {
    format!(
        "<g class=\"edgeLabel\" transform=\"translate({},{})\"><text x=\"0\" y=\"5\" text-anchor=\"middle\" font-family=\"Arial,sans-serif\" font-size=\"{}\" fill=\"#333\">{}</text></g>",
        x, y, font_size, label
    )
}

/// Render an empty edge-label placeholder.
pub fn edge_label_empty(edge_id: &str) -> String {
    format!(
        "<g class=\"edgeLabel\"><g class=\"label\" data-id=\"{}\" transform=\"translate(0, 0)\"><foreignObject width=\"0\" height=\"0\"><div xmlns=\"http://www.w3.org/1999/xhtml\" class=\"labelBkg\" style=\"display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;\"><span class=\"edgeLabel \"></span></div></foreignObject></g></g>",
        edge_id
    )
}

// ---------------------------------------------------------------------------
// Note cluster
// ---------------------------------------------------------------------------

/// Render the background `<rect>` for a note-cluster compound group.
pub fn note_cluster_rect(cluster_id: &str, x: &str, y: &str, w: &str, h: &str) -> String {
    format!(
        "<g class=\"note-cluster\" id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\"></rect></g>",
        cluster_id, x, y, w, h
    )
}

// ---------------------------------------------------------------------------
// Composite state (cluster) rendering
// ---------------------------------------------------------------------------

/// Render the opening `<g>` tag for a composite state group, with translate.
pub fn composite_root_group(tx: &str, ty: &str) -> String {
    format!("<g class=\"root\" transform=\"translate({}, {})\">", tx, ty)
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

/// Render the `stateDiagram-barbEnd` arrowhead marker definition.
pub fn barb_end_marker(id: &str) -> String {
    let mut m = String::new();
    m.push_str("<defs><marker id=\"");
    m.push_str(id);
    m.push_str("_stateDiagram-barbEnd\" refX=\"19\" refY=\"7\" markerWidth=\"20\" markerHeight=\"14\" markerUnits=\"userSpaceOnUse\" orient=\"auto\"><path d=\"M 19,7 L9,13 L14,7 L9,1 Z\"></path></marker></defs>");
    m
}
