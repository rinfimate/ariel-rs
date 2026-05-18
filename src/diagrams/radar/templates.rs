//! SVG template functions for the radar renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element for a radar diagram.
pub fn svg_root(w: &str, h: &str) -> String {
    format!(
        r#"<svg id="mermaid-radar" xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" role="graphics-document">"#,
    )
}

/// Render the CSS `<style>` block for a radar diagram.
pub fn style_block(ff: &str, tc: &str) -> String {
    format!("<style>#mermaid-radar{{font-family:{ff};font-size:14px;fill:{tc};}}</style>",)
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element.
pub fn title_text(cx: &str, ff: &str, fs: &str, tc: &str, text: &str) -> String {
    format!(
        r#"<text x="{cx}" y="24" text-anchor="middle" font-family="{ff}" font-size="{fs}" font-weight="bold" fill="{tc}">{text}</text>"#,
    )
}

// ---------------------------------------------------------------------------
// Graticule (grid rings)
// ---------------------------------------------------------------------------

/// Render a circular graticule ring.
pub fn graticule_circle(cx: &str, cy: &str, r: &str, sc: &str) -> String {
    format!(
        r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="{sc}" stroke-opacity="0.4" stroke-width="1"/>"#,
    )
}

/// Render a polygon graticule ring.
pub fn graticule_polygon(pts: &str, sc: &str) -> String {
    format!(
        r#"<polygon points="{pts}" fill="none" stroke="{sc}" stroke-opacity="0.4" stroke-width="1"/>"#,
    )
}

/// Render a graticule tick label.
pub fn graticule_tick_label(x: &str, y: &str, ff: &str, tc: &str, text: &str) -> String {
    format!(
        r#"<text x="{x}" y="{y}" font-family="{ff}" font-size="9" fill="{tc}" text-anchor="middle">{text}</text>"#,
    )
}

// ---------------------------------------------------------------------------
// Axes (spokes)
// ---------------------------------------------------------------------------

/// Render a radial spoke line.
pub fn axis_spoke(cx: &str, cy: &str, ax: &str, ay: &str, sc: &str) -> String {
    format!(
        r#"<line x1="{cx}" y1="{cy}" x2="{ax}" y2="{ay}" stroke="{sc}" stroke-opacity="0.6" stroke-width="1"/>"#,
    )
}

/// Render an axis label text element.
#[allow(clippy::too_many_arguments)]
pub fn axis_label(
    lx: &str,
    ly: &str,
    dy: &str,
    anchor: &str,
    ff: &str,
    fs: &str,
    tc: &str,
    text: &str,
) -> String {
    format!(
        r#"<text x="{lx}" y="{ly}" dy="{dy}" text-anchor="{anchor}" font-family="{ff}" font-size="{fs}" fill="{tc}">{text}</text>"#,
    )
}

// ---------------------------------------------------------------------------
// Data curves
// ---------------------------------------------------------------------------

/// Render a data curve `<path>`.
pub fn curve_path(d: &str, color: &str) -> String {
    format!(
        r#"<path d="{d}" fill="{color}" fill-opacity="0.2" stroke="{color}" stroke-width="2" stroke-linejoin="round"/>"#,
    )
}

/// Render a data point `<circle>`.
pub fn data_point(px: &str, py: &str, color: &str) -> String {
    format!(r#"<circle cx="{px}" cy="{py}" r="3" fill="{color}"/>"#,)
}

// ---------------------------------------------------------------------------
// Legend
// ---------------------------------------------------------------------------

/// Render a legend colour swatch rect.
pub fn legend_rect(x: &str, y: &str, bw: &str, bh: &str, color: &str) -> String {
    format!(r#"<rect x="{x}" y="{y}" width="{bw}" height="{bh}" fill="{color}"/>"#,)
}

/// Render a legend label text.
pub fn legend_label(x: &str, y: &str, ff: &str, fs: &str, tc: &str, text: &str) -> String {
    format!(
        r#"<text x="{x}" y="{y}" dominant-baseline="middle" font-family="{ff}" font-size="{fs}" fill="{tc}">{text}</text>"#,
    )
}
