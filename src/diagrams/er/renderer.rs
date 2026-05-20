// ER diagram renderer — port of erRenderer-unified.ts + erBox.ts (Mermaid v11)
//
// erRenderer-unified.ts uses:
//   nodeSpacing = conf.nodeSpacing || 140 → dagre nodesep (unused for single column)
//   rankSpacing = conf.rankSpacing || 80  → dagre ranksep
//   setupViewPortForSVG(svg, padding=8, ...)
//
// erBox.ts entity sizing (htmlLabels=false path):
//   PADDING     = diagramPadding = 20
//   TEXT_PADDING = entityPadding * 1.25 = 15 * 1.25 = 18.75  (non-htmlLabels)
//   No-attr:  h = FO_H + 2 * PADDING * 1.5 = 24 + 60 = 84
//             w = max(minEntityWidth, label_w + PADDING * 2)
//   With-attr: row_h = FO_H + TEXT_PADDING = 42.75
//              h     = (n_attrs + 1) * row_h
//              w     = max(label_w + ATTR_PADDING*2, sum_col_widths_with_padding)
//
// Dagre graph: marginx=8, marginy=8, nodesep=100, ranksep=100
// ViewBox: svgBounds expanded by padding=8 on all sides

use super::constants::*;
use super::parser::{Attribute, AttributeKeyType, ErDiagram, Identification};
use super::templates::{
    attr_row_rect, build_css, entity_box_rect, entity_group_open, entity_h_divider,
    entity_v_divider, esc, fo_label, marker_end, marker_start, midpoint, render_markers,
    render_relationship, self_loop_edge_label, self_loop_path_end, self_loop_path_mid,
    self_loop_path_start,
};
use crate::text::measure;
use crate::theme::Theme;
use dagre_dgl_rs::graph::{EdgeLabel, Graph, GraphLabel, NodeLabel};
use dagre_dgl_rs::layout::layout;

// ── Measurement helpers ────────────────────────────────────────────────────────

fn tw(text: &str, font_size: f64) -> f64 {
    measure(text, font_size).0 * TEXT_SCALE
}

// ── Entity geometry ───────────────────────────────────────────────────────────

struct EntityGeom {
    width: f64,
    height: f64,
    // Whether label is centered (no-attr) or top-aligned (with-attr)
    label_centered: bool,
    // Column x-offsets from entity left (for with-attr entities)
    type_col_x: f64,
    name_col_x: f64,
    key_col_x: f64,
    comm_col_x: f64,
    has_key: bool,
    has_comment: bool,
}

fn compute_entity_geom(entity_name: &str, alias: &str, attrs: &[Attribute]) -> EntityGeom {
    let label_text = if !alias.is_empty() && alias != entity_name {
        alias
    } else {
        entity_name
    };
    let label_w = tw(label_text, FONT_SIZE);

    if attrs.is_empty() {
        // erBox.ts no-attr: drawRect with labelPaddingX=DIAGRAM_PADDING, labelPaddingY=DIAGRAM_PADDING*1.5
        let h = (FO_H + 2.0 * DIAGRAM_PADDING * 1.5).max(MIN_ENTITY_H); // = 84
        let w = (label_w + 2.0 * DIAGRAM_PADDING).max(MIN_ENTITY_W);
        return EntityGeom {
            width: w,
            height: h,
            label_centered: true,
            type_col_x: 0.0,
            name_col_x: 0.0,
            key_col_x: 0.0,
            comm_col_x: 0.0,
            has_key: false,
            has_comment: false,
        };
    }

    // erBox.ts with-attr (non-htmlLabels path):
    //   PADDING *= 1.25 = 25, TEXT_PADDING = entityPadding * 1.25 = 18.75
    let has_key = attrs.iter().any(|a| !a.attribute_key_type_list.is_empty());
    let has_comment = attrs.iter().any(|a| !a.attribute_comment.is_empty());

    // Per-column max text widths (erBox.ts line 103-156)
    let mut max_type_tw: f64 = 0.0;
    let mut max_name_tw: f64 = 0.0;
    let mut max_key_tw: f64 = 0.0;
    let mut max_comm_tw: f64 = 0.0;

    for attr in attrs {
        max_type_tw = max_type_tw.max(tw(&attr.attribute_type, FONT_SIZE));
        max_name_tw = max_name_tw.max(tw(&attr.attribute_name, FONT_SIZE));
        if has_key {
            max_key_tw = max_key_tw.max(tw(&attr_key_str(attr), FONT_SIZE));
        }
        if has_comment {
            max_comm_tw = max_comm_tw.max(tw(&attr.attribute_comment, FONT_SIZE));
        }
    }

    // erBox.ts: maxTypeWidth = max(typeBBox.width + PADDING), i.e. text_w + ATTR_PADDING
    let mut max_type_col = max_type_tw + ATTR_PADDING;
    let mut max_name_col = max_name_tw + ATTR_PADDING;
    let mut max_key_col = if has_key {
        max_key_tw + ATTR_PADDING
    } else {
        0.0
    };
    let mut max_comm_col = if has_comment {
        max_comm_tw + ATTR_PADDING
    } else {
        0.0
    };

    // erBox.ts: keysPresent = maxKeysWidth > PADDING
    let keys_present = max_key_col > ATTR_PADDING;
    let comment_present = max_comm_col > ATTR_PADDING;
    if !keys_present {
        max_key_col = 0.0;
    }
    if !comment_present {
        max_comm_col = 0.0;
    }

    // totalWidthSections: columns that exist
    let total_sections =
        2.0 + if keys_present { 1.0 } else { 0.0 } + if comment_present { 1.0 } else { 0.0 };

    let max_width = max_type_col + max_name_col + max_key_col + max_comm_col;

    // erBox.ts line 173-186: expand columns if label is wider
    let label_span = label_w + ATTR_PADDING * 2.0;
    if label_span > max_width {
        let diff = label_span - max_width;
        let per_col = diff / total_sections;
        max_type_col += per_col;
        max_name_col += per_col;
        if keys_present {
            max_key_col += per_col;
        }
        if comment_present {
            max_comm_col += per_col;
        }
    }

    // shapeBBox.width ≈ label_w (entity label dominates when wider than attrs)
    // erBox.ts: w = max(shapeBBox.width + PADDING*2, node.width, maxWidth)
    let w = (label_w + ATTR_PADDING * 2.0)
        .max(max_type_col + max_name_col + max_key_col + max_comm_col)
        .max(MIN_ENTITY_W);

    // h = (n_attrs + 1) * ROW_H
    let h = (attrs.len() as f64 + 1.0) * ROW_H;

    // Column x positions from entity left
    let type_col_x = 0.0;
    let name_col_x = type_col_x + max_type_col;
    let key_col_x = name_col_x + max_name_col;
    let comm_col_x = key_col_x + max_key_col;

    EntityGeom {
        width: w,
        height: h,
        label_centered: false,
        type_col_x,
        name_col_x,
        key_col_x,
        comm_col_x,
        has_key: keys_present,
        has_comment: comment_present,
    }
}

fn attr_key_str(attr: &Attribute) -> String {
    attr.attribute_key_type_list
        .iter()
        .map(|k| match k {
            AttributeKeyType::PK => "PK",
            AttributeKeyType::FK => "FK",
            AttributeKeyType::UK => "UK",
        })
        .collect::<Vec<_>>()
        .join(",")
}

// ── SVG entity rendering (erBox.ts style) ─────────────────────────────────────
//
// Text labels use <foreignObject> so resvg (used in visual regression) skips them
// while browsers render them correctly — matching the reference SVG structure.

#[allow(clippy::too_many_arguments)]
fn render_entity_svg(
    entity_name: &str,
    alias: &str,
    attrs: &[Attribute],
    geom: &EntityGeom,
    tx: f64,
    ty: f64,
    entity_id: &str,
    vars: &crate::theme::ThemeVars,
) -> String {
    let mut s = String::new();
    let w = geom.width;
    let h = geom.height;
    let label_text = if !alias.is_empty() && alias != entity_name {
        alias
    } else {
        entity_name
    };
    let label_w = tw(label_text, FONT_SIZE);
    let fill = vars.primary_color;
    let stroke = vars.primary_border;

    s.push_str(&entity_group_open(entity_id, tx, ty));
    s.push_str(&entity_box_rect(w, h, fill, stroke));

    if geom.label_centered {
        // No-attr: label centered (drawRect behavior)
        // erBox.ts: g.label translate(-label_w/2, -FO_H/2) → from entity top-left: (w/2-label_w/2, h/2-FO_H/2)
        s.push_str(&fo_label(
            w / 2.0 - label_w / 2.0,
            h / 2.0 - FO_H / 2.0,
            label_w,
            FO_H,
            label_text,
            "",
        ));
    } else {
        // With-attr: entity name label centered in the name row (y=0 to y=ROW_H)
        s.push_str(&fo_label(
            w / 2.0 - label_w / 2.0,
            (ROW_H - FO_H) / 2.0,
            label_w,
            FO_H,
            label_text,
            "",
        ));

        // Horizontal divider after name row
        s.push_str(&entity_h_divider(ROW_H, w, stroke));

        // Attribute rows
        for (idx, attr) in attrs.iter().enumerate() {
            let content_row_index = idx + 1;
            let is_even = content_row_index % 2 == 0;
            let row_fill = if is_even { vars.primary_color } else { "white" };
            let row_y = ROW_H + idx as f64 * ROW_H;

            s.push_str(&attr_row_rect(
                if is_even {
                    "attributeBoxEven"
                } else {
                    "attributeBoxOdd"
                },
                row_y,
                w,
                ROW_H,
                row_fill,
                stroke,
            ));

            // Attr text FO: centered vertically in the row box
            // row spans row_y to row_y+ROW_H; FO height=FO_H → center at row_y + ROW_H/2
            let attr_fo_y = row_y + (ROW_H - FO_H) / 2.0;
            let type_w = tw(&attr.attribute_type, FONT_SIZE);
            let name_w = tw(&attr.attribute_name, FONT_SIZE);

            s.push_str(&fo_label(
                geom.type_col_x + ATTR_PADDING / 2.0,
                attr_fo_y,
                type_w,
                FO_H,
                &attr.attribute_type,
                "",
            ));
            s.push_str(&fo_label(
                geom.name_col_x + ATTR_PADDING / 2.0,
                attr_fo_y,
                name_w,
                FO_H,
                &attr.attribute_name,
                "",
            ));

            if geom.has_key {
                let key_str = attr_key_str(attr);
                let key_w = tw(&key_str, FONT_SIZE);
                s.push_str(&fo_label(
                    geom.key_col_x + ATTR_PADDING / 2.0,
                    attr_fo_y,
                    key_w,
                    FO_H,
                    &key_str,
                    "",
                ));
            }
            if geom.has_comment {
                let comm_w = tw(&attr.attribute_comment, FONT_SIZE);
                s.push_str(&fo_label(
                    geom.comm_col_x + ATTR_PADDING / 2.0,
                    attr_fo_y,
                    comm_w,
                    FO_H,
                    &attr.attribute_comment,
                    "font-style:italic;",
                ));
            }
        }

        // Vertical dividers
        s.push_str(&entity_v_divider(geom.name_col_x, ROW_H, h, stroke));
        if geom.has_key {
            s.push_str(&entity_v_divider(geom.key_col_x, ROW_H, h, stroke));
        }
        if geom.has_comment {
            s.push_str(&entity_v_divider(geom.comm_col_x, ROW_H, h, stroke));
        }
    }

    s.push_str("</g>");
    s
}

fn get_edge_name(rel: &super::parser::ErRelationship) -> String {
    format!("{}{}{}", rel.entity_a, rel.role_a, rel.entity_b).replace(' ', "")
}

// ── Main render ───────────────────────────────────────────────────────────────

pub fn render(diag: &ErDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let svg_id = "mermaid-svg";

    // Compute entity geometries
    let geoms: Vec<EntityGeom> = diag
        .entities
        .iter()
        .map(|e| compute_entity_geom(&e.id, &e.alias, &e.attributes))
        .collect();

    // Build dagre graph (marginx=marginy=8, same as setupViewPortForSVG padding)
    let mut g = Graph::with_options(true, false, true);
    g.set_graph(GraphLabel {
        rankdir: Some("TB".to_string()),
        nodesep: Some(NODESEP),
        edgesep: Some(EDGESEP),
        ranksep: Some(RANKSEP),
        marginx: Some(MARGINX),
        marginy: Some(MARGINY),
        ..Default::default()
    });

    for (i, entity) in diag.entities.iter().enumerate() {
        g.set_node(
            &entity.id,
            NodeLabel {
                width: geoms[i].width,
                height: geoms[i].height,
                ..Default::default()
            },
        );
    }

    // JS: for self-loops (start==end), split into 3 edges through 2 virtual nodes (10×10)
    // This replicates the "cyclic-special" logic in the mermaid bundle so dagre places
    // the virtual nodes in the same ranks as the real downstream nodes, producing the
    // correct 2-column layout (e.g. PERSON vs ADDRESS/CITY in er_optional).
    let mut cyclic_map: std::collections::HashMap<String, (String, String)> = Default::default();

    for (ci, rel) in diag.relationships.iter().enumerate() {
        let edge_name = get_edge_name(rel);
        if g.node_opt(&rel.entity_a).is_none() || g.node_opt(&rel.entity_b).is_none() {
            continue;
        }
        if rel.entity_a == rel.entity_b {
            // Self-loop: create two tiny virtual nodes
            let sp1 = format!("__cyclic_{ci}_sp1");
            let sp2 = format!("__cyclic_{ci}_sp2");
            g.set_node(
                &sp1,
                NodeLabel {
                    width: 10.0,
                    height: 10.0,
                    ..Default::default()
                },
            );
            g.set_node(
                &sp2,
                NodeLabel {
                    width: 10.0,
                    height: 10.0,
                    ..Default::default()
                },
            );
            g.set_edge(
                &rel.entity_a,
                &sp1,
                EdgeLabel::default(),
                Some(&format!("{edge_name}-cyc0")),
            );
            g.set_edge(
                &sp1,
                &sp2,
                EdgeLabel::default(),
                Some(&format!("{edge_name}-cyc1")),
            );
            g.set_edge(
                &sp2,
                &rel.entity_b,
                EdgeLabel::default(),
                Some(&format!("{edge_name}-cyc2")),
            );
            cyclic_map.insert(edge_name, (sp1, sp2));
        } else {
            g.set_edge(
                &rel.entity_a,
                &rel.entity_b,
                EdgeLabel::default(),
                Some(&edge_name),
            );
        }
    }

    layout(&mut g);

    let css = build_css(svg_id, ff, &vars);
    let markers = render_markers(svg_id, &vars);

    // adjustEntities: translate to (node.x - w/2, node.y - h/2)
    let mut entities_svg = String::new();
    let mut content_min_x = f64::INFINITY;
    let mut content_min_y = f64::INFINITY;
    let mut content_max_x = f64::NEG_INFINITY;
    let mut content_max_y = f64::NEG_INFINITY;

    for (i, entity) in diag.entities.iter().enumerate() {
        if let Some(node) = g.node_opt(&entity.id) {
            if let (Some(cx), Some(cy)) = (node.x, node.y) {
                let tx = cx - geoms[i].width / 2.0;
                let ty = cy - geoms[i].height / 2.0;
                let eid = format!("{svg_id}-entity-{}", entity.id.replace(' ', "_"));
                entities_svg.push_str(&render_entity_svg(
                    &entity.id,
                    &entity.alias,
                    &entity.attributes,
                    &geoms[i],
                    tx,
                    ty,
                    &eid,
                    &vars,
                ));
                content_min_x = content_min_x.min(tx);
                content_min_y = content_min_y.min(ty);
                content_max_x = content_max_x.max(tx + geoms[i].width);
                content_max_y = content_max_y.max(ty + geoms[i].height);
            }
        }
    }

    let mut rels_svg = String::new();
    for rel in diag.relationships.iter() {
        let edge_name = get_edge_name(rel);

        if let Some((sp1, sp2)) = cyclic_map.get(&edge_name) {
            let lc = vars.line_color;
            let dasharray = if rel.rel_spec.rel_type == Identification::NonIdentifying {
                " stroke-dasharray:8,8;"
            } else {
                ""
            };
            // Self-loop: 3 separate paths matching JS "cyclic-special" structure
            // edge1: entity→sp1  (start marker only)
            // edgeMid: sp1→sp2  (no markers, carries the label)
            // edge2: sp2→entity  (end marker only)
            let mut pts0 = edge_points(&g, &rel.entity_a, sp1, &format!("{edge_name}-cyc0"));
            let pts1 = edge_points(&g, sp1, sp2, &format!("{edge_name}-cyc1"));
            let mut pts2 = edge_points(&g, sp2, &rel.entity_b, &format!("{edge_name}-cyc2"));

            for pts in [&pts0, &pts1, &pts2] {
                for &(px, py) in pts.iter() {
                    content_min_x = content_min_x.min(px);
                    content_min_y = content_min_y.min(py);
                    content_max_x = content_max_x.max(px);
                    content_max_y = content_max_y.max(py);
                }
            }

            // Fix attachment points: force edge to exit/enter entity on its BOTTOM edge
            // Dagre may route through the LEFT edge when sp1 is very far left; we
            // recompute the boundary intersection using the bottom edge instead.
            if let Some(node) = g.node_opt(&rel.entity_a) {
                if let (Some(cx), Some(cy)) = (node.x, node.y) {
                    let entity_idx = diag.entities.iter().position(|e| e.id == rel.entity_a);
                    let h = entity_idx.map(|i| geoms[i].height).unwrap_or(84.0);
                    let bottom = cy + h / 2.0;
                    // Fix pts0 first point: use PERSON-center → sp1-center direction
                    // to find the BOTTOM-edge exit. Using pts0[1] as direction gives
                    // a left-edge exit when sp1 is very far left (different from ref).
                    if !pts0.is_empty() {
                        if let Some(sp1_node) = g.node_opt(sp1) {
                            if let (Some(sp1_x), Some(sp1_y)) = (sp1_node.x, sp1_node.y) {
                                let dy = sp1_y - cy;
                                if dy.abs() > 0.001 {
                                    let t = (bottom - cy) / dy;
                                    let ax = cx + t * (sp1_x - cx);
                                    let ew = entity_idx.map(|i| geoms[i].width).unwrap_or(0.0);
                                    let attach_x = ax.clamp(cx - ew / 2.0, cx + ew / 2.0);
                                    pts0[0] = (attach_x, bottom);
                                }
                            }
                        }
                    }
                    // Fix pts2 last point: intersection of pts2[n-2]→(cx,cy) with bottom edge
                    let n = pts2.len();
                    if n >= 2 {
                        let (wx, wy) = pts2[n - 2];
                        let dy = cy - wy;
                        if dy.abs() > 0.001 {
                            let t = (bottom - wy) / dy;
                            let attach_x = wx + t * (cx - wx);
                            pts2[n - 1] = (attach_x, bottom);
                        }
                    }
                }
            }

            // edge1: start marker (cardB), no end marker
            if !pts0.is_empty() {
                let d = crate::svg::curve_basis_path(&pts0);
                let ms = marker_start(rel, svg_id);
                rels_svg.push_str(&self_loop_path_start(&d, lc, dasharray, &ms));
            }
            // edgeMid: no markers, label here
            if !pts1.is_empty() {
                let d = crate::svg::curve_basis_path(&pts1);
                rels_svg.push_str(&self_loop_path_mid(&d, lc, dasharray));
                // Label at midpoint of middle segment
                if !rel.role_a.is_empty() {
                    let (raw_lx, ly) = midpoint(&pts1);
                    let lbl_w = tw(&rel.role_a, REL_FONT_SIZE);
                    let fo_h = REL_FONT_SIZE * 1.5;
                    // Clamp label x so FO left edge >= content_min_x (no viewBox expansion needed)
                    let lx = raw_lx.max(content_min_x + lbl_w / 2.0);
                    rels_svg.push_str(&self_loop_edge_label(
                        lx,
                        ly,
                        -lbl_w / 2.0,
                        -fo_h / 2.0,
                        lbl_w,
                        fo_h,
                        REL_FONT_SIZE,
                        &esc(&rel.role_a),
                    ));
                }
            }
            // edge2: end marker (cardA), no start marker
            if !pts2.is_empty() {
                let d = crate::svg::curve_basis_path(&pts2);
                let me = marker_end(rel, svg_id);
                rels_svg.push_str(&self_loop_path_end(&d, lc, dasharray, &me));
            }
        } else {
            let points = edge_points(&g, &rel.entity_a, &rel.entity_b, &edge_name);
            if !points.is_empty() {
                for &(px, py) in &points {
                    content_min_x = content_min_x.min(px);
                    content_min_y = content_min_y.min(py);
                    content_max_x = content_max_x.max(px);
                    content_max_y = content_max_y.max(py);
                }
                rels_svg.push_str(&render_relationship(rel, &points, svg_id, &vars));
            }
        }
    }

    // setupViewPortForSVG: viewBox = (x-pad, y-pad, w+2*pad, h+2*pad)
    if content_min_x.is_infinite() {
        content_min_x = 0.0;
        content_min_y = 0.0;
        content_max_x = 200.0;
        content_max_y = 200.0;
    }
    let vb_x = content_min_x - VB_PAD;
    let vb_y = content_min_y - VB_PAD;
    let vb_w = (content_max_x - content_min_x) + VB_PAD * 2.0;
    let vb_h = (content_max_y - content_min_y) + VB_PAD * 2.0;

    format!(
        "<svg id=\"{svg_id}\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" \
         class=\"erDiagram\" style=\"max-width:{vb_w:.3}px;\" \
         viewBox=\"{vb_x:.3} {vb_y:.3} {vb_w:.3} {vb_h:.3}\">\
         <style>{css}</style>\
         {markers}\
         {rels_svg}\
         {entities_svg}\
         </svg>"
    )
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn edge_points(g: &Graph, from: &str, to: &str, name: &str) -> Vec<(f64, f64)> {
    g.edge_label_named(from, to, name)
        .and_then(|lbl| {
            lbl.points
                .as_ref()
                .map(|pts| pts.iter().map(|p| (p.x, p.y)).collect())
        })
        .unwrap_or_default()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_default_theme() {
        let input =
            "erDiagram\n    CUSTOMER ||--o{ ORDER : places\n    ORDER ||--|{ LINE-ITEM : contains";
        let diag = super::super::parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        insta::assert_snapshot!(svg);
    }
}
