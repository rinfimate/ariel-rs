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
    let hh = FORK_JOIN_HEIGHT / 2.0;
    format!("<g id=\"{dom_id}\" class=\"node statediagram-state\"><rect class=\"fork-join\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" style=\"fill:{f};stroke:{f};\"></rect></g>",
        dom_id=dom_id, x=fmt(cx-hw), y=fmt(cy-hh), w=fmt(FORK_JOIN_WIDTH), h=fmt(FORK_JOIN_HEIGHT), f=vars.fork_join_fill)
}
pub fn node_choice(dom_id: &str, cx: f64, cy: f64, vars: &ThemeVars) -> String {
    let s = CHOICE_SIZE;
    let pts = format!(
        "{},{} {},{} {},{} {},{}",
        fmt(cx),
        fmt(cy - s),
        fmt(cx + s),
        fmt(cy),
        fmt(cx),
        fmt(cy + s),
        fmt(cx - s),
        fmt(cy)
    );
    format!("<g id=\"{dom_id}\" class=\"node statediagram-state\"><polygon points=\"{pts}\" style=\"fill:{f};stroke:{s};stroke-width:1px;\"></polygon></g>",
        dom_id=dom_id, pts=pts, f=vars.primary_color, s=vars.state_end_bg)
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
    format!("<g id=\"{dom_id}\" class=\"node statediagram-state\"><rect class=\"basic\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" rx=\"5\" ry=\"5\" style=\"fill:{f};stroke:{s};stroke-width:1px;\"></rect><text x=\"{cx}\" y=\"{cy}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"{fs}\" fill=\"{tc}\">{lbl}</text></g>",
        dom_id=dom_id, x=fmt(x), y=fmt(y), w=fmt(w), h=fmt(h), cx=fmt(cx), cy=fmt(cy),
        f=vars.primary_color, s=vars.state_end_bg, fs=FONT_SIZE as u32, tc=vars.primary_text, lbl=esc(label))
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
    let x = cx - w / 2.0;
    let y = cy - h / 2.0;
    format!("<g id=\"{dom_id}\" class=\"node statediagram-note\"><rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" style=\"fill:{f};stroke:{s};stroke-width:1px;\"></rect><text x=\"{cx}\" y=\"{cy}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"{fs}\" fill=\"{tc}\">{lbl}</text></g>",
        dom_id=dom_id, x=fmt(x), y=fmt(y), w=fmt(w), h=fmt(h), cx=fmt(cx), cy=fmt(cy),
        f=vars.note_bg, s=vars.note_border, fs=FONT_SIZE as u32, tc=vars.note_text_color, lbl=esc(label))
}
