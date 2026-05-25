use super::constants::DIVIDER_FILL;
use super::constants::*;
use crate::theme::ThemeVars;

pub fn fmt(v: f64) -> String {
    crate::diagrams::util::fmt(v)
}
pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub fn arrow_marker(color: &str) -> String {
    format!("<defs><marker id=\"state-barbEnd\" refX=\"19\" refY=\"7\" markerWidth=\"20\" markerHeight=\"14\" markerUnits=\"userSpaceOnUse\" orient=\"auto\"><path d=\"M 19,7 L9,13 L14,7 L9,1 Z\" style=\"fill:{color};\"/></marker></defs>")
}
pub fn node_start(dom_id: &str, cx: f64, cy: f64, vars: &ThemeVars) -> String {
    format!("<g id=\"{dom_id}\" class=\"node statediagram-state\"><circle class=\"state-start\" cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" style=\"fill:{f};stroke:{f};\"></circle></g>",
        dom_id=dom_id, cx=fmt(cx), cy=fmt(cy), r=fmt(START_RADIUS), f=vars.state_start_fill)
}
pub fn node_end(dom_id: &str, cx: f64, cy: f64, vars: &ThemeVars) -> String {
    format!("<g class=\"node default\" id=\"{dom_id}\" data-look=\"classic\" \
             transform=\"translate({cx},{cy})\"><g class=\"outer-path\">\
             <circle r=\"{ro}\" cx=\"0\" cy=\"0\" fill=\"{ef}\" stroke=\"{lc}\" stroke-width=\"2\"></circle>\
             <circle r=\"{ri}\" cx=\"0\" cy=\"0\" fill=\"{bg}\" stroke=\"{bg}\"></circle>\
             </g></g>",
        dom_id=dom_id, cx=fmt(cx), cy=fmt(cy),
        ro=fmt(END_OUTER_RADIUS), ri=fmt(END_INNER_RADIUS),
        ef=vars.state_end_fill, lc=vars.line_color, bg=vars.state_end_bg)
}
pub fn node_fork_join(dom_id: &str, cx: f64, cy: f64, vars: &ThemeVars) -> String {
    let hw = FORK_JOIN_WIDTH / 2.0;
    let hh = FORK_JOIN_VISIBLE_H / 2.0;
    format!("<g id=\"{dom_id}\" class=\"node statediagram-state\"><rect class=\"fork-join\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" style=\"fill:{f};stroke:{f};\"></rect></g>",
        dom_id=dom_id, x=fmt(cx-hw), y=fmt(cy-hh), w=fmt(FORK_JOIN_WIDTH), h=fmt(FORK_JOIN_VISIBLE_H), f=vars.fork_join_fill)
}
pub fn node_choice(dom_id: &str, cx: f64, cy: f64, vars: &ThemeVars) -> String {
    // Match Mermaid's rough.js two-path rendering for the choice diamond.
    // Coordinates are relative to center (0,0) with CHOICE_SIZE=14.
    // Path 1: smooth Bezier fill. Path 2: sketchy stroke overlay (8 segments).
    format!(
        "<g id=\"{dom_id}\" class=\"node statediagram-state\" transform=\"translate({cx},{cy})\"><g>\
<path d=\"M0 14 C5.028 8.972,10.057 3.943,14 0 C10.879 -3.121,7.757 -6.243,0 -14 C-2.980 -11.020,-5.960 -8.040,-14 0 C-9.627 4.373,-5.254 8.746,0 14\" \
stroke=\"none\" stroke-width=\"0\" fill=\"{f}\" style=\"\"></path>\
<path d=\"M0 14 C2.903 11.097 5.805 8.194 14 0 \
M0 14 C4.876 9.124 9.752 4.248 14 0 \
M14 0 C10.989 -3.011 7.978 -6.022 0 -14 \
M14 0 C8.534 -5.466 3.067 -10.933 0 -14 \
M0 -14 C-4.852 -9.148 -9.704 -4.296 -14 0 \
M0 -14 C-5.419 -8.581 -10.839 -3.161 -14 0 \
M-14 0 C-10.702 3.298 -7.404 6.596 0 14 \
M-14 0 C-8.675 5.325 -3.350 10.650 0 14\" \
stroke=\"{s}\" stroke-width=\"1.3\" fill=\"none\" stroke-dasharray=\"0 0\" style=\"\"></path>\
</g></g>",
        dom_id = dom_id,
        cx = fmt(cx),
        cy = fmt(cy),
        f = vars.primary_color,
        s = vars.state_end_bg,
    )
}
pub fn node_rect(
    dom_id: &str,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    label: &str,
    vars: &ThemeVars,
) -> String {
    let x = cx - w / 2.0;
    let y = cy - h / 2.0;
    let lbl_g = crate::diagrams::util::label_tspan(
        cx,
        cy,
        &esc(label),
        FONT_SIZE,
        vars.primary_text,
        "middle",
        "",
        vars.font_family,
    );
    format!("<g id=\"{dom_id}\" class=\"node statediagram-state\"><rect class=\"basic\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" rx=\"5\" ry=\"5\" style=\"fill:{f};stroke:{s};stroke-width:1px;\"></rect>{lbl_g}</g>",
        dom_id=dom_id, x=fmt(x), y=fmt(y), w=fmt(w), h=fmt(h),
        f=vars.primary_color, s=vars.state_end_bg, lbl_g=lbl_g)
}
pub fn node_note(
    dom_id: &str,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    label: &str,
    vars: &ThemeVars,
) -> String {
    let hw = w / 2.0;
    let hh = h / 2.0;
    let (nhw, nhh) = (-hw, -hh);
    // Rough.js-style two-path border: fill polygon + 8 cubic-Bezier stroke segments.
    // Control-point fractions match reference rough.js output for note boxes.
    let stroke_d = format!(
        "M{nhw} {nhh} C{t1c1} {nhh}, {t1c2} {nhh}, {hw} {nhh} \
M{nhw} {nhh} C{t2c1} {nhh}, {t2c2} {nhh}, {hw} {nhh} \
M{hw} {nhh} C{hw} {r1c1}, {hw} {r1c2}, {hw} {hh} \
M{hw} {nhh} C{hw} {r2c1}, {hw} {r2c2}, {hw} {hh} \
M{hw} {hh} C{b1c1} {hh}, {b1c2} {hh}, {nhw} {hh} \
M{hw} {hh} C{b2c1} {hh}, {b2c2} {hh}, {nhw} {hh} \
M{nhw} {hh} C{nhw} {l1c1}, {nhw} {l1c2}, {nhw} {nhh} \
M{nhw} {hh} C{nhw} {l2c1}, {nhw} {l2c2}, {nhw} {nhh}",
        nhw = fmt(nhw),
        nhh = fmt(nhh),
        hw = fmt(hw),
        hh = fmt(hh),
        t1c1 = fmt(-hw * 0.294),
        t1c2 = fmt(hw * 0.413),
        t2c1 = fmt(-hw * 0.471),
        t2c2 = fmt(hw * 0.059),
        r1c1 = fmt(-hh * 0.381),
        r1c2 = fmt(hh * 0.237),
        r2c1 = fmt(-hh * 0.514),
        r2c2 = fmt(-hh * 0.029),
        b1c1 = fmt(hw * 0.340),
        b1c2 = fmt(-hw * 0.320),
        b2c1 = fmt(hw * 0.461),
        b2c2 = fmt(-hw * 0.078),
        l1c1 = fmt(hh * 0.244),
        l1c2 = fmt(-hh * 0.512),
        l2c1 = fmt(hh * 0.342),
        l2c2 = fmt(-hh * 0.316),
    );
    let lbl_g = crate::diagrams::util::label_tspan(
        0.0,
        0.0,
        &esc(label),
        FONT_SIZE,
        vars.note_text_color,
        "middle",
        "",
        vars.font_family,
    );
    format!(
        "<g id=\"{dom_id}\" class=\"node statediagram-note\" transform=\"translate({cx},{cy})\">\
<g class=\"basic label-container outer-path\">\
<path d=\"M{nhw} {nhh} L{hw} {nhh} L{hw} {hh} L{nhw} {hh}\" stroke=\"none\" stroke-width=\"0\" fill=\"{f}\" style=\"\"></path>\
<path d=\"{stroke_d}\" stroke=\"{s}\" stroke-width=\"1.3\" fill=\"none\" stroke-dasharray=\"0 0\" style=\"\"></path>\
</g>\
{lbl_g}\
</g>",
        dom_id = dom_id,
        cx = fmt(cx),
        cy = fmt(cy),
        nhw = fmt(nhw),
        nhh = fmt(nhh),
        hw = fmt(hw),
        hh = fmt(hh),
        f = vars.note_bg,
        s = vars.note_border,
        stroke_d = stroke_d,
        lbl_g = lbl_g,
    )
}

// ---------------------------------------------------------------------------
// Top-level SVG and renderer-built composites
// ---------------------------------------------------------------------------

/// Render the state diagram outer `<svg>` element.
#[allow(clippy::too_many_arguments)]
pub fn svg_root(vx: f64, vy: f64, vw: f64, vh: f64, css: &str, marker: &str, body: &str) -> String {
    format!(
        "<svg id=\"mermaid-svg\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" \
         viewBox=\"{vx} {vy} {vw} {vh}\" style=\"max-width:{vw}px;\">{css}{marker}{body}</svg>",
        vx = fmt(vx),
        vy = fmt(vy),
        vw = fmt(vw),
        vh = fmt(vh),
    )
}

/// Render a note-group placeholder `<g>` (empty container).
pub fn note_group_placeholder(dom_id: &str) -> String {
    format!("<g class=\"statediagram-state statediagram-note-group\" id=\"{dom_id}\"></g>",)
}

/// Render a translate `<g>` wrapper used to position an extracted sub-graph.
pub fn translate_group_open(tx: f64, ty: f64) -> String {
    format!("<g transform=\"translate({},{})\">", fmt(tx), fmt(ty),)
}

/// Render a divider-style cluster `<g>` (dashed background, no children rendered).
#[allow(clippy::too_many_arguments)]
pub fn cluster_divider(dom_id: &str, x: f64, y: f64, w: f64, h: f64, stroke: &str) -> String {
    format!(
        "<g class=\"statediagram-state statediagram-cluster statediagram-cluster-alt\" id=\"{dom_id}\">\
         <rect class=\"divider\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" \
         style=\"fill:{DIVIDER_FILL};stroke:{s};stroke-dasharray:10,10;\"></rect></g>",
        dom_id = dom_id, x = fmt(x), y = fmt(y), w = fmt(w), h = fmt(h), s = stroke,
    )
}

/// Render a full compound cluster `<g>` with outer rect, title text and inner rect.
#[allow(clippy::too_many_arguments)]
pub fn cluster_compound(
    dom_id: &str,
    rx: f64,
    ry: f64,
    rw: f64,
    outer_rh: f64,
    tcx: f64,
    tcy: f64,
    inner_y: f64,
    inner_h: f64,
    primary_color: &str,
    primary_border: &str,
    font_size: u32,
    fc: &str,
    label: &str,
    bg: &str,
) -> String {
    format!(
        "<g class=\"statediagram-state statediagram-cluster\" id=\"{dom_id}\">\
         <rect class=\"outer\" x=\"{rx}\" y=\"{ry}\" width=\"{rw}\" height=\"{rh}\" \
         rx=\"5\" ry=\"5\" style=\"fill:{pc};stroke:{pb};stroke-width:1px;\"></rect>\
         <text x=\"{tcx}\" y=\"{tcy}\" text-anchor=\"middle\" dominant-baseline=\"middle\" \
         font-size=\"{fs}\" fill=\"{fc}\">{lbl}</text>\
         <rect class=\"inner\" x=\"{rx}\" y=\"{iy}\" width=\"{rw}\" height=\"{ih}\" \
         style=\"fill:{bg};stroke:{pb};stroke-width:1px;\"></rect></g>",
        dom_id = dom_id,
        rx = fmt(rx),
        ry = fmt(ry),
        rw = fmt(rw),
        rh = fmt(outer_rh),
        tcx = fmt(tcx),
        tcy = fmt(tcy),
        iy = fmt(inner_y),
        ih = fmt(inner_h),
        pc = primary_color,
        pb = primary_border,
        fs = font_size,
        fc = fc,
        lbl = label,
        bg = bg,
    )
}

/// Render a state transition `<path>` (with optional marker-end attribute already formed).
pub fn transition_path(d: &str, color: &str, dash: &str, marker_end_attr: &str) -> String {
    format!(
        "<path d=\"{d}\" class=\"transition\" fill=\"none\" stroke=\"{color}\" style=\"{dash}\"{marker_end_attr}></path>",
    )
}

/// Render the inline `<style>` block for the state diagram.
#[allow(clippy::too_many_arguments)]
pub fn css(
    pc: &str,
    pb: &str,
    sc: &str,
    tc: &str,
    nb: &str,
    nst: &str,
    fs: u32,
    ff: &str,
) -> String {
    format!(
        "<style>\
         .statediagram-cluster rect{{fill:{pc};stroke:{pb};}}\
         .statediagram-state rect.basic{{fill:{pc};stroke:{pb};}}\
         .node circle.state-start{{fill:{sc};}}\
         .node .fork-join{{fill:{sc};}}\
         .transition{{stroke:{tc};fill:none;}}\
         .statediagram-state rect.divider{{stroke-dasharray:10,10;fill:{DIVIDER_FILL};}}\
         .statediagram-note rect{{fill:{nb};stroke:{nst};}}\
         .note-edge{{stroke-dasharray:5;}}\
         text{{font-family:{ff};font-size:{fs}px;}}\
         </style>",
    )
}

/// Render an edge label group with background rect and label_tspan.
#[allow(clippy::too_many_arguments)]
pub fn edge_label_group(x: f64, y: f64, rect_w: f64, rect_h: f64, bg: &str, lbl_g: &str) -> String {
    format!(
        "<g class=\"edgeLabel\" transform=\"translate({x},{y})\"><rect x=\"{rx}\" y=\"{ry}\" width=\"{rw}\" height=\"{rh}\" fill=\"{bg}\" fill-opacity=\"0.5\" stroke=\"none\"/>{lbl_g}</g>",
        x = fmt(x),
        y = fmt(y),
        rx = fmt(-rect_w / 2.0),
        ry = fmt(-rect_h / 2.0),
        rw = fmt(rect_w),
        rh = fmt(rect_h),
    )
}
