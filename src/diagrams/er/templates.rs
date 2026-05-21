// ER diagram SVG templates — port of erRenderer-unified.ts + erBox.ts (Mermaid v11)

use super::constants::*;
use super::parser::{Cardinality, ErRelationship, Identification};
use crate::theme::ThemeVars;

// ── Utilities ─────────────────────────────────────────────────────────────────

pub use crate::diagrams::util::esc;

// ── Markers ───────────────────────────────────────────────────────────────────

pub fn render_markers(svg_id: &str, vars: &ThemeVars) -> String {
    let s = vars.line_color;
    format!(
        "<defs>\
         <marker id=\"{svg_id}-ONLY_ONE_START\" refX=\"0\" refY=\"9\" markerWidth=\"18\" markerHeight=\"18\" orient=\"auto\">\
         <path stroke=\"{s}\" fill=\"none\" d=\"M9,0 L9,18 M15,0 L15,18\"/></marker>\
         <marker id=\"{svg_id}-ONLY_ONE_END\" refX=\"18\" refY=\"9\" markerWidth=\"18\" markerHeight=\"18\" orient=\"auto\">\
         <path stroke=\"{s}\" fill=\"none\" d=\"M3,0 L3,18 M9,0 L9,18\"/></marker>\
         <marker id=\"{svg_id}-ZERO_OR_ONE_START\" refX=\"0\" refY=\"9\" markerWidth=\"30\" markerHeight=\"18\" orient=\"auto\">\
         <circle stroke=\"{s}\" fill=\"white\" cx=\"21\" cy=\"9\" r=\"6\"/>\
         <path stroke=\"{s}\" fill=\"none\" d=\"M9,0 L9,18\"/></marker>\
         <marker id=\"{svg_id}-ZERO_OR_ONE_END\" refX=\"30\" refY=\"9\" markerWidth=\"30\" markerHeight=\"18\" orient=\"auto\">\
         <circle stroke=\"{s}\" fill=\"white\" cx=\"9\" cy=\"9\" r=\"6\"/>\
         <path stroke=\"{s}\" fill=\"none\" d=\"M21,0 L21,18\"/></marker>\
         <marker id=\"{svg_id}-ONE_OR_MORE_START\" refX=\"18\" refY=\"18\" markerWidth=\"45\" markerHeight=\"36\" orient=\"auto\">\
         <path stroke=\"{s}\" fill=\"none\" d=\"M0,18 Q 18,0 36,18 Q 18,36 0,18 M42,9 L42,27\"/></marker>\
         <marker id=\"{svg_id}-ONE_OR_MORE_END\" refX=\"27\" refY=\"18\" markerWidth=\"45\" markerHeight=\"36\" orient=\"auto\">\
         <path stroke=\"{s}\" fill=\"none\" d=\"M3,9 L3,27 M9,18 Q27,0 45,18 Q27,36 9,18\"/></marker>\
         <marker id=\"{svg_id}-ZERO_OR_MORE_START\" refX=\"18\" refY=\"18\" markerWidth=\"57\" markerHeight=\"36\" orient=\"auto\">\
         <circle stroke=\"{s}\" fill=\"white\" cx=\"48\" cy=\"18\" r=\"6\"/>\
         <path stroke=\"{s}\" fill=\"none\" d=\"M0,18 Q18,0 36,18 Q18,36 0,18\"/></marker>\
         <marker id=\"{svg_id}-ZERO_OR_MORE_END\" refX=\"39\" refY=\"18\" markerWidth=\"57\" markerHeight=\"36\" orient=\"auto\">\
         <circle stroke=\"{s}\" fill=\"white\" cx=\"9\" cy=\"18\" r=\"6\"/>\
         <path stroke=\"{s}\" fill=\"none\" d=\"M21,18 Q39,0 57,18 Q39,36 21,18\"/></marker>\
         </defs>"
    )
}

pub fn marker_start(rel: &ErRelationship, svg_id: &str) -> String {
    let name = match rel.rel_spec.card_b {
        Cardinality::ZeroOrOne => "ZERO_OR_ONE_START",
        Cardinality::ZeroOrMore => "ZERO_OR_MORE_START",
        Cardinality::OneOrMore => "ONE_OR_MORE_START",
        Cardinality::OnlyOne => "ONLY_ONE_START",
        Cardinality::MdParent => "ONLY_ONE_START",
    };
    format!("url(#{svg_id}-{name})")
}

pub fn marker_end(rel: &ErRelationship, svg_id: &str) -> String {
    let name = match rel.rel_spec.card_a {
        Cardinality::ZeroOrOne => "ZERO_OR_ONE_END",
        Cardinality::ZeroOrMore => "ZERO_OR_MORE_END",
        Cardinality::OneOrMore => "ONE_OR_MORE_END",
        Cardinality::OnlyOne => "ONLY_ONE_END",
        Cardinality::MdParent => "ONLY_ONE_END",
    };
    format!("url(#{svg_id}-{name})")
}

// ── Entity label (native SVG text) ───────────────────────────────────────────

pub fn fo_label(x: f64, y: f64, _w: f64, h: f64, text: &str, style: &str, pt: &str) -> String {
    // Vertical center within the label box: y offset = h/2
    let text_y = y + h / 2.0;
    // italic style means attribute_comment column
    let font_style = if style.contains("italic") {
        " font-style=\"italic\""
    } else {
        ""
    };
    format!(
        "<g class=\"label\" transform=\"translate({x:.3},0)\">\
         <text font-family=\"Arial, sans-serif\" \
         font-size=\"{FONT_SIZE}\"{font_style} fill=\"{pt}\" \
         text-anchor=\"start\" dominant-baseline=\"middle\" \
         x=\"0\" y=\"{text_y:.3}\">{escaped}</text></g>",
        x = x,
        text_y = text_y,
        font_style = font_style,
        pt = pt,
        escaped = esc(text),
    )
}

// ── Relationship SVG ──────────────────────────────────────────────────────────

pub fn render_relationship(
    rel: &ErRelationship,
    points: &[(f64, f64)],
    svg_id: &str,
    vars: &ThemeVars,
) -> String {
    let mut s = String::new();
    let lc = vars.line_color;

    let path_d = crate::svg::curve_basis_path(points);

    let dasharray = if rel.rel_spec.rel_type == Identification::NonIdentifying {
        " stroke-dasharray:8,8;"
    } else {
        ""
    };

    let ms = marker_start(rel, svg_id);
    let me = marker_end(rel, svg_id);

    s.push_str(&format!(
        "<path class=\"er relationshipLine\" d=\"{path_d}\" \
         style=\"fill:none;stroke:{lc};{dasharray}\" \
         marker-start=\"{ms}\" marker-end=\"{me}\"/>"
    ));

    // Edge label with background rect
    if !rel.role_a.is_empty() && points.len() >= 2 {
        let (lx, ly) = midpoint(points);
        let lbl_w = crate::text_browser_metrics::measure_browser(&rel.role_a, REL_FONT_SIZE).0;
        s.push_str(&edge_label_text(
            lx,
            ly,
            &esc(&rel.role_a),
            vars.primary_text,
            vars.er_relation_label_bg,
            lbl_w,
        ));
    }

    s
}

// ── Entity SVG building blocks ────────────────────────────────────────────────

/// Render the opening `<g>` wrapper for an entity.
pub fn entity_group_open(entity_id: &str, tx: f64, ty: f64) -> String {
    format!(
        "<g id=\"{entity_id}\" transform=\"translate({tx:.3},{ty:.3})\">",
        entity_id = entity_id,
        tx = tx,
        ty = ty,
    )
}

/// Render the outer entity box `<rect>`.
pub fn entity_box_rect(w: f64, h: f64, fill: &str, stroke: &str) -> String {
    format!(
        "<rect class=\"er entityBox\" x=\"0\" y=\"0\" width=\"{w:.3}\" height=\"{h:.3}\" \
         fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"1\"/>",
        w = w,
        h = h,
        fill = fill,
        stroke = stroke,
    )
}

/// Render a horizontal divider `<line>` inside an entity.
pub fn entity_h_divider(row_h: f64, w: f64, stroke: &str) -> String {
    format!(
        "<line x1=\"0\" y1=\"{:.3}\" x2=\"{:.3}\" y2=\"{:.3}\" stroke=\"{stroke}\" stroke-width=\"1\"/>",
        row_h,
        w,
        row_h,
        stroke = stroke,
    )
}

/// Render an attribute row `<rect>` inside an entity.
pub fn attr_row_rect(class: &str, y: f64, w: f64, h: f64, fill: &str, stroke: &str) -> String {
    format!(
        "<rect class=\"er {class}\" x=\"0\" y=\"{y:.3}\" width=\"{w:.3}\" height=\"{h:.3}\" \
         fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"1\"/>",
        class = class,
        y = y,
        w = w,
        h = h,
        fill = fill,
        stroke = stroke,
    )
}

/// Render a vertical divider `<line>` inside an entity.
pub fn entity_v_divider(x: f64, y1: f64, y2: f64, stroke: &str) -> String {
    format!(
        "<line x1=\"{x:.3}\" y1=\"{y1:.3}\" x2=\"{x:.3}\" y2=\"{y2:.3}\" stroke=\"{stroke}\" stroke-width=\"1\"/>",
        x = x,
        y1 = y1,
        y2 = y2,
        stroke = stroke,
    )
}

/// Render a self-loop path with only a start marker (no end marker).
pub fn self_loop_path_start(d: &str, lc: &str, dasharray: &str, ms: &str) -> String {
    format!(
        "<path class=\"er relationshipLine\" d=\"{d}\" \
         style=\"fill:none;stroke:{lc};{dasharray}\" marker-start=\"{ms}\"/>",
        d = d,
        lc = lc,
        dasharray = dasharray,
        ms = ms,
    )
}

/// Render a self-loop middle path with no markers.
pub fn self_loop_path_mid(d: &str, lc: &str, dasharray: &str) -> String {
    format!(
        "<path class=\"er relationshipLine\" d=\"{d}\" \
         style=\"fill:none;stroke:{lc};{dasharray}\"/>",
        d = d,
        lc = lc,
        dasharray = dasharray,
    )
}

/// Render the self-loop edge label as a native SVG `<text>`.
#[allow(clippy::too_many_arguments)]
pub fn self_loop_edge_label(
    lx: f64,
    ly: f64,
    _ox: f64,
    _oy: f64,
    _lbl_w: f64,
    _fo_h: f64,
    rel_font_size: f64,
    text: &str,
    pt: &str,
) -> String {
    format!(
        "<text class=\"edgeLabel\" font-family=\"Arial, sans-serif\" \
         font-size=\"{rel_font_size}\" fill=\"{pt}\" text-anchor=\"middle\" \
         dominant-baseline=\"middle\" x=\"{lx:.3}\" y=\"{ly:.3}\">{text}</text>",
        lx = lx,
        ly = ly,
        rel_font_size = rel_font_size,
        pt = pt,
        text = text,
    )
}

/// Render a self-loop path with only an end marker (no start marker).
pub fn self_loop_path_end(d: &str, lc: &str, dasharray: &str, me: &str) -> String {
    format!(
        "<path class=\"er relationshipLine\" d=\"{d}\" \
         style=\"fill:none;stroke:{lc};{dasharray}\" marker-end=\"{me}\"/>",
        d = d,
        lc = lc,
        dasharray = dasharray,
        me = me,
    )
}

// ── Top-level SVG wrapper ─────────────────────────────────────────────────────

/// Render the outer `<svg>` element with markers, relationships and entities.
#[allow(clippy::too_many_arguments)]
pub fn svg_root(
    svg_id: &str,
    vb_x: f64,
    vb_y: f64,
    vb_w: f64,
    vb_h: f64,
    markers: &str,
    rels_svg: &str,
    entities_svg: &str,
) -> String {
    format!(
        "<svg id=\"{svg_id}\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" \
         class=\"erDiagram\" style=\"max-width:{vb_w:.3}px;\" \
         viewBox=\"{vb_x:.3} {vb_y:.3} {vb_w:.3} {vb_h:.3}\">\
         {markers}\
         {rels_svg}\
         {entities_svg}\
         </svg>",
    )
}

// ── Edge label helper ─────────────────────────────────────────────────────────

/// Render a relationship edge label as a native SVG `<text>`.
pub fn edge_label_text(x: f64, y: f64, text: &str, pt: &str, bg: &str, text_w: f64) -> String {
    // Height = REL_FONT_SIZE × 1.5 (line-height), matching Mermaid's foreignObject height.
    // No extra padding — the foreignObject background fills exactly text_w × line_h.
    let rw = text_w;
    let rh = REL_FONT_SIZE * 1.5;
    let rx = x - rw / 2.0;
    let ry = y - rh / 2.0;
    format!(
        "<rect class=\"relationshipLabelBox\" x=\"{rx:.3}\" y=\"{ry:.3}\" width=\"{rw:.3}\" height=\"{rh:.3}\" \
         fill=\"{bg}\"/>\
         <text class=\"edgeLabel er relationshipLabel\" \
         font-family=\"Arial, sans-serif\" \
         font-size=\"{REL_FONT_SIZE}\" fill=\"{pt}\" \
         text-anchor=\"middle\" dominant-baseline=\"middle\" \
         x=\"{x:.3}\" y=\"{y:.3}\">{text}</text>",
        rx = rx, ry = ry, rw = rw, rh = rh,
        bg = bg, x = x, y = y, pt = pt, text = text,
    )
}

// ── Internal helper ───────────────────────────────────────────────────────────

pub fn midpoint(points: &[(f64, f64)]) -> (f64, f64) {
    if points.is_empty() {
        return (0.0, 0.0);
    }
    let n = points.len();
    let mid = n / 2;
    if n % 2 == 1 {
        points[mid]
    } else {
        (
            (points[mid - 1].0 + points[mid].0) / 2.0,
            (points[mid - 1].1 + points[mid].1) / 2.0,
        )
    }
}
