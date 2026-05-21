// State diagram SVG templates — faithful port of Mermaid stateRenderer-v3-unified.ts

use super::constants::*;
use crate::theme::ThemeVars;

// ── Utilities ─────────────────────────────────────────────────────────────────

pub use crate::diagrams::util::{esc, fmt};

// ── SVG root ──────────────────────────────────────────────────────────────────

pub fn svg_root(svg_id: &str, vb_x: f64, vb_y: f64, vb_w: f64, vb_h: f64) -> String {
    format!(
        "<svg id=\"{svg_id}\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" \
         xmlns:xlink=\"http://www.w3.org/1999/xlink\" class=\"statediagram\" \
         font-family=\"Arial, sans-serif\" style=\"max-width: {w}px;\" viewBox=\"{vx} {vy} {w} {h}\" \
         role=\"graphics-document document\" aria-roledescription=\"stateDiagram\">",
        w = fmt(vb_w),
        h = fmt(vb_h),
        vx = fmt(vb_x),
        vy = fmt(vb_y),
    )
}

// ── Markers ───────────────────────────────────────────────────────────────────

pub fn markers(svg_id: &str, line_color: &str) -> String {
    format!(
        "<defs>\
         <marker id=\"{id}-dependencyEnd\" refX=\"19\" refY=\"7\" \
         markerWidth=\"20\" markerHeight=\"28\" orient=\"auto\">\
         <path d=\"M 19,7 L9,13 L14,7 L9,1 Z\" \
         fill=\"{lc}\" stroke=\"{lc}\" stroke-width=\"1\"></path></marker>\
         <marker id=\"{id}_stateDiagram-barbEnd\" refX=\"19\" refY=\"7\" \
         markerWidth=\"20\" markerHeight=\"14\" markerUnits=\"userSpaceOnUse\" orient=\"auto\">\
         <path d=\"M 19,7 L9,13 L14,7 L9,1 Z\" \
         fill=\"{lc}\" stroke=\"{lc}\" stroke-width=\"1\"></path></marker>\
         </defs>",
        id = svg_id,
        lc = line_color,
    )
}

pub fn drop_shadow_filter(svg_id: &str) -> String {
    format!(
        "<defs><filter id=\"{svg_id}-drop-shadow\" height=\"130%\" width=\"130%\">\
         <feDropShadow dx=\"4\" dy=\"4\" stdDeviation=\"0\" flood-opacity=\"0.06\" \
         flood-color=\"#000000\"></feDropShadow></filter></defs>"
    )
}

// ── Cluster (composite state box) ────────────────────────────────────────────

pub fn composite_cluster(
    dom_id: &str,
    label: &str,
    graph_w: f64,
    graph_h: f64,
    _label_tw: f64,
    vars: &ThemeVars,
) -> String {
    let sp = CLUSTER_PADDING;
    let title_h = CLUSTER_TITLE_H;
    let rect_w = graph_w - 2.0 * sp;
    let rect_h = graph_h - 2.0 * sp - 4.0;
    let inner_y = sp + title_h + 2.0;
    let inner_h = rect_h - title_h - 6.0;
    let text_cx = graph_w / 2.0;
    let text_cy = sp + title_h / 2.0;
    format!(
        "<g class=\" statediagram-state statediagram-cluster \" id=\"{dom_id}\" \
         data-id=\"{label}\" data-look=\"classic\">\
         <g><rect class=\"outer\" x=\"{sp}\" y=\"{sp}\" width=\"{rw}\" height=\"{rh}\" \
         rx=\"5\" ry=\"5\" style=\"fill:{pc};stroke:{pb};stroke-width:1px;\"></rect></g>\
         <text x=\"{tcx}\" y=\"{tcy}\" text-anchor=\"middle\" dominant-baseline=\"middle\" \
         font-size=\"{fs}\" fill=\"{fill}\">{lbl}</text>\
         <rect class=\"inner\" x=\"{sp}\" y=\"{iy}\" width=\"{rw}\" height=\"{ih}\" \
         style=\"fill:{bg};stroke:{pb};stroke-width:1px;\"></rect>\
         </g>",
        dom_id = dom_id,
        label = label,
        sp = fmt(sp),
        rw = fmt(rect_w),
        rh = fmt(rect_h),
        pc = vars.primary_color,
        pb = vars.state_end_bg,
        tcx = fmt(text_cx),
        tcy = fmt(text_cy),
        fs = fmt(FONT_SIZE),
        fill = vars.xychart_axis_color,
        lbl = esc(label),
        iy = fmt(inner_y),
        ih = fmt(inner_h),
        bg = vars.state_composit_bg,
    )
}

pub fn note_cluster(dom_id: &str, x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        "<g class=\"note-cluster\" id=\"{dom_id}\">\
         <rect rx=\"0\" ry=\"0\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" \
         fill=\"none\"></rect></g>",
        x = fmt(x),
        y = fmt(y),
        w = fmt(w),
        h = fmt(h),
    )
}

// ── Nodes ─────────────────────────────────────────────────────────────────────

pub fn node_state_start(dom_id: &str, cx: f64, cy: f64, vars: &ThemeVars) -> String {
    format!(
        "<g class=\"node default\" id=\"{dom_id}\" data-look=\"classic\" \
         transform=\"translate({cx},{cy})\">\
         <circle class=\"state-start\" r=\"{r}\" cx=\"0\" cy=\"0\" \
         fill=\"{lc}\" stroke=\"{lc}\"></circle></g>",
        r = fmt(START_R),
        cx = fmt(cx),
        cy = fmt(cy),
        lc = vars.state_start_fill,
    )
}

pub fn node_state_end(dom_id: &str, cx: f64, cy: f64, vars: &ThemeVars) -> String {
    format!(
        "<g class=\"node default\" id=\"{dom_id}\" data-look=\"classic\" \
         transform=\"translate({cx},{cy})\">\
         <g class=\"outer-path\">\
         <circle r=\"{ro}\" cx=\"0\" cy=\"0\" fill=\"{ef}\" stroke=\"{lc}\" stroke-width=\"2\"></circle>\
         <circle r=\"{ri}\" cx=\"0\" cy=\"0\" fill=\"{bg}\" stroke=\"{bg}\"></circle>\
         </g></g>",
        ro = fmt(END_OUTER_R), ri = fmt(END_INNER_R),
        cx = fmt(cx), cy = fmt(cy),
        ef = vars.state_end_fill, lc = vars.line_color, bg = vars.state_end_bg,
    )
}

pub fn node_fork_join(dom_id: &str, cx: f64, cy: f64, w: f64, h: f64, line_color: &str) -> String {
    format!(
        "<g class=\"node statediagram-state \" id=\"{dom_id}\" data-look=\"classic\" \
         transform=\"translate({cx},{cy})\">\
         <rect class=\"fork-join\" style=\"fill:{lc};stroke:{lc};\" \
         x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\"></rect></g>",
        x = fmt(-w / 2.0),
        y = fmt(-h / 2.0),
        w = fmt(w),
        h = fmt(h),
        cx = fmt(cx),
        cy = fmt(cy),
        lc = line_color,
    )
}

pub fn node_choice(dom_id: &str, cx: f64, cy: f64, vars: &ThemeVars) -> String {
    let r = CHOICE_R;
    let pts = format!(
        "0,{a} {b},{c} 0,{d} {e},0",
        a = fmt(-r),
        b = fmt(r),
        c = fmt(0.0),
        d = fmt(r),
        e = fmt(-r)
    );
    format!(
        "<g class=\"node statediagram-state \" id=\"{dom_id}\" data-look=\"classic\" \
         transform=\"translate({cx},{cy})\">\
         <polygon class=\"choice-state\" points=\"{pts}\" \
         style=\"fill:{fill};stroke:{stroke};\"></polygon></g>",
        pts = pts,
        cx = fmt(cx),
        cy = fmt(cy),
        fill = vars.primary_color,
        stroke = vars.primary_border,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn node_note(
    dom_id: &str,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    _label: &str,
    label_html: &str,
    vars: &ThemeVars,
) -> String {
    let hw = w / 2.0;
    let hh = h / 2.0;
    format!(
        "<g class=\"node statediagram-note \" id=\"{dom_id}\" data-look=\"classic\" \
         transform=\"translate({cx},{cy})\">\
         <g class=\"basic label-container outer-path\">\
         <path d=\"M{x1} {y1} L{x2} {y1} L{x2} {y2} L{x1} {y2}\" \
         stroke=\"none\" stroke-width=\"0\" style=\"fill:{fill};stroke:none;\"></path>\
         <path d=\"M{x1} {y1} L{x2} {y1} M{x2} {y1} L{x2} {y2} \
         M{x2} {y2} L{x1} {y2} M{x1} {y2} L{x1} {y1}\" \
         style=\"fill:none;stroke:{stroke};stroke-width:1.3;\"></path></g>\
         {label_html}</g>",
        dom_id = dom_id,
        cx = fmt(cx),
        cy = fmt(cy),
        x1 = fmt(-hw),
        y1 = fmt(-hh),
        x2 = fmt(hw),
        y2 = fmt(hh),
        fill = vars.note_bg,
        stroke = vars.note_border,
        label_html = label_html,
    )
}

pub fn node_rect(
    dom_id: &str,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    vars: &ThemeVars,
    label_html: &str,
) -> String {
    format!(
        "<g class=\"node  statediagram-state \" id=\"{dom_id}\" data-look=\"classic\" \
         transform=\"translate({cx},{cy})\">\
         <rect class=\"basic label-container\" rx=\"5\" ry=\"5\" \
         style=\"fill:{pc};stroke:{pb};stroke-width:1px;\" \
         x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\"></rect>\
         {label_html}</g>",
        dom_id = dom_id,
        cx = fmt(cx),
        cy = fmt(cy),
        pc = vars.primary_color,
        pb = vars.node_border,
        x = fmt(-w / 2.0),
        y = fmt(-h / 2.0),
        w = fmt(w),
        h = fmt(h),
        label_html = label_html,
    )
}

// ── Label fragments (plain text) ──────────────────────────────────────────────

pub fn text_state_label(text: &str, fill: &str) -> String {
    format!(
        "<g class=\"label\" style=\"\" transform=\"translate(0,0)\">\
         <text class=\"stateText\" x=\"0\" y=\"0\" font-size=\"{fs}\" \
         text-anchor=\"middle\" dominant-baseline=\"middle\" fill=\"{fill}\">{text}</text></g>",
        fs = fmt(FONT_SIZE),
        fill = fill,
        text = esc(text),
    )
}

pub fn text_note_label(text: &str, fill: &str) -> String {
    format!(
        "<text class=\"noteText\" x=\"0\" y=\"0\" fill=\"{fill}\" \
         font-size=\"{fs}\" text-anchor=\"middle\" dominant-baseline=\"middle\">{text}</text>",
        fill = fill,
        fs = fmt(FONT_SIZE),
        text = esc(text),
    )
}

pub fn text_composite_label(text: &str, hh: f64) -> String {
    format!(
        "<text class=\"stateText\" x=\"0\" y=\"{y}\" \
         font-size=\"{fs}\" text-anchor=\"middle\" dominant-baseline=\"middle\">{text}</text>",
        y = fmt(-hh + 16.0),
        fs = fmt(FONT_SIZE),
        text = esc(text),
    )
}

// ── Edges ─────────────────────────────────────────────────────────────────────

pub fn edge_path(
    d: &str,
    id: &str,
    classes: &str,
    line_color: &str,
    dasharray: &str,
    marker: &str,
) -> String {
    format!(
        "<path d=\"{d}\" id=\"{id}\" class=\" {classes}\" \
         fill=\"none\" stroke=\"{lc}\" stroke-width=\"1\" \
         stroke-dasharray=\"{dash}\" data-edge=\"true\" data-id=\"{id}\" \
         marker-end=\"{marker}\"></path>",
        d = d,
        id = id,
        classes = classes,
        lc = line_color,
        dash = dasharray,
        marker = marker,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn text_edge_label(
    mx: f64,
    my: f64,
    ox: f64,
    oy: f64,
    w: f64,
    id: &str,
    text: &str,
    fill: &str,
    bg: &str,
) -> String {
    format!(
        "<g class=\"edgeLabel\" transform=\"translate({mx},{my})\">\
         <rect x=\"{ox}\" y=\"{oy}\" width=\"{w}\" height=\"24\" \
         fill=\"{bg}\" stroke=\"none\"></rect>\
         <text x=\"0\" y=\"{ty}\" text-anchor=\"middle\" dominant-baseline=\"middle\" \
         font-size=\"{fs}\" fill=\"{fill}\" data-id=\"{id}\">{text}</text></g>",
        mx = fmt(mx),
        my = fmt(my),
        ox = fmt(ox),
        oy = fmt(oy),
        w = fmt(w),
        ty = fmt(oy + 12.0),
        fs = fmt(FONT_SIZE),
        fill = fill,
        bg = bg,
        id = id,
        text = esc(text),
    )
}

pub fn edge_label_empty() -> &'static str {
    "<g class=\"edgeLabel\"></g>"
}

/// Render a composite inner group translate wrapper.
pub fn composite_inner_group(tx: f64, ty: f64, inner_svg: &str) -> String {
    format!(
        "<g class=\"root\" transform=\"translate({},{})\">{}</g>",
        fmt(tx),
        fmt(ty),
        inner_svg
    )
}
