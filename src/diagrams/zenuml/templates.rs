//! SVG template functions for the ZenUML renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for a ZenUML diagram (includes arrow markers).
pub fn svg_root(total_w: f64, total_h: f64, content: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.1} {:.1}" width="100%" style="max-width:{:.0}px"><defs><marker id="zenuml-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M 0 0 L 10 5 L 0 10 z" class="zenuml-arrow-head"/></marker><marker id="zenuml-arrow-open" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M 0 0 L 10 5 L 0 10" class="zenuml-arrow-head-open"/></marker></defs>{}</svg>"#,
        total_w, total_h, total_w, content,
    )
}

/// Render the empty SVG placeholder when no participants are present.
pub fn empty_svg(title: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 60"><text x="10" y="30" font-size="14">{}</text></svg>"#,
        title,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element.
pub fn title_text(cx: f64, title: &str) -> String {
    format!(
        r#"<text class="zenuml-title" x="{:.1}" y="20" text-anchor="middle" font-size="16" font-weight="bold">{}</text>"#,
        cx, title,
    )
}

// ---------------------------------------------------------------------------
// Participant boxes
// ---------------------------------------------------------------------------

/// Render a participant `<rect>` background.
pub fn participant_rect(class: &str, x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        r#"<rect class="{}" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="4"/>"#,
        class, x, y, w, h,
    )
}

/// Render the actor head `<circle>`.
pub fn actor_head(cx: f64, cy: f64) -> String {
    format!(
        r#"<circle class="zenuml-actor-head" cx="{:.1}" cy="{:.1}" r="6"/>"#,
        cx, cy,
    )
}

/// Render the database-top ellipse.
pub fn db_top_ellipse(cx: f64, cy: f64, rx: f64) -> String {
    format!(
        r#"<ellipse class="zenuml-db-top" cx="{:.1}" cy="{:.1}" rx="{:.1}" ry="4"/>"#,
        cx, cy, rx,
    )
}

/// Render the participant label `<text>`.
pub fn participant_label(x: f64, y: f64, font_size: f64, label: &str) -> String {
    format!(
        r#"<text class="zenuml-part-label" x="{:.1}" y="{:.1}" text-anchor="middle" font-size="{}">{}</text>"#,
        x, y, font_size, label,
    )
}

// ---------------------------------------------------------------------------
// Lifelines
// ---------------------------------------------------------------------------

/// Render a participant lifeline `<line>`.
pub fn lifeline(cx: f64, y_top: f64, y_bottom: f64) -> String {
    format!(
        r#"<line class="zenuml-lifeline" x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}"/>"#,
        cx, y_top, cx, y_bottom,
    )
}

// ---------------------------------------------------------------------------
// Messages / arrows
// ---------------------------------------------------------------------------

/// Render a sync or async message `<line>` with arrowhead.
pub fn message_line(dash_class: &str, from_x: f64, y: f64, to_x: f64) -> String {
    format!(
        r#"<line class="{}" x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" marker-end="url(#zenuml-arrow)"/>"#,
        dash_class, from_x, y, to_x, y,
    )
}

/// Render the label background `<rect>` for a message.
pub fn message_label_bg(x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        r#"<rect class="zenuml-label-bg" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}"/>"#,
        x, y, w, h,
    )
}

/// Render the message label `<text>`.
pub fn message_label_text(cx: f64, y: f64, font_size: f64, label: &str) -> String {
    format!(
        r#"<text class="zenuml-msg-label" x="{:.1}" y="{:.1}" text-anchor="middle" font-size="{}">{}</text>"#,
        cx, y, font_size, label,
    )
}

/// Render a return (dashed) `<line>` with open arrowhead.
pub fn return_line(from_x: f64, y: f64, to_x: f64) -> String {
    format!(
        r#"<line class="zenuml-return" x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" marker-end="url(#zenuml-arrow-open)"/>"#,
        from_x, y, to_x, y,
    )
}

/// Render the return value label `<text>`.
pub fn return_label_text(cx: f64, y: f64, font_size: f64, label: &str) -> String {
    format!(
        r#"<text class="zenuml-msg-label" x="{:.1}" y="{:.1}" text-anchor="middle" font-size="{}">{}</text>"#,
        cx, y, font_size, label,
    )
}

/// Render a self-call cubic Bezier `<path>`.
pub fn self_call_path(lx: f64, arrow_y: f64) -> String {
    format!(
        r#"<path class="zenuml-arrow" d="M {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1}" fill="none" marker-end="url(#zenuml-arrow)"/>"#,
        lx,
        arrow_y,
        lx + 30.0,
        arrow_y - 10.0,
        lx,
        arrow_y + 20.0,
    )
}

/// Render the label for a self-call.
pub fn self_call_label(lx: f64, y: f64, font_size: f64, label: &str) -> String {
    format!(
        r#"<text class="zenuml-msg-label" x="{:.1}" y="{:.1}" font-size="{}">{}</text>"#,
        lx + 35.0,
        y,
        font_size,
        label,
    )
}

// ---------------------------------------------------------------------------
// Control blocks
// ---------------------------------------------------------------------------

/// Render a control-structure block `<rect>`.
pub fn block_rect(class: &str, x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        r#"<rect class="{}" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}"/>"#,
        class, x, y, w, h,
    )
}

/// Render the block header label `<text>`.
pub fn block_label(x: f64, y: f64, font_size: f64, header: &str) -> String {
    format!(
        r#"<text class="zenuml-block-label" x="{:.1}" y="{:.1}" font-size="{}">{}</text>"#,
        x, y, font_size, header,
    )
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

/// Render a comment `<text>` element (italic styling via CSS class).
pub fn comment_text(x: f64, y: f64, font_size: f64, text: &str) -> String {
    format!(
        r#"<text class="zenuml-comment" x="{:.1}" y="{:.1}" font-size="{}" font-style="italic">{}</text>"#,
        x, y, font_size, text,
    )
}

// ---------------------------------------------------------------------------
// Creation arrow
// ---------------------------------------------------------------------------

/// Render a "new X" creation message arrow label `<text>`.
#[allow(dead_code)]
pub fn creation_label(cx: f64, y: f64, font_size: f64, label: &str) -> String {
    format!(
        r#"<text class="zenuml-msg-label" x="{:.1}" y="{:.1}" text-anchor="middle" font-size="{}">{}</text>"#,
        cx, y, font_size, label,
    )
}
