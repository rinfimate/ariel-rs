//! SVG template functions for the kanban renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.

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
        r#"<g class="cluster-label " transform="translate({tx:.4}, {ty:.0})"><foreignObject width="160" height="24"><div xmlns="http://www.w3.org/1999/xhtml" style="display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;"><span class="nodeLabel "><p>{label}</p></span></div></foreignObject></g>"#,
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
pub fn item_label_fo(tx: f64, ty: f64, fw: f64, mw: f64, label: &str) -> String {
    format!(
        r#"<g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect><foreignObject width="{fw:.6}" height="24"><div style="text-align: center; display: table-cell; white-space: nowrap; line-height: 1.5; max-width: {mw:.0}px;" xmlns="http://www.w3.org/1999/xhtml"><span style="text-align:left !important" class="nodeLabel markdown-node-label"><p>{label}</p></span></div></foreignObject></g>"#,
    )
}

/// Render an item label with fixed width (non-estimated).
pub fn item_label_fo_fixed(tx: f64, ty: f64, fw: f64, mw: f64, label: &str) -> String {
    format!(
        r#"<g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect><foreignObject width="{fw:.0}" height="24"><div style="text-align: center; display: table-cell; white-space: nowrap; line-height: 1.5; max-width: {mw:.0}px;" xmlns="http://www.w3.org/1999/xhtml"><span style="text-align:left !important" class="nodeLabel markdown-node-label"><p>{label}</p></span></div></foreignObject></g>"#,
    )
}

/// Render an empty secondary item label placeholder (0×0 foreignObject).
pub fn item_label_empty(tx: f64, ty: f64, mw: f64) -> String {
    format!(
        r#"<g class="label" style="text-align:left !important" transform="translate({tx:.2}, {ty:.2})"><rect></rect><foreignObject width="0" height="0"><div style="text-align: center; display: table-cell; white-space: nowrap; line-height: 1.5; max-width: {mw:.0}px;" xmlns="http://www.w3.org/1999/xhtml"><span style="text-align:left !important" class="nodeLabel "></span></div></foreignObject></g>"#,
    )
}
