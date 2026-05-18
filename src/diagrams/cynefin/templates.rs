//! SVG template functions for the Cynefin diagram renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for a Cynefin diagram.
pub fn svg_root(tw: &str, th: &str) -> String {
    format!(
        r#"<svg id="mermaid-cynefin" xmlns="http://www.w3.org/2000/svg" width="{tw}" height="{th}" viewBox="0 0 {tw} {th}" role="graphics-document">"#,
        tw = tw,
        th = th,
    )
}

/// Render the CSS `<style>` block for a Cynefin diagram.
pub fn style_block(ff: &str, fs: f64, fsl: f64, tc: &str, title_c: &str) -> String {
    format!(
        r#"<style>
#mermaid-cynefin {{ font-family: {ff}; font-size: {fs}px; }}
.cynefinBoundary {{ stroke: {tc}; stroke-width: 2; opacity: 0.5; }}
.cynefinCliff {{ stroke: {tc}; stroke-width: 4; opacity: 0.7; }}
.cynefinConfusion {{ stroke: {tc}; stroke-width: 2; }}
.cynefinDomainLabel {{ font-weight: bold; font-size: {fsl}px; }}
.cynefinSubtitle {{ font-size: 11px; fill: {tc}; opacity: 0.75; }}
.cynefinItem {{ stroke: none; }}
.cynefinItemText {{ font-size: 12px; fill: {tc}; }}
.cynefinItemOverflow {{ stroke: none; opacity: 0.6; }}
.cynefinArrowLine {{ stroke: {tc}; stroke-width: 1.5; }}
.cynefinArrowLabel {{ font-size: 11px; fill: {tc}; }}
.cynefinArrowHead {{ fill: {tc}; }}
.cynefinTitle {{ font-size: 18px; font-weight: bold; fill: {title_c}; }}
</style>"#,
        ff = ff,
        fs = fs,
        fsl = fsl,
        tc = tc,
        title_c = title_c,
    )
}

// ---------------------------------------------------------------------------
// Arrow marker
// ---------------------------------------------------------------------------

/// Render the arrowhead `<marker>` definition for Cynefin transition arrows.
pub fn arrowhead_marker(mid: &str) -> String {
    format!(
        r#"<defs><marker id="{mid}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" class="cynefinArrowHead"/></marker></defs>"#,
        mid = mid,
    )
}

// ---------------------------------------------------------------------------
// Domain backgrounds
// ---------------------------------------------------------------------------

/// Render a domain background `<rect>`.
pub fn domain_rect(x: &str, y: &str, w: &str, h: &str, fill: &str) -> String {
    format!(
        r#"<rect class="cynefinDomain" x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}" fill-opacity="0.4" stroke="none"/>"#,
        x = x,
        y = y,
        w = w,
        h = h,
        fill = fill,
    )
}

// ---------------------------------------------------------------------------
// Boundary paths
// ---------------------------------------------------------------------------

/// Render a wavy boundary `<path>`.
pub fn boundary_path(d: &str) -> String {
    format!(
        r#"<path class="cynefinBoundary" d="{d}" fill="none"/>"#,
        d = d,
    )
}

/// Render the cliff boundary `<path>`.
pub fn cliff_path(d: &str) -> String {
    format!(r#"<path class="cynefinCliff" d="{d}" fill="none"/>"#, d = d,)
}

/// Render the confusion ellipse `<path>`.
pub fn confusion_path(d: &str, fill: &str) -> String {
    format!(
        r#"<path class="cynefinConfusion" d="{d}" fill="{fill}" fill-opacity="0.5"/>"#,
        d = d,
        fill = fill,
    )
}

// ---------------------------------------------------------------------------
// Labels
// ---------------------------------------------------------------------------

/// Render a domain name label `<text>`.
pub fn domain_label(x: &str, y: &str, ff: &str, tc: &str, text: &str) -> String {
    format!(
        r#"<text class="cynefinDomainLabel" x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{ff}" fill="{tc}">{text}</text>"#,
        x = x,
        y = y,
        ff = ff,
        tc = tc,
        text = text,
    )
}

/// Render a domain subtitle `<text>`.
pub fn domain_subtitle(x: &str, y: &str, ff: &str, text: &str) -> String {
    format!(
        r#"<text class="cynefinSubtitle" x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{ff}">{text}</text>"#,
        x = x,
        y = y,
        ff = ff,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Item badges
// ---------------------------------------------------------------------------

/// Render a Cynefin item badge group (rect + text).
#[allow(clippy::too_many_arguments)]
pub fn item_badge(
    x: &str,
    y: &str,
    w: &str,
    h: &str,
    fill: &str,
    tx: &str,
    ty: &str,
    ff: &str,
    text: &str,
) -> String {
    format!(
        r#"<g><rect class="cynefinItem" x="{x}" y="{y}" width="{w}" height="{h}" rx="4" ry="4" fill="{fill}" fill-opacity="0.95"/><text class="cynefinItemText" x="{tx}" y="{ty}" text-anchor="middle" dominant-baseline="central" font-family="{ff}">{text}</text></g>"#,
        x = x,
        y = y,
        w = w,
        h = h,
        fill = fill,
        tx = tx,
        ty = ty,
        ff = ff,
        text = text,
    )
}

/// Render an overflow badge group (faded rect + text).
#[allow(clippy::too_many_arguments)]
pub fn item_overflow_badge(
    x: &str,
    y: &str,
    w: &str,
    h: &str,
    fill: &str,
    tx: &str,
    ty: &str,
    ff: &str,
    text: &str,
) -> String {
    format!(
        r#"<g><rect class="cynefinItemOverflow" x="{x}" y="{y}" width="{w}" height="{h}" rx="4" ry="4" fill="{fill}" fill-opacity="0.6"/><text class="cynefinItemText" x="{tx}" y="{ty}" text-anchor="middle" dominant-baseline="central" font-family="{ff}">{text}</text></g>"#,
        x = x,
        y = y,
        w = w,
        h = h,
        fill = fill,
        tx = tx,
        ty = ty,
        ff = ff,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Transition arrows
// ---------------------------------------------------------------------------

/// Render a Cynefin transition arrow `<path>`.
pub fn transition_arrow(
    x1: &str,
    y1: &str,
    cpx: &str,
    cpy: &str,
    x2: &str,
    y2: &str,
    mid: &str,
) -> String {
    format!(
        r#"<path class="cynefinArrowLine" d="M{x1},{y1} Q{cpx},{cpy} {x2},{y2}" fill="none" marker-end="url(#{mid})"/>"#,
        x1 = x1,
        y1 = y1,
        cpx = cpx,
        cpy = cpy,
        x2 = x2,
        y2 = y2,
        mid = mid,
    )
}

/// Render a Cynefin transition arrow label `<text>`.
pub fn transition_label(x: &str, y: &str, ff: &str, text: &str) -> String {
    format!(
        r#"<text class="cynefinArrowLabel" x="{x}" y="{y}" text-anchor="middle" dominant-baseline="auto" font-family="{ff}">{text}</text>"#,
        x = x,
        y = y,
        ff = ff,
        text = text,
    )
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

/// Render the Cynefin diagram title `<text>`.
pub fn title_text(cx: &str, y: &str, ff: &str, text: &str) -> String {
    format!(
        r#"<text class="cynefinTitle" x="{cx}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{ff}">{text}</text>"#,
        cx = cx,
        y = y,
        ff = ff,
        text = text,
    )
}
