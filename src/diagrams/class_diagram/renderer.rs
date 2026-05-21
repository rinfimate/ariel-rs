// Mermaid class diagram renderer — faithful port of classRenderer-v3.ts
// Uses dagre for layout; SVG structure mirrors reference output.

use super::constants::*;
use super::parser::{ClassDiagram, ClassNode, ClassRelation, EndType, LineStyle};
use super::templates::{
    self as tmpl, build_markers, drop_shadow_filter, drop_shadow_filter_small, edge_label_empty,
    edge_label_text, esc, fmt, svg_root, terminal_label_text_source, terminal_label_text_target,
};
use crate::text::measure;
use crate::text_browser_metrics::measure_browser;
use crate::theme::{Theme, ThemeVars};
use dagre_dgl_rs::graph::{EdgeLabel, Graph, GraphLabel, NodeLabel, Point};
use dagre_dgl_rs::layout::layout;

// ─── Public entry points ──────────────────────────────────────────────────────

pub fn render(diag: &ClassDiagram, theme: Theme, _use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    render_inner(diag, &vars)
}

fn render_inner(diag: &ClassDiagram, vars: &ThemeVars) -> String {
    let mut g = Graph::with_options(false, true, true);
    g.set_graph(GraphLabel {
        rankdir: Some(diag.direction.clone()),
        nodesep: Some(50.0),
        ranksep: Some(50.0),
        marginx: Some(8.0),
        marginy: Some(8.0),
        ..Default::default()
    });

    // Compute node sizes and add to graph
    let node_sizes: Vec<(String, f64, f64)> = diag
        .class_order
        .iter()
        .filter_map(|id| {
            let cls = diag.classes.get(id)?;
            let (w, h) = class_box_size(cls);
            Some((id.clone(), w, h))
        })
        .collect();

    for (id, w, h) in &node_sizes {
        g.set_node(
            id,
            NodeLabel {
                width: *w,
                height: *h,
                ..Default::default()
            },
        );
    }

    // Add edges — include label dimensions so dagre expands ranksep for labeled edges.
    // Edge labels in class diagrams are 24 px tall (one line); width is measured text.
    // Apply CONTENT_SCALE so dagre reserves the same space as the browser-rendered label.
    for (i, rel) in diag.relations.iter().enumerate() {
        let key = Some(format!("e{}", i));
        let (lbl_w, lbl_h) = if !rel.title.is_empty() {
            let (tw, _) = measure(&rel.title, FONT_SIZE);
            (tw * CONTENT_SCALE, 24.0)
        } else {
            (0.0, 0.0)
        };
        g.set_edge(
            &rel.id1,
            &rel.id2,
            EdgeLabel {
                minlen: Some(1),
                weight: Some(1.0),
                width: Some(lbl_w),
                height: Some(lbl_h),
                labelpos: Some("c".to_string()),
                ..Default::default()
            },
            key.as_deref(),
        );
    }

    layout(&mut g);

    let graph_w_dagre = g.graph().width.unwrap_or(200.0);
    let graph_h = g.graph().height.unwrap_or(200.0);

    // Mermaid's setupViewPortForSVG uses getBBox() on the full SVG, which includes cardinality
    // (edgeTerminal) labels.  In Mermaid's DOM structure, the terminal label foreignObject is a
    // direct child of the edgeTerminals group (no centering offset), so its CSS-styled width
    // (text.len() * 9 px) adds to the right edge of the bounding box.  We replicate that here
    // so our viewBox width matches the reference.
    //
    // Formula (mirrors Mermaid/browser getBBox result):
    //   content_right = max over all terminal labels of (terminal_cx + style_w_px)
    //   content_left  = marginx (leftmost node left edge ≈ marginx after dagre translate)
    //   viewBox_width = max(graph_w_dagre, content_right + marginx)
    let margin_x = 8.0_f64;
    let graph_w = f64::max(graph_w_dagre, margin_x);

    let svg_id = "mermaid-svg";

    let mut out = String::new();

    out.push_str(&svg_root(svg_id, &fmt(graph_w), &fmt(graph_h)));

    out.push_str("<g>");
    out.push_str(&build_markers(svg_id, vars.primary_color, vars.line_color));
    out.push_str("</g>");

    out.push_str(r#"<g class="root">"#);

    // clusters (none for basic class diagrams)
    out.push_str(r#"<g class="clusters"></g>"#);

    // edgePaths
    out.push_str(r#"<g class="edgePaths">"#);
    for (i, rel) in diag.relations.iter().enumerate() {
        let edge_key = format!("e{}", i);
        let e = dagre_dgl_rs::graph::Edge::named(&rel.id1, &rel.id2, &edge_key);
        if let Some(lbl) = g.edge(&e) {
            let pts = lbl.points.clone().unwrap_or_default();
            if pts.len() >= 2 {
                let edge_id = format!("{}-id_{}_{}_{}", svg_id, rel.id1, rel.id2, i + 1);
                let pts = trim_end(
                    &trim_start(&pts, start_trim(&rel.start)),
                    end_trim(&rel.end),
                );
                let path_d = edge_path(&pts);
                let is_dashed = rel.line_style == LineStyle::Dashed;
                let classes = if is_dashed {
                    " edge-thickness-normal edge-pattern-dashed relation"
                } else {
                    " edge-thickness-normal edge-pattern-solid relation"
                };
                let dasharray = if is_dashed { "3" } else { "0" };
                let marker_start = marker_start_attr(svg_id, rel);
                let marker_end = marker_end_attr(svg_id, rel);
                out.push_str(&tmpl::edge_path(
                    &path_d,
                    &edge_id,
                    classes,
                    vars.line_color,
                    dasharray,
                    &marker_start,
                    &marker_end,
                ));
            }
        }
    }
    out.push_str("</g>");

    // edgeLabels
    out.push_str(r#"<g class="edgeLabels">"#);
    for (i, rel) in diag.relations.iter().enumerate() {
        let edge_key = format!("e{}", i);
        let e = dagre_dgl_rs::graph::Edge::named(&rel.id1, &rel.id2, &edge_key);
        if let Some(lbl_data) = g.edge(&e) {
            let pts = lbl_data.points.clone().unwrap_or_default();
            // apts = pts (raw dagre points) — the raw endpoint is the node boundary which
            // correctly anchors cardinality labels near the arrowhead via calcTerminalLabelPosition.
            let apts = pts.clone();
            if !rel.title.is_empty() {
                let mid = midpoint(&pts);
                let (raw_fo_w, _) = measure(&rel.title, TITLE_FONT_SIZE);
                let fo_w = raw_fo_w * CONTENT_SCALE;
                out.push_str(&edge_label_text(
                    &fmt(mid.0),
                    &fmt(mid.1),
                    &fmt(-fo_w / 2.0),
                    &fmt(fo_w),
                    vars.primary_color,
                    vars.font_family,
                    vars.primary_text,
                    &esc(&rel.title),
                ));
            } else {
                out.push_str(&edge_label_empty());
            }

            // Render start/end cardinality labels (title1 = near id1, title2 = near id2)
            // Mermaid: terminalMarkerSize = arrowTypeStart/End ? 10 : 0
            // (10 when there is a marker/arrow at that end, 0 for plain/none)
            // title1: source label — placed beside the source arrowhead
            if !rel.title1.is_empty() && apts.len() >= 2 {
                let start_marker_size: f64 = if rel.start == EndType::None {
                    0.0
                } else {
                    10.0
                };
                let (cx, cy) =
                    calc_terminal_label_position(start_marker_size, TerminalPos::StartRight, &apts);
                let w1 = measure_browser(&rel.title1, 11.0).0;
                out.push_str(&terminal_label_text_source(
                    &fmt(cx - 5.0),
                    &fmt(cy + 5.0),
                    vars.font_family,
                    vars.primary_text,
                    &esc(&rel.title1),
                    w1,
                ));
            }
            // title2: target label — placed beside the arrowhead tip
            if !rel.title2.is_empty() && apts.len() >= 2 {
                let (cx, cy) = label_pos_near(&apts, false);
                let w2 = measure_browser(&rel.title2, 11.0).0;
                out.push_str(&terminal_label_text_target(
                    &fmt(cx),
                    &fmt(cy),
                    vars.font_family,
                    vars.primary_text,
                    &esc(&rel.title2),
                    w2,
                ));
            }
        }
    }
    out.push_str("</g>");

    // nodes
    out.push_str(r#"<g class="nodes">"#);
    for (class_idx, id) in diag.class_order.iter().enumerate() {
        if let Some(cls) = diag.classes.get(id) {
            if let Some(n) = g.node_opt(id) {
                let cx = n.x.unwrap_or(0.0);
                let cy = n.y.unwrap_or(0.0);
                let w = n.width;
                let h = n.height;
                let dom_id = format!("{}-classId-{}-{}", svg_id, id, class_idx);
                out.push_str(&render_class_node(cls, cx, cy, w, h, vars, &dom_id));
            }
        }
    }
    out.push_str("</g>");

    out.push_str("</g>"); // root

    out.push_str(&drop_shadow_filter(svg_id));
    out.push_str(&drop_shadow_filter_small(svg_id));

    out.push_str("</svg>");
    out
}

// ─── Class box sizing ─────────────────────────────────────────────────────────

/// Returns the height of a non-empty section (n > 0 guaranteed).
/// Mermaid: (n+1)*24px per section when rows > 0.
fn section_h_nonzero(rows: usize) -> f64 {
    (rows as f64 + 1.0) * MEMBER_ROW_H
}

/// Compute the total width and height of a class box.
///
/// Width formula mirrors Mermaid's DOM-based layout (classBox.ts / textHelper):
///   • Annotation and label groups are **centred** at x=0.
///   • Member and method groups are **left-aligned** starting at x=0.
///   • After layout the shapeSvg bbox spans:
///       x_min = −max(ann_w, name_w) / 2
///       x_max = max(max(ann_w, name_w)/2, max_content_w)
///       bbox_w = x_max − x_min
///   • The enclosing rectangle adds H_PAD on each side:
///       hw = bbox_w / 2 + H_PAD   →   full_w = bbox_w + 2*H_PAD
fn class_box_size(cls: &ClassNode) -> (f64, f64) {
    // ── Centred items: class name + annotations ──────────────────────────────
    // Apply NAME_SCALE to the bold class name and CONTENT_SCALE to italic annotations,
    // matching browser foreignObject rendering widths derived from reference SVGs.
    let (raw_name_w, _) = measure(&cls.label, FONT_SIZE);
    let name_w = raw_name_w * NAME_SCALE;

    let mut max_centred_w: f64 = name_w;
    for ann in &cls.annotations {
        // Use actual guillemet characters (U+00AB, U+00BB) that Mermaid displays —
        // these are narrower than ASCII "<<>>" and match the reference foreignObject widths.
        // Measure with actual Unicode chars; render with HTML entities to avoid encoding issues.
        let (raw_w, _) = measure(&format!("\u{00AB}{}\u{00BB}", ann), FONT_SIZE);
        max_centred_w = max_centred_w.max(raw_w * CONTENT_SCALE);
    }

    // ── Left-aligned items: members + methods ────────────────────────────────
    // Apply CONTENT_SCALE to regular text (member/method display strings).
    let mut max_content_w: f64 = 0.0;
    for m in &cls.members {
        let (raw_w, _) = measure(&m.display_text(), FONT_SIZE);
        max_content_w = max_content_w.max(raw_w * CONTENT_SCALE);
    }
    for m in &cls.methods {
        let (raw_w, _) = measure(&m.display_text(), FONT_SIZE);
        max_content_w = max_content_w.max(raw_w * CONTENT_SCALE);
    }

    // ── shapeSvg bbox width ──────────────────────────────────────────────────
    let half_centred = max_centred_w / 2.0;
    let x_max = f64::max(half_centred, max_content_w);
    let bbox_w = x_max + half_centred; // x_max − (−half_centred)

    // ── Full box width = bbox_w + 2*H_PAD, with a minimum ───────────────────
    let w = (bbox_w + H_PAD * 2.0).max(MIN_BOX_W);

    // ── Height ───────────────────────────────────────────────────────────────
    //   annotations:      ann_rows * 24
    //   header:           48  (always)
    //   members section:  section_h(member_rows)  — 18 if empty, (n+1)*24 if non-empty
    //   methods section:  section_h(method_rows)
    //
    // Mermaid DOM observation: when annotations are present and the members section
    // is empty, the classBox.ts bounding-box calculation produces an extra 6px for
    // the empty members section (24 instead of 18) AND an extra 6px in the methods
    // section when methods are non-empty.  Together this adds 12px in that case.
    let ann_rows = cls.annotations.len();
    let member_rows = cls.members.len();
    let method_rows = cls.methods.len();

    // Section heights — derived from Mermaid classBox.ts DOM measurements.
    // When one section is empty and the other is not, Mermaid's GAP/2=6px floor on
    // membersGroupHeight shifts the layout, producing section sizes of 24px (not 18).
    //   (m=0, me=0): members=18, methods=18
    //   (m>0, me=0): members=(m+1)*24, methods=24       ← methods floor = 24
    //   (m=0, me>0): members=24, methods=(me+1)*24 + 6  ← members floor=24; +6 shift
    //   (m>0, me>0): members=(m+1)*24, methods=(me+1)*24
    let (members_h, methods_h) = match (member_rows, method_rows) {
        (0, 0) => (EMPTY_SECTION_H, EMPTY_SECTION_H),
        (m, 0) => (section_h_nonzero(m), MEMBER_ROW_H),
        (0, me) => (MEMBER_ROW_H, section_h_nonzero(me) + 6.0),
        (m, me) => (section_h_nonzero(m), section_h_nonzero(me)),
    };

    let h = ann_rows as f64 * ANNOTATION_H + HEADER_H + members_h + methods_h;

    (w, h)
}

// ─── Node rendering ───────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn render_class_node(
    cls: &ClassNode,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    vars: &ThemeVars,
    dom_id: &str,
) -> String {
    let hw = w / 2.0;
    let hh = h / 2.0;
    let pb = vars.primary_border;
    let pt = vars.primary_text;
    let pf = vars.primary_color;

    let mut s = String::new();
    s.push_str(&tmpl::node_group(dom_id, &fmt(cx), &fmt(cy)));

    // Outer rectangle (filled, no stroke for shadow layer)
    s.push_str(&tmpl::node_outer_path(
        &fmt(-hw),
        &fmt(-hh),
        &fmt(hw),
        &fmt(hh),
        pf,
    ));
    // Sketchy border path (matches Mermaid neo-classic look)
    s.push_str(&tmpl::node_border_path(
        &fmt(-hw),
        &fmt(-hh),
        &fmt(hw),
        &fmt(hh),
        &fmt(-hw * 0.6),
        &fmt(hw * 0.4),
        &fmt(hw * 0.5),
        &fmt(-hw * 0.2),
        &fmt(-hh * 0.6),
        &fmt(hh * 0.5),
        &fmt(hh * 0.5),
        &fmt(-hh * 0.1),
        pb,
    ));

    // Y layout (all positions relative to node centre = 0, box spans -hh to +hh):
    //
    //   -hh ───────────────────── box top
    //        ann_rows * 24        annotation rows
    //   div1 ──────────────────── members divider  (= -hh + ann*24 + 48)
    //        section_h(members)   member rows       (0 rows → 18, n rows → (n+1)*24)
    //   div2 ──────────────────── methods divider  (= div1 + section_h(members))
    //        section_h(methods)   method rows
    //   +hh ───────────────────── box bottom

    let ann_rows = cls.annotations.len();
    let member_rows = cls.members.len();
    let method_rows = cls.methods.len();

    let ann_top_y = -hh;
    let div1_y = ann_top_y + ann_rows as f64 * ANNOTATION_H + HEADER_H;

    // Members section height — must match class_box_size() exactly.
    let members_section_h = match (member_rows, method_rows) {
        (0, 0) => EMPTY_SECTION_H,
        (0, _) => MEMBER_ROW_H, // floor = 24
        (m, _) => section_h_nonzero(m),
    };
    let div2_y = div1_y + members_section_h;

    // ── Annotation group ──────────────────────────────────────────────────────────
    let region_h = ann_rows as f64 * ANNOTATION_H + HEADER_H;
    let content_h = (ann_rows as f64 + 1.0) * ANNOTATION_H;
    let vert_pad = (region_h - content_h) / 2.0;
    let ann_group_y = if ann_rows > 0 {
        ann_top_y + vert_pad
    } else {
        ann_top_y + ann_rows as f64 * ANNOTATION_H + HEADER_H / 2.0
    };
    s.push_str(&tmpl::annotation_group(&fmt(ann_group_y)));
    for (i, ann) in cls.annotations.iter().enumerate() {
        let ann_text = format!("&laquo;{}&raquo;", esc(ann));
        let row_centre_rel = i as f64 * ANNOTATION_H + ANNOTATION_H / 2.0;
        s.push_str(&tmpl::annotation_text(
            &fmt(row_centre_rel),
            FONT_SIZE,
            pt,
            &ann_text,
        ));
    }
    s.push_str("</g>");

    // ── Label group (class name) ──────────────────────────────────────────────────
    let header_centre_y = ann_top_y + ann_rows as f64 * ANNOTATION_H + HEADER_H / 2.0;
    let (raw_name_fo_w, _) = measure(&cls.label, TITLE_FONT_SIZE);
    let name_fo_w = raw_name_fo_w * NAME_SCALE;
    s.push_str(&tmpl::label_group_text(
        &fmt(-name_fo_w / 2.0),
        &fmt(header_centre_y),
        &fmt(name_fo_w / 2.0),
        TITLE_FONT_SIZE,
        pt,
        &esc(&cls.label),
    ));

    // ── Members group ──────────────────────────────────────────────────────────────
    let members_group_y = div1_y + MEMBER_ROW_H;
    s.push_str(&tmpl::members_group(
        &fmt(-hw + H_PAD),
        &fmt(members_group_y),
    ));
    for (i, m) in cls.members.iter().enumerate() {
        let text = m.display_text();
        let row_y = i as f64 * MEMBER_ROW_H;
        s.push_str(&tmpl::member_row_text(
            &fmt(row_y),
            FONT_SIZE,
            pt,
            &esc(&text),
        ));
    }
    s.push_str("</g>");

    // ── Methods group ──────────────────────────────────────────────────────────────
    let methods_group_y = div2_y + MEMBER_ROW_H;
    s.push_str(&tmpl::methods_group(
        &fmt(-hw + H_PAD),
        &fmt(methods_group_y),
    ));
    for (i, m) in cls.methods.iter().enumerate() {
        let text = m.display_text();
        let row_y = i as f64 * MEMBER_ROW_H;
        s.push_str(&tmpl::member_row_text(
            &fmt(row_y),
            FONT_SIZE,
            pt,
            &esc(&text),
        ));
    }
    s.push_str("</g>");

    // ── Dividers ───────────────────────────────────────────────────────────────────
    // div1: between header and members section
    s.push_str(&tmpl::divider_path(
        &fmt(-hw),
        &fmt(div1_y),
        &fmt(-hw * 0.4),
        &fmt(hw * 0.4),
        &fmt(hw),
        pb,
    ));
    // div2: between members and methods section
    s.push_str(&tmpl::divider_path(
        &fmt(-hw),
        &fmt(div2_y),
        &fmt(-hw * 0.4),
        &fmt(hw * 0.4),
        &fmt(hw),
        pb,
    ));

    s.push_str("</g>"); // node
    s
}

// ─── Marker helpers ───────────────────────────────────────────────────────────

fn marker_start_attr(svg_id: &str, rel: &ClassRelation) -> String {
    match &rel.start {
        EndType::None => String::new(),
        EndType::Extension => format!(r#" marker-start="url(#{}_class-extensionStart)""#, svg_id),
        EndType::Composition => {
            format!(r#" marker-start="url(#{}_class-compositionStart)""#, svg_id)
        }
        EndType::Aggregation => {
            format!(r#" marker-start="url(#{}_class-aggregationStart)""#, svg_id)
        }
        EndType::Arrow => format!(r#" marker-start="url(#{}_class-dependencyStart)""#, svg_id),
    }
}

fn marker_end_attr(svg_id: &str, rel: &ClassRelation) -> String {
    match &rel.end {
        EndType::None => String::new(),
        EndType::Extension => format!(r#" marker-end="url(#{}_class-extensionEnd)""#, svg_id),
        EndType::Composition => format!(r#" marker-end="url(#{}_class-compositionEnd)""#, svg_id),
        EndType::Aggregation => format!(r#" marker-end="url(#{}_class-aggregationEnd)""#, svg_id),
        EndType::Arrow => format!(r#" marker-end="url(#{}_class-dependencyEnd)""#, svg_id),
    }
}

// ─── Edge path ────────────────────────────────────────────────────────────────

/// Arrowhead overhang = (tip_x - refX) for each End marker type.
/// The dagre edge endpoint lands on the node boundary; trimming pulls it back
/// so the arrowhead tip touches the boundary instead of being buried inside.
fn end_trim(end: &EndType) -> f64 {
    match end {
        EndType::Extension | EndType::Composition | EndType::Aggregation => 17.0,
        EndType::Arrow => 8.0,
        EndType::None => 0.0,
    }
}

fn start_trim(start: &EndType) -> f64 {
    match start {
        EndType::Extension | EndType::Composition | EndType::Aggregation => 17.0,
        EndType::Arrow => 8.0,
        EndType::None => 0.0,
    }
}

/// Trim `amount` units off the END of the last segment (toward source).
fn trim_end(pts: &[Point], amount: f64) -> Vec<Point> {
    if amount <= 0.0 || pts.len() < 2 {
        return pts.to_vec();
    }
    let mut result = pts.to_vec();
    let n = result.len();
    let last = result[n - 1].clone();
    let prev = result[n - 2].clone();
    let dx = last.x - prev.x;
    let dy = last.y - prev.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= amount {
        result.truncate(n - 1);
    } else {
        let frac = (len - amount) / len;
        result[n - 1] = Point {
            x: prev.x + dx * frac,
            y: prev.y + dy * frac,
        };
    }
    result
}

/// Trim `amount` units off the START of the first segment (toward target).
fn trim_start(pts: &[Point], amount: f64) -> Vec<Point> {
    if amount <= 0.0 || pts.len() < 2 {
        return pts.to_vec();
    }
    let mut result = pts.to_vec();
    let first = result[0].clone();
    let next = result[1].clone();
    let dx = next.x - first.x;
    let dy = next.y - first.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= amount {
        result.remove(0);
    } else {
        let frac = amount / len;
        result[0] = Point {
            x: first.x + dx * frac,
            y: first.y + dy * frac,
        };
    }
    result
}

fn edge_path(pts: &[Point]) -> String {
    let pairs: Vec<(f64, f64)> = pts.iter().map(|p| (p.x, p.y)).collect();
    crate::svg::curve_basis_path(&pairs)
}

fn midpoint(pts: &[Point]) -> (f64, f64) {
    if pts.is_empty() {
        return (0.0, 0.0);
    }
    let mid = pts.len() / 2;
    (pts[mid].x, pts[mid].y)
}

// ─── Terminal label positioning — faithful port of Mermaid utils.ts ──────────
//
// Mermaid uses calcTerminalLabelPosition(terminalMarkerSize, position, points)
// from packages/mermaid/src/utils.ts via positionEdgeLabel in edges.js.
//
// title1 (near source) → position 'start_right'
// title2 (near target) → position 'end_left'
//
/// Place a cardinality label directly beside the arrowhead.
/// is_source=true → label near the edge start; false → label near the edge end (arrowhead tip).
/// Offset: 10px perpendicular right of the edge direction, 20px back from the endpoint.
fn label_pos_near(pts: &[Point], is_source: bool) -> (f64, f64) {
    let n = pts.len();
    let (tip, prev) = if is_source {
        (&pts[0], &pts[1])
    } else {
        (&pts[n - 1], &pts[n - 2])
    };
    let dx = tip.x - prev.x;
    let dy = tip.y - prev.y;
    let len = (dx * dx + dy * dy).sqrt().max(1e-9);
    let (dirx, diry) = (dx / len, dy / len); // unit vector pointing FROM prev TO tip
                                             // travel direction = from tip TOWARD prev (forward along edge from source perspective)
                                             // perp left of travel direction = (diry, -dirx)
    let (lx, ly) = (diry, -dirx);

    if is_source {
        // Source label: LEFT of arrow, forward (toward target) from tip
        let perp = 14.0;
        let fwd = 17.0;
        // forward = (-dirx, -diry) = direction source→target
        let x = tip.x + lx * perp + (-dirx) * fwd;
        let y = tip.y + ly * perp + (-diry) * fwd;
        (x, y)
    } else {
        // Target label: box left edge aligned with arrow x + 10px right, 12px back from arrowhead
        let back = 17.0;
        let x = tip.x - dirx * back + 10.0;
        let y = tip.y - diry * back;
        (x, y)
    }
}

// terminalMarkerSize = 10 when an arrow marker is present, 0 otherwise.

#[derive(Clone, Copy)]
enum TerminalPos {
    StartRight,
    EndLeft,
}

/// Port of utils.calcTerminalLabelPosition.
/// Returns (x, y) for the outer group transform of the terminal label.
fn calc_terminal_label_position(
    terminal_marker_size: f64,
    position: TerminalPos,
    points: &[Point],
) -> (f64, f64) {
    // For end positions, reverse the point list so we always traverse from source.
    let fwd: Vec<(f64, f64)> = points.iter().map(|p| (p.x, p.y)).collect();
    let rev: Vec<(f64, f64)> = fwd.iter().cloned().rev().collect();
    let pts_owned: Vec<(f64, f64)> = match position {
        TerminalPos::StartRight => fwd,
        TerminalPos::EndLeft => rev,
    };
    let pts_ref: &[(f64, f64)] = &pts_owned;

    let distance_to_cardinality_point = 25.0 + terminal_marker_size;
    // We need calculatePoint over Point slices — use a helper that accepts (f64,f64) tuples.
    let center = {
        let mut prev: Option<(f64, f64)> = None;
        let mut remaining = distance_to_cardinality_point;
        let mut result = pts_ref[pts_ref.len() - 1];
        for &p in pts_ref {
            if let Some(prev_p) = prev {
                let dx = p.0 - prev_p.0;
                let dy = p.1 - prev_p.1;
                let seg_len = (dx * dx + dy * dy).sqrt();
                if seg_len == 0.0 {
                    prev = Some(p);
                    continue;
                }
                if seg_len < remaining {
                    remaining -= seg_len;
                } else {
                    let ratio = remaining / seg_len;
                    result = (
                        (1.0 - ratio) * prev_p.0 + ratio * p.0,
                        (1.0 - ratio) * prev_p.1 + ratio * p.1,
                    );
                    break;
                }
            }
            prev = Some(p);
        }
        result
    };

    let d = 10.0 + terminal_marker_size * 0.5;
    let p0 = pts_ref[0];
    let angle = f64::atan2(p0.1 - center.1, p0.0 - center.0);

    let (x, y) = match position {
        TerminalPos::StartRight => {
            // sin(angle)*d + (p0.x + center.x)/2
            // -cos(angle)*d + (p0.y + center.y)/2
            let x = angle.sin() * d + (p0.0 + center.0) / 2.0;
            let y = -angle.cos() * d + (p0.1 + center.1) / 2.0;
            (x, y)
        }
        TerminalPos::EndLeft => {
            // Mermaid source: sin(angle)*d + (p0.x+center.x)/2 - 5
            //                 -cos(angle)*d + (p0.y+center.y)/2 - 5
            let x = angle.sin() * d + (p0.0 + center.0) / 2.0 - 5.0;
            let y = -angle.cos() * d + (p0.1 + center.1) / 2.0 - 5.0;
            (x, y)
        }
    };

    (x, y)
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    const CLASS_BASIC: &str = "classDiagram\n    class Animal {\n        +String name\n        +int age\n        +makeSound() void\n    }\n    class Dog {\n        +String breed\n        +fetch() void\n    }\n    Animal <|-- Dog";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(CLASS_BASIC).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains("<svg"), "missing <svg tag");
        assert!(svg.contains("Animal"), "missing class name");
        assert!(svg.contains("Dog"), "missing class name");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(CLASS_BASIC).diagram;
        let svg = render(&diag, Theme::Dark, false);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn snapshot_default_theme() {
        let diag = parser::parse(CLASS_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
