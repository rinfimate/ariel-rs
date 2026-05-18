//! SVG template functions for the ER diagram renderer.
//!
//! Each function takes typed parameters and returns a `String`.
//! No rendering logic lives here — only string formatting.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Top-level SVG structure
// ---------------------------------------------------------------------------

/// Render the outer SVG wrapper for an ER diagram.
pub fn svg_root(id: &str, w: f64, h: f64, css: &str) -> String {
    format!(
        r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" class="erDiagram" style="max-width:{w:.3}px;" viewBox="0 0 {w:.3} {h:.3}"><style>{css}</style>"#,
        id = id,
        w = w,
        h = h,
        css = css,
    )
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

/// Render the `onlyOneStart` crow's-foot marker (two bars, refX=0).
pub fn marker_only_one_start(id: &str) -> String {
    format!(
        r#"<defs><marker id="{id}" class="marker onlyOne er" refX="0" refY="9" markerWidth="18" markerHeight="18" orient="auto"><path d="M9,0 L9,18 M15,0 L15,18"/></marker></defs>"#,
        id = id,
    )
}

/// Render the `onlyOneEnd` crow's-foot marker (two bars, refX=18).
pub fn marker_only_one_end(id: &str) -> String {
    format!(
        r#"<defs><marker id="{id}" class="marker onlyOne er" refX="18" refY="9" markerWidth="18" markerHeight="18" orient="auto"><path d="M3,0 L3,18 M9,0 L9,18"/></marker></defs>"#,
        id = id,
    )
}

/// Render the `zeroOrOneStart` crow's-foot marker (circle + bar, refX=0).
pub fn marker_zero_or_one_start(id: &str) -> String {
    format!(
        r#"<defs><marker id="{id}" class="marker zeroOrOne er" refX="0" refY="9" markerWidth="30" markerHeight="18" orient="auto"><circle fill="white" cx="21" cy="9" r="6"/><path d="M9,0 L9,18"/></marker></defs>"#,
        id = id,
    )
}

/// Render the `zeroOrOneEnd` crow's-foot marker (circle + bar, refX=30).
pub fn marker_zero_or_one_end(id: &str) -> String {
    format!(
        r#"<defs><marker id="{id}" class="marker zeroOrOne er" refX="30" refY="9" markerWidth="30" markerHeight="18" orient="auto"><circle fill="white" cx="9" cy="9" r="6"/><path d="M21,0 L21,18"/></marker></defs>"#,
        id = id,
    )
}

/// Render the `oneOrMoreStart` crow's-foot marker (crow's foot + single bar).
pub fn marker_one_or_more_start(id: &str) -> String {
    format!(
        r#"<defs><marker id="{id}" class="marker oneOrMore er" refX="18" refY="18" markerWidth="45" markerHeight="36" orient="auto"><path d="M0,18 Q 18,0 36,18 Q 18,36 0,18 M42,9 L42,27"/></marker></defs>"#,
        id = id,
    )
}

/// Render the `oneOrMoreEnd` crow's-foot marker (single bar + crow's foot).
pub fn marker_one_or_more_end(id: &str) -> String {
    format!(
        r#"<defs><marker id="{id}" class="marker oneOrMore er" refX="27" refY="18" markerWidth="45" markerHeight="36" orient="auto"><path d="M3,9 L3,27 M9,18 Q27,0 45,18 Q27,36 9,18"/></marker></defs>"#,
        id = id,
    )
}

/// Render the `zeroOrMoreStart` crow's-foot marker (crow's foot + circle).
pub fn marker_zero_or_more_start(id: &str) -> String {
    format!(
        r#"<defs><marker id="{id}" class="marker zeroOrMore er" refX="18" refY="18" markerWidth="57" markerHeight="36" orient="auto"><circle fill="white" cx="48" cy="18" r="6"/><path d="M0,18 Q18,0 36,18 Q18,36 0,18"/></marker></defs>"#,
        id = id,
    )
}

/// Render the `zeroOrMoreEnd` crow's-foot marker (circle + crow's foot).
pub fn marker_zero_or_more_end(id: &str) -> String {
    format!(
        r#"<defs><marker id="{id}" class="marker zeroOrMore er" refX="39" refY="18" markerWidth="57" markerHeight="36" orient="auto"><circle fill="white" cx="9" cy="18" r="6"/><path d="M21,18 Q39,0 57,18 Q39,36 21,18"/></marker></defs>"#,
        id = id,
    )
}

// ---------------------------------------------------------------------------
// Entity rendering
// ---------------------------------------------------------------------------

/// Render the entity outer box `<rect>`.
pub fn entity_outer_rect(x: f64, y: f64, w: f64, h: f64, fill: &str, stroke: &str) -> String {
    format!(
        r#"<rect class="entityBox" x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{h:.3}" fill="{fill}" stroke="{stroke}" stroke-width="1"/>"#,
        x = x,
        y = y,
        w = w,
        h = h,
        fill = fill,
        stroke = stroke,
    )
}

/// Render the entity header band `<rect>` (for entities with attributes).
pub fn entity_header_rect(x: f64, y: f64, w: f64, h: f64, fill: &str, stroke: &str) -> String {
    format!(
        r#"<rect x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{h:.3}" fill="{fill}" stroke="{stroke}" stroke-width="1"/>"#,
        x = x,
        y = y,
        w = w,
        h = h,
        fill = fill,
        stroke = stroke,
    )
}

/// Render an entity name `<foreignObject>` label.
pub fn entity_name_fo(x: f64, y: f64, w: f64, name: &str) -> String {
    format!(
        r#"<foreignObject x="{x:.3}" y="{y:.3}" width="{w:.3}" height="24"><div xmlns="http://www.w3.org/1999/xhtml" style="display:table-cell;white-space:nowrap;line-height:1.5;max-width:200px;text-align:center;"><span class="nodeLabel">{name}</span></div></foreignObject>"#,
        x = x,
        y = y,
        w = w,
        name = name,
    )
}

/// Render the separator line below the entity header.
pub fn entity_separator_line(x1: f64, y: f64, x2: f64, stroke: &str) -> String {
    format!(
        r#"<line x1="{x1:.3}" y1="{y:.3}" x2="{x2:.3}" y2="{y:.3}" stroke="{stroke}" stroke-width="1"/>"#,
        x1 = x1,
        y = y,
        x2 = x2,
        stroke = stroke,
    )
}

/// Render an attribute row background `<rect>`.
pub fn attr_row_rect(x: f64, y: f64, w: f64, h: f64, fill: &str, stroke: &str) -> String {
    format!(
        r#"<rect x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{h:.3}" fill="{fill}" stroke="{stroke}" stroke-width="1"/>"#,
        x = x,
        y = y,
        w = w,
        h = h,
        fill = fill,
        stroke = stroke,
    )
}

/// Render an attribute text `<foreignObject>` (type, name, key, or comment column).
pub fn attr_text_fo(x: f64, y: f64, w: f64, fo_h: f64, text: &str) -> String {
    format!(
        r#"<foreignObject x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{fo_h}"><div xmlns="http://www.w3.org/1999/xhtml" style="display:table-cell;white-space:nowrap;line-height:1.5;max-width:200px;text-align:start;"><span class="nodeLabel">{text}</span></div></foreignObject>"#,
        x = x,
        y = y,
        w = w,
        fo_h = fo_h,
        text = text,
    )
}

/// Render an italic attribute text `<foreignObject>` (comment column).
pub fn attr_text_fo_italic(x: f64, y: f64, w: f64, fo_h: f64, text: &str) -> String {
    format!(
        r#"<foreignObject x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{fo_h}"><div xmlns="http://www.w3.org/1999/xhtml" style="display:table-cell;white-space:nowrap;line-height:1.5;max-width:200px;text-align:start;font-style:italic;"><span class="nodeLabel">{text}</span></div></foreignObject>"#,
        x = x,
        y = y,
        w = w,
        fo_h = fo_h,
        text = text,
    )
}

/// Render a vertical column divider `<line>` within an attribute row.
pub fn attr_divider_line(x: f64, y1: f64, y2: f64, stroke: &str) -> String {
    format!(
        r#"<line x1="{x:.3}" y1="{y1:.3}" x2="{x:.3}" y2="{y2:.3}" stroke="{stroke}" stroke-width="1"/>"#,
        x = x,
        y1 = y1,
        y2 = y2,
        stroke = stroke,
    )
}

// ---------------------------------------------------------------------------
// Relationship rendering
// ---------------------------------------------------------------------------

/// Render a relationship line `<path>`.
pub fn relationship_path(d: &str, color: &str, dash: &str, ms: &str, me: &str) -> String {
    format!(
        r#"<path d="{d}" class="relationshipLine" stroke="{color}" stroke-width="1" fill="none" stroke-dasharray="{dash}" marker-start="{ms}" marker-end="{me}"/>"#,
        d = d,
        color = color,
        dash = dash,
        ms = ms,
        me = me,
    )
}

/// Render a relationship label using `<foreignObject>`.
pub fn rel_label_fo(x: f64, y: f64, tx: f64, ty: f64, fw: f64, fo_h: f64, label: &str) -> String {
    format!(
        r#"<g class="edgeLabel" transform="translate({x:.3},{y:.3})"><g class="label" transform="translate({tx:.3},{ty:.3})"><foreignObject width="{fw:.3}" height="{fo_h}"><div xmlns="http://www.w3.org/1999/xhtml" class="labelBkg" style="display:table-cell;white-space:nowrap;line-height:1.5;max-width:200px;text-align:center;"><span class="edgeLabel"><p>{label}</p></span></div></foreignObject></g></g>"#,
        x = x,
        y = y,
        tx = tx,
        ty = ty,
        fw = fw,
        fo_h = fo_h,
        label = label,
    )
}
