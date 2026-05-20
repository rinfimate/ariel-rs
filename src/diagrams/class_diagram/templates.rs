//! SVG template functions for the class diagram renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

use crate::theme::ThemeVars;

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::{esc, fmt};

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

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

pub fn build_css(id: &str, vars: &ThemeVars) -> String {
    let pb = vars.primary_border;
    let pf = vars.primary_color;
    let lc = vars.line_color;
    let ff = vars.font_family;
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
    c.push_str(&format!(
        "#{id} g.classGroup text{{fill:{pb};stroke:none;font-family:{ff};font-size:10px;}}"
    ));
    c.push_str(&format!(
        "#{id} g.classGroup text .title{{font-weight:bolder;}}"
    ));
    c.push_str(&format!("#{id} .cluster-label text{{fill:#333;}}"));
    c.push_str(&format!("#{id} .cluster-label span{{color:#333;}}"));
    c.push_str(&format!(
        "#{id} .cluster-label span p{{background-color:transparent;}}"
    ));
    c.push_str(&format!(
        "#{id} .cluster rect{{fill:#ffffde;stroke:#aaaa33;stroke-width:1px;}}"
    ));
    c.push_str(&format!("#{id} .cluster text{{fill:#333;}}"));
    c.push_str(&format!("#{id} .cluster span{{color:#333;}}"));
    c.push_str(&format!(
        "#{id} .nodeLabel,#{id} .edgeLabel{{color:#131300;}}"
    ));
    c.push_str(&format!(
        "#{id} .noteLabel .nodeLabel,#{id} .noteLabel .edgeLabel{{color:black;}}"
    ));
    c.push_str(&format!("#{id} .edgeLabel .label rect{{fill:{pf};}}"));
    c.push_str(&format!("#{id} .label text{{fill:#131300;}}"));
    c.push_str(&format!("#{id} .labelBkg{{background:{pf};}}"));
    c.push_str(&format!("#{id} .edgeLabel .label span{{background:{pf};}}"));
    c.push_str(&format!("#{id} .classTitle{{font-weight:bolder;}}"));
    c.push_str(&format!("#{id} .node rect,#{id} .node circle,#{id} .node ellipse,#{id} .node polygon,#{id} .node path{{fill:{pf};stroke:{pb};stroke-width:1;}}"));
    c.push_str(&format!("#{id} .divider{{stroke:{pb};stroke-width:1;}}"));
    c.push_str(&format!("#{id} g.clickable{{cursor:pointer;}}"));
    c.push_str(&format!(
        "#{id} g.classGroup rect{{fill:{pf};stroke:{pb};}}"
    ));
    c.push_str(&format!(
        "#{id} g.classGroup line{{stroke:{pb};stroke-width:1;}}"
    ));
    c.push_str(&format!(
        "#{id} .classLabel .box{{stroke:none;stroke-width:0;fill:{pf};opacity:0.5;}}"
    ));
    c.push_str(&format!(
        "#{id} .classLabel .label{{fill:{pb};font-size:10px;}}"
    ));
    c.push_str(&format!(
        "#{id} .relation{{stroke:{lc};stroke-width:1;fill:none;}}"
    ));
    c.push_str(&format!("#{id} .dashed-line{{stroke-dasharray:3;}}"));
    c.push_str(&format!("#{id} .dotted-line{{stroke-dasharray:1 2;}}"));
    c.push_str(&format!("#{id} [id$=\"-compositionStart\"],#{id} .composition{{fill:#333333!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-compositionEnd\"],#{id} .composition{{fill:#333333!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-dependencyStart\"],#{id} .dependency{{fill:#333333!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-dependencyEnd\"],#{id} .dependency{{fill:#333333!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-extensionStart\"],#{id} .extension{{fill:transparent!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-extensionEnd\"],#{id} .extension{{fill:transparent!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-aggregationStart\"],#{id} .aggregation{{fill:transparent!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-aggregationEnd\"],#{id} .aggregation{{fill:transparent!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-lollipopStart\"],#{id} .lollipop{{fill:{pf}!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!("#{id} [id$=\"-lollipopEnd\"],#{id} .lollipop{{fill:{pf}!important;stroke:#333333!important;stroke-width:1;}}"));
    c.push_str(&format!(
        "#{id} .edgeTerminals{{font-size:11px;line-height:initial;}}"
    ));
    c.push_str(&format!(
        "#{id} .classTitleText{{text-anchor:middle;font-size:18px;fill:#333;}}"
    ));
    c.push_str(&format!("#{id} .edgeLabel[data-look=\"neo\"]{{background-color:rgba(232,232,232, 0.8);text-align:center;}}"));
    c.push_str(&format!(
        "#{id} .edgeLabel[data-look=\"neo\"] p{{background-color:rgba(232,232,232, 0.8);}}"
    ));
    c.push_str(&format!("#{id} .edgeLabel[data-look=\"neo\"] rect{{opacity:0.5;background-color:rgba(232,232,232, 0.8);fill:rgba(232,232,232, 0.8);}}"));
    c.push_str(&format!("#{id} .label-icon{{display:inline-block;height:1em;overflow:visible;vertical-align:-0.125em;}}"));
    c.push_str(&format!(
        "#{id} .node .label-icon path{{fill:currentColor;stroke:revert;stroke-width:revert;}}"
    ));
    c.push_str(&format!("#{id} .node .neo-node{{stroke:{pb};}}"));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node rect,#{id} [data-look=\"neo\"].cluster rect,#{id} [data-look=\"neo\"].node polygon{{stroke:{pb};filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node path{{stroke:{pb};stroke-width:1px;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node .outer-path{{filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node .neo-line path{{stroke:{pb};filter:none;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node circle{{stroke:{pb};filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node circle .state-start{{fill:#000000;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].icon-shape .icon{{fill:{pb};filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!("#{id} [data-look=\"neo\"].icon-shape .icon-neo path{{stroke:{pb};filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!("#{id} :root{{--mermaid-font-family:{ff};}}"));
    c
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

pub fn build_markers(id: &str) -> String {
    let mut m = String::new();
    m.push_str(&format!(r#"<defs><marker id="{id}_class-aggregationStart" class="marker aggregation class" refX="18" refY="7" markerWidth="190" markerHeight="240" orient="auto"><path d="M 18,7 L9,13 L1,7 L9,1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-aggregationEnd" class="marker aggregation class" refX="1" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 18,7 L9,13 L1,7 L9,1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-aggregationStart-margin" class="marker aggregation class" refX="15" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><path d="M 18,7 L9,13 L1,7 L9,1 Z" style="stroke-width: 2;"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-aggregationEnd-margin" class="marker aggregation class" refX="1" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse"><path d="M 18,7 L9,13 L1,7 L9,1 Z" style="stroke-width: 2;"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-extensionStart" class="marker extension class" refX="18" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse"><path d="M 1,7 L18,13 V 1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-extensionEnd" class="marker extension class" refX="1" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 1,1 V 13 L18,7 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<marker id="{id}_class-extensionStart-margin" class="marker extension class" refX="18" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse" viewBox="0 0 20 14"><polygon points="10,7 18,13 18,1" style="stroke-width: 2; stroke-dasharray: 0;"></polygon></marker>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-extensionEnd-margin" class="marker extension class" refX="9" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse" viewBox="0 0 20 14"><polygon points="10,1 10,13 18,7" style="stroke-width: 2; stroke-dasharray: 0;"></polygon></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-compositionStart" class="marker composition class" refX="18" refY="7" markerWidth="190" markerHeight="240" orient="auto"><path d="M 18,7 L9,13 L1,7 L9,1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-compositionEnd" class="marker composition class" refX="1" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 18,7 L9,13 L1,7 L9,1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-compositionStart-margin" class="marker composition class" refX="15" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><path viewBox="0 0 15 15" d="M 18,7 L9,13 L1,7 L9,1 Z" style="stroke-width: 0;"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-compositionEnd-margin" class="marker composition class" refX="3.5" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse"><path d="M 18,7 L9,13 L1,7 L9,1 Z" style="stroke-width: 0;"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-dependencyStart" class="marker dependency class" refX="6" refY="7" markerWidth="190" markerHeight="240" orient="auto"><path d="M 5,7 L9,13 L1,7 L9,1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-dependencyEnd" class="marker dependency class" refX="13" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 18,7 L9,13 L14,7 L9,1 Z"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-dependencyStart-margin" class="marker dependency class" refX="4" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><path d="M 5,7 L9,13 L1,7 L9,1 Z" style="stroke-width: 0;"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-dependencyEnd-margin" class="marker dependency class" refX="16" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse"><path d="M 18,7 L9,13 L14,7 L9,1 Z" style="stroke-width: 0;"></path></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-lollipopStart" class="marker lollipop class" refX="13" refY="7" markerWidth="190" markerHeight="240" orient="auto"><circle fill="transparent" cx="7" cy="7" r="6"></circle></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-lollipopEnd" class="marker lollipop class" refX="1" refY="7" markerWidth="190" markerHeight="240" orient="auto"><circle fill="transparent" cx="7" cy="7" r="6"></circle></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-lollipopStart-margin" class="marker lollipop class" refX="13" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><circle fill="transparent" cx="7" cy="7" r="6" stroke-width="2"></circle></marker></defs>"#));
    m.push_str(&format!(r#"<defs><marker id="{id}_class-lollipopEnd-margin" class="marker lollipop class" refX="1" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><circle fill="transparent" cx="7" cy="7" r="6" stroke-width="2"></circle></marker></defs>"#));
    m
}
