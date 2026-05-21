//! SVG template functions for the sequence diagram renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for a sequence diagram.
pub fn svg_root(id: &str, max_w: u64, vbx: f64, vby: i64, vbw: u64, vbh: u64) -> String {
    format!(
        r##"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="max-width: {mw}px;" viewBox="{vbx} {vby} {vbw} {vbh}" role="graphics-document document" aria-roledescription="sequence">"##,
        id = id,
        mw = max_w,
        vbx = vbx,
        vby = vby,
        vbw = vbw,
        vbh = vbh,
    )
}

// ---------------------------------------------------------------------------
// Actor rendering
// ---------------------------------------------------------------------------

/// Render a rectangular actor box (`<rect>`).
#[allow(clippy::too_many_arguments)]
pub fn actor_rect(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    name: &str,
    cls: &str,
    pf: &str,
    pb: &str,
) -> String {
    format!(
        r##"<rect x="{x}" y="{y}" fill="{pf}" stroke="{pb}" stroke-width="1" width="{w}" height="{h}" name="{name}" rx="3" ry="3" class="actor {cls}"></rect>"##,
        x = x,
        y = y,
        w = w,
        h = h,
        name = name,
        cls = cls,
        pf = pf,
        pb = pb,
    )
}

/// Render the actor text label `<text>` centred in a box.
pub fn actor_text(cx: f64, cy: f64, font_size: f64, name: &str, tc: &str) -> String {
    format!(
        r##"<text x="{cx}" y="{cy}" dominant-baseline="central" alignment-baseline="central" class="actor actor-box" style="text-anchor: middle; font-size: {fs}px; font-weight: 400; font-family: Arial, sans-serif; fill: {tc};"><tspan x="{cx}" dy="0">{name}</tspan></text>"##,
        cx = cx,
        cy = cy,
        fs = font_size,
        name = name,
        tc = tc,
    )
}

// ---------------------------------------------------------------------------
// Lifeline
// ---------------------------------------------------------------------------

/// Render the vertical lifeline `<line>` for an actor.
pub fn lifeline(ai: usize, cx: f64, y_start: f64, y_end: f64, name: &str, pb: &str) -> String {
    format!(
        r##"<line id="actor{ai}" x1="{cx}" y1="{ys}" x2="{cx}" y2="{ye}" class="actor-line 200" stroke-width="2" stroke="{pb}" name="{name}" data-et="life-line" data-id="{name}"></line>"##,
        ai = ai,
        cx = cx,
        ys = y_start,
        ye = y_end,
        name = name,
        pb = pb,
    )
}

/// Render the `<g>` root group for a top-row participant box.
pub fn participant_root_group(ai: usize, name: &str) -> String {
    format!(
        r##"<g id="root-{ai}" data-et="participant" data-type="participant" data-id="{name}">"##,
        ai = ai,
        name = name,
    )
}

// ---------------------------------------------------------------------------
// Activation boxes
// ---------------------------------------------------------------------------

/// Render an activation bar `<rect>` on a lifeline.
pub fn activation_rect(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    cls: &str,
    fill: &str,
    stroke: &str,
) -> String {
    format!(
        r##"<rect x="{x}" y="{y}" fill="{fill}" stroke="{stroke}" width="{w}" height="{h}" class="{cls}"></rect>"##,
        x = x,
        y = y,
        w = w,
        h = h,
        cls = cls,
        fill = fill,
        stroke = stroke,
    )
}

// ---------------------------------------------------------------------------
// Notes
// ---------------------------------------------------------------------------

/// Render a note background `<rect>`.
pub fn note_rect(x: f64, y: f64, w: f64, h: f64, note_bg: &str, note_border: &str) -> String {
    format!(
        r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{note_bg}" stroke="{note_border}" class="note"></rect>"##,
        x = x,
        y = y,
        w = w,
        h = h,
        note_bg = note_bg,
        note_border = note_border,
    )
}

/// Render the text inside a note box.
pub fn note_text(tx: f64, ty: f64, font_size: u32, text: &str, text_color: &str) -> String {
    format!(
        r##"<text x="{tx}" y="{ty}" text-anchor="middle" dominant-baseline="central" fill="{text_color}" class="noteText" style="font-family: Arial, sans-serif; font-size: {fs}px; font-weight: 400;">{text}</text>"##,
        tx = tx,
        ty = ty,
        fs = font_size,
        text = text,
        text_color = text_color,
    )
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Render a straight message `<line>`.
#[allow(clippy::too_many_arguments)]
pub fn message_line(
    x1: f64,
    y: f64,
    x2: f64,
    cls: &str,
    id: usize,
    from: &str,
    to: &str,
    marker: &str,
    dash: &str,
    lc: &str,
) -> String {
    format!(
        r##"<line x1="{x1}" y1="{y}" x2="{x2}" y2="{y}" class="{cls}" data-et="message" data-id="i{id}" data-from="{from}" data-to="{to}" stroke-width="1.5" stroke="{lc}"{marker}{dash}></line>"##,
        x1 = x1,
        y = y,
        x2 = x2,
        cls = cls,
        id = id,
        from = from,
        to = to,
        marker = marker,
        dash = dash,
        lc = lc,
    )
}

/// Render a self-message cubic Bezier `<path>`.
#[allow(clippy::too_many_arguments)]
pub fn message_self_path(
    sx: f64,
    ly: f64,
    cx1: f64,
    cy1: f64,
    cx2: f64,
    cy2: f64,
    ey: f64,
    cls: &str,
    id: usize,
    from: &str,
    to: &str,
    marker: &str,
    dash: &str,
    lc: &str,
) -> String {
    format!(
        r##"<path d="M {sx},{ly} C {cx1},{cy1} {cx2},{cy2} {sx},{ey}" class="{cls}" data-et="message" data-id="i{id}" data-from="{from}" data-to="{to}" stroke-width="1.5" stroke="{lc}" fill="none"{marker}{dash}></path>"##,
        sx = sx,
        ly = ly,
        cx1 = cx1,
        cy1 = cy1,
        cx2 = cx2,
        cy2 = cy2,
        ey = ey,
        cls = cls,
        id = id,
        from = from,
        to = to,
        marker = marker,
        dash = dash,
        lc = lc,
    )
}

/// Render the message label `<text>` above the arrow line.
pub fn message_label_text(
    tx: f64,
    ty: f64,
    font_size: u32,
    text: &str,
    text_color: &str,
) -> String {
    format!(
        r##"<text x="{tx}" y="{ty}" text-anchor="middle" dominant-baseline="auto" fill="{tc}" class="messageText" style="font-family: Arial, sans-serif; font-size: {fs}px; font-weight: 400;">{text}</text>"##,
        tx = tx,
        ty = ty,
        fs = font_size,
        text = text,
        tc = text_color,
    )
}

/// Render the sequence-number filled circle for autonumber.
pub fn seq_number_circle(cx: f64, cy: f64, circle_fill: &str) -> String {
    format!(
        r##"<circle cx="{cx}" cy="{cy}" r="12" fill="{circle_fill}" class="seqnum-circle"></circle>"##,
        cx = cx,
        cy = cy,
        circle_fill = circle_fill,
    )
}

/// Render the sequence-number text inside the circle.
pub fn seq_number_text(cx: f64, ty: f64, n: usize, text_fill: &str) -> String {
    format!(
        r##"<text x="{cx}" y="{ty}" font-family="sans-serif" font-size="12px" text-anchor="middle" dominant-baseline="central" fill="{text_fill}" class="sequenceNumber">{n}</text>"##,
        cx = cx,
        ty = ty,
        n = n,
        text_fill = text_fill,
    )
}

// ---------------------------------------------------------------------------
// Control structures (loop / alt / opt / par)
// ---------------------------------------------------------------------------

/// Render the outer group wrapper for a control structure.
pub fn control_group_open(idx: usize, x1: f64, y1: f64, x2: f64, y2: f64, pb: &str) -> String {
    format!(
        r##"<g data-et="control-structure" data-id="i{idx}">
<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y1}" class="loopLine" stroke="{pb}" stroke-width="2" stroke-dasharray="2,2" fill="{pb}"></line>
<line x1="{x2}" y1="{y1}" x2="{x2}" y2="{y2}" class="loopLine" stroke="{pb}" stroke-width="2" stroke-dasharray="2,2" fill="{pb}"></line>
<line x1="{x1}" y1="{y2}" x2="{x2}" y2="{y2}" class="loopLine" stroke="{pb}" stroke-width="2" stroke-dasharray="2,2" fill="{pb}"></line>
<line x1="{x1}" y1="{y1}" x2="{x1}" y2="{y2}" class="loopLine" stroke="{pb}" stroke-width="2" stroke-dasharray="2,2" fill="{pb}"></line>"##,
        idx = idx,
        x1 = x1,
        y1 = y1,
        x2 = x2,
        y2 = y2,
        pb = pb,
    )
}

/// Render a dashed section-divider line inside a control structure.
pub fn control_section_divider(x1: f64, x2: f64, sy: f64, pb: &str) -> String {
    format!(
        r##"<line x1="{x1}" y1="{sy}" x2="{x2}" y2="{sy}" class="loopLine" stroke="{pb}" stroke-width="2" stroke-dasharray="3,3" fill="{pb}"></line>"##,
        x1 = x1,
        x2 = x2,
        sy = sy,
        pb = pb,
    )
}

/// Render the label-badge polygon and kind text for a control structure.
#[allow(clippy::too_many_arguments)]
pub fn control_badge(
    p1: &str,
    p2: &str,
    p3: &str,
    p4: &str,
    p5: &str,
    cx: f64,
    cy: f64,
    font_size: u32,
    kind: &str,
    pf: &str,
    pb: &str,
    tc: &str,
) -> String {
    format!(
        r##"<polygon points="{p1} {p2} {p3} {p4} {p5}" class="labelBox" fill="{pf}" stroke="{pb}"></polygon><text x="{cx}" y="{cy}" text-anchor="middle" dominant-baseline="middle" alignment-baseline="middle" class="labelText" style="font-family: Arial, sans-serif; font-size: {fs}px; font-weight: 400; fill: {tc};">{kind}</text>"##,
        p1 = p1,
        p2 = p2,
        p3 = p3,
        p4 = p4,
        p5 = p5,
        cx = cx,
        cy = cy,
        fs = font_size,
        kind = kind,
        pf = pf,
        pb = pb,
        tc = tc,
    )
}

/// Render the main condition label for a control structure.
pub fn control_label_text(cx: f64, cy: f64, font_size: u32, label: &str, tc: &str) -> String {
    format!(
        r##"<text x="{cx}" y="{cy}" text-anchor="middle" class="loopText" style="font-family: Arial, sans-serif; font-size: {fs}px; font-weight: 400; fill: {tc};"><tspan x="{cx}">[{label}]</tspan></text>"##,
        cx = cx,
        cy = cy,
        fs = font_size,
        label = label,
        tc = tc,
    )
}

/// Render a section-title label inside an alt/par divider.
pub fn control_section_title(cx: f64, cy: f64, font_size: u32, label: &str, tc: &str) -> String {
    format!(
        r##"<text x="{cx}" y="{cy}" text-anchor="middle" class="sectionTitle" style="font-family: Arial, sans-serif; font-size: {fs}px; font-weight: 400; fill: {tc};">[{label}]</text>"##,
        cx = cx,
        cy = cy,
        fs = font_size,
        label = label,
        tc = tc,
    )
}

// ---------------------------------------------------------------------------
// Actor-man (stick figure)
// ---------------------------------------------------------------------------

/// Render an actor-man (stick figure) participant box.
///
/// `cx` = centre x, `box_y` = top of the actor box area,
/// `idx` = unique index for element ids, `actor_width` / `actor_height` for
/// the bounding circle `width`/`height` attributes, `font_size` = label px.
#[allow(clippy::too_many_arguments)]
pub fn actor_man(
    cls: &str,
    esc_name: &str,
    idx: usize,
    cx: f64,
    cy: f64, // circle centre y
    r: f64,
    ts: f64, // torso start y
    te: f64, // torso end y
    al: f64, // arm left x
    ar: f64, // arm right x
    ay: f64, // arm y
    ll: f64, // leg left x
    rl: f64, // leg right x
    ls: f64, // leg start y
    le: f64, // leg end y
    ty: f64, // text y
    actor_width: f64,
    actor_height: f64,
    font_size: u32,
    pf: &str,
    pb: &str,
    tc: &str,
) -> String {
    format!(
        concat!(
            r##"<g class="actor-man {cls}" name="{name}" data-et="participant" data-type="actor" data-id="{name}" style="stroke: {pb};">"##,
            r##"<line id="actor-man-torso{idx}" x1="{cx}" y1="{ts}" x2="{cx}" y2="{te}" stroke-width="2"></line>"##,
            r##"<line id="actor-man-arms{idx}" x1="{al}" y1="{ay}" x2="{ar}" y2="{ay}" stroke-width="2"></line>"##,
            r##"<line x1="{ll}" y1="{le}" x2="{cx}" y2="{ls}" stroke-width="2"></line>"##,
            r##"<line x1="{cx}" y1="{ls}" x2="{rl}" y2="{le}" stroke-width="2"></line>"##,
            r##"<circle cx="{cx}" cy="{cy}" r="{r}" width="{w}" height="{h}" fill="{pf}" stroke-width="2"></circle>"##,
            r##"<text x="{cx}" y="{ty}" dominant-baseline="central" alignment-baseline="central" class="actor actor-man" fill="{tc}" stroke="none" style="text-anchor: middle; font-size: {fs}px; font-weight: 400; font-family: Arial, sans-serif;"><tspan x="{cx}" dy="0">{name}</tspan></text>"##,
            r##"</g>"##,
        ),
        cls = cls,
        name = esc_name,
        idx = idx,
        cx = cx,
        cy = cy,
        r = r,
        ts = ts,
        te = te,
        al = al,
        ar = ar,
        ay = ay,
        ll = ll,
        rl = rl,
        ls = ls,
        le = le,
        w = actor_width,
        h = actor_height,
        ty = ty,
        fs = font_size,
        pf = pf,
        pb = pb,
        tc = tc,
    )
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::esc;

// ---------------------------------------------------------------------------
// Arrow marker defs
// ---------------------------------------------------------------------------

pub fn defs_svg(id: &str, line_color: &str, marker_fill: &str) -> String {
    format!(
        r##"<defs><marker id="{id}-arrowhead" refX="7.9" refY="5" markerUnits="userSpaceOnUse" markerWidth="12" markerHeight="12" orient="auto-start-reverse"><path fill="{mf}" d="M -1 0 L 10 5 L 0 10 z"></path></marker></defs>
<defs><marker id="{id}-crosshead" markerWidth="15" markerHeight="8" orient="auto" refX="4" refY="4.5"><path fill="none" stroke="{mf}" stroke-width="1pt" d="M 1,2 L 6,7 M 6,2 L 1,7" style="stroke-dasharray: 0, 0;"></path></marker></defs>
<defs><marker id="{id}-filled-head" refX="15.5" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path fill="{mf}" d="M 18,7 L9,13 L14,7 L9,1 Z"></path></marker></defs>
<defs><marker id="{id}-sequencenumber" refX="15" refY="15" markerWidth="60" markerHeight="40" orient="auto"><circle cx="15" cy="15" r="6" fill="{mf}"></circle></marker></defs>
<defs><marker id="{id}-stickTopArrowHead" refX="7.5" refY="7" markerUnits="userSpaceOnUse" markerWidth="12" markerHeight="12" orient="auto-start-reverse"><path d="M 0 0 L 7 7" stroke="{lc}" stroke-width="1.5" fill="none"></path></marker></defs>
<defs><marker id="{id}-stickBottomArrowHead" refX="7.5" refY="0" markerUnits="userSpaceOnUse" markerWidth="12" markerHeight="12" orient="auto-start-reverse"><path d="M 0 7 L 7 0" stroke="{lc}" stroke-width="1.5" fill="none"></path></marker></defs>"##,
        id = id,
        lc = line_color,
        mf = marker_fill,
    )
}
