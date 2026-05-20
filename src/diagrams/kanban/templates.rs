//! SVG template functions for the kanban renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

pub use crate::diagrams::util::esc;

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

use super::constants::{SECTION_HUES, SECTION_L, SECTION_L_0, SECTION_L_0_DARK, SECTION_L_DARK};

fn section_text_color(i: usize) -> &'static str {
    match i {
        2 => "#ffffff",
        _ => "black",
    }
}

pub fn build_css(svg_id: &str, ff: &str) -> String {
    let mut s = format!(
        concat!(
            "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}",
            "#{id} p{{margin:0;}}",
            "#{id} .edge{{stroke-width:3;}}",
        ),
        id = svg_id,
        ff = ff,
    );

    // section--1 (default/fallback)
    s.push_str(&format!(
        concat!(
            "#{id} .section--1 rect,#{id} .section--1 path,#{id} .section--1 circle,",
            "#{id} .section--1 polygon,#{id} .section--1 path{{",
            "fill:hsl(240, 100%, {l});stroke:hsl(240, 100%, {l});}}",
            "#{id} .section--1 text{{fill:#ffffff;}}",
        ),
        id = svg_id,
        l = SECTION_L,
    ));

    // section-0
    s.push_str(&format!(
        concat!(
            "#{id} .section-0 rect,#{id} .section-0 path,#{id} .section-0 circle,",
            "#{id} .section-0 polygon,#{id} .section-0 path{{",
            "fill:hsl({h}, 100%, {l});stroke:hsl({h}, 100%, {l});}}",
            "#{id} .section-0 text{{fill:black;}}",
        ),
        id = svg_id,
        h = SECTION_HUES[0],
        l = SECTION_L_0,
    ));

    // section-1 through section-10
    for (i, &hue) in SECTION_HUES.iter().enumerate().skip(1) {
        let text_color = section_text_color(i);
        let (l, _ld) = if i == 0 {
            (SECTION_L_0, SECTION_L_0_DARK)
        } else {
            (SECTION_L, SECTION_L_DARK)
        };
        s.push_str(&format!(
            concat!(
                "#{id} .section-{idx} rect,#{id} .section-{idx} path,#{id} .section-{idx} circle,",
                "#{id} .section-{idx} polygon,#{id} .section-{idx} path{{",
                "fill:hsl({h}, 100%, {l});stroke:hsl({h}, 100%, {l});}}",
                "#{id} .section-{idx} text{{fill:{tc};}}",
            ),
            id = svg_id,
            idx = i,
            h = hue,
            l = l,
            tc = text_color,
        ));
    }

    // Node (card) styles
    s.push_str(&format!(
        concat!(
            "#{id} .node rect,#{id} .node circle,#{id} .node ellipse,",
            "#{id} .node polygon,#{id} .node path{{",
            "fill:white;stroke:#9370DB;stroke-width:1px;}}",
            "#{id} .kanban-ticket-link{{fill:white;stroke:#9370DB;text-decoration:underline;}}",
        ),
        id = svg_id,
    ));

    // kanban-label
    s.push_str(&format!(
        concat!(
            "#{id} .kanban-label{{",
            "dy:1em;alignment-baseline:middle;text-anchor:middle;",
            "dominant-baseline:middle;text-align:center;}}",
        ),
        id = svg_id,
    ));

    // cluster-label / label
    s.push_str(&format!(
        "#{id} .cluster-label,#{id} .label{{color:#333;fill:#333;}}",
        id = svg_id,
    ));

    // section-root
    s.push_str(&format!(
        concat!(
            "#{id} .section-root rect,#{id} .section-root path,",
            "#{id} .section-root circle,#{id} .section-root polygon{{",
            "fill:hsl(240, 100%, 46.2745098039%);}}",
            "#{id} .section-root text{{fill:#ffffff;}}",
        ),
        id = svg_id,
    ));

    s
}

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer `<svg>` element for a kanban diagram.
pub fn svg_root(id: &str, mw: f64, vbx: i64, vby: i64, vbw: u64, vbh: u64) -> String {
    format!(
        r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="max-width: {mw:.0}px;" viewBox="{vbx} {vby} {vbw} {vbh}" role="graphics-document document" aria-roledescription="kanban">"#,
    )
}

/// Render an empty kanban SVG (no sections).
pub fn empty_svg() -> &'static str {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"></svg>"#
}

// ---------------------------------------------------------------------------
// Section (column)
// ---------------------------------------------------------------------------

/// Render the opening `<g>` for a section/column cluster.
pub fn section_group_open(sec_idx: usize, svg_id: &str, id: &str) -> String {
    format!(
        r#"<g class="cluster undefined section-{sec_idx}" id="{svg_id}-{id}" data-look="classic">"#,
    )
}

/// Render the column background `<rect>`.
pub fn section_rect(x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        r#"<rect style="" rx="5" ry="5" x="{x:.0}" y="{y:.0}" width="{w:.0}" height="{h:.0}"></rect>"#,
    )
}

/// Render the column header label using `<foreignObject>`.
pub fn section_label_fo(tx: f64, ty: f64, label: &str) -> String {
    format!(
        r#"<g class="cluster-label " transform="translate({tx:.4}, {ty:.0})"><foreignObject width="160" height="24"><div xmlns="http://www.w3.org/1999/xhtml" style="display: block; white-space: nowrap; line-height: 1.5; width: 160px; text-align: center;"><span class="nodeLabel "><p>{label}</p></span></div></foreignObject></g>"#,
    )
}

// ---------------------------------------------------------------------------
// Item (card) nodes
// ---------------------------------------------------------------------------

/// Render the opening `<g>` for an item node.
pub fn item_group_open(svg_id: &str, id: &str, cx: f64, cy: f64) -> String {
    format!(
        r#"<g class="node undefined " id="{svg_id}-{id}" transform="translate({cx:.0}, {cy:.0})">"#,
    )
}

/// Render a circle-shaped item card.
pub fn item_circle(r: f64) -> String {
    format!(r#"<circle class="basic label-container" cx="0" cy="0" r="{r:.2}" style=""/>"#,)
}

/// Render a hexagon-shaped item card.
pub fn item_hexagon(pts: &str) -> String {
    format!(r#"<polygon class="basic label-container __APA__" points="{pts}" style=""/>"#,)
}

/// Render a default (no-border) item card rect.
pub fn item_default_rect(x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        r#"<rect class="basic label-container __APA__" style="" rx="5" ry="5" x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}"></rect>"#,
    )
}

/// Render a rounded/rect item card with explicit rx.
pub fn item_rect(rx: f64, x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        r#"<rect class="basic label-container __APA__" style="" rx="{rx:.0}" ry="{rx:.0}" x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}"></rect>"#,
    )
}

/// Render an item label using `<foreignObject>` (primary label).
pub fn item_label_fo(tx: f64, ty: f64, fw: f64, mw: f64, fh: f64, label: &str) -> String {
    format!(
        r#"<g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect><foreignObject width="{fw:.6}" height="{fh:.0}"><div style="text-align: center; display: table-cell; white-space: normal; word-wrap: break-word; line-height: 1.5; max-width: {mw:.0}px;" xmlns="http://www.w3.org/1999/xhtml"><span style="text-align:left !important" class="nodeLabel markdown-node-label"><p>{label}</p></span></div></foreignObject></g>"#,
    )
}

/// Render an item label with fixed width and dynamic height.
pub fn item_label_fo_fixed(tx: f64, ty: f64, fw: f64, mw: f64, fh: f64, label: &str) -> String {
    format!(
        r#"<g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect><foreignObject width="{fw:.0}" height="{fh:.0}"><div style="text-align: center; display: table-cell; white-space: normal; word-wrap: break-word; line-height: 1.5; max-width: {mw:.0}px;" xmlns="http://www.w3.org/1999/xhtml"><span style="text-align:left !important" class="nodeLabel markdown-node-label"><p>{label}</p></span></div></foreignObject></g>"#,
    )
}

/// Render an empty secondary item label placeholder (0×0 foreignObject).
pub fn item_label_empty(tx: f64, ty: f64, mw: f64) -> String {
    format!(
        r#"<g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect><foreignObject width="0" height="0"><div style="text-align: center; display: table-cell; white-space: normal; word-wrap: break-word; line-height: 1.5; max-width: {mw:.0}px;" xmlns="http://www.w3.org/1999/xhtml"><span style="text-align:left !important" class="nodeLabel "></span></div></foreignObject></g>"#,
    )
}

/// Render a ticket link `<a>` wrapping a label foreignObject.
pub fn ticket_link(url: &str, tx: f64, ty: f64, label: &str) -> String {
    format!(
        r#"<a class="kanban-ticket-link" xlink:href="{url}" target="_blank"><g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect><foreignObject width="60" height="24"><div style="text-align: center; display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 175px;" xmlns="http://www.w3.org/1999/xhtml"><span style="text-align:left !important" class="nodeLabel"><p>{label}</p></span></div></foreignObject></g></a>"#,
        url = url,
        tx = tx,
        ty = ty,
        label = label,
    )
}

/// Render an assignee label foreignObject.
pub fn assignee_label(tx: f64, ty: f64, fw: f64, label: &str) -> String {
    format!(
        r#"<g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect><foreignObject width="{fw:.0}" height="24"><div style="text-align: center; display: table-cell; white-space: nowrap; line-height: 1.5; max-width: {fw:.0}px;" xmlns="http://www.w3.org/1999/xhtml"><span style="text-align:left !important" class="nodeLabel"><p>{label}</p></span></div></foreignObject></g>"#,
        tx = tx,
        ty = ty,
        fw = fw,
        label = label,
    )
}

/// Render a priority indicator vertical `<line>`.
pub fn priority_line(x: f64, y1: f64, y2: f64, color: &str) -> String {
    format!(
        r#"<line x1="{x}" y1="{y1}" x2="{x}" y2="{y2}" stroke-width="4" stroke="{color}"></line>"#,
        x = x,
        y1 = y1,
        y2 = y2,
        color = color,
    )
}
