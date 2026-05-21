//! SVG template functions for the class diagram renderer.
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

/// Render the outer SVG wrapper for a class diagram.
pub fn svg_root(id: &str, w: &str, h: &str) -> String {
    format!(
        r##"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" class="classDiagram" style="max-width: {w}px;" viewBox="0 0 {w} {h}" role="graphics-document document" aria-roledescription="class">"##,
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
    stroke: &str,
    dasharray: &str,
    marker_start: &str,
    marker_end: &str,
) -> String {
    format!(
        r##"<path d="{d}" id="{eid}" class="{cls}" fill="none" stroke="{stroke}" stroke-width="1" stroke-dasharray="{dasharray}" data-edge="true" data-et="edge" data-id="{eid}" data-look="classic"{ms}{me}></path>"##,
        d = path_d,
        eid = edge_id,
        cls = classes,
        stroke = stroke,
        dasharray = dasharray,
        ms = marker_start,
        me = marker_end,
    )
}

// ---------------------------------------------------------------------------
// Edge labels
// ---------------------------------------------------------------------------

/// Render a class diagram edge label as a plain SVG `<text>` element.
pub fn edge_label_text(
    mx: &str,
    my: &str,
    ox: &str,
    fw: &str,
    pf: &str,
    ff: &str,
    text_color: &str,
    text: &str,
) -> String {
    format!(
        r##"<g class="edgeLabel" transform="translate({mx}, {my})"><rect x="{ox}" y="-12" width="{fw}" height="24" fill="{pf}" stroke="none"></rect><text x="0" y="5" text-anchor="middle" font-family="{ff}" font-size="16" fill="{text_color}">{text}</text></g>"##,
        mx = mx,
        my = my,
        ox = ox,
        fw = fw,
        pf = pf,
        ff = ff,
        text_color = text_color,
        text = text,
    )
}

/// Render an empty edge label placeholder.
pub fn edge_label_empty() -> String {
    r##"<g class="edgeLabel"></g>"##.to_string()
}

/// Render a terminal label (cardinality) as a plain SVG `<text>` element.
/// Target-end cardinality label: text left-aligned, anchored at arrowhead x.
pub fn terminal_label_text_target(
    cx: &str,
    cy: &str,
    ff: &str,
    fill: &str,
    text: &str,
    _text_w: f64,
) -> String {
    format!(
        r##"<g class="edgeTerminals" transform="translate({cx}, {cy})"><text x="2" y="0" text-anchor="start" dominant-baseline="middle" font-family="{ff}" font-size="11" fill="{fill}">{text}</text></g>"##,
        cx = cx,
        cy = cy,
        ff = ff,
        fill = fill,
        text = text,
    )
}

/// Source-end cardinality label: text centered at group position.
pub fn terminal_label_text_source(
    cx: &str,
    cy: &str,
    ff: &str,
    fill: &str,
    text: &str,
    _text_w: f64,
) -> String {
    format!(
        r##"<g class="edgeTerminals" transform="translate({cx}, {cy})"><text x="0" y="0" text-anchor="middle" dominant-baseline="middle" font-family="{ff}" font-size="11" fill="{fill}">{text}</text></g>"##,
        cx = cx,
        cy = cy,
        ff = ff,
        fill = fill,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Node rendering
// ---------------------------------------------------------------------------

/// Render the outer `<g class="node">` wrapper for a class node.
pub fn node_group(dom_id: &str, cx: &str, cy: &str) -> String {
    format!(
        r##"<g class="node default " id="{did}" data-look="classic" transform="translate({cx}, {cy})">"##,
        did = dom_id,
        cx = cx,
        cy = cy,
    )
}

/// Render the filled outer path for the class box shadow layer.
pub fn node_outer_path(x1: &str, y1: &str, x2: &str, y2: &str, pf: &str) -> String {
    format!(
        r##"<g class="basic label-container outer-path"><path d="M{x1} {y1} L{x2} {y1} L{x2} {y2} L{x1} {y2}" stroke="none" stroke-width="0" fill="{pf}" style=""></path>"##,
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
        r##"<path d="M{x1} {y1} C{cx1} {y1},{cx2} {y1},{x2} {y1} M{x2} {y1} C{x2} {cy1},{x2} {cy2},{x2} {y2} M{x2} {y2} C{cx3} {y2},{cx4} {y2},{x1} {y2} M{x1} {y2} C{x1} {cy3},{x1} {cy4},{x1} {y1}" stroke="{pb}" stroke-width="1.3" fill="none" stroke-dasharray="0 0" style=""></path></g>"##,
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
        r##"<g class="annotation-group text" transform="translate(0, {})">"##,
        y,
    )
}

/// Render a single annotation row as plain SVG `<text>`.
pub fn annotation_text(y: &str, fs: f64, pb: &str, text: &str) -> String {
    format!(
        r##"<text x="0" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="Arial,sans-serif" font-size="{fs}" fill="{pb}">{text}</text>"##,
        y = y,
        fs = fs,
        pb = pb,
        text = text,
    )
}

/// Render the label group (class name) as plain SVG `<text>`.
pub fn label_group_text(ox: &str, gy: &str, hw: &str, fs: f64, pb: &str, text: &str) -> String {
    format!(
        r##"<g class="label-group text" transform="translate({ox}, {gy})"><text x="{hw}" y="0" text-anchor="middle" dominant-baseline="middle" font-family="Arial,sans-serif" font-size="{fs}" fill="{pb}" font-weight="bold">{text}</text></g>"##,
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
        r##"<g class="members-group text" transform="translate({ox}, {gy})">"##,
        ox = ox,
        gy = gy,
    )
}

/// Render the methods group wrapper `<g>`.
pub fn methods_group(ox: &str, gy: &str) -> String {
    format!(
        r##"<g class="methods-group text" transform="translate({ox}, {gy})">"##,
        ox = ox,
        gy = gy,
    )
}

/// Render a member/method row as plain SVG `<text>`.
pub fn member_row_text(y: &str, fs: f64, pb: &str, text: &str) -> String {
    format!(
        r##"<text x="0" y="{y}" dominant-baseline="middle" font-family="Arial,sans-serif" font-size="{fs}" fill="{pb}">{text}</text>"##,
        y = y,
        fs = fs,
        pb = pb,
        text = text,
    )
}

/// Render a class box divider `<path>` (cubic bezier).
pub fn divider_path(x1: &str, y: &str, cx1: &str, cx2: &str, x2: &str, pb: &str) -> String {
    format!(
        r##"<g class="divider" style=""><path d="M{x1} {y} C{cx1} {y},{cx2} {y},{x2} {y}" stroke="{pb}" stroke-width="1.3" fill="none" stroke-dasharray="0 0" style=""></path></g>"##,
        x1 = x1,
        y = y,
        cx1 = cx1,
        cx2 = cx2,
        x2 = x2,
        pb = pb,
    )
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

pub fn build_markers(id: &str, primary_color: &str, line_color: &str) -> String {
    let mut m = String::new();
    // aggregation: fill=transparent, stroke=line_color
    m.push_str(&format!(r##"<defs><marker id="{id}_class-aggregationStart" class="marker aggregation class" refX="18" refY="7" markerWidth="190" markerHeight="240" orient="auto"><path d="M 18,7 L9,13 L1,7 L9,1 Z" fill="transparent" stroke="{line_color}" stroke-width="1"></path></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-aggregationEnd" class="marker aggregation class" refX="1" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 18,7 L9,13 L1,7 L9,1 Z" fill="transparent" stroke="{line_color}" stroke-width="1"></path></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-aggregationStart-margin" class="marker aggregation class" refX="15" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><path d="M 18,7 L9,13 L1,7 L9,1 Z" fill="transparent" stroke="{line_color}" stroke-width="2"></path></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-aggregationEnd-margin" class="marker aggregation class" refX="1" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse"><path d="M 18,7 L9,13 L1,7 L9,1 Z" fill="transparent" stroke="{line_color}" stroke-width="2"></path></marker></defs>"##));
    // extension: fill=transparent, stroke=line_color
    m.push_str(&format!(r##"<defs><marker id="{id}_class-extensionStart" class="marker extension class" refX="18" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse"><path d="M 1,7 L18,13 V 1 Z" fill="transparent" stroke="{line_color}" stroke-width="1"></path></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-extensionEnd" class="marker extension class" refX="1" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 1,1 V 13 L18,7 Z" fill="transparent" stroke="{line_color}" stroke-width="1"></path></marker></defs>"##));
    m.push_str(&format!(r##"<marker id="{id}_class-extensionStart-margin" class="marker extension class" refX="18" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse" viewBox="0 0 20 14"><polygon points="10,7 18,13 18,1" fill="transparent" stroke="{line_color}" stroke-width="2" stroke-dasharray="0"></polygon></marker>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-extensionEnd-margin" class="marker extension class" refX="9" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse" viewBox="0 0 20 14"><polygon points="10,1 10,13 18,7" fill="transparent" stroke="{line_color}" stroke-width="2" stroke-dasharray="0"></polygon></marker></defs>"##));
    // composition: fill=line_color, stroke=line_color
    m.push_str(&format!(r##"<defs><marker id="{id}_class-compositionStart" class="marker composition class" refX="18" refY="7" markerWidth="190" markerHeight="240" orient="auto"><path d="M 18,7 L9,13 L1,7 L9,1 Z" fill="{line_color}" stroke="{line_color}" stroke-width="1"></path></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-compositionEnd" class="marker composition class" refX="1" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 18,7 L9,13 L1,7 L9,1 Z" fill="{line_color}" stroke="{line_color}" stroke-width="1"></path></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-compositionStart-margin" class="marker composition class" refX="15" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><path viewBox="0 0 15 15" d="M 18,7 L9,13 L1,7 L9,1 Z" fill="{line_color}" stroke="{line_color}" stroke-width="0"></path></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-compositionEnd-margin" class="marker composition class" refX="3.5" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse"><path d="M 18,7 L9,13 L1,7 L9,1 Z" fill="{line_color}" stroke="{line_color}" stroke-width="0"></path></marker></defs>"##));
    // dependency: fill=line_color, stroke=line_color
    m.push_str(&format!(r##"<defs><marker id="{id}_class-dependencyStart" class="marker dependency class" refX="6" refY="7" markerWidth="190" markerHeight="240" orient="auto"><path d="M 5,7 L9,13 L1,7 L9,1 Z" fill="{line_color}" stroke="{line_color}" stroke-width="1"></path></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-dependencyEnd" class="marker dependency class" refX="13" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 18,7 L9,13 L14,7 L9,1 Z" fill="{line_color}" stroke="{line_color}" stroke-width="1"></path></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-dependencyStart-margin" class="marker dependency class" refX="4" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><path d="M 5,7 L9,13 L1,7 L9,1 Z" fill="{line_color}" stroke="{line_color}" stroke-width="0"></path></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-dependencyEnd-margin" class="marker dependency class" refX="16" refY="7" markerWidth="20" markerHeight="28" orient="auto" markerUnits="userSpaceOnUse"><path d="M 18,7 L9,13 L14,7 L9,1 Z" fill="{line_color}" stroke="{line_color}" stroke-width="0"></path></marker></defs>"##));
    // lollipop: fill=primary_color, stroke=line_color
    m.push_str(&format!(r##"<defs><marker id="{id}_class-lollipopStart" class="marker lollipop class" refX="13" refY="7" markerWidth="190" markerHeight="240" orient="auto"><circle fill="{primary_color}" stroke="{line_color}" stroke-width="1" cx="7" cy="7" r="6"></circle></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-lollipopEnd" class="marker lollipop class" refX="1" refY="7" markerWidth="190" markerHeight="240" orient="auto"><circle fill="{primary_color}" stroke="{line_color}" stroke-width="1" cx="7" cy="7" r="6"></circle></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-lollipopStart-margin" class="marker lollipop class" refX="13" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><circle fill="{primary_color}" stroke="{line_color}" stroke-width="2" cx="7" cy="7" r="6"></circle></marker></defs>"##));
    m.push_str(&format!(r##"<defs><marker id="{id}_class-lollipopEnd-margin" class="marker lollipop class" refX="1" refY="7" markerWidth="190" markerHeight="240" orient="auto" markerUnits="userSpaceOnUse"><circle fill="{primary_color}" stroke="{line_color}" stroke-width="2" cx="7" cy="7" r="6"></circle></marker></defs>"##));
    m
}
