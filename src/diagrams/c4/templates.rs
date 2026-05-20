//! SVG template functions for the C4 diagram renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

use super::constants::{CLOCK_ICON_PATH, COMPUTER_ICON_PATH, DATABASE_ICON_PATH};

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::{esc, fmt};

pub fn fmt_int(v: f64) -> String {
    format!("{}", v.round() as i64)
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

/// Render the forward arrowhead `<marker>` definition for C4 relationships.
pub fn marker_arrowhead(id: &str) -> String {
    format!(
        "<marker id=\"{id}-arrowhead\" refX=\"9\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"12\" markerHeight=\"12\" orient=\"auto\"><path d=\"M 0 0 L 10 5 L 0 10 z\"></path></marker>",
        id = id,
    )
}

/// Render the reverse arrowhead `<marker>` definition for C4 bidirectional relationships.
pub fn marker_arrowend(id: &str) -> String {
    format!(
        "<marker id=\"{id}-arrowend\" refX=\"1\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"12\" markerHeight=\"12\" orient=\"auto\"><path d=\"M 10 0 L 0 5 L 10 10 z\"></path></marker>",
        id = id,
    )
}

/// Render the diagram title `<text>`.
pub fn title_text(title_x: f64, text: &str) -> String {
    format!(
        "<text x=\"{title_x}\" y=\"20\">{text}</text>",
        title_x = title_x,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// SVG root
// ---------------------------------------------------------------------------

/// Render the outer SVG element for a C4 diagram.
pub fn svg_root(svg_id: &str, svg_w: f64, vb_y: f64, vb_h: f64) -> String {
    format!(
        "<svg id=\"{id}\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" \
xmlns:xlink=\"http://www.w3.org/1999/xlink\" style=\"max-width: {w}px;\" \
viewBox=\"0 {vby} {vbw} {vbh}\" role=\"graphics-document document\" aria-roledescription=\"c4\">",
        id = svg_id,
        w = fmt(svg_w),
        vby = fmt(vb_y),
        vbw = fmt(svg_w),
        vbh = fmt(vb_h),
    )
}

// ---------------------------------------------------------------------------
// Shape element rendering
// ---------------------------------------------------------------------------

/// Render the database cylinder shape (two ellipse-capped path segments).
pub fn shape_db_path(fill: &str, stroke: &str, x: f64, y: f64, half: f64, h: f64) -> String {
    format!(
        "<path fill=\"{fill}\" stroke-width=\"0.5\" stroke=\"{stroke}\" d=\"M{x},{y}c0,-10 {half},-10 {half},-10c0,0 {half},0 {half},10l0,{h}c0,10 -{half},10 -{half},10c0,0 -{half},0 -{half},-10l0,-{h}\"></path>\
<path fill=\"none\" stroke-width=\"0.5\" stroke=\"{stroke}\" d=\"M{x},{y}c0,10 {half},10 {half},10c0,0 {half},0 {half},-10\"></path>",
        fill = fill,
        stroke = stroke,
        x = fmt(x),
        y = fmt(y),
        half = fmt(half),
        h = fmt(h),
    )
}

/// Render the standard element rectangle.
pub fn shape_rect(x: f64, y: f64, fill: &str, stroke: &str, w: f64, h: f64) -> String {
    format!(
        "<rect x=\"{x}\" y=\"{y}\" fill=\"{fill}\" stroke=\"{stroke}\" width=\"{w}\" height=\"{h}\" rx=\"2.5\" ry=\"2.5\" stroke-width=\"0.5\"></rect>",
        x = fmt(x),
        y = fmt(y),
        fill = fill,
        stroke = stroke,
        w = fmt(w),
        h = fmt(h),
    )
}

/// Render the stereotype italic `<text>` element for a shape.
pub fn shape_stereo_text(fill: &str, ff: &str, tw: &str, x: f64, y: f64, text: &str) -> String {
    format!(
        "<text fill=\"{fill}\" font-family=\"{ff}\" font-size=\"{fs}\" font-style=\"italic\" lengthAdjust=\"spacing\" textLength=\"{tw}\" x=\"{x}\" y=\"{y}\">{text}</text>",
        fill = fill,
        ff = ff,
        fs = fmt_int(12.0),
        tw = tw,
        x = fmt(x),
        y = fmt(y),
        text = text,
    )
}

/// Render a person image (48×48 PNG) for a C4 element.
pub fn shape_person_image(cx: f64, y: f64, png: &str) -> String {
    format!(
        "<image width=\"48\" height=\"48\" x=\"{x}\" y=\"{y}\" xlink:href=\"data:image/png;base64,{png}\"></image>",
        x = fmt(cx - 24.0),
        y = fmt(y),
        png = png,
    )
}

/// Render the shape bold-16px label `<text>`.
pub fn shape_label_text(cx: f64, y: f64, fill: &str, ff: &str, text: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" dominant-baseline=\"middle\" fill=\"{fill}\" style=\"text-anchor: middle; font-size: 16px; font-weight: bold; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">{text}</tspan></text>",
        x = fmt(cx),
        y = fmt(y),
        fill = fill,
        ff = ff,
        text = text,
    )
}

/// Render the shape normal-14px description `<text>`.
pub fn shape_descr_text(cx: f64, y: f64, fill: &str, ff: &str, text: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" dominant-baseline=\"middle\" fill=\"{fill}\" style=\"text-anchor: middle; font-size: 14px; font-weight: normal; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">{text}</tspan></text>",
        x = fmt(cx),
        y = fmt(y),
        fill = fill,
        ff = ff,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Boundary rendering
// ---------------------------------------------------------------------------

/// Render the boundary dashed rectangle.
pub fn boundary_rect(x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        "<rect x=\"{x}\" y=\"{y}\" fill=\"none\" stroke=\"{sc}\" width=\"{w}\" height=\"{h}\" rx=\"2.5\" ry=\"2.5\" stroke-width=\"1\" stroke-dasharray=\"7.0,7.0\"></rect>",
        x = fmt(x),
        y = fmt(y),
        sc = "#444444",
        w = fmt(w),
        h = fmt(h),
    )
}

/// Render the boundary bold-16px label `<text>`.
pub fn boundary_label_text(cx: f64, y: f64, fill: &str, ff: &str, text: &str) -> String {
    format!(
        "<text x=\"{cx}\" y=\"{y}\" dominant-baseline=\"middle\" fill=\"{fill}\" style=\"text-anchor: middle; font-size: 16px; font-weight: bold; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">{text}</tspan></text>",
        cx = fmt(cx),
        y = fmt(y),
        fill = fill,
        ff = ff,
        text = text,
    )
}

/// Render the boundary type normal-14px `<text>`.
pub fn boundary_type_text(cx: f64, y: f64, fill: &str, ff: &str, text: &str) -> String {
    format!(
        "<text x=\"{cx}\" y=\"{y}\" dominant-baseline=\"middle\" fill=\"{fill}\" style=\"text-anchor: middle; font-size: 14px; font-weight: normal; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">[{text}]</tspan></text>",
        cx = fmt(cx),
        y = fmt(y),
        fill = fill,
        ff = ff,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Relationship rendering
// ---------------------------------------------------------------------------

/// Render a straight-line relationship with optional bidirectional marker.
pub fn rel_line(sx: f64, sy: f64, ex: f64, ey: f64, svg_id: &str, marker_start: &str) -> String {
    format!(
        "<line x1=\"{sx}\" y1=\"{sy}\" x2=\"{ex}\" y2=\"{ey}\" stroke-width=\"1\" stroke=\"{sc}\" marker-end=\"url(#{id}-arrowhead)\"{ms}  style=\"fill: none;\"></line>",
        sx = fmt(sx),
        sy = fmt(sy),
        ex = fmt(ex),
        ey = fmt(ey),
        sc = "#444444",
        id = svg_id,
        ms = marker_start,
    )
}

/// Render a curved (quadratic bezier) relationship.
#[allow(clippy::too_many_arguments)]
pub fn rel_curve(
    sx: f64,
    sy: f64,
    ctrl_x: f64,
    ctrl_y: f64,
    ex: f64,
    ey: f64,
    svg_id: &str,
    marker_start: &str,
) -> String {
    format!(
        "<path fill=\"none\" stroke-width=\"1\" stroke=\"{sc}\" d=\"M{sx},{sy} Q{cx},{cy} {ex},{ey}\" marker-end=\"url(#{id}-arrowhead)\"{ms}></path>",
        sc = "#444444",
        sx = fmt(sx),
        sy = fmt(sy),
        cx = fmt(ctrl_x),
        cy = fmt(ctrl_y),
        ex = fmt(ex),
        ey = fmt(ey),
        id = svg_id,
        ms = marker_start,
    )
}

/// Render the relationship label `<text>`.
pub fn rel_label(tx: f64, ty: f64, ff: &str, text: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" dominant-baseline=\"middle\" fill=\"{fill}\" style=\"text-anchor: middle; font-size: 12px; font-weight: normal; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">{text}</tspan></text>",
        x = fmt(tx),
        y = fmt(ty),
        fill = "#444444",
        ff = ff,
        text = text,
    )
}

/// Render the relationship technology label `<text>` (italic, 10px).
pub fn rel_techn_label(tx: f64, ty: f64, ff: &str, text: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" dominant-baseline=\"middle\" fill=\"{fill}\" style=\"text-anchor: middle; font-size: 10px; font-style: italic; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">{text}</tspan></text>",
        x = fmt(tx),
        y = fmt(ty),
        fill = "#444444",
        ff = ff,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Icon symbol defs
// ---------------------------------------------------------------------------

/// Render the `<defs><symbol>` block for the computer icon.
pub fn symbol_computer(id: &str) -> String {
    format!(
        "<defs><symbol id=\"{id}-computer\" width=\"24\" height=\"24\">\
         <path transform=\"scale(.5)\" d=\"{p}\"></path></symbol></defs>",
        id = id,
        p = COMPUTER_ICON_PATH,
    )
}

/// Render the `<defs><symbol>` block for the database icon.
pub fn symbol_database(id: &str) -> String {
    format!(
        "<defs><symbol id=\"{id}-database\" fill-rule=\"evenodd\" clip-rule=\"evenodd\">\
         <path transform=\"scale(.5)\" d=\"{p}\"></path></symbol></defs>",
        id = id,
        p = DATABASE_ICON_PATH,
    )
}

/// Render the `<defs><symbol>` block for the clock icon.
pub fn symbol_clock(id: &str) -> String {
    format!(
        "<defs><symbol id=\"{id}-clock\" width=\"24\" height=\"24\">\
         <path transform=\"scale(.5)\" d=\"{p}\"></path></symbol></defs>",
        id = id,
        p = CLOCK_ICON_PATH,
    )
}

// ---------------------------------------------------------------------------
// Marker defs
// ---------------------------------------------------------------------------

/// Render all four `<marker>` defs for C4 relationships.
pub fn all_markers(id: &str) -> String {
    format!(
        "<defs><marker id=\"{id}-arrowhead\" refX=\"9\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"12\" markerHeight=\"12\" orient=\"auto\"><path d=\"M 0 0 L 10 5 L 0 10 z\"></path></marker></defs>\
         <defs><marker id=\"{id}-arrowend\" refX=\"1\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"12\" markerHeight=\"12\" orient=\"auto\"><path d=\"M 10 0 L 0 5 L 10 10 z\"></path></marker></defs>\
         <defs><marker id=\"{id}-crosshead\" markerWidth=\"15\" markerHeight=\"8\" orient=\"auto\" refX=\"16\" refY=\"4\"><path fill=\"black\" stroke=\"#000000\" stroke-width=\"1px\" d=\"M 9,2 V 6 L16,4 Z\" style=\"stroke-dasharray: 0, 0;\"></path><path fill=\"none\" stroke=\"#000000\" stroke-width=\"1px\" d=\"M 0,1 L 6,7 M 6,1 L 0,7\" style=\"stroke-dasharray: 0, 0;\"></path></marker></defs>\
         <defs><marker id=\"{id}-filled-head\" refX=\"18\" refY=\"7\" markerWidth=\"20\" markerHeight=\"28\" orient=\"auto\"><path d=\"M 18,7 L9,13 L14,7 L9,1 Z\"></path></marker></defs>",
        id = id,
    )
}

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

/// Render the C4 diagram CSS block.
pub fn build_style(id: &str, ff: &str) -> String {
    format!(
        "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}\
         #{id} p{{margin:0;}}\
         #{id} .person{{stroke:hsl(240, 60%, 86.2745098039%);fill:#ECECFF;}}\
         #{id} :root{{--mermaid-font-family:{ff};}}",
        id = id,
        ff = ff,
    )
}
