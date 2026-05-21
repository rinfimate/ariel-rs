//! SVG template functions for the event modeling renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No layout logic lives here — only SVG string assembly.
#![allow(dead_code)]

pub use crate::diagrams::util::{esc, fmt};

// ---------------------------------------------------------------------------
// SVG root
// ---------------------------------------------------------------------------

/// Outer SVG wrapper matching Mermaid's `setupGraphViewbox` output.
///
/// Mermaid uses `width="100%"`, `style="max-width: Xpx"`, and a
/// `viewBox` offset by `-padding` on each axis so content drawn at (0,0)
/// has equal whitespace on all four sides.
pub fn svg_root(vbx: &str, vby: &str, vbw: &str, vbh: &str, max_w: &str) -> String {
    format!(
        r#"<svg id="mermaid-eventmodeling" xmlns="http://www.w3.org/2000/svg" width="100%" style="max-width: {max_w}px;" viewBox="{vbx} {vby} {vbw} {vbh}" role="graphics-document">"#,
    )
}

// ---------------------------------------------------------------------------
// Defs: arrowhead marker (renderD3Relation → defs append in Mermaid)
// ---------------------------------------------------------------------------

/// `<defs><marker>` arrowhead identical to Mermaid JS output.
pub fn arrowhead_marker(id: &str, color: &str) -> String {
    format!(
        r#"<defs><marker id="{id}" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="{color}"/></marker></defs>"#,
    )
}

// ---------------------------------------------------------------------------
// Swimlane (renderD3Swimlane)
// ---------------------------------------------------------------------------

/// Render a swimlane background rect + label text.
/// Mirrors the D3 `g.em-swimlane` group from `renderD3Swimlane`.
#[allow(clippy::too_many_arguments)]
pub fn swimlane(
    y: f64,
    w: f64,
    h: f64,
    bg: &str,
    bg_stroke: &str,
    lx: f64,
    ly: f64,
    ff: &str,
    fs: f64,
    tc: &str,
    label: &str,
) -> String {
    format!(
        r#"<g class="em-swimlane"><rect x="0" y="{y}" width="{w}" height="{h}" rx="3" fill="{bg}" stroke="{bg_stroke}"/><text font-weight="bold" x="{lx}" y="{ly}" font-family="{ff}" font-size="{fs}" fill="{tc}">{label}</text></g>"#,
        y = fmt(y),
        w = fmt(w),
        h = fmt(h),
        lx = fmt(lx),
        ly = fmt(ly),
        fs = fs,
    )
}

// ---------------------------------------------------------------------------
// Box (renderD3Box)
// ---------------------------------------------------------------------------

/// Render a box group: rect + SVG text label centred in the rect.
///
/// Mermaid JS uses a `foreignObject` with HTML content.  Since SVG foreignObject
/// is not rendered by resvg (the PNG comparison pipeline), we use a plain SVG
/// `<text>` element centred on the rect instead, preserving the same geometry.
#[allow(clippy::too_many_arguments)]
pub fn em_box(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    stroke: &str,
    fill: &str,
    ff: &str,
    fs: f64,
    tc: &str,
    text: &str,
) -> String {
    let cx = fmt(x + w / 2.0);
    let cy = fmt(y + h / 2.0);
    format!(
        r#"<g class="em-box"><rect x="{x}" y="{y}" rx="3" width="{w}" height="{h}" stroke="{stroke}" fill="{fill}"/><text font-family="{ff}" font-size="{fs}" font-weight="bold" fill="{tc}" text-anchor="middle" dominant-baseline="middle" x="{cx}" y="{cy}">{text}</text></g>"#,
        x = fmt(x),
        y = fmt(y),
        w = fmt(w),
        h = fmt(h),
        fs = fs,
        cx = cx,
        cy = cy,
    )
}

// ---------------------------------------------------------------------------
// Relation arrow (renderD3Relation)
// ---------------------------------------------------------------------------

/// Render a straight-line relation arrow between two boxes.
pub fn relation_path(
    fill: &str,
    stroke: &str,
    arrowhead_id: &str,
    sx: f64,
    sy: f64,
    tx: f64,
    ty: f64,
) -> String {
    format!(
        r#"<path class="em-relation" fill="{fill}" stroke="{stroke}" stroke-width="1" marker-end="url(#{arrowhead_id})" d="M{sx} {sy} L{tx} {ty}"/>"#,
        sx = fmt(sx),
        sy = fmt(sy),
        tx = fmt(tx),
        ty = fmt(ty),
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the diagram title text (placed at top-centre).
pub fn title_text(cx: f64, ff: &str, fs: f64, tc: &str, text: &str) -> String {
    format!(
        r#"<text class="em-title" x="{cx}" y="16" text-anchor="middle" font-family="{ff}" font-size="{fs}" fill="{tc}">{text}</text>"#,
        cx = fmt(cx),
        fs = fs,
    )
}
