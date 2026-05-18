//! SVG template functions for the class diagram renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for a class diagram.
pub fn svg_root(id: &str, w: &str, h: &str) -> String {
    format!(
        r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" class="classDiagram" style="max-width: {w}px;" viewBox="0 0 {w} {h}" role="graphics-document document" aria-roledescription="class">"#,
        id = id,
        w = w,
        h = h,
    )
}

// ---------------------------------------------------------------------------
// Drop-shadow filters
// ---------------------------------------------------------------------------

/// Render the standard drop-shadow `<defs><filter>` (130% × 130%).
pub fn drop_shadow_filter(id: &str) -> String {
    format!(
        "<defs><filter id=\"{id}-drop-shadow\" height=\"130%\" width=\"130%\"><feDropShadow dx=\"4\" dy=\"4\" stdDeviation=\"0\" flood-opacity=\"0.06\" flood-color=\"#000000\"></feDropShadow></filter></defs>",
        id = id,
    )
}

/// Render the small drop-shadow `<defs><filter>` (150% × 150%).
pub fn drop_shadow_filter_small(id: &str) -> String {
    format!(
        "<defs><filter id=\"{id}-drop-shadow-small\" height=\"150%\" width=\"150%\"><feDropShadow dx=\"2\" dy=\"2\" stdDeviation=\"0\" flood-opacity=\"0.06\" flood-color=\"#000000\"></feDropShadow></filter></defs>",
        id = id,
    )
}

// ---------------------------------------------------------------------------
// Edge paths
// ---------------------------------------------------------------------------

/// Render a class diagram edge `<path>`.
pub fn edge_path(
    path_d: &str,
    edge_id: &str,
    classes: &str,
    marker_start: &str,
    marker_end: &str,
) -> String {
    format!(
        r#"<path d="{d}" id="{eid}" class="{cls}" style=";;;" data-edge="true" data-et="edge" data-id="{eid}" data-look="classic"{ms}{me}></path>"#,
        d = path_d,
        eid = edge_id,
        cls = classes,
        ms = marker_start,
        me = marker_end,
    )
}

// ---------------------------------------------------------------------------
// Edge labels
// ---------------------------------------------------------------------------

/// Render a class diagram edge label using `<foreignObject>`.
pub fn edge_label_fo(mx: &str, my: &str, eid: &str, ox: &str, fw: &str, text: &str) -> String {
    format!(
        r#"<g class="edgeLabel" transform="translate({mx}, {my})"><g class="label" data-id="{eid}" transform="translate({ox}, -12)"><foreignObject width="{fw}" height="24"><div xmlns="http://www.w3.org/1999/xhtml" class="labelBkg" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;"><span class="edgeLabel "><p>{text}</p></span></div></foreignObject></g></g>"#,
        mx = mx,
        my = my,
        eid = eid,
        ox = ox,
        fw = fw,
        text = text,
    )
}

/// Render a class diagram edge label using a plain SVG `<text>` element.
pub fn edge_label_text(
    mx: &str,
    my: &str,
    ox: &str,
    fw: &str,
    pf: &str,
    ff: &str,
    text: &str,
) -> String {
    format!(
        r##"<g class="edgeLabel" transform="translate({mx}, {my})"><rect x="{ox}" y="-12" width="{fw}" height="24" fill="{pf}" stroke="none"></rect><text x="0" y="5" text-anchor="middle" font-family="{ff}" font-size="16" fill="#131300">{text}</text></g>"##,
        mx = mx,
        my = my,
        ox = ox,
        fw = fw,
        pf = pf,
        ff = ff,
        text = text,
    )
}

/// Render an empty edge label placeholder.
pub fn edge_label_empty(eid: &str) -> String {
    format!(
        r#"<g class="edgeLabel"><g class="label" data-id="{eid}" transform="translate(0, 0)"><foreignObject width="0" height="0"><div xmlns="http://www.w3.org/1999/xhtml" class="labelBkg" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;"><span class="edgeLabel "></span></div></foreignObject></g></g>"#,
        eid = eid,
    )
}

/// Render a terminal label (cardinality) using `<foreignObject>`.
pub fn terminal_label_fo(cx: &str, cy: &str, fw: &str, sw: usize, text: &str) -> String {
    format!(
        r#"<g class="edgeTerminals" transform="translate({cx}, {cy})"><g class="inner" transform="translate(0, -8.25)"><foreignObject width="{fw}" height="16.5" style="width: {sw}px; height: 12px;"><div xmlns="http://www.w3.org/1999/xhtml" style="display: table-cell; white-space: nowrap; line-height: 1.5;"><span class="edgeLabel "><p>{text}</p></span></div></foreignObject></g></g>"#,
        cx = cx,
        cy = cy,
        fw = fw,
        sw = sw,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Node rendering
// ---------------------------------------------------------------------------

/// Render the outer `<g class="node">` wrapper for a class node.
pub fn node_group(dom_id: &str, cx: &str, cy: &str) -> String {
    format!(
        r#"<g class="node default " id="{did}" data-look="classic" transform="translate({cx}, {cy})">"#,
        did = dom_id,
        cx = cx,
        cy = cy,
    )
}

/// Render the filled outer path for the class box shadow layer.
pub fn node_outer_path(x1: &str, y1: &str, x2: &str, y2: &str, pf: &str) -> String {
    format!(
        r#"<g class="basic label-container outer-path"><path d="M{x1} {y1} L{x2} {y1} L{x2} {y2} L{x1} {y2}" stroke="none" stroke-width="0" fill="{pf}" style=""></path>"#,
        x1 = x1,
        y1 = y1,
        x2 = x2,
        y2 = y2,
        pf = pf,
    )
}

/// Render the sketchy border path for a class box (neo-classic look).
#[allow(clippy::too_many_arguments)]
pub fn node_border_path(
    x1: &str,
    y1: &str,
    x2: &str,
    y2: &str,
    cx1: &str,
    cx2: &str,
    cx3: &str,
    cx4: &str,
    cy1: &str,
    cy2: &str,
    cy3: &str,
    cy4: &str,
    pb: &str,
) -> String {
    format!(
        r#"<path d="M{x1} {y1} C{cx1} {y1},{cx2} {y1},{x2} {y1} M{x2} {y1} C{x2} {cy1},{x2} {cy2},{x2} {y2} M{x2} {y2} C{cx3} {y2},{cx4} {y2},{x1} {y2} M{x1} {y2} C{x1} {cy3},{x1} {cy4},{x1} {y1}" stroke="{pb}" stroke-width="1.3" fill="none" stroke-dasharray="0 0" style=""></path></g>"#,
        x1 = x1,
        y1 = y1,
        x2 = x2,
        y2 = y2,
        cx1 = cx1,
        cx2 = cx2,
        cx3 = cx3,
        cx4 = cx4,
        cy1 = cy1,
        cy2 = cy2,
        cy3 = cy3,
        cy4 = cy4,
        pb = pb,
    )
}

/// Render the annotation group wrapper `<g>`.
pub fn annotation_group(y: &str) -> String {
    format!(
        r#"<g class="annotation-group text" transform="translate(0, {})">"#,
        y,
    )
}

/// Render a single annotation row using `<foreignObject>`.
pub fn annotation_fo(ox: &str, y: &str, fw: &str, text: &str) -> String {
    format!(
        r#"<g class="label" style="font-style: italic" transform="translate({ox}, {y})"><foreignObject width="{fw}" height="24"><div xmlns="http://www.w3.org/1999/xhtml" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;"><span class="nodeLabel markdown-node-label" style=""><p>{text}</p></span></div></foreignObject></g>"#,
        ox = ox,
        y = y,
        fw = fw,
        text = text,
    )
}

/// Render a single annotation row as plain SVG `<text>`.
pub fn annotation_text(y: &str, fs: f64, pb: &str, text: &str) -> String {
    format!(
        r#"<text x="0" y="{y}" text-anchor="middle" font-family="Arial,sans-serif" font-size="{fs}" fill="{pb}" font-style="italic">{text}</text>"#,
        y = y,
        fs = fs,
        pb = pb,
        text = text,
    )
}

/// Render the label group (class name) using `<foreignObject>`.
pub fn label_group_fo(ox: &str, gy: &str, fw: &str, text: &str) -> String {
    format!(
        r#"<g class="label-group text" transform="translate({ox}, {gy})"><g class="label" style="font-weight: bolder" transform="translate(0,-12)"><foreignObject width="{fw}" height="24"><div xmlns="http://www.w3.org/1999/xhtml" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 100px; text-align: center;"><span class="nodeLabel markdown-node-label" style=""><p>{text}</p></span></div></foreignObject></g></g>"#,
        ox = ox,
        gy = gy,
        fw = fw,
        text = text,
    )
}

/// Render the label group (class name) as plain SVG `<text>`.
pub fn label_group_text(ox: &str, gy: &str, hw: &str, fs: f64, pb: &str, text: &str) -> String {
    format!(
        r#"<g class="label-group text" transform="translate({ox}, {gy})"><text x="{hw}" y="5" text-anchor="middle" font-family="Arial,sans-serif" font-size="{fs}" fill="{pb}" font-weight="bold">{text}</text></g>"#,
        ox = ox,
        gy = gy,
        hw = hw,
        fs = fs,
        pb = pb,
        text = text,
    )
}

/// Render the members group wrapper `<g>`.
pub fn members_group(ox: &str, gy: &str) -> String {
    format!(
        r#"<g class="members-group text" transform="translate({ox}, {gy})">"#,
        ox = ox,
        gy = gy,
    )
}

/// Render the methods group wrapper `<g>`.
pub fn methods_group(ox: &str, gy: &str) -> String {
    format!(
        r#"<g class="methods-group text" transform="translate({ox}, {gy})">"#,
        ox = ox,
        gy = gy,
    )
}

/// Render a member/method row using `<foreignObject>`.
pub fn member_row_fo(y: &str, fw: &str, text: &str) -> String {
    format!(
        r#"<g class="label" style="" transform="translate(0,{y})"><foreignObject width="{fw}" height="24"><div xmlns="http://www.w3.org/1999/xhtml" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 150px; text-align: center;"><span class="nodeLabel markdown-node-label" style=""><p>{text}</p></span></div></foreignObject></g>"#,
        y = y,
        fw = fw,
        text = text,
    )
}

/// Render a member/method row as plain SVG `<text>`.
pub fn member_row_text(y: &str, fs: f64, pb: &str, text: &str) -> String {
    format!(
        r#"<text x="0" y="{y}" font-family="Arial,sans-serif" font-size="{fs}" fill="{pb}">{text}</text>"#,
        y = y,
        fs = fs,
        pb = pb,
        text = text,
    )
}

/// Render a class box divider `<path>` (cubic bezier).
pub fn divider_path(x1: &str, y: &str, cx1: &str, cx2: &str, x2: &str, pb: &str) -> String {
    format!(
        r#"<g class="divider" style=""><path d="M{x1} {y} C{cx1} {y},{cx2} {y},{x2} {y}" stroke="{pb}" stroke-width="1.3" fill="none" stroke-dasharray="0 0" style=""></path></g>"#,
        x1 = x1,
        y = y,
        cx1 = cx1,
        cx2 = cx2,
        x2 = x2,
        pb = pb,
    )
}
