//! SVG template functions for the wardley renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for a wardley map.
pub fn svg_root(total_w: f64, total_h: f64, content: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.1} {:.1}" width="100%" style="max-width:{:.0}px">{}</svg>"#,
        total_w, total_h, total_w, content,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element.
pub fn title_text(x: f64, title_font: f64, title: &str) -> String {
    format!(
        r#"<text class="wardley-title" x="{:.1}" y="20" text-anchor="middle" font-size="{}">{}</text>"#,
        x, title_font, title,
    )
}

// ---------------------------------------------------------------------------
// Background and borders
// ---------------------------------------------------------------------------

/// Render the plot area background `<rect>`.
pub fn bg_rect(canvas_w: f64, canvas_h: f64) -> String {
    format!(
        r#"<rect class="wardley-bg" x="0" y="0" width="{:.1}" height="{:.1}"/>"#,
        canvas_w, canvas_h,
    )
}

/// Render the plot area border `<rect>`.
pub fn border_rect(canvas_w: f64, canvas_h: f64) -> String {
    format!(
        r#"<rect class="wardley-border" x="0" y="0" width="{:.1}" height="{:.1}"/>"#,
        canvas_w, canvas_h,
    )
}

/// Render an evolution stage background `<rect>`.
pub fn stage_bg_rect(x_start: f64, w: f64, canvas_h: f64, fill: &str) -> String {
    format!(
        r#"<rect x="{:.1}" y="0" width="{:.1}" height="{:.1}" fill="{}" opacity="0.5"/>"#,
        x_start, w, canvas_h, fill,
    )
}

/// Render a stage boundary vertical `<line>`.
pub fn stage_boundary_line(x: f64, canvas_h: f64) -> String {
    format!(
        r#"<line class="wardley-axis-line" x1="{:.1}" y1="0" x2="{:.1}" y2="{:.1}"/>"#,
        x, x, canvas_h,
    )
}

/// Render a stage label `<text>` at the bottom of the stage.
pub fn stage_label(x_center: f64, y: f64, axis_font: f64, label: &str) -> String {
    format!(
        r#"<text class="wardley-stage-label" x="{:.1}" y="{:.1}" text-anchor="middle" font-size="{}">{}</text>"#,
        x_center, y, axis_font, label,
    )
}

// ---------------------------------------------------------------------------
// Axis labels
// ---------------------------------------------------------------------------

/// Render the "Visible" Y-axis label at the top-left.
pub fn axis_label_visible(axis_font: f64) -> String {
    format!(
        r#"<text class="wardley-axis-label" x="-5" y="0" text-anchor="end" font-size="{}" dominant-baseline="hanging">Visible</text>"#,
        axis_font,
    )
}

/// Render the "Invisible" Y-axis label at the bottom-left.
pub fn axis_label_invisible(canvas_h: f64, axis_font: f64) -> String {
    format!(
        r#"<text class="wardley-axis-label" x="-5" y="{:.1}" text-anchor="end" font-size="{}">Invisible</text>"#,
        canvas_h, axis_font,
    )
}

/// Render the "Evolution" X-axis label at the bottom-center.
pub fn axis_label_evolution(canvas_w: f64, y: f64, axis_font: f64) -> String {
    format!(
        r#"<text class="wardley-axis-label" x="{:.1}" y="{:.1}" text-anchor="middle" font-size="{}">Evolution</text>"#,
        canvas_w / 2.0,
        y,
        axis_font,
    )
}

// ---------------------------------------------------------------------------
// Arrow marker
// ---------------------------------------------------------------------------

/// Render the wardley arrowhead `<defs><marker>` definition.
pub fn arrow_marker() -> String {
    r#"<defs><marker id="wardley-arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M 0 0 L 10 5 L 0 10 z" class="wardley-arrow-head"/></marker></defs>"#.to_string()
}

// ---------------------------------------------------------------------------
// Links
// ---------------------------------------------------------------------------

/// Render a link `<line>` with optional dasharray.
pub fn link_line(x1: f64, y1: f64, x2: f64, y2: f64, dasharray: &str) -> String {
    format!(
        r#"<line class="wardley-link" x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}"{} marker-end="url(#wardley-arrow)"/>"#,
        x1, y1, x2, y2, dasharray,
    )
}

/// Render a link label `<text>` at the midpoint of a link.
pub fn link_label(mx: f64, my: f64, axis_font: f64, label: &str) -> String {
    format!(
        r#"<text class="wardley-link-label" x="{:.1}" y="{:.1}" text-anchor="middle" font-size="{}">{}</text>"#,
        mx, my, axis_font, label,
    )
}

// ---------------------------------------------------------------------------
// Nodes
// ---------------------------------------------------------------------------

/// Render a node circle (`<circle>`).
pub fn node_circle(class: &str, cx: f64, cy: f64, r: f64, fill_overlay: &str) -> String {
    format!(
        r#"<circle class="{}" cx="{:.1}" cy="{:.1}" r="{:.1}"{}"/>"#,
        class, cx, cy, r, fill_overlay,
    )
}

/// Render the inertia vertical `<line>` next to a node.
pub fn inertia_line(cx: f64, cy: f64, node_radius: f64) -> String {
    format!(
        r#"<line class="wardley-inertia" x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}"/>"#,
        cx + node_radius + 3.0,
        cy - 8.0,
        cx + node_radius + 3.0,
        cy + 8.0,
    )
}

/// Render a node label `<text>` above the node circle.
pub fn node_label(cx: f64, cy: f64, node_radius: f64, font_size: f64, label: &str) -> String {
    format!(
        r#"<text class="wardley-label" x="{:.1}" y="{:.1}" text-anchor="middle" font-size="{}">{}</text>"#,
        cx,
        cy - node_radius - 4.0,
        font_size,
        label,
    )
}

/// Render a note background `<rect>`.
pub fn note_rect(cx: f64, cy: f64, tw: f64, font_size: f64) -> String {
    format!(
        r#"<rect class="wardley-note-box" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="3"/>"#,
        cx - tw / 2.0 - 4.0,
        cy - font_size - 2.0,
        tw + 8.0,
        font_size + 6.0,
    )
}

/// Render a note label `<text>`.
pub fn note_text(cx: f64, cy: f64, font_size: f64, label: &str) -> String {
    format!(
        r#"<text class="wardley-note" x="{:.1}" y="{:.1}" text-anchor="middle" font-size="{}">{}</text>"#,
        cx, cy, font_size, label,
    )
}

// ---------------------------------------------------------------------------
// Annotations
// ---------------------------------------------------------------------------

/// Render an annotation circle.
pub fn annotation_circle(cx: f64, cy: f64) -> String {
    format!(
        r#"<circle class="wardley-annotation" cx="{:.1}" cy="{:.1}" r="8"/>"#,
        cx, cy,
    )
}

/// Render an annotation number `<text>`.
pub fn annotation_number(cx: f64, cy: f64, axis_font: f64, number: u32) -> String {
    format!(
        r#"<text class="wardley-annotation-num" x="{:.1}" y="{:.1}" text-anchor="middle" dominant-baseline="middle" font-size="{}">{}</text>"#,
        cx, cy, axis_font, number,
    )
}

// ---------------------------------------------------------------------------
// Main group
// ---------------------------------------------------------------------------

/// Render the opening `<g transform="translate(gx,gy)">` group for axes + content.
pub fn main_group_open(gx: f64, gy: f64) -> String {
    format!(r#"<g transform="translate({:.1},{:.1})">"#, gx, gy)
}
