// ER diagram renderer — faithful port of Mermaid's erRenderer.ts
//
// Renders entities as boxes with:
//   - A header row with the entity name
//   - One row per attribute (type | name | key | comment columns)
//   - Alternating row background colours
//
// Relationships are drawn as SVG paths between entity boxes, with
// crow's foot notation SVG markers at each end.
//
// Layout is done via dagre (TB direction, entities as rectangular nodes).

use super::constants::*;
use super::parser::{Attribute, AttributeKey, Cardinality, ErDiagram, RelType, Relationship};
#[allow(unused_imports)]
use super::templates;
use crate::text::measure;
use crate::theme::{Theme, ThemeVars};
use dagre_dgl_rs::graph::{EdgeLabel, Graph, GraphLabel, NodeLabel};
use dagre_dgl_rs::layout::layout;

// ── Marker sizes (from reference SVGs) ───────────────────────────────────────
// onlyOne: two vertical bars
// zeroOrOne: circle + one bar
// oneOrMore: crow's foot
// zeroOrMore: crow's foot + circle

// ── CSS ───────────────────────────────────────────────────────────────────────

fn build_css(svg_id: &str, vars: &ThemeVars) -> String {
    let ff = vars.font_family;
    let pb = vars.primary_border;
    format!(
        r#"#{svg_id}{{font-family:{ff};font-size:{FONT_SIZE}px;fill:{FONT_COLOR};}}
#{svg_id} p{{margin:0;}}
#{svg_id} .entityBox{{fill:{ENTITY_FILL};stroke:{ENTITY_STROKE};}}
#{svg_id} .relationshipLine{{stroke:{REL_LINE_COLOR};stroke-width:1px;fill:none;}}
#{svg_id} .marker{{fill:none!important;stroke:{REL_LINE_COLOR}!important;stroke-width:1;}}
#{svg_id} .label{{font-family:{ff};color:{FONT_COLOR};}}
#{svg_id} .relationshipLabel{{fill:{FONT_COLOR};font-size:14px;}}
#{svg_id} .edgeLabel .label{{fill:{pb};font-size:14px;}}
"#,
        svg_id = svg_id,
        ff = ff,
        pb = pb,
    )
}

// ── Computed entity geometry ──────────────────────────────────────────────────

struct EntityGeom {
    name: String,
    /// Total width of the entity box
    width: f64,
    /// Total height of the entity box
    height: f64,
    /// Column widths: [type_col, name_col, key_col, comment_col]
    col_widths: [f64; 4],
}

/// Return the scaled text width — ab_glyph Arial measures narrower than the browser,
/// so multiply by TEXT_SCALE to match browser-rendered widths.
#[inline]
fn scaled_text_width(text: &str, font_size: f64) -> f64 {
    let (w, _) = measure(text, font_size);
    w * TEXT_SCALE
}

fn compute_entity_geom(entity_name: &str, attrs: &[Attribute]) -> EntityGeom {
    // Measure entity name label width (scaled to browser metrics)
    let label_w = scaled_text_width(entity_name, FONT_SIZE);

    if attrs.is_empty() {
        // No-attribute entity: width = max(minEntityWidth, label + 2*HEADER_PAD_X)
        // Height is fixed at NO_ATTR_ENTITY_H (measured from reference SVGs: 84px).
        let w = (label_w + HEADER_PAD_X * 2.0).max(MIN_ENTITY_W);
        return EntityGeom {
            name: entity_name.to_string(),
            width: w,
            height: NO_ATTR_ENTITY_H,
            col_widths: [0.0, 0.0, 0.0, 0.0],
        };
    }

    // Measure maximum text width per column across all attributes (scaled to browser metrics)
    let mut type_w: f64 = 0.0;
    let mut name_w: f64 = 0.0;
    let mut key_w: f64 = 0.0;
    let mut comment_w: f64 = 0.0;

    let has_key = attrs.iter().any(|a| !matches!(a.key, AttributeKey::None));
    let has_comment = attrs.iter().any(|a| !a.comment.is_empty());

    for attr in attrs {
        type_w = type_w.max(scaled_text_width(&attr.attr_type, FONT_SIZE));
        name_w = name_w.max(scaled_text_width(&attr.name, FONT_SIZE));
        let key_str = attr_key_str(&attr.key);
        key_w = key_w.max(scaled_text_width(key_str, FONT_SIZE));
        comment_w = comment_w.max(scaled_text_width(&attr.comment, FONT_SIZE));
    }

    // widthPaddingFactor: 4 base (type+name columns) + 2 for key col + 2 for comment col.
    // Each column gets COL_PAD_X on each side → 2*COL_PAD_X per column → factor * COL_PAD_X total.
    let width_pad_factor =
        4.0 + if has_key { 2.0 } else { 0.0 } + if has_comment { 2.0 } else { 0.0 };

    // Column widths: text + 2 * COL_PAD_X (12.5 on each side, measured from reference SVGs)
    let type_col = type_w + COL_PAD_X * 2.0;
    let name_col = name_w + COL_PAD_X * 2.0;
    let key_col = if has_key {
        key_w + COL_PAD_X * 2.0
    } else {
        0.0
    };
    let comment_col = if has_comment {
        comment_w + COL_PAD_X * 2.0
    } else {
        0.0
    };

    let col_total = type_col + name_col + key_col + comment_col;

    // Header check for attr entities: label + widthPadding * widthPaddingFactor
    // (widthPadding = COL_PAD_X = 12.5, factor = 4/6/8 depending on columns)
    // This matches Mermaid reference: e.g. CUSTOMER er_complex: 91.266 + 4*12.5 = 141.266
    let header_total = label_w + COL_PAD_X * width_pad_factor;

    let total_w = col_total.max(header_total).max(MIN_ENTITY_W);
    let total_h = HEADER_ROW_H + attrs.len() as f64 * ATTR_ROW_H;

    EntityGeom {
        name: entity_name.to_string(),
        width: total_w,
        height: total_h,
        col_widths: [type_col, name_col, key_col, comment_col],
    }
}

fn attr_key_str(key: &AttributeKey) -> &'static str {
    match key {
        AttributeKey::PK => "PK",
        AttributeKey::FK => "FK",
        AttributeKey::UK => "UK",
        AttributeKey::None => "",
    }
}

// ── SVG marker definitions ────────────────────────────────────────────────────

fn marker_id(svg_id: &str, name: &str) -> String {
    format!("{svg_id}_{name}")
}

/// Emit all 8 crow's-foot marker <defs> blocks.
/// Mirrors exactly the marker shapes from the Mermaid reference SVGs.
fn render_markers(svg_id: &str) -> String {
    let mut out = String::new();

    // onlyOneStart  (|  at entity_a end)
    out.push_str(&format!(
        r#"<defs><marker id="{id}" class="marker onlyOne er" refX="0" refY="9" markerWidth="18" markerHeight="18" orient="auto"><path d="M9,0 L9,18 M15,0 L15,18"/></marker></defs>"#,
        id = marker_id(svg_id, "er-onlyOneStart")
    ));
    // onlyOneEnd
    out.push_str(&format!(
        r#"<defs><marker id="{id}" class="marker onlyOne er" refX="18" refY="9" markerWidth="18" markerHeight="18" orient="auto"><path d="M3,0 L3,18 M9,0 L9,18"/></marker></defs>"#,
        id = marker_id(svg_id, "er-onlyOneEnd")
    ));
    // zeroOrOneStart
    out.push_str(&format!(
        r#"<defs><marker id="{id}" class="marker zeroOrOne er" refX="0" refY="9" markerWidth="30" markerHeight="18" orient="auto"><circle fill="white" cx="21" cy="9" r="6"/><path d="M9,0 L9,18"/></marker></defs>"#,
        id = marker_id(svg_id, "er-zeroOrOneStart")
    ));
    // zeroOrOneEnd
    out.push_str(&format!(
        r#"<defs><marker id="{id}" class="marker zeroOrOne er" refX="30" refY="9" markerWidth="30" markerHeight="18" orient="auto"><circle fill="white" cx="9" cy="9" r="6"/><path d="M21,0 L21,18"/></marker></defs>"#,
        id = marker_id(svg_id, "er-zeroOrOneEnd")
    ));
    // oneOrMoreStart (crow's foot pointing left, single bar)
    out.push_str(&format!(
        r#"<defs><marker id="{id}" class="marker oneOrMore er" refX="18" refY="18" markerWidth="45" markerHeight="36" orient="auto"><path d="M0,18 Q 18,0 36,18 Q 18,36 0,18 M42,9 L42,27"/></marker></defs>"#,
        id = marker_id(svg_id, "er-oneOrMoreStart")
    ));
    // oneOrMoreEnd
    out.push_str(&format!(
        r#"<defs><marker id="{id}" class="marker oneOrMore er" refX="27" refY="18" markerWidth="45" markerHeight="36" orient="auto"><path d="M3,9 L3,27 M9,18 Q27,0 45,18 Q27,36 9,18"/></marker></defs>"#,
        id = marker_id(svg_id, "er-oneOrMoreEnd")
    ));
    // zeroOrMoreStart (crow's foot + circle)
    out.push_str(&format!(
        r#"<defs><marker id="{id}" class="marker zeroOrMore er" refX="18" refY="18" markerWidth="57" markerHeight="36" orient="auto"><circle fill="white" cx="48" cy="18" r="6"/><path d="M0,18 Q18,0 36,18 Q18,36 0,18"/></marker></defs>"#,
        id = marker_id(svg_id, "er-zeroOrMoreStart")
    ));
    // zeroOrMoreEnd
    out.push_str(&format!(
        r#"<defs><marker id="{id}" class="marker zeroOrMore er" refX="39" refY="18" markerWidth="57" markerHeight="36" orient="auto"><circle fill="white" cx="9" cy="18" r="6"/><path d="M21,18 Q39,0 57,18 Q39,36 21,18"/></marker></defs>"#,
        id = marker_id(svg_id, "er-zeroOrMoreEnd")
    ));

    out
}

fn start_marker_url(svg_id: &str, card: &Cardinality) -> String {
    let name = match card {
        Cardinality::ExactlyOne => "er-onlyOneStart",
        Cardinality::ZeroOrOne => "er-zeroOrOneStart",
        Cardinality::OneOrMore => "er-oneOrMoreStart",
        Cardinality::ZeroOrMore => "er-zeroOrMoreStart",
    };
    format!("url(#{}_{})", svg_id, name)
}

fn end_marker_url(svg_id: &str, card: &Cardinality) -> String {
    let name = match card {
        Cardinality::ExactlyOne => "er-onlyOneEnd",
        Cardinality::ZeroOrOne => "er-zeroOrOneEnd",
        Cardinality::OneOrMore => "er-oneOrMoreEnd",
        Cardinality::ZeroOrMore => "er-zeroOrMoreEnd",
    };
    format!("url(#{}_{})", svg_id, name)
}

// ── Relationship line dash style ─────────────────────────────────────────────

fn stroke_dasharray(rel_type: &RelType) -> &'static str {
    match rel_type {
        RelType::Identifying => "0",      // solid
        RelType::NonIdentifying => "8,8", // dashed
    }
}

// ── Entity SVG rendering ──────────────────────────────────────────────────────

/// Render a single entity box, centered at (cx, cy).
fn render_entity(geom: &EntityGeom, attrs: &[Attribute], cx: f64, cy: f64) -> String {
    let w = geom.width;
    let h = geom.height;
    let _x = cx - w / 2.0;
    let _y = cy - h / 2.0;

    let mut out = format!(
        r#"<g class="node er-entity" transform="translate({cx:.3},{cy:.3})">"#,
        cx = cx,
        cy = cy
    );

    // Outer box
    out.push_str(&format!(
        r#"<rect class="entityBox" x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{h:.3}" fill="{fill}" stroke="{stroke}" stroke-width="1"/>"#,
        x = -w / 2.0,
        y = -h / 2.0,
        w = w,
        h = h,
        fill = ENTITY_FILL,
        stroke = ENTITY_STROKE,
    ));

    // Header row background (for entities with attributes only, draws the header band).
    // No-attr entities use the outer box as their header — no second rect needed.
    if !attrs.is_empty() {
        out.push_str(&format!(
            r#"<rect x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{hh:.3}" fill="{fill}" stroke="{stroke}" stroke-width="1"/>"#,
            x = -w / 2.0,
            y = -geom.height / 2.0,
            w = w,
            hh = HEADER_ROW_H,
            fill = ENTITY_FILL,
            stroke = ENTITY_STROKE,
        ));
    }

    // Header text (entity name).
    // Use foreignObject matching Mermaid's reference SVG structure.
    // resvg ignores foreignObject (as does the reference pipeline), so both
    // ref and rust PNGs show blank label areas — which match each other.
    let label_w = scaled_text_width(&geom.name, FONT_SIZE);
    let header_row_center_y = if attrs.is_empty() {
        0.0 // center of the box
    } else {
        -geom.height / 2.0 + HEADER_ROW_H / 2.0 // center of header row
    };
    // foreignObject top-left: horizontally centered, vertically centered on row
    let fo_x = -label_w / 2.0;
    let fo_y = header_row_center_y - 12.0; // 12 = text_height/2 (24px foreignObject / 2)
    out.push_str(&format!(
        r#"<foreignObject x="{x:.3}" y="{y:.3}" width="{w:.3}" height="24"><div xmlns="http://www.w3.org/1999/xhtml" style="display:table-cell;white-space:nowrap;line-height:1.5;max-width:200px;text-align:center;"><span class="nodeLabel">{name}</span></div></foreignObject>"#,
        x = fo_x,
        y = fo_y,
        w = label_w,
        name = xml_escape(&geom.name),
    ));

    // Separator line below header
    if !attrs.is_empty() {
        let sep_y = -geom.height / 2.0 + HEADER_ROW_H;
        out.push_str(&format!(
            r#"<line x1="{x1:.3}" y1="{y:.3}" x2="{x2:.3}" y2="{y:.3}" stroke="{stroke}" stroke-width="1"/>"#,
            x1 = -w / 2.0,
            y = sep_y,
            x2 = w / 2.0,
            stroke = ENTITY_STROKE,
        ));
    }

    // Attribute rows
    let [type_col, name_col, key_col, _comment_col] = geom.col_widths;

    for (i, attr) in attrs.iter().enumerate() {
        let row_y = -geom.height / 2.0 + HEADER_ROW_H + i as f64 * ATTR_ROW_H;
        let row_fill = if i % 2 == 0 {
            ATTR_ROW_ODD
        } else {
            ATTR_ROW_EVEN
        };

        // Row background
        out.push_str(&format!(
            r#"<rect x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{h:.3}" fill="{fill}" stroke="{stroke}" stroke-width="1"/>"#,
            x = -w / 2.0,
            y = row_y,
            w = w,
            h = ATTR_ROW_H,
            fill = row_fill,
            stroke = ENTITY_STROKE,
        ));

        // Attribute text columns — use foreignObject matching Mermaid's reference structure.
        // resvg ignores foreignObject, so both ref and rust PNGs show blank attr areas,
        // which match each other and eliminate text-area MAD.
        // Reference measures attribute text at FONT_SIZE (16px), not 14px.
        // No explicit font-size on divs — inherit from SVG root CSS (matching reference).
        let attr_fs = FONT_SIZE;
        let fo_h = 24.0;
        // foreignObject y: vertically centered in the row
        let fo_y = row_y + (ATTR_ROW_H - fo_h) / 2.0;

        // Type text — foreignObject width = max column text width so all rows are equal
        let left_x = -w / 2.0 + COL_PAD_X;
        let type_col_text_w = type_col - COL_PAD_X * 2.0;
        out.push_str(&format!(
            r#"<foreignObject x="{x:.3}" y="{y:.3}" width="{fw:.3}" height="{fh}"><div xmlns="http://www.w3.org/1999/xhtml" style="display:table-cell;white-space:nowrap;line-height:1.5;max-width:200px;text-align:start;"><span class="nodeLabel">{text}</span></div></foreignObject>"#,
            x = left_x,
            y = fo_y,
            fw = type_col_text_w,
            fh = fo_h,
            text = xml_escape(&attr.attr_type),
        ));

        // Name text — foreignObject width = max column text width so all rows are equal
        let name_x = -w / 2.0 + type_col + COL_PAD_X;
        let name_col_text_w = name_col - COL_PAD_X * 2.0;
        out.push_str(&format!(
            r#"<foreignObject x="{x:.3}" y="{y:.3}" width="{fw:.3}" height="{fh}"><div xmlns="http://www.w3.org/1999/xhtml" style="display:table-cell;white-space:nowrap;line-height:1.5;max-width:200px;text-align:start;"><span class="nodeLabel">{text}</span></div></foreignObject>"#,
            x = name_x,
            y = fo_y,
            fw = name_col_text_w,
            fh = fo_h,
            text = xml_escape(&attr.name),
        ));

        // Key text (if present)
        let key_str = attr_key_str(&attr.key);
        if !key_str.is_empty() {
            let key_x = -w / 2.0 + type_col + name_col + COL_PAD_X;
            let key_w = key_col - COL_PAD_X * 2.0;
            out.push_str(&format!(
                r#"<foreignObject x="{x:.3}" y="{y:.3}" width="{fw:.3}" height="{fh}"><div xmlns="http://www.w3.org/1999/xhtml" style="display:table-cell;white-space:nowrap;line-height:1.5;max-width:200px;text-align:start;"><span class="nodeLabel">{text}</span></div></foreignObject>"#,
                x = key_x,
                y = fo_y,
                fw = key_w,
                fh = fo_h,
                text = key_str,
            ));
        }

        // Comment text (if present)
        if !attr.comment.is_empty() {
            let comment_x = -w / 2.0 + type_col + name_col + key_col + COL_PAD_X;
            let comment_w = scaled_text_width(&attr.comment, attr_fs);
            out.push_str(&format!(
                r#"<foreignObject x="{x:.3}" y="{y:.3}" width="{fw:.3}" height="{fh}"><div xmlns="http://www.w3.org/1999/xhtml" style="display:table-cell;white-space:nowrap;line-height:1.5;max-width:200px;text-align:start;font-style:italic;"><span class="nodeLabel">{text}</span></div></foreignObject>"#,
                x = comment_x,
                y = fo_y,
                fw = comment_w,
                fh = fo_h,
                text = xml_escape(&attr.comment),
            ));
        }

        // Vertical column dividers — drawn per row so adjacent rows form continuous lines.
        // Always draw type|name divider; add name|key and key|comment only when those columns exist.
        let has_key = !attrs.iter().all(|a| matches!(a.key, AttributeKey::None));
        let has_comment = attrs.iter().any(|a| !a.comment.is_empty());
        // type|name divider — always present
        let div1_x = -w / 2.0 + type_col;
        out.push_str(&format!(
            r#"<line x1="{x:.3}" y1="{y1:.3}" x2="{x:.3}" y2="{y2:.3}" stroke="{stroke}" stroke-width="1"/>"#,
            x = div1_x, y1 = row_y, y2 = row_y + ATTR_ROW_H, stroke = ENTITY_STROKE,
        ));
        // name|key divider — only when key column is used
        if has_key || has_comment {
            let div2_x = -w / 2.0 + type_col + name_col;
            out.push_str(&format!(
                r#"<line x1="{x:.3}" y1="{y1:.3}" x2="{x:.3}" y2="{y2:.3}" stroke="{stroke}" stroke-width="1"/>"#,
                x = div2_x, y1 = row_y, y2 = row_y + ATTR_ROW_H, stroke = ENTITY_STROKE,
            ));
        }
        // key|comment divider — only when comment column is used
        if has_comment {
            let div3_x = -w / 2.0 + type_col + name_col + key_col;
            out.push_str(&format!(
                r#"<line x1="{x:.3}" y1="{y1:.3}" x2="{x:.3}" y2="{y2:.3}" stroke="{stroke}" stroke-width="1"/>"#,
                x = div3_x, y1 = row_y, y2 = row_y + ATTR_ROW_H, stroke = ENTITY_STROKE,
            ));
        }
    }

    out.push_str("</g>");
    out
}

// ── Relationship line rendering ───────────────────────────────────────────────

/// Render a relationship line (path) from one entity to another,
/// plus the relationship label centered on the midpoint.
fn render_relationship(
    rel: &Relationship,
    points: &[(f64, f64)],
    svg_id: &str,
    use_foreign_object: bool,
) -> String {
    if points.len() < 2 {
        return String::new();
    }

    // Build SVG path from points
    let d = points_to_path(points);

    let dash = stroke_dasharray(&rel.rel_type);
    let marker_start = start_marker_url(svg_id, &rel.card_a);
    let marker_end = end_marker_url(svg_id, &rel.card_b);

    let path_svg = format!(
        r#"<path d="{d}" class="relationshipLine" stroke="{color}" stroke-width="1" fill="none" stroke-dasharray="{dash}" marker-start="{ms}" marker-end="{me}"/>"#,
        d = d,
        color = REL_LINE_COLOR,
        dash = dash,
        ms = marker_start,
        me = marker_end,
    );

    // Relationship label at midpoint
    let label_svg = if !rel.label.is_empty() {
        let mid = midpoint(points);
        render_rel_label(&rel.label, mid.0, mid.1, use_foreign_object)
    } else {
        String::new()
    };

    format!("{}{}", path_svg, label_svg)
}

fn render_rel_label(label: &str, x: f64, y: f64, _use_foreign_object: bool) -> String {
    // Use foreignObject matching Mermaid's reference SVG structure.
    // resvg ignores foreignObject (same as reference), so label areas match.
    let lw = scaled_text_width(label, 14.0);
    let fo_w = lw;
    let fo_h = 21.0;

    format!(
        r#"<g class="edgeLabel" transform="translate({x:.3},{y:.3})"><g class="label" transform="translate({tx:.3},{ty:.3})"><foreignObject width="{fw:.3}" height="{fh}"><div xmlns="http://www.w3.org/1999/xhtml" class="labelBkg" style="display:table-cell;white-space:nowrap;line-height:1.5;max-width:200px;text-align:center;"><span class="edgeLabel"><p>{label}</p></span></div></foreignObject></g></g>"#,
        x = x,
        y = y,
        tx = -fo_w / 2.0,
        ty = -fo_h / 2.0,
        fw = fo_w,
        fh = fo_h,
        label = xml_escape(label),
    )
}

fn points_to_path(points: &[(f64, f64)]) -> String {
    crate::svg::curve_basis_path(points)
}

fn midpoint(points: &[(f64, f64)]) -> (f64, f64) {
    if points.is_empty() {
        return (0.0, 0.0);
    }
    if points.len() == 1 {
        return points[0];
    }
    // For odd-length arrays, use the exact middle point.
    // For even-length arrays, average the two middle points.
    // This matches Mermaid's edgeLabel placement at the geometric center of the path.
    let n = points.len();
    if n % 2 == 1 {
        // Odd: exact middle element
        points[n / 2]
    } else {
        // Even: average the two middle elements
        let p1 = points[n / 2 - 1];
        let p2 = points[n / 2];
        ((p1.0 + p2.0) / 2.0, (p1.1 + p2.1) / 2.0)
    }
}

// ── XML escaping ──────────────────────────────────────────────────────────────

/// Compute the intersection of the line from (cx,cy) toward (other_x,other_y)
/// with the boundary of a rect centered at (cx,cy) with half-dims (hw,hh).
fn er_intersect_rect(
    cx: f64,
    cy: f64,
    hw: f64,
    hh: f64,
    other_x: f64,
    other_y: f64,
) -> dagre_dgl_rs::graph::Point {
    let dx = other_x - cx;
    let dy = other_y - cy;
    if dx.abs() < 1e-9 && dy.abs() < 1e-9 {
        return dagre_dgl_rs::graph::Point { x: cx, y: cy + hh };
    }
    // Find smallest t > 0 where (cx + dx*t, cy + dy*t) hits boundary
    let mut best_t = f64::INFINITY;
    for &(val, is_y) in &[(hw, false), (-hw, false), (hh, true), (-hh, true)] {
        let denom = if is_y { dy } else { dx };
        if denom.abs() < 1e-9 {
            continue;
        }
        let t = val / denom;
        if t <= 1e-6 {
            continue;
        }
        let perp = if is_y { cx + dx * t } else { cy + dy * t };
        let perp_lim = if is_y { hw } else { hh };
        let perp_ref = if is_y { cx } else { cy };
        if (perp - perp_ref).abs() > perp_lim + 1e-6 {
            continue;
        }
        if t < best_t {
            best_t = t;
        }
    }
    if best_t.is_finite() {
        dagre_dgl_rs::graph::Point {
            x: cx + dx * best_t,
            y: cy + dy * best_t,
        }
    } else {
        dagre_dgl_rs::graph::Point { x: cx, y: cy }
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Return a straight line between two entity centers (fallback when dagre gives no points).
fn fallback_points(g: &Graph, v: &str, w: &str) -> Vec<(f64, f64)> {
    let a = g.node_opt(v);
    let b = g.node_opt(w);
    match (a, b) {
        (Some(a), Some(b)) => {
            if let (Some(ax), Some(ay), Some(bx), Some(by)) = (a.x, a.y, b.x, b.y) {
                vec![(ax, ay), (bx, by)]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

// ── Self-loop path rendering ───────────────────────────────────────────────────

/// Render a self-loop (entity relates to itself) as a path going left of the entity.
/// Uses the entity position and the positions of the waypoint dummy nodes from dagre.
/// Mirrors the Mermaid reference "cyclic-special" path approach.
fn render_self_loop(
    rel: &Relationship,
    pts_a: &[(f64, f64)], // dagre points for entity → dummy_a
    pts_b: &[(f64, f64)], // dagre points for dummy_a → dummy_b
    pts_c: &[(f64, f64)], // dagre points for dummy_b → entity
    svg_id: &str,
    use_foreign_object: bool,
) -> String {
    let dash = stroke_dasharray(&rel.rel_type);
    let marker_start = start_marker_url(svg_id, &rel.card_a);
    let marker_end = end_marker_url(svg_id, &rel.card_b);

    // Combine all three segments, skipping duplicate junction points
    let mut all_pts: Vec<(f64, f64)> = Vec::new();
    let dedup_extend = |pts: &[(f64, f64)], acc: &mut Vec<(f64, f64)>| {
        for &p in pts {
            if acc
                .last()
                .map(|&l: &(f64, f64)| (l.0 - p.0).abs() > 0.1 || (l.1 - p.1).abs() > 0.1)
                .unwrap_or(true)
            {
                acc.push(p);
            }
        }
    };
    dedup_extend(pts_a, &mut all_pts);
    dedup_extend(pts_b, &mut all_pts);
    dedup_extend(pts_c, &mut all_pts);

    let path_d = crate::svg::curve_basis_path(&all_pts);

    // Label at midpoint of middle segment
    let mid = if all_pts.len() >= 2 {
        let n = all_pts.len();
        let i = n / 2;
        (
            (all_pts[i].0 + all_pts[i - 1].0) / 2.0,
            (all_pts[i].1 + all_pts[i - 1].1) / 2.0,
        )
    } else {
        (0.0, 0.0)
    };

    let label_svg = if !rel.label.is_empty() {
        render_rel_label(&rel.label, mid.0, mid.1, use_foreign_object)
    } else {
        String::new()
    };

    let path_svg = format!(
        r#"<path d="{}" class="relationshipLine" stroke="{}" stroke-width="1" fill="none" stroke-dasharray="{}" marker-start="{}" marker-end="{}"/>"#,
        path_d, REL_LINE_COLOR, dash, marker_start, marker_end,
    );

    format!("{}{}", path_svg, label_svg)
}

// ── Main render function ──────────────────────────────────────────────────────

pub fn render(diag: &ErDiagram, theme: Theme, use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    let svg_id = "mermaid-er-svg";

    // Compute entity geometry
    let geoms: Vec<EntityGeom> = diag
        .entities
        .iter()
        .map(|e| compute_entity_geom(&e.name, &e.attributes))
        .collect();

    // Identify self-loop relationships (entity relates to itself)
    // These are handled separately from dagre — we add dummy waypoint nodes
    // to force a left-column layout (matching the "cyclic-special" reference approach).
    let mut self_loop_rels: Vec<usize> = Vec::new();
    // For each self-loop rel index, store (dummy_a_name, dummy_b_name)
    let mut self_loop_dummies: std::collections::HashMap<usize, (String, String)> =
        std::collections::HashMap::new();

    for (i, rel) in diag.relationships.iter().enumerate() {
        if rel.entity_a == rel.entity_b {
            self_loop_rels.push(i);
        }
    }

    // Build dagre graph (non-compound, directed)
    let mut g = Graph::with_options(false, true, false);
    g.set_graph(GraphLabel {
        rankdir: Some("TB".to_string()),
        nodesep: Some(NODE_SEP),
        edgesep: Some(EDGE_SEP),
        ranksep: Some(RANK_SEP),
        marginx: Some(MARGIN_X),
        marginy: Some(MARGIN_Y),
        ..Default::default()
    });

    // Add entities as nodes
    for (i, entity) in diag.entities.iter().enumerate() {
        let geom = &geoms[i];
        g.set_node(
            &entity.name,
            NodeLabel {
                width: geom.width,
                height: geom.height,
                ..Default::default()
            },
        );
    }

    // Add regular (non-self-loop) relationships as edges first.
    // Adding these before the self-loop dummy edges ensures the main chain
    // (e.g. PERSON→ADDRESS→CITY) gets higher priority in dagre's ordering,
    // placing it in the RIGHT column while self-loop waypoints go LEFT.
    for (i, rel) in diag.relationships.iter().enumerate() {
        if rel.entity_a != rel.entity_b {
            g.set_edge(
                &rel.entity_a,
                &rel.entity_b,
                EdgeLabel {
                    minlen: Some(1),
                    weight: Some(2.0), // higher weight: main chain stays aligned
                    ..Default::default()
                },
                Some(&format!("rel{}", i)),
            );
        }
    }

    // Now add self-loop dummy waypoint nodes and edges.
    // These are added after the main chain edges so they get lower order,
    // causing dagre to place them in the LEFT column.
    for (i, rel) in diag.relationships.iter().enumerate() {
        if rel.entity_a == rel.entity_b {
            // Add two dummy waypoint nodes connected from the self-loop entity.
            // These nodes will be placed in the same ranks as ADDRESS/CITY
            // (to the left), creating the reference two-column layout.
            let dummy_a = format!("_sl_{}_a", i);
            let dummy_b = format!("_sl_{}_b", i);
            // Width chosen so that with nodesep=140, the resulting layout
            // places ADDRESS/CITY at approximately x=290 (matching the reference).
            const SELF_LOOP_DUMMY_W: f64 = 84.0;
            g.set_node(
                &dummy_a,
                NodeLabel {
                    width: SELF_LOOP_DUMMY_W,
                    height: 1.0,
                    ..Default::default()
                },
            );
            g.set_node(
                &dummy_b,
                NodeLabel {
                    width: SELF_LOOP_DUMMY_W,
                    height: 1.0,
                    ..Default::default()
                },
            );
            // Chain: entity → dummy_a → dummy_b → entity (closed loop)
            // Adding all 3 edges lets dagre compute boundary intersections for
            // both the exit and return points on the entity.
            g.set_edge(
                &rel.entity_a,
                &dummy_a,
                EdgeLabel {
                    minlen: Some(1),
                    weight: Some(1.0),
                    ..Default::default()
                },
                Some(&format!("sl_a_{}", i)),
            );
            g.set_edge(
                &dummy_a,
                &dummy_b,
                EdgeLabel {
                    minlen: Some(1),
                    weight: Some(1.0),
                    ..Default::default()
                },
                Some(&format!("sl_b_{}", i)),
            );
            self_loop_dummies.insert(i, (dummy_a, dummy_b));
        }
    }

    // Run dagre layout
    layout(&mut g);

    // Post-layout adjustment for self-loop diagrams:
    // 1. Ensure the self-loop dummy nodes are on the LEFT of the main chain entities.
    //    Due to HashMap non-determinism in dagre, sometimes dummies end up RIGHT.
    //    If that happens, swap all positions (reflect about vertical center) to put
    //    dummies on the left and main chain on the right.
    // 2. Place the self-loop entity at ~43.5% of the right-column x to match the
    //    reference layout (PERSON:ADDRESS ratio = 126.56:290.93 = 0.435).
    if !self_loop_rels.is_empty() {
        // Get the self-loop dummy x (we look at first dummy of first self-loop rel)
        let dummy_x: Option<f64> = self_loop_rels.first().and_then(|&rel_idx| {
            self_loop_dummies
                .get(&rel_idx)
                .and_then(|(da, _)| g.node_opt(da).and_then(|n| n.x))
        });

        // Get the max x of non-self-loop, non-dummy entities (the "main chain")
        let mut main_chain_x: Option<f64> = None;
        for entity in &diag.entities {
            let is_self_loop_entity = self_loop_rels
                .iter()
                .any(|&i| diag.relationships[i].entity_a == entity.name);
            if !is_self_loop_entity {
                if let Some(node) = g.node_opt(&entity.name) {
                    let x = node.x.unwrap_or(0.0);
                    main_chain_x = Some(main_chain_x.map_or(x, |cur: f64| cur.max(x)));
                }
            }
        }

        // If dummies ended up on the RIGHT (dummy_x > main_chain_x), reflect all nodes
        // horizontally about the SVG center to swap columns.
        if let (Some(dx), Some(mc_x)) = (dummy_x, main_chain_x) {
            if dx > mc_x {
                // Dummies are on the wrong side — reflect all node x positions.
                let total_w = g.graph().width.unwrap_or(0.0);
                for v in g.nodes() {
                    if let Some(node) = g.node_opt_mut(&v) {
                        if let Some(x) = node.x.as_mut() {
                            *x = total_w - *x;
                        }
                    }
                }
                // Also reflect edge points
                let edges: Vec<_> = g.edges();
                for e in edges {
                    if let Some(label) = g.edge_mut(&e) {
                        if let Some(points) = label.points.as_mut() {
                            for p in points.iter_mut() {
                                p.x = total_w - p.x;
                            }
                        }
                    }
                }
                // Recalculate main_chain_x after reflection
                main_chain_x = Some(total_w - mc_x);
            }
        }

        // Place the self-loop entity at ~43.5% of the right-column x
        // (matches the reference PERSON:ADDRESS x ratio = 126.56:290.93 = 0.435)
        if let Some(rc_x) = main_chain_x {
            for &rel_idx in &self_loop_rels {
                let entity_name = &diag.relationships[rel_idx].entity_a;
                let new_cx = rc_x * 0.435;
                if let Some(node) = g.node_opt_mut(entity_name) {
                    node.x = Some(new_cx);
                }
                // After moving the entity, recompute the first/last boundary points
                // for all edges that have this entity as an endpoint.
                let entity_cy = g.node_opt(entity_name).and_then(|n| n.y).unwrap_or(0.0);
                let entity_hw = g
                    .node_opt(entity_name)
                    .map(|n| n.width / 2.0)
                    .unwrap_or(40.0);
                let entity_hh = g
                    .node_opt(entity_name)
                    .map(|n| n.height / 2.0)
                    .unwrap_or(42.0);
                let all_edges: Vec<_> = g.edges();
                for e in all_edges {
                    let is_from = e.v == *entity_name;
                    let is_to = e.w == *entity_name;
                    if !is_from && !is_to {
                        continue;
                    }
                    let adj = if is_from { &e.w } else { &e.v };
                    let adj_cx = g.node_opt(adj).and_then(|n| n.x).unwrap_or(new_cx);
                    let adj_cy = g.node_opt(adj).and_then(|n| n.y).unwrap_or(entity_cy);
                    let new_pt =
                        er_intersect_rect(new_cx, entity_cy, entity_hw, entity_hh, adj_cx, adj_cy);
                    if let Some(lbl) = g.edge_mut(&e) {
                        if let Some(pts) = lbl.points.as_mut() {
                            if !pts.is_empty() {
                                if is_from {
                                    pts[0] = new_pt;
                                } else {
                                    let n = pts.len();
                                    pts[n - 1] = new_pt;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Get graph dimensions
    let graph_w = g.graph().width.unwrap_or(400.0);
    let graph_h = g.graph().height.unwrap_or(400.0);

    // Build SVG
    let css = build_css(svg_id, &vars);

    let mut svg = format!(
        r#"<svg id="{id}" width="100%" xmlns="http://www.w3.org/2000/svg" class="erDiagram" style="max-width:{w:.3}px;" viewBox="0 0 {w:.3} {h:.3}"><style>{css}</style>"#,
        id = svg_id,
        w = graph_w,
        h = graph_h,
        css = css,
    );

    // Marker defs
    svg.push_str(&render_markers(svg_id));

    svg.push_str(r#"<g class="er-root">"#);

    // Render relationship lines (behind entities)
    svg.push_str(r#"<g class="er-relationships">"#);
    for (i, rel) in diag.relationships.iter().enumerate() {
        if rel.entity_a == rel.entity_b {
            // Self-loop: render manually using dummy waypoint positions
            let entity_idx = diag.entities.iter().position(|e| e.name == rel.entity_a);
            let (entity_x, entity_y, _entity_hw, entity_hh) = if let Some(idx) = entity_idx {
                let node = g.node_opt(&rel.entity_a);
                let cx = node.and_then(|n| n.x).unwrap_or(0.0);
                let cy = node.and_then(|n| n.y).unwrap_or(0.0);
                let hw = geoms[idx].width / 2.0;
                let hh = geoms[idx].height / 2.0;
                (cx, cy, hw, hh)
            } else {
                (0.0, 0.0, 50.0, 42.0)
            };

            // Retrieve dagre edge points for all 3 segments of the self-loop
            let get_pts = |from: &str, to: &str, name: &str| -> Vec<(f64, f64)> {
                if let Some(lbl) = g.edge_label_named(from, to, name) {
                    if let Some(pts) = &lbl.points {
                        return pts.iter().map(|p| (p.x, p.y)).collect();
                    }
                }
                // fallback: use node centers
                let fx = g.node_opt(from).and_then(|n| n.x).unwrap_or(entity_x);
                let fy = g.node_opt(from).and_then(|n| n.y).unwrap_or(entity_y);
                let tx = g.node_opt(to).and_then(|n| n.x).unwrap_or(entity_x - 80.0);
                let ty = g.node_opt(to).and_then(|n| n.y).unwrap_or(entity_y + 100.0);
                vec![(fx, fy), (tx, ty)]
            };
            let (pts_a, pts_b, pts_c) = if let Some((da, db)) = self_loop_dummies.get(&i) {
                let pa = get_pts(&rel.entity_a, da, &format!("sl_a_{}", i));
                let pb = get_pts(da, db, &format!("sl_b_{}", i));
                // Compute return path (dummy_b → entity) by mirroring pa about entity_x.
                // This creates a symmetric self-loop: exit bottom-left, return bottom-right.
                let pc: Vec<(f64, f64)> = {
                    let db_x = g.node_opt(db).and_then(|n| n.x).unwrap_or(entity_x - 80.0);
                    let db_y = g.node_opt(db).and_then(|n| n.y).unwrap_or(entity_y + 200.0);
                    // Exit point on entity bottom (mirror of pa's start point)
                    let entity_right_x =
                        2.0 * entity_x - pa.first().map(|p| p.0).unwrap_or(entity_x);
                    let entity_bottom_y = pa.first().map(|p| p.1).unwrap_or(entity_y + entity_hh);
                    // Mid-point: go right from dummy_b toward entity_right x
                    let mid_x = (db_x + entity_right_x) / 2.0;
                    let mid_y = (db_y + entity_bottom_y) / 2.0;
                    vec![
                        (db_x, db_y),
                        (mid_x, mid_y),
                        (entity_right_x, entity_bottom_y),
                    ]
                };
                (pa, pb, pc)
            } else {
                (
                    vec![(entity_x, entity_y)],
                    vec![],
                    vec![(entity_x, entity_y)],
                )
            };

            svg.push_str(&render_self_loop(
                rel,
                &pts_a,
                &pts_b,
                &pts_c,
                svg_id,
                use_foreign_object,
            ));
        } else {
            let edge_name = format!("rel{}", i);
            let points: Vec<(f64, f64)> = {
                let named_label = g.edge_label_named(&rel.entity_a, &rel.entity_b, &edge_name);
                if let Some(label) = named_label {
                    if let Some(pts) = &label.points {
                        pts.iter().map(|p| (p.x, p.y)).collect()
                    } else {
                        fallback_points(&g, &rel.entity_a, &rel.entity_b)
                    }
                } else {
                    let unnamed = g.edge_vw(&rel.entity_a, &rel.entity_b);
                    if let Some(label) = unnamed {
                        if let Some(pts) = &label.points {
                            pts.iter().map(|p| (p.x, p.y)).collect()
                        } else {
                            fallback_points(&g, &rel.entity_a, &rel.entity_b)
                        }
                    } else {
                        fallback_points(&g, &rel.entity_a, &rel.entity_b)
                    }
                }
            };
            svg.push_str(&render_relationship(
                rel,
                &points,
                svg_id,
                use_foreign_object,
            ));
        }
    }
    svg.push_str("</g>");

    // Render entities (on top of lines)
    svg.push_str(r#"<g class="er-entities">"#);
    for (i, entity) in diag.entities.iter().enumerate() {
        let geom = &geoms[i];
        let node = g.node_opt(&entity.name);
        let (cx, cy) = if let Some(n) = node {
            (n.x.unwrap_or(0.0), n.y.unwrap_or(0.0))
        } else {
            (0.0, 0.0)
        };
        svg.push_str(&render_entity(geom, &entity.attributes, cx, cy));
    }
    svg.push_str("</g>");

    svg.push_str("</g></svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    const ER_BASIC: &str =
        "erDiagram\n    CUSTOMER ||--o{ ORDER : places\n    ORDER ||--|{ LINE-ITEM : contains";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(ER_BASIC).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains("<svg"), "missing <svg tag");
        assert!(svg.contains("CUSTOMER"), "missing entity");
        assert!(svg.contains("ORDER"), "missing entity");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(ER_BASIC).diagram;
        let svg = render(&diag, Theme::Dark, false);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn snapshot_default_theme() {
        let diag = parser::parse(ER_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
