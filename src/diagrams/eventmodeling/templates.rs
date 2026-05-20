//! SVG template functions for the event modeling renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::{esc, fmt};

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for an event modeling diagram.
pub fn svg_root(w: &str, h: &str) -> String {
    format!(
        r#"<svg id="mermaid-eventmodeling" xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" role="graphics-document">"#,
        w = w,
        h = h,
    )
}

/// Render the CSS `<style>` block for an event modeling diagram.
pub fn style_block(ff: &str, fs: f64, tc: &str) -> String {
    format!(
        r#"<style>
#mermaid-eventmodeling {{ font-family: {ff}; font-size: {fs}px; }}
.em-swimlane rect {{ fill-opacity: 0.08; }}
.em-swimlane text {{ font-weight: bold; fill: {tc}; }}
.em-box rect {{ rx: 3; ry: 3; }}
.em-relation {{ stroke-width: 1; }}
</style>"#,
        ff = ff,
        fs = fs,
        tc = tc,
    )
}

// ---------------------------------------------------------------------------
// Arrowhead marker
// ---------------------------------------------------------------------------

/// Render the arrowhead `<marker>` definition for event modeling relations.
pub fn arrowhead_marker(mid: &str, color: &str) -> String {
    format!(
        r#"<defs><marker id="{mid}" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="{color}"/></marker></defs>"#,
        mid = mid,
        color = color,
    )
}

// ---------------------------------------------------------------------------
// Swimlane rendering
// ---------------------------------------------------------------------------

/// Render a swimlane background group (rect + label text).
#[allow(clippy::too_many_arguments)]
pub fn swimlane(
    y: &str,
    w: &str,
    h: &str,
    bg: &str,
    lx: &str,
    ly: &str,
    ff: &str,
    fs: f64,
    tc: &str,
    label: &str,
) -> String {
    format!(
        r#"<g class="em-swimlane"><rect x="0" y="{y}" width="{w}" height="{h}" rx="3" fill="{bg}" stroke="rgb(240,240,240)"/><text font-weight="bold" x="{lx}" y="{ly}" font-family="{ff}" font-size="{fs}" fill="{tc}">{label}</text></g>"#,
        y = y,
        w = w,
        h = h,
        bg = bg,
        lx = lx,
        ly = ly,
        ff = ff,
        fs = fs,
        tc = tc,
        label = label,
    )
}

// ---------------------------------------------------------------------------
// Box rendering
// ---------------------------------------------------------------------------

/// Render a box group (rect + foreignObject label).
#[allow(clippy::too_many_arguments)]
pub fn box_group(
    x: &str,
    y: &str,
    w: &str,
    h: &str,
    stroke: &str,
    fill: &str,
    fx: &str,
    fy: &str,
    fw: &str,
    fh: &str,
    ff: &str,
    fs: f64,
    text: &str,
) -> String {
    format!(
        r#"<g class="em-box"><rect x="{x}" y="{y}" rx="3" width="{w}" height="{h}" stroke="{stroke}" fill="{fill}"/><foreignObject x="{fx}" y="{fy}" width="{fw}" height="{fh}"><div xmlns="http://www.w3.org/1999/xhtml" style="display:table;height:100%;width:100%;"><span style="display:table-cell;text-align:center;vertical-align:middle;font-family:{ff};font-size:{fs}px;color:#fff;">{text}</span></div></foreignObject></g>"#,
        x = x,
        y = y,
        w = w,
        h = h,
        stroke = stroke,
        fill = fill,
        fx = fx,
        fy = fy,
        fw = fw,
        fh = fh,
        ff = ff,
        fs = fs,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Relation rendering
// ---------------------------------------------------------------------------

/// Render a relation arrow `<path>`.
pub fn relation_path(
    fill: &str,
    stroke: &str,
    arrowhead: &str,
    sx: &str,
    sy: &str,
    tx: &str,
    ty: &str,
) -> String {
    format!(
        r#"<path class="em-relation" fill="{fill}" stroke="{stroke}" stroke-width="1" marker-end="url(#{arrowhead})" d="M{sx} {sy} L{tx} {ty}"/>"#,
        fill = fill,
        stroke = stroke,
        arrowhead = arrowhead,
        sx = sx,
        sy = sy,
        tx = tx,
        ty = ty,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the event modeling diagram title `<text>`.
pub fn title_text(cx: &str, ff: &str, fs: f64, tc: &str, text: &str) -> String {
    format!(
        r#"<text class="em-title" x="{cx}" y="16" text-anchor="middle" font-family="{ff}" font-size="{fs}" fill="{tc}">{text}</text>"#,
        cx = cx,
        ff = ff,
        fs = fs,
        tc = tc,
        text = text,
    )
}
