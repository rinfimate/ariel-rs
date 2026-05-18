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
        r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="max-width: {mw}px;" viewBox="{vbx} {vby} {vbw} {vbh}" role="graphics-document document" aria-roledescription="sequence">"#,
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
pub fn actor_rect(x: f64, y: f64, w: f64, h: f64, name: &str, cls: &str) -> String {
    format!(
        r##"<rect x="{x}" y="{y}" fill="#eaeaea" stroke="#666" width="{w}" height="{h}" name="{name}" rx="3" ry="3" class="actor {cls}"></rect>"##,
        x = x,
        y = y,
        w = w,
        h = h,
        name = name,
        cls = cls,
    )
}

/// Render the actor text label `<text>` centred in a box.
pub fn actor_text(cx: f64, cy: f64, font_size: f64, name: &str) -> String {
    format!(
        r#"<text x="{cx}" y="{cy}" dominant-baseline="central" alignment-baseline="central" class="actor actor-box" style="text-anchor: middle; font-size: {fs}px; font-weight: 400; font-family: Arial, sans-serif;"><tspan x="{cx}" dy="0">{name}</tspan></text>"#,
        cx = cx,
        cy = cy,
        fs = font_size,
        name = name,
    )
}

// ---------------------------------------------------------------------------
// Lifeline
// ---------------------------------------------------------------------------

/// Render the vertical lifeline `<line>` for an actor.
pub fn lifeline(ai: usize, cx: f64, y_start: f64, y_end: f64, name: &str) -> String {
    format!(
        r##"<line id="actor{ai}" x1="{cx}" y1="{ys}" x2="{cx}" y2="{ye}" class="actor-line 200" stroke-width="0.5px" style="stroke:#9370DB;" name="{name}" data-et="life-line" data-id="{name}"></line>"##,
        ai = ai,
        cx = cx,
        ys = y_start,
        ye = y_end,
        name = name,
    )
}

/// Render the `<g>` root group for a top-row participant box.
pub fn participant_root_group(ai: usize, name: &str) -> String {
    format!(
        r#"<g id="root-{ai}" data-et="participant" data-type="participant" data-id="{name}">"#,
        ai = ai,
        name = name,
    )
}

// ---------------------------------------------------------------------------
// Activation boxes
// ---------------------------------------------------------------------------

/// Render an activation bar `<rect>` on a lifeline.
pub fn activation_rect(x: f64, y: f64, w: f64, h: f64, cls: &str) -> String {
    format!(
        r##"<rect x="{x}" y="{y}" fill="#f4f4f4" stroke="#666" width="{w}" height="{h}" class="{cls}"></rect>"##,
        x = x,
        y = y,
        w = w,
        h = h,
        cls = cls,
    )
}

// ---------------------------------------------------------------------------
// Notes
// ---------------------------------------------------------------------------

/// Render a note background `<rect>`.
pub fn note_rect(x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" class="note"></rect>"#,
        x = x,
        y = y,
        w = w,
        h = h,
    )
}

/// Render the text inside a note box.
pub fn note_text(tx: f64, ty: f64, font_size: u32, text: &str) -> String {
    format!(
        r#"<text x="{tx}" y="{ty}" text-anchor="middle" dominant-baseline="central" class="noteText" style="font-family: Arial, sans-serif; font-size: {fs}px; font-weight: 400;">{text}</text>"#,
        tx = tx,
        ty = ty,
        fs = font_size,
        text = text,
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
) -> String {
    format!(
        r#"<line x1="{x1}" y1="{y}" x2="{x2}" y2="{y}" class="{cls}" data-et="message" data-id="i{id}" data-from="{from}" data-to="{to}" stroke-width="2" stroke="none"{marker}{dash}></line>"#,
        x1 = x1,
        y = y,
        x2 = x2,
        cls = cls,
        id = id,
        from = from,
        to = to,
        marker = marker,
        dash = dash,
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
) -> String {
    format!(
        r#"<path d="M {sx},{ly} C {cx1},{cy1} {cx2},{cy2} {sx},{ey}" class="{cls}" data-et="message" data-id="i{id}" data-from="{from}" data-to="{to}" stroke-width="2" stroke="none"{marker}{dash}></path>"#,
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
    )
}

/// Render the message label `<text>` above the arrow line.
pub fn message_label_text(tx: f64, ty: f64, font_size: u32, text: &str) -> String {
    format!(
        r#"<text x="{tx}" y="{ty}" text-anchor="middle" dominant-baseline="auto" class="messageText" style="font-family: Arial, sans-serif; font-size: {fs}px; font-weight: 400;">{text}</text>"#,
        tx = tx,
        ty = ty,
        fs = font_size,
        text = text,
    )
}

/// Render the sequence-number filled circle for autonumber.
pub fn seq_number_circle(cx: f64, cy: f64) -> String {
    format!(
        r##"<circle cx="{cx}" cy="{cy}" r="12" fill="#333" class="seqnum-circle"></circle>"##,
        cx = cx,
        cy = cy,
    )
}

/// Render the sequence-number text inside the circle.
pub fn seq_number_text(cx: f64, ty: f64, n: usize) -> String {
    format!(
        r#"<text x="{cx}" y="{ty}" font-family="sans-serif" font-size="12px" text-anchor="middle" dominant-baseline="central" fill="white" class="sequenceNumber">{n}</text>"#,
        cx = cx,
        ty = ty,
        n = n,
    )
}

// ---------------------------------------------------------------------------
// Control structures (loop / alt / opt / par)
// ---------------------------------------------------------------------------

/// Render the outer group wrapper for a control structure.
pub fn control_group_open(idx: usize, x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    format!(
        r#"<g data-et="control-structure" data-id="i{idx}">
<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y1}" class="loopLine"></line>
<line x1="{x2}" y1="{y1}" x2="{x2}" y2="{y2}" class="loopLine"></line>
<line x1="{x1}" y1="{y2}" x2="{x2}" y2="{y2}" class="loopLine"></line>
<line x1="{x1}" y1="{y1}" x2="{x1}" y2="{y2}" class="loopLine"></line>"#,
        idx = idx,
        x1 = x1,
        y1 = y1,
        x2 = x2,
        y2 = y2,
    )
}

/// Render a dashed section-divider line inside a control structure.
pub fn control_section_divider(x1: f64, x2: f64, sy: f64) -> String {
    format!(
        r#"<line x1="{x1}" y1="{sy}" x2="{x2}" y2="{sy}" class="loopLine" style="stroke-dasharray: 3, 3;"></line>"#,
        x1 = x1,
        x2 = x2,
        sy = sy,
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
) -> String {
    format!(
        r##"<polygon points="{p1} {p2} {p3} {p4} {p5}" class="labelBox" fill="#ECECFF" stroke="#9370DB"></polygon><text x="{cx}" y="{cy}" text-anchor="middle" dominant-baseline="middle" alignment-baseline="middle" class="labelText" style="font-family: Arial, sans-serif; font-size: {fs}px; font-weight: 400;">{kind}</text>"##,
        p1 = p1,
        p2 = p2,
        p3 = p3,
        p4 = p4,
        p5 = p5,
        cx = cx,
        cy = cy,
        fs = font_size,
        kind = kind,
    )
}

/// Render the main condition label for a control structure.
pub fn control_label_text(cx: f64, cy: f64, font_size: u32, label: &str) -> String {
    format!(
        r#"<text x="{cx}" y="{cy}" text-anchor="middle" class="loopText" style="font-family: Arial, sans-serif; font-size: {fs}px; font-weight: 400;"><tspan x="{cx}">[{label}]</tspan></text>"#,
        cx = cx,
        cy = cy,
        fs = font_size,
        label = label,
    )
}

/// Render a section-title label inside an alt/par divider.
pub fn control_section_title(cx: f64, cy: f64, font_size: u32, label: &str) -> String {
    format!(
        r#"<text x="{cx}" y="{cy}" text-anchor="middle" class="sectionTitle" style="font-family: Arial, sans-serif; font-size: {fs}px; font-weight: 400;">[{label}]</text>"#,
        cx = cx,
        cy = cy,
        fs = font_size,
        label = label,
    )
}
