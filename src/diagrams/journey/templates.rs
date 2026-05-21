//! SVG template functions for the journey renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::esc;

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element for a journey diagram.
pub fn svg_root(id: &str, max_w: i64, vw: i64, vh: i64, h: i64) -> String {
    format!(
        "<svg id=\"{id}\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" style=\"max-width: {max_w}px;\" viewBox=\"0 -25 {vw} {vh}\" preserveAspectRatio=\"xMinYMin meet\" height=\"{h}\" role=\"graphics-document document\" aria-roledescription=\"journey\">",
    )
}

/// Render the arrowhead `<defs><marker>` definition.
pub fn arrowhead_marker(id: &str) -> String {
    format!(
        "<defs><marker id=\"{id}-arrowhead\" refX=\"5\" refY=\"2\" markerWidth=\"6\" markerHeight=\"4\" orient=\"auto\"><path d=\"M 0,0 V 4 L6,2 Z\"></path></marker></defs>",
    )
}

// ---------------------------------------------------------------------------
// Actor legend
// ---------------------------------------------------------------------------

/// Render an actor legend circle (left panel).
pub fn actor_circle(cy: i64, pos: usize, color: &str) -> String {
    format!(
        "<circle cx=\"20\" cy=\"{cy}\" class=\"actor-{pos}\" fill=\"{color}\" stroke=\"#000\" r=\"7\"></circle>",
    )
}

/// Render an actor legend label (left panel).
pub fn actor_label(ty: i64, name: &str, text_color: &str) -> String {
    format!("<text x=\"40\" y=\"{ty}\" class=\"legend\" fill=\"{text_color}\" font-family=\"trebuchet ms, verdana, arial, sans-serif\"><tspan x=\"50\">{name}</tspan></text>",)
}

// ---------------------------------------------------------------------------
// Section header
// ---------------------------------------------------------------------------

/// Render the section header `<rect>`.
pub fn section_rect(x: i64, fill: &str, w: i64, h: i64, si: usize) -> String {
    format!(
        "<g><rect x=\"{x}\" y=\"50\" fill=\"{fill}\" stroke=\"#666\" width=\"{w}\" height=\"{h}\" rx=\"3\" ry=\"3\" class=\"journey-section section-type-{si}\"></rect>",
    )
}

/// Render the section header label as a native SVG `<text>`.
#[allow(clippy::too_many_arguments)]
pub fn section_label(
    _x: i64,
    _w: i64,
    _h: i64,
    si: usize,
    tx: i64,
    ty: i64,
    label: &str,
    ff: &str,
    text_fill: &str,
) -> String {
    format!(
        "<text x=\"{tx}\" y=\"{ty}\" dominant-baseline=\"central\" alignment-baseline=\"central\" fill=\"{text_fill}\" class=\"journey-section section-type-{si}\" style=\"text-anchor: middle; font-size: 14px; font-family: {ff};\">\
            <tspan x=\"{tx}\" dy=\"0\">{label}</tspan>\
        </text></g>",
    )
}

// ---------------------------------------------------------------------------
// Task elements
// ---------------------------------------------------------------------------

/// Render a vertical dashed task separator line.
pub fn task_line(id: &str, i: usize, cx: i64, top: i64, bottom: i64) -> String {
    format!(
        "<line id=\"{id}-task{i}\" x1=\"{cx}\" y1=\"{top}\" x2=\"{cx}\" y2=\"{bottom}\" class=\"task-line\" stroke-width=\"1px\" stroke-dasharray=\"4 2\" stroke=\"#666\"></line>",
    )
}

/// Render the face circle for a task score.
pub fn face_circle(cx: i64, cy: i64) -> String {
    format!(
        "<circle cx=\"{cx}\" cy=\"{cy}\" class=\"face\" r=\"15\" fill=\"#FFF8DC\" stroke=\"#999\" stroke-width=\"2\" overflow=\"visible\"></circle>",
    )
}

/// Render the two eye circles for a task face.
pub fn face_eyes(elx: i64, erx: i64, ey: i64) -> String {
    format!(
        "<g>\
            <circle cx=\"{elx}\" cy=\"{ey}\" r=\"1.5\" stroke-width=\"2\" fill=\"#666\" stroke=\"#666\"></circle>\
            <circle cx=\"{erx}\" cy=\"{ey}\" r=\"1.5\" stroke-width=\"2\" fill=\"#666\" stroke=\"#666\"></circle>",
    )
}

/// Render a smile mouth path for score >= 4.
pub fn mouth_smile(tx: i64, ty: i64) -> String {
    format!(
        "<path class=\"mouth\" stroke=\"#666\" fill=\"#666\" d=\"M7.5,0A7.5,7.5,0,1,1,-7.5,0L-6.818,0A6.818,6.818,0,1,0,6.818,0Z\" transform=\"translate({tx},{ty})\"></path>",
    )
}

/// Render a neutral mouth line for score == 3.
pub fn mouth_neutral(x1: i64, x2: i64, my: i64) -> String {
    format!(
        "<line class=\"mouth\" stroke=\"#666\" x1=\"{x1}\" y1=\"{my}\" x2=\"{x2}\" y2=\"{my}\" stroke-width=\"1px\"></line>",
    )
}

/// Render a frown mouth path for score <= 2.
pub fn mouth_frown(tx: i64, ty: i64) -> String {
    format!(
        "<path class=\"mouth\" stroke=\"#666\" fill=\"#666\" d=\"M-7.5,0A7.5,7.5,0,1,1,7.5,0L6.818,0A6.818,6.818,0,1,0,-6.818,0Z\" transform=\"translate({tx},{ty})\"></path>",
    )
}

/// Render the task background rectangle.
pub fn task_rect(x: i64, y: i64, fill: &str, w: i64, h: i64, si: usize) -> String {
    format!(
        "<rect x=\"{x}\" y=\"{y}\" fill=\"{fill}\" stroke=\"#666\" width=\"{w}\" height=\"{h}\" rx=\"3\" ry=\"3\" class=\"task task-type-{si}\"></rect>",
    )
}

/// Render an actor dot on a task box.
pub fn actor_dot(cx: i64, cy: i64, pos: usize, color: &str, name: &str) -> String {
    format!(
        "<circle cx=\"{cx}\" cy=\"{cy}\" class=\"actor-{pos}\" fill=\"{color}\" stroke=\"#000\" r=\"7\"><title>{name}</title></circle>",
    )
}

/// Render a task label as a native SVG `<text>`.
#[allow(clippy::too_many_arguments)]
pub fn task_label(
    _x: i64,
    _y: i64,
    _w: i64,
    _h: i64,
    tx: i64,
    ty: i64,
    label: &str,
    ff: &str,
    text_fill: &str,
) -> String {
    format!(
        "<text x=\"{tx}\" y=\"{ty}\" dominant-baseline=\"central\" alignment-baseline=\"central\" fill=\"{text_fill}\" class=\"task\" style=\"text-anchor: middle; font-size: 14px; font-family: {ff};\">\
            <tspan x=\"{tx}\" dy=\"0\">{label}</tspan>\
        </text>",
    )
}

// ---------------------------------------------------------------------------
// Title and activity line
// ---------------------------------------------------------------------------

/// Render the diagram title `<text>` element.
pub fn title_text(tx: i64, _ff: &str, title: &str, text_fill: &str) -> String {
    format!(
        r##"<text x="{tx}" font-size="4ex" font-weight="bold" y="25" fill="{text_fill}" font-family="&quot;trebuchet ms&quot;, verdana, arial, sans-serif">{title}</text>"##,
    )
}

/// Render the horizontal activity arrow line.
pub fn activity_line(x1: i64, y: i64, x2: i64, id: &str) -> String {
    format!(
        "<line x1=\"{x1}\" y1=\"{y}\" x2=\"{x2}\" y2=\"{y}\" stroke-width=\"4\" stroke=\"black\" marker-end=\"url(#{id}-arrowhead)\"></line>",
    )
}

// ---------------------------------------------------------------------------
// Empty / fallback
// ---------------------------------------------------------------------------

/// Render an empty journey SVG placeholder.
pub fn empty_svg(id: &str) -> String {
    format!(
        "<svg id=\"{id}\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 50\"><text x=\"10\" y=\"30\">Empty Journey</text></svg>",
    )
}
