//! SVG template functions for the venn diagram renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::{esc, fmt};

pub fn fmt_floor(v: f64) -> String {
    format!("{}", v.floor() as i64)
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG root element for a venn diagram.
pub fn svg_root(id: &str, width: &str, height: &str) -> String {
    format!(
        r##"<svg id="{id}" xmlns="http://www.w3.org/2000/svg" width="100%" viewBox="0 0 {w} {h}" style="max-width: {w}px;" role="graphics-document document" aria-roledescription="venn">"##,
        id = id,
        w = width,
        h = height,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element matching vennRenderer.ts output.
/// In the reference: x="50%", y = 32*scale, font-size = "{32*scale}px", style fill.
pub fn title_text_venn(
    y: &str,
    scale: f64,
    text_color: &str,
    text: &str,
    font_family: &str,
) -> String {
    let fs = format!("{}px", fmt_f(32.0 * scale));
    format!(
        r##"<text class="venn-title" font-size="{fs}" text-anchor="middle" dominant-baseline="middle" x="50%" y="{y}" style="fill: {tc}; font-family: {ff};">{text}</text>"##,
        fs = fs,
        y = y,
        tc = text_color,
        ff = font_family,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Main group
// ---------------------------------------------------------------------------

/// Render the opening `<g>` element for the main translate group.
pub fn main_group_open(ty: &str) -> String {
    format!(r##"<g transform="translate(0, {ty})">"##)
}

// ---------------------------------------------------------------------------
// Venn circle group
// ---------------------------------------------------------------------------

/// Render the opening `<g>` for a venn-circle set group.
pub fn venn_circle_group_open(set_index: usize, set_id: &str) -> String {
    format!(r##"<g class="venn-area venn-circle venn-set-{set_index}" data-venn-sets="{set_id}">"##)
}

/// Render the `<path>` element for a venn circle area.
pub fn venn_circle_path(
    d: &str,
    fill_opacity: &str,
    fill: &str,
    stroke: &str,
    stroke_width: &str,
) -> String {
    format!(
        r##"<path d="{d}" style="fill-opacity: {fill_opacity}; fill: {fill}; stroke: {stroke}; stroke-width: {stroke_width}; stroke-opacity: 0.95;"></path>"##
    )
}

/// Render the `<text>` element for a venn set label.
pub fn venn_set_label_open(
    tx: &str,
    ty: &str,
    text_color: &str,
    font_size: &str,
    font_family: &str,
) -> String {
    format!(
        r##"<text class="label" text-anchor="middle" dy=".35em" x="{tx}" y="{ty}" style="fill: {text_color}; font-size: {font_size}px; font-family: {font_family};">"##
    )
}

/// Render a `<tspan>` inside a venn label.
pub fn venn_label_tspan(tx: &str, ty: &str, label: &str) -> String {
    format!(r##"<tspan x="{tx}" y="{ty}" dy="0.35em">{label}</tspan>"##)
}

// ---------------------------------------------------------------------------
// Venn intersection group
// ---------------------------------------------------------------------------

/// Render the opening `<g>` for a venn-intersection group.
pub fn venn_intersection_group_open(data_venn: &str) -> String {
    format!(r##"<g class="venn-area venn-intersection" data-venn-sets="{data_venn}">"##)
}

/// Render the `<path>` element for a venn intersection area.
pub fn venn_intersection_path(d: &str, fill_opacity: &str, fill: &str) -> String {
    format!(r##"<path d="{d}" style="fill-opacity: {fill_opacity}; fill: {fill};"></path>"##)
}

fn fmt_f(v: f64) -> String {
    if v == v.trunc() {
        format!("{}", v as i64)
    } else {
        format!("{:.2}", v)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}
