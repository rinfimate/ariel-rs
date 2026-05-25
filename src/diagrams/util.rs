//! Shared SVG utility functions used across all diagram renderers.

/// Format a float for SVG: integer if whole, else trim trailing zeros.
pub fn fmt(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e12 {
        return format!("{}", v as i64);
    }
    let s = format!("{:.13}", v);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

/// XML-escape a string for SVG text content and attribute values.
pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Render a single-line SVG text label using Mermaid's tspan-with-dy pattern,
/// with the conventional inner-label-group offset (`translate(cx, cy - FONT_SIZE/1.88)`).
/// Use this for node labels where the outer container is the NODE group; the label-group
/// sits inside with a -8.5 (for 16px) offset.
///
/// For labels where the translate is already on the label-group itself (no extra wrapper),
/// use [`label_tspan_raw`] which omits the offset.
#[allow(clippy::too_many_arguments)]
pub fn label_tspan(
    cx: f64,
    cy: f64,
    label: &str,
    font_size: f64,
    color: &str,
    anchor: &str,
    extra_text_attrs: &str,
    font_family: &str,
) -> String {
    let group_y = cy - font_size / 1.882; // -8.5 for 16px
    label_tspan_raw(
        cx,
        group_y,
        label,
        font_size,
        color,
        anchor,
        extra_text_attrs,
        font_family,
    )
}

/// Same as [`label_tspan`] but without the inner-group offset; the translate goes directly
/// to the position the label-group is rendered at. Use this for renderers like requirement
/// where the body text label is placed at an absolute y without a wrapping node group.
#[allow(clippy::too_many_arguments)]
pub fn label_tspan_raw(
    cx: f64,
    gy: f64,
    label: &str,
    font_size: f64,
    color: &str,
    anchor: &str,
    extra_text_attrs: &str,
    font_family: &str,
) -> String {
    let text_y = -(font_size * 0.631); // -10.1 for 16px
    format!(
        "<g class=\"label\" transform=\"translate({cx},{gy})\"><text text-anchor=\"{anchor}\" y=\"{ty}\" font-family=\"{ff}\" font-size=\"{fs}\" fill=\"{color}\"{extra}><tspan x=\"0\" y=\"-0.1em\" dy=\"1.1em\"><tspan>{label}</tspan></tspan></text></g>",
        cx = fmt(cx),
        gy = fmt(gy),
        ty = fmt(text_y),
        anchor = anchor,
        ff = font_family,
        fs = font_size,
        color = color,
        extra = extra_text_attrs,
        label = label,
    )
}
