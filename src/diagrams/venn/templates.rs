//! SVG template functions for the venn diagram renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG root element for a venn diagram.
pub fn svg_root(id: &str, width: &str, height: &str) -> String {
    format!(
        r#"<svg id="{id}" xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" role="graphics-document">"#,
        id = id,
        w = width,
        h = height,
    )
}

/// Render the embedded `<style>` block for a venn diagram.
pub fn style_block(svg_id: &str, font_family: &str, text_color: &str) -> String {
    format!(
        "<style>#{id}{{font-family:{ff};font-size:14px;}} .venn-circle path{{fill-opacity:0.25;}} .venn-circle text,.venn-intersection text{{text-anchor:middle;dominant-baseline:middle;fill:{tc};}}</style>",
        id = svg_id,
        ff = font_family,
        tc = text_color,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element.
pub fn title_text(
    cx: &str,
    y: &str,
    font_family: &str,
    font_size: &str,
    text_color: &str,
    text: &str,
) -> String {
    format!(
        r#"<text class="venn-title" x="{cx}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{ff}" font-size="{fs}" fill="{tc}">{text}</text>"#,
        cx = cx,
        y = y,
        ff = font_family,
        fs = font_size,
        tc = text_color,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Set circles
// ---------------------------------------------------------------------------

/// Render the opening `<g>` for a venn-circle set group.
pub fn venn_circle_group_open(idx: usize) -> String {
    format!(r#"<g class="venn-circle venn-set-{idx}">"#, idx = idx % 8,)
}

/// Render the set circle `<path>` with fill/stroke styling.
pub fn set_circle_path(path_d: &str, color: &str, stroke_w: &str) -> String {
    format!(
        r#"<path d="{d}" style="fill:{color};fill-opacity:0.1;stroke:{color};stroke-width:{sw};stroke-opacity:0.95;"/>"#,
        d = path_d,
        color = color,
        sw = stroke_w,
    )
}

/// Render the set label `<text>`.
pub fn set_label_text(x: &str, y: &str, font_size: &str, text_color: &str, text: &str) -> String {
    format!(
        r#"<text x="{x}" y="{y}" style="font-size:{fs}px;fill:{tc};">{text}</text>"#,
        x = x,
        y = y,
        fs = font_size,
        tc = text_color,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Intersections
// ---------------------------------------------------------------------------

/// Render the transparent intersection marker `<path>`.
pub fn intersection_path(path_d: &str) -> String {
    format!(r#"<path d="{d}" style="fill-opacity:0;"/>"#, d = path_d,)
}

/// Render the intersection label `<text>`.
pub fn intersection_label_text(
    x: &str,
    y: &str,
    font_size: &str,
    text_color: &str,
    text: &str,
) -> String {
    format!(
        r#"<text x="{x}" y="{y}" style="font-size:{fs}px;fill:{tc};">{text}</text>"#,
        x = x,
        y = y,
        fs = font_size,
        tc = text_color,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Text nodes
// ---------------------------------------------------------------------------

/// Render a venn text-node `<text>` element.
pub fn text_node(
    x: &str,
    y: &str,
    font_family: &str,
    font_size: &str,
    text_color: &str,
    text: &str,
) -> String {
    format!(
        r#"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{ff}" font-size="{fs}" fill="{tc}">{text}</text>"#,
        x = x,
        y = y,
        ff = font_family,
        fs = font_size,
        tc = text_color,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Transform group
// ---------------------------------------------------------------------------

/// Render a `<g transform="translate(0,{y})">` group for shifting diagram content.
pub fn translate_group(y: &str) -> String {
    format!(r#"<g transform="translate(0,{title_h})">"#, title_h = y,)
}
