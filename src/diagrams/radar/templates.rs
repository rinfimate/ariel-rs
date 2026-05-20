//! SVG template functions for the radar renderer.
//!
//! Faithful port of Mermaid JS styles.ts and renderer.ts output structure.
//! Uses CSS classes (radarGraticule, radarAxisLine, radarAxisLabel, etc.)
//! to match the reference SVG exactly.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::{esc, fmt};

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element.
/// Mermaid JS sets width="100%" with max-width style and a square viewBox.
pub fn svg_root(total_w: &str, total_h: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="max-width: {total_w}px;" viewBox="0 0 {total_w} {total_h}" role="graphics-document document" aria-roledescription="radar">"#,
    )
}

/// Render the CSS `<style>` block.
/// Mirrors Mermaid JS styles.ts genIndexStyles() and the static class definitions.
#[allow(clippy::too_many_arguments)]
pub fn style_block(
    ff: &str,
    font_size: &str,
    title_color: &str,
    axis_color: &str,
    axis_label_font_size: &str,
    graticule_color: &str,
    graticule_opacity: &str,
    graticule_stroke_width: &str,
    legend_font_size: &str,
    curve_styles: &str,
) -> String {
    format!(
        r#"<style>svg{{font-family:{ff};font-size:{font_size}px;fill:#333;}}.radarTitle{{font-size:{font_size}px;color:{title_color};dominant-baseline:hanging;text-anchor:middle;}}.radarAxisLine{{stroke:{axis_color};stroke-width:2;}}.radarAxisLabel{{dominant-baseline:middle;text-anchor:middle;font-size:{axis_label_font_size}px;color:{axis_color};}}.radarGraticule{{fill:{graticule_color};fill-opacity:{graticule_opacity};stroke:{graticule_color};stroke-width:{graticule_stroke_width};}}.radarLegendText{{text-anchor:start;font-size:{legend_font_size}px;dominant-baseline:hanging;}}{curve_styles}</style>"#,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element.
/// Placed at (x=0, y=-(height/2 + marginTop)) relative to the translated group.
pub fn title_text(y: &str, text: &str) -> String {
    format!(r#"<text class="radarTitle" x="0" y="{y}">{text}</text>"#,)
}

// ---------------------------------------------------------------------------
// Graticule (grid rings)
// ---------------------------------------------------------------------------

/// Render a circular graticule ring (centred at origin via g transform).
pub fn graticule_circle(r: &str) -> String {
    format!(r#"<circle r="{r}" class="radarGraticule"></circle>"#,)
}

/// Render a polygon graticule ring.
pub fn graticule_polygon(pts: &str) -> String {
    format!(r#"<polygon points="{pts}" class="radarGraticule"></polygon>"#,)
}

// ---------------------------------------------------------------------------
// Axes (spokes)
// ---------------------------------------------------------------------------

/// Render a radial spoke line from origin to (x2, y2).
pub fn axis_line(x2: &str, y2: &str) -> String {
    format!(r#"<line x1="0" y1="0" x2="{x2}" y2="{y2}" class="radarAxisLine"></line>"#,)
}

/// Render an axis label text element.
pub fn axis_label(x: &str, y: &str, text: &str) -> String {
    format!(r#"<text x="{x}" y="{y}" class="radarAxisLabel">{text}</text>"#,)
}

// ---------------------------------------------------------------------------
// Data curves
// ---------------------------------------------------------------------------

/// Render a data curve `<path>` using CSS class for colour.
pub fn curve_path(d: &str, index: usize) -> String {
    format!(r#"<path d="{d}" class="radarCurve-{index}"></path>"#,)
}

/// Render a data polygon using CSS class for colour.
pub fn curve_polygon(pts: &str, index: usize) -> String {
    format!(r#"<polygon points="{pts}" class="radarCurve-{index}"></polygon>"#,)
}

// ---------------------------------------------------------------------------
// Legend
// ---------------------------------------------------------------------------

/// Render the legend group wrapper with translate.
pub fn legend_group_open(tx: &str, ty: &str) -> String {
    format!(r#"<g transform="translate({tx}, {ty})">"#,)
}

/// Render a legend colour swatch rect using CSS class.
pub fn legend_rect(index: usize) -> String {
    format!(r#"<rect width="12" height="12" class="radarLegendBox-{index}"></rect>"#,)
}

/// Render a legend label text.
pub fn legend_label(text: &str) -> String {
    format!(r#"<text x="16" y="0" class="radarLegendText">{text}</text>"#,)
}

/// Render the centered drawing group wrapper `<g><g transform="translate(cx, cy)">`.
pub fn centered_group_open(cx: &str, cy: &str) -> String {
    format!(
        r#"<g><g transform="translate({cx}, {cy})">"#,
        cx = cx,
        cy = cy,
    )
}

/// Render a CSS curve style string for a single radar curve index.
pub fn curve_style_entry(i: usize, color: &str, fill_opacity: f64, stroke_width: f64) -> String {
    format!(
        ".radarCurve-{i}{{color:{c};fill:{c};fill-opacity:{fo};stroke:{c};stroke-width:{sw};}}\
         .radarLegendBox-{i}{{fill:{c};fill-opacity:{fo};stroke:{c};}}",
        i = i,
        c = color,
        fo = fill_opacity,
        sw = stroke_width,
    )
}
