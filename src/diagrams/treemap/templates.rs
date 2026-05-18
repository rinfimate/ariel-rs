//! SVG template functions for the treemap renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper element for the treemap.
pub fn svg_root(max_w: &str, vb_x: &str, vb_y: &str, vb_w: &str, vb_h: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="100%" style="max-width: {mw}px;" viewBox="{vx} {vy} {vw} {vh}" role="graphics-document" class="flowchart">"#,
        mw = max_w,
        vx = vb_x,
        vy = vb_y,
        vw = vb_w,
        vh = vb_h,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element.
pub fn title_text(cx: &str, ty: &str, font_family: &str, text: &str) -> String {
    format!(
        "<text x=\"{cx}\" y=\"{ty}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"{ff}\" font-size=\"14\" font-weight=\"bold\" fill=\"#333\" class=\"treemapTitle\">{text}</text>",
        cx = cx,
        ty = ty,
        ff = font_family,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Leaf tile
// ---------------------------------------------------------------------------

/// Render a leaf tile background `<rect>`.
pub fn leaf_rect(x: &str, y: &str, w: &str, h: &str) -> String {
    format!(
        r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" class="treemapLeaf" fill="transparent" style="" fill-opacity="0.3" stroke="transparent" stroke-width="3"/>"#,
        x = x,
        y = y,
        w = w,
        h = h,
    )
}

/// Render the leaf label `<text>` (dominant-baseline=middle).
pub fn leaf_label_text(
    cx: &str,
    cy: &str,
    font_family: &str,
    font_size: i32,
    color: &str,
    text: &str,
) -> String {
    format!(
        r#"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{ff}" font-size="{fs}" fill="{color}" class="treemapLabel" style="text-anchor: middle; dominant-baseline: middle; font-size: {fs}px;fill:{color};">{text}</text>"#,
        x = cx,
        y = cy,
        ff = font_family,
        fs = font_size,
        color = color,
        text = text,
    )
}

/// Render the leaf value `<text>` (dominant-baseline=hanging).
pub fn leaf_value_text(
    cx: &str,
    y: &str,
    font_family: &str,
    font_size: i32,
    color: &str,
    text: &str,
) -> String {
    format!(
        r#"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="hanging" font-family="{ff}" font-size="{fs}" fill="{color}" class="treemapValue" style="text-anchor: middle; dominant-baseline: hanging; font-size: {fs}px; fill: {color};">{text}</text>"#,
        x = cx,
        y = y,
        ff = font_family,
        fs = font_size,
        color = color,
        text = text,
    )
}
