//! SVG template functions for the kanban renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::esc;

// ---------------------------------------------------------------------------
// Section colour helpers (inline — no CSS required)
// ---------------------------------------------------------------------------

use super::constants::{SECTION_HUES, SECTION_L, SECTION_L_0};
use crate::theme::Theme;

// Dark theme cScale colors for kanban sections (Mermaid dark cScale palette, section-1 first).
const DARK_SECTION_FILLS: &[&str] = &[
    "hsl(321.6393442623, 65.5913978495%, 28.2352941176%)",
    "hsl(194.4, 16.5562913907%, 39.6078431373%)",
    "hsl(23.0769230769, 49.0566037736%, 30.7843137255%)",
    "hsl(0, 83.3333333333%, 33.5294117647%)",
    "hsl(289.1666666667, 100%, 24.1176470588%)",
    "hsl(35.1315789474, 98.7012987013%, 40.1960784314%)",
    "hsl(106.1538461538, 84.4155844156%, 25.0980392157%)",
    "hsl(235, 21.4285714286%, 20.9803921569%)",
];

// Forest theme cScale colors (section-1 first).
const FOREST_SECTION_FILLS: &[&str] = &[
    "hsl(78.1578947368, 58.4615384615%, 84.5098039216%)",
    "hsl(108.1578947368, 58.4615384615%, 74.5098039216%)",
    "hsl(138.1578947368, 58.4615384615%, 74.5098039216%)",
    "hsl(168.1578947368, 58.4615384615%, 74.5098039216%)",
    "hsl(198.1578947368, 58.4615384615%, 74.5098039216%)",
    "hsl(228.1578947368, 58.4615384615%, 74.5098039216%)",
    "hsl(288.1578947368, 58.4615384615%, 74.5098039216%)",
];

// Neutral theme section fills (section-1 first, greyscale, 6-entry cycle matching Mermaid CSS).
const NEUTRAL_SECTION_FILLS: &[&str] = &[
    "hsl(0, 0%, 43.3333333333%)",
    "hsl(0, 0%, 83.3333333333%)",
    "hsl(0, 0%, 56.6666666667%)",
    "hsl(0, 0%, 70%)",
    "hsl(0, 0%, 96.6666666667%)",
    "#FFF",
];
// Neutral section header text colours — only index 0 (section-1, dark bg) uses light text.
const NEUTRAL_SECTION_TEXTS: &[&str] = &["#F4F4F4", "#333", "#333", "#333", "#333", "#333"];

/// Return the `fill` HSL color string for a given section index and theme.
/// sec_idx is 1-based (first column = 1), so subtract 1 for 0-based palette indexing.
pub fn section_fill_color(sec_idx: usize, theme: Theme) -> String {
    let i = sec_idx.saturating_sub(1);
    match theme {
        Theme::Dark => DARK_SECTION_FILLS[i % DARK_SECTION_FILLS.len()].to_string(),
        Theme::Forest => FOREST_SECTION_FILLS[i % FOREST_SECTION_FILLS.len()].to_string(),
        Theme::Neutral => NEUTRAL_SECTION_FILLS[i % NEUTRAL_SECTION_FILLS.len()].to_string(),
        _ => {
            let hue = SECTION_HUES[sec_idx % SECTION_HUES.len()];
            let l = if sec_idx == 0 { SECTION_L_0 } else { SECTION_L };
            format!("hsl({hue}, 100%, {l})")
        }
    }
}

/// Per-section header text colour (for themes with varying text per section index).
pub fn section_header_text(sec_idx: usize, theme: Theme) -> &'static str {
    match theme {
        Theme::Neutral => {
            let i = sec_idx.saturating_sub(1);
            NEUTRAL_SECTION_TEXTS[i % NEUTRAL_SECTION_TEXTS.len()]
        }
        Theme::Dark => "#ccc",
        // Default/forest: Mermaid's themeColor.ts picks contrast based on bg luminosity.
        // Only section-2 (hue 270°, purple) is dark enough to need white text — all
        // other sections in the 11-entry cycle use black. Verified from ref CSS.
        _ => {
            if sec_idx % 11 == 2 {
                "#ffffff"
            } else {
                "#333333"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element for a kanban diagram.
#[allow(clippy::too_many_arguments)]
pub fn svg_root(id: &str, mw: f64, vbx: i64, vby: i64, vbw: u64, vbh: u64, ff: &str) -> String {
    format!(
        r##"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" font-family="{ff}" style="max-width: {mw:.0}px;" viewBox="{vbx} {vby} {vbw} {vbh}" role="graphics-document document" aria-roledescription="kanban">"##,
    )
}

/// Render an empty kanban SVG (no sections).
pub fn empty_svg() -> &'static str {
    r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"></svg>"##
}

// ---------------------------------------------------------------------------
// Section (column)
// ---------------------------------------------------------------------------

/// Render the opening `<g>` for a section/column cluster.
pub fn section_group_open(sec_idx: usize, svg_id: &str, id: &str) -> String {
    format!(
        r##"<g class="cluster undefined section-{sec_idx}" id="{svg_id}-{id}" data-look="classic">"##,
    )
}

/// Render the column background `<rect>` with inline section color.
pub fn section_rect(x: f64, y: f64, w: f64, h: f64, sec_idx: usize, theme: Theme) -> String {
    let fill = section_fill_color(sec_idx, theme);
    format!(
        r##"<rect style="fill:{fill};stroke:{fill};" rx="5" ry="5" x="{x:.0}" y="{y:.0}" width="{w:.0}" height="{h:.0}"></rect>"##,
    )
}

/// Render the column header label as a native SVG `<text>`.
pub fn section_label_fo(tx: f64, ty: f64, label: &str, primary_text: &str, ff: &str) -> String {
    let lbl_g =
        crate::diagrams::util::label_tspan(80.0, 12.0, label, 16.0, primary_text, "middle", "", ff);
    format!(r##"<g class="cluster-label " transform="translate({tx:.4}, {ty:.0})">{lbl_g}</g>"##,)
}

// ---------------------------------------------------------------------------
// Item (card) nodes
// ---------------------------------------------------------------------------

/// Render the opening `<g>` for an item node.
pub fn item_group_open(svg_id: &str, id: &str, cx: f64, cy: f64) -> String {
    format!(
        r##"<g class="node undefined " id="{svg_id}-{id}" transform="translate({cx:.0}, {cy:.0})">"##,
    )
}

/// Render a circle-shaped item card.
pub fn item_circle(r: f64, primary_border: &str, bg: &str) -> String {
    format!(
        r##"<circle class="basic label-container" cx="0" cy="0" r="{r:.2}" style="fill:{bg};stroke:{primary_border};stroke-width:1px;"/>"##,
        primary_border = primary_border,
        bg = bg,
    )
}

/// Render a hexagon-shaped item card.
pub fn item_hexagon(pts: &str, primary_border: &str, bg: &str) -> String {
    format!(
        r##"<polygon class="basic label-container __APA__" points="{pts}" style="fill:{bg};stroke:{primary_border};stroke-width:1px;"/>"##,
        primary_border = primary_border,
        bg = bg,
    )
}

/// Render a default (no-border) item card rect.
pub fn item_default_rect(x: f64, y: f64, w: f64, h: f64, primary_border: &str, bg: &str) -> String {
    format!(
        r##"<rect class="basic label-container __APA__" style="fill:{bg};stroke:{primary_border};stroke-width:1px;" rx="5" ry="5" x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}"></rect>"##,
        primary_border = primary_border,
        bg = bg,
    )
}

/// Render a rounded/rect item card with explicit rx.
pub fn item_rect(
    rx: f64,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    primary_border: &str,
    bg: &str,
) -> String {
    format!(
        r##"<rect class="basic label-container __APA__" style="fill:{bg};stroke:{primary_border};stroke-width:1px;" rx="{rx:.0}" ry="{rx:.0}" x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}"></rect>"##,
        primary_border = primary_border,
        bg = bg,
    )
}

/// Render a multi-line item label using `<tspan>` per line, vertically centered.
/// `lines` — pre-wrapped text lines. `line_h` — line height in px.
pub fn item_label_wrapped(
    tx: f64,
    ty: f64,
    lines: &[String],
    line_h: f64,
    primary_text: &str,
    ff: &str,
) -> String {
    let n = lines.len() as f64;
    // Center the text block: first line center = -(n-1)*line_h/2
    let y0 = -((n - 1.0) * line_h / 2.0);
    let mut tspans = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            tspans.push_str(&format!(r#"<tspan x="0" dy="0">{}</tspan>"#, esc(line)));
        } else {
            tspans.push_str(&format!(
                r#"<tspan x="0" dy="{lh:.0}">{line}</tspan>"#,
                lh = line_h,
                line = esc(line)
            ));
        }
    }
    format!(
        r##"<g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect><text font-family="{ff}" font-size="16" fill="{primary_text}" text-anchor="start" dominant-baseline="middle" x="0" y="{y0:.1}">{tspans}</text></g>"##,
        primary_text = primary_text,
    )
}

/// Render an item label as a native SVG `<text>` (primary label) — kept for metadata case.
#[allow(clippy::too_many_arguments)]
pub fn item_label_fo(
    tx: f64,
    ty: f64,
    _fw: f64,
    _mw: f64,
    fh: f64,
    label: &str,
    primary_text: &str,
    ff: &str,
) -> String {
    let y = fh / 2.0;
    let lbl_g =
        crate::diagrams::util::label_tspan(0.0, y, label, 16.0, primary_text, "start", "", ff);
    format!(
        r##"<g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect>{lbl_g}</g>"##,
    )
}

/// Render an item label with fixed width and dynamic height as a native SVG `<text>`.
#[allow(clippy::too_many_arguments)]
pub fn item_label_fo_fixed(
    tx: f64,
    ty: f64,
    _fw: f64,
    _mw: f64,
    fh: f64,
    label: &str,
    primary_text: &str,
    ff: &str,
) -> String {
    let y = fh / 2.0;
    let lbl_g =
        crate::diagrams::util::label_tspan(0.0, y, label, 16.0, primary_text, "start", "", ff);
    format!(
        r##"<g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect>{lbl_g}</g>"##,
    )
}

/// Render an empty secondary item label placeholder (no foreignObject).
pub fn item_label_empty(tx: f64, ty: f64, _mw: f64) -> String {
    format!(
        r##"<g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect></g>"##,
    )
}

/// Render a ticket link `<a>` wrapping a native SVG `<text>` label.
#[allow(clippy::too_many_arguments)]
pub fn ticket_link(
    url: &str,
    tx: f64,
    ty: f64,
    label: &str,
    primary_text: &str,
    ff: &str,
) -> String {
    format!(
        r##"<a class="kanban-ticket-link" xlink:href="{url}" target="_blank" style="text-decoration:none;"><g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect><text font-family="{ff}" font-size="16" fill="{primary_text}" text-anchor="start" dominant-baseline="middle" text-decoration="underline" x="0" y="12">{label}</text></g></a>"##,
        url = url,
        tx = tx,
        ty = ty,
        label = label,
        primary_text = primary_text,
    )
}

/// Render an assignee label as a native SVG `<text>`.
pub fn assignee_label(
    tx: f64,
    ty: f64,
    _fw: f64,
    label: &str,
    primary_text: &str,
    ff: &str,
) -> String {
    format!(
        r##"<g class="label" style="text-align:right !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect><text font-family="{ff}" font-size="16" fill="{primary_text}" text-anchor="end" dominant-baseline="middle" x="0" y="12">{label}</text></g>"##,
        tx = tx,
        ty = ty,
        label = label,
        primary_text = primary_text,
    )
}

/// Render a priority indicator vertical `<line>`.
pub fn priority_line(x: f64, y1: f64, y2: f64, color: &str) -> String {
    format!(
        r##"<line x1="{x}" y1="{y1}" x2="{x}" y2="{y2}" stroke-width="4" stroke="{color}"></line>"##,
        x = x,
        y1 = y1,
        y2 = y2,
        color = color,
    )
}
