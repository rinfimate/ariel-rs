//! SVG template functions for the railroad renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element for a railroad diagram.
pub fn svg_root(total_w: f64, total_h: f64) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" class="railroad-diagram" viewBox="0 0 {total_w:.1} {total_h:.1}" width="100%" style="max-width:{total_w:.0}px">"#,
    )
}

/// Render an empty railroad SVG placeholder.
pub fn empty_svg(title: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 60"><text x="10" y="30" font-size="14">{title}</text></svg>"#,
    )
}

// ---------------------------------------------------------------------------
// Style block
// ---------------------------------------------------------------------------

/// Render the CSS `<style>` block for railroad diagrams.
pub fn style_block(ff: &str) -> String {
    format!(
        r#"<style>
.railroad-diagram {{ font-family: {ff}; }}
.railroad-title {{ fill: #333; font-family: {ff}; }}
.railroad-rule-name {{ font-family: {ff}; }}
.railroad-comment {{ font-family: {ff}; }}
.railroad-terminal rect {{ fill: #FFFFC0; stroke: #000; stroke-width: 2; }}
.railroad-nonterminal rect {{ fill: #fff; stroke: #000; stroke-width: 2; }}
.railroad-special rect {{ fill: #F0E0FF; stroke: #8800CC; stroke-width: 2; stroke-dasharray: 4,2; }}
.railroad-line {{ stroke: #000; stroke-width: 2; fill: none; }}
.railroad-start, .railroad-end {{ fill: #000; }}
</style>"#,
    )
}

// ---------------------------------------------------------------------------
// Title and rule name
// ---------------------------------------------------------------------------

/// Render the diagram title text element.
pub fn title_text(x: f64, title: &str) -> String {
    format!(
        r#"<text class="railroad-title" x="{x:.1}" y="18" text-anchor="middle" font-size="16" font-weight="bold">{title}</text>"#,
    )
}

/// Render a rule name label.
pub fn rule_name_text(
    baseline_y: f64,
    font_size: f64,
    color: &str,
    ff: &str,
    name: &str,
) -> String {
    format!(
        r#"<text class="railroad-rule-name" x="0" y="{baseline_y:.1}" font-size="{font_size}" fill="{color}" font-family="{ff}">{name}</text>"#,
    )
}

/// Render a rule comment label.
pub fn comment_text(y: f64, font_size: f64, ff: &str, text: &str) -> String {
    format!(
        "<text class=\"railroad-comment\" x=\"0.0\" y=\"{y:.1}\" font-size=\"{font_size}\" fill=\"#888888\" font-style=\"italic\" font-family=\"{ff}\">{text}</text>",
    )
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

/// Render a start marker circle.
pub fn start_marker(cx: f64, cy: f64, r: f64, fill: &str, stroke: &str, sw: f64) -> String {
    format!(
        r#"<circle class="railroad-start" cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/>"#,
    )
}

/// Render an end marker circle (outer).
pub fn end_marker_outer(cx: f64, cy: f64, r: f64, fill: &str, stroke: &str, sw: f64) -> String {
    format!(
        r#"<circle class="railroad-end" cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/>"#,
    )
}

/// Render an end marker circle (inner ring).
pub fn end_marker_inner(cx: f64, cy: f64, r: f64, stroke: &str, sw: f64) -> String {
    format!(
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="none" stroke="{stroke}" stroke-width="{sw}"/>"#,
    )
}

// ---------------------------------------------------------------------------
// Connector lines
// ---------------------------------------------------------------------------

/// Render a horizontal connector line.
pub fn connector_line(x1: f64, y1: f64, x2: f64, y2: f64, color: &str, sw: f64) -> String {
    format!(
        r#"<line class="railroad-line" x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{color}" stroke-width="{sw}"/>"#,
    )
}

// ---------------------------------------------------------------------------
// Terminal / non-terminal nodes
// ---------------------------------------------------------------------------

/// Render a terminal node (rounded rect + label).
#[allow(clippy::too_many_arguments)]
pub fn terminal_node(
    width: f64,
    height: f64,
    fill: &str,
    stroke: &str,
    sw: f64,
    lx: f64,
    ly: f64,
    font_size: f64,
    ff: &str,
    text: &str,
) -> String {
    format!(
        r#"<g class="railroad-terminal"><rect x="0" y="0" width="{width:.1}" height="{height:.1}" rx="10" ry="10" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/><text x="{lx:.1}" y="{ly:.1}" text-anchor="middle" dominant-baseline="middle" font-size="{font_size}" font-family="{ff}" fill="black">{text}</text></g>"#,
    )
}

/// Render a non-terminal node (plain rect + label).
#[allow(clippy::too_many_arguments)]
pub fn nonterminal_node(
    width: f64,
    height: f64,
    fill: &str,
    stroke: &str,
    sw: f64,
    lx: f64,
    ly: f64,
    font_size: f64,
    ff: &str,
    text: &str,
) -> String {
    format!(
        r#"<g class="railroad-nonterminal"><rect x="0" y="0" width="{width:.1}" height="{height:.1}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/><text x="{lx:.1}" y="{ly:.1}" text-anchor="middle" dominant-baseline="middle" font-size="{font_size}" font-family="{ff}" fill="black">{text}</text></g>"#,
    )
}

/// Render a special node (dashed rect + label).
#[allow(clippy::too_many_arguments)]
pub fn special_node(
    width: f64,
    height: f64,
    sw: f64,
    lx: f64,
    ly: f64,
    font_size: f64,
    ff: &str,
    text: &str,
) -> String {
    format!(
        "<g class=\"railroad-special\"><rect x=\"0\" y=\"0\" width=\"{width:.1}\" height=\"{height:.1}\" fill=\"#F0E0FF\" stroke=\"#8800CC\" stroke-width=\"{sw}\" stroke-dasharray=\"4,2\"/><text x=\"{lx:.1}\" y=\"{ly:.1}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"{font_size}\" font-family=\"{ff}\" fill=\"#8800CC\">{text}</text></g>",
    )
}

// ---------------------------------------------------------------------------
// Path elements
// ---------------------------------------------------------------------------

/// Render a railroad path element (for arcs/bypasses/loops).
pub fn path_el(d: &str, color: &str, sw: f64) -> String {
    format!(
        r#"<path class="railroad-line" d="{d}" fill="none" stroke="{color}" stroke-width="{sw}"/>"#,
    )
}
