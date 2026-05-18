//! SVG template functions for the C4 diagram renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

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

// ---------------------------------------------------------------------------
// Boundary rendering
// ---------------------------------------------------------------------------

/// Render the boundary rectangle with dashed stroke.
pub fn boundary_rect(rx: f64, ry: f64, rw: f64, rh: f64) -> String {
    format!(
        "<rect x=\"{rx}\" y=\"{ry}\" fill=\"none\" stroke=\"#444444\" width=\"{rw}\" height=\"{rh}\" rx=\"2.5\" ry=\"2.5\" stroke-width=\"1\" stroke-dasharray=\"7.0,7.0\"></rect>",
        rx = rx, ry = ry, rw = rw, rh = rh,
    )
}

// ---------------------------------------------------------------------------
// Element rendering
// ---------------------------------------------------------------------------

/// Render the element background rectangle.
pub fn element_rect(ex: f64, ey: f64, fill: &str, stroke: &str, ew: f64, eh: f64) -> String {
    format!(
        "<rect x=\"{ex}\" y=\"{ey}\" fill=\"{fill}\" stroke=\"{stroke}\" width=\"{ew}\" height=\"{eh}\" rx=\"2.5\" ry=\"2.5\" stroke-width=\"0.5\"></rect>",
        ex = ex, ey = ey, fill = fill, stroke = stroke, ew = ew, eh = eh,
    )
}

/// Render a Person element image (48×48 base64 PNG icon).
pub fn person_image(img_x: f64, img_y: f64, png: &str) -> String {
    format!(
        "<image width=\"48\" height=\"48\" x=\"{img_x}\" y=\"{img_y}\" xlink:href=\"data:image/png;base64,{png}\"></image>",
        img_x = img_x, img_y = img_y, png = png,
    )
}

// ---------------------------------------------------------------------------
// Text elements
// ---------------------------------------------------------------------------

/// Render a centred bold 16px `<text>` element (used for element names and boundary labels).
pub fn center_text_bold16(cx: f64, y: f64, fill: &str, text: &str, ff: &str) -> String {
    format!(
        "<text x=\"{cx}\" y=\"{y}\" dominant-baseline=\"middle\" fill=\"{fill}\" style=\"text-anchor: middle; font-size: 16px; font-weight: bold; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">{text}</tspan></text>",
        cx = cx, y = y, fill = fill, text = text, ff = ff,
    )
}

/// Render a centred normal 14px `<text>` element (used for descriptions and type labels).
pub fn center_text_normal14(cx: f64, y: f64, fill: &str, text: &str, ff: &str) -> String {
    format!(
        "<text x=\"{cx}\" y=\"{y}\" dominant-baseline=\"middle\" fill=\"{fill}\" style=\"text-anchor: middle; font-size: 14px; font-weight: normal; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">{text}</tspan></text>",
        cx = cx, y = y, fill = fill, text = text, ff = ff,
    )
}

/// Render a stereotype italic 12px `<text>` element.
pub fn stereo_text(
    fill: &str,
    ff: &str,
    stereo_len: u32,
    stereo_x: f64,
    stereo_y: f64,
    text: &str,
) -> String {
    format!(
        "<text fill=\"{fill}\" font-family=\"{ff}\" font-size=\"12\" font-style=\"italic\" lengthAdjust=\"spacing\" textLength=\"{stereo_len}\" x=\"{stereo_x}\" y=\"{stereo_y}\">{text}</text>",
        fill = fill, ff = ff, stereo_len = stereo_len, stereo_x = stereo_x, stereo_y = stereo_y, text = text,
    )
}

// ---------------------------------------------------------------------------
// Relationship rendering
// ---------------------------------------------------------------------------

/// Render a straight-line relationship as an SVG `<line>`.
pub fn rel_line(sx: f64, sy: f64, ex: f64, ey: f64, svg_id: &str, marker_start: &str) -> String {
    format!(
        "<line x1=\"{sx}\" y1=\"{sy}\" x2=\"{ex}\" y2=\"{ey}\" stroke-width=\"1\" stroke=\"#444444\" marker-end=\"url(#{svg_id}-arrowhead)\"{ms}  style=\"fill: none;\"></line>",
        sx = sx, sy = sy, ex = ex, ey = ey, svg_id = svg_id, ms = marker_start,
    )
}

/// Render a curved relationship as an SVG quadratic bezier `<path>`.
pub fn rel_curve(sx: f64, sy: f64, qx: f64, qy: f64, ex: f64, ey: f64, svg_id: &str) -> String {
    format!(
        "<path fill=\"none\" stroke-width=\"1\" stroke=\"#444444\" d=\"M{sx},{sy} Q{qx},{qy} {ex},{ey}\" marker-end=\"url(#{svg_id}-arrowhead)\"></path>",
        sx = sx, sy = sy, qx = qx, qy = qy, ex = ex, ey = ey, svg_id = svg_id,
    )
}

/// Render the relationship label `<text>`.
pub fn rel_label(lbl_x: f64, lbl_y: f64, ff: &str, text: &str) -> String {
    format!(
        "<text x=\"{lbl_x}\" y=\"{lbl_y}\" dominant-baseline=\"middle\" fill=\"#444444\" style=\"text-anchor: middle; font-size: 12px; font-weight: normal; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">{text}</tspan></text>",
        lbl_x = lbl_x, lbl_y = lbl_y, ff = ff, text = text,
    )
}

/// Render the relationship technology label `<text>` (italic, 10px).
pub fn rel_techn_label(lbl_x: f64, lbl_y: f64, ff: &str, text: &str) -> String {
    format!(
        "<text x=\"{lbl_x}\" y=\"{lbl_y}\" dominant-baseline=\"middle\" fill=\"#444444\" style=\"text-anchor: middle; font-size: 10px; font-style: italic; font-family: {ff};\"><tspan dy=\"0\" alignment-baseline=\"mathematical\">[{text}]</tspan></text>",
        lbl_x = lbl_x, lbl_y = lbl_y, ff = ff, text = text,
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
