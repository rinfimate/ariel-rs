// ER diagram SVG templates — port of erRenderer-unified.ts + erBox.ts (Mermaid v11)

use super::constants::*;
use super::parser::{Cardinality, ErRelationship, Identification};
use crate::theme::ThemeVars;

// ── Utilities ─────────────────────────────────────────────────────────────────

pub use crate::diagrams::util::esc;

// ── CSS ───────────────────────────────────────────────────────────────────────

pub fn build_css(svg_id: &str, ff: &str, vars: &ThemeVars) -> String {
    let pc = vars.primary_color;
    let pb = vars.primary_border;
    let lc = vars.line_color;
    let tc = vars.primary_text;
    format!(
        "#{svg_id}{{font-family:{ff};font-size:{FONT_SIZE}px;fill:{tc};}}\
         #{svg_id} .er.entityBox{{fill:{pc};stroke:{pb};}}\
         #{svg_id} .er.entityLabel{{fill:{tc};}}\
         #{svg_id} .er.attributeBoxOdd{{fill:white;stroke:{pb};}}\
         #{svg_id} .er.attributeBoxEven{{fill:{pc};stroke:{pb};}}\
         #{svg_id} .er.relationshipLine{{stroke:{lc};fill:none;}}\
         #{svg_id} .er.relationshipLabel{{fill:{tc};}}\
         #{svg_id} .er.relationshipLabelBox{{fill:white;opacity:0.85;}}",
    )
}

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

// ── Entity label (foreignObject) ─────────────────────────────────────────────

pub fn fo_label(x: f64, y: f64, w: f64, h: f64, text: &str, style: &str) -> String {
    format!(
        "<g class=\"label\" transform=\"translate({x:.3},{y:.3})\">\
         <foreignObject width=\"{w:.3}\" height=\"{h:.3}\">\
         <div xmlns=\"http://www.w3.org/1999/xhtml\" \
         style=\"display:table-cell;white-space:nowrap;line-height:1.5;\
         max-width:200px;text-align:center;{style}\">\
         <span class=\"nodeLabel\">{text}</span>\
         </div></foreignObject></g>",
        text = esc(text)
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

    // Edge label as foreignObject (matches reference erRenderer-unified edgeLabel structure)
    if !rel.role_a.is_empty() && points.len() >= 2 {
        let (lx, ly) = midpoint(points);
        let lbl_w = crate::text::measure(&rel.role_a, REL_FONT_SIZE).0 * TEXT_SCALE;
        let fo_h = REL_FONT_SIZE * 1.5; // 14 * 1.5 = 21
        s.push_str(&format!(
            "<g class=\"edgeLabel\" transform=\"translate({:.3},{:.3})\">\
             <g class=\"label\" transform=\"translate({:.3},{:.3})\">\
             <foreignObject width=\"{:.3}\" height=\"{:.3}\" style=\"font-size:{REL_FONT_SIZE}px;\">\
             <div xmlns=\"http://www.w3.org/1999/xhtml\" \
             class=\"labelBkg\" style=\"display:table-cell;white-space:nowrap;\
             line-height:1.5;text-align:center;\">\
             <span class=\"edgeLabel\">{}</span>\
             </div></foreignObject></g></g>",
            lx, ly,
            -lbl_w / 2.0, -fo_h / 2.0,
            lbl_w, fo_h,
            esc(&rel.role_a)
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

/// Render the self-loop edge label.
#[allow(clippy::too_many_arguments)]
pub fn self_loop_edge_label(
    lx: f64,
    ly: f64,
    ox: f64,
    oy: f64,
    lbl_w: f64,
    fo_h: f64,
    rel_font_size: f64,
    text: &str,
) -> String {
    format!(
        "<g class=\"edgeLabel\" transform=\"translate({lx:.3},{ly:.3})\">\
         <g class=\"label\" transform=\"translate({ox:.3},{oy:.3})\">\
         <foreignObject width=\"{lbl_w:.3}\" height=\"{fo_h:.3}\" style=\"font-size:{rel_font_size}px;\">\
         <div xmlns=\"http://www.w3.org/1999/xhtml\" class=\"labelBkg\" \
         style=\"display:table-cell;white-space:nowrap;line-height:1.5;text-align:center;\">\
         <span class=\"edgeLabel\">{text}</span>\
         </div></foreignObject></g></g>",
        lx = lx,
        ly = ly,
        ox = ox,
        oy = oy,
        lbl_w = lbl_w,
        fo_h = fo_h,
        rel_font_size = rel_font_size,
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
