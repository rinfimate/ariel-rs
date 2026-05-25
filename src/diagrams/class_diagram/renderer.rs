// Mermaid class diagram renderer — faithful port of classRenderer-v3.ts
// Uses dagre for layout; SVG structure mirrors reference output.

use super::constants::*;
use super::parser::{ClassDiagram, ClassNode, ClassRelation, EndType, LineStyle};
use super::templates::{
    self as tmpl, build_markers, drop_shadow_filter, drop_shadow_filter_small, edge_label_empty,
    edge_label_text, esc, fmt, svg_root, terminal_label_text,
};
use crate::backends::layout;
use crate::backends::{measure, measure_bold};
use crate::text_browser_metrics::measure_browser;
use crate::theme::{Theme, ThemeVars};
use dagre_dgl_rs::graph::{EdgeLabel, Graph, GraphLabel, NodeLabel, Point};

/// Pre-computed layout for a namespace's interior. Built once per namespace before
/// the outer dagre layout runs, then used to (1) size the namespace's opaque node in
/// the outer graph and (2) compose final child positions after outer layout.
struct NsLayout {
    /// Outer node dimensions in the main graph (= inner sub-graph size + padding + title bar).
    outer_w: f64,
    outer_h: f64,
    /// Each child's (x, y) within the sub-graph coordinate frame.
    child_positions: std::collections::HashMap<String, (f64, f64)>,
    /// Where the sub-graph origin sits relative to the namespace's outer top-left corner
    /// (i.e. padding on the left, padding + title bar on the top).
    sub_origin_x: f64,
    sub_origin_y: f64,
}

// ─── Public entry points ──────────────────────────────────────────────────────

pub fn render(diag: &ClassDiagram, theme: Theme, _use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    render_inner(diag, &vars)
}

fn render_inner(diag: &ClassDiagram, vars: &ThemeVars) -> String {
    // Build a lookup: class_id -> namespace_name
    let mut class_ns: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for (ns_name, ns_classes) in &diag.namespaces {
        for cls_id in ns_classes {
            class_ns.insert(cls_id.clone(), ns_name.clone());
        }
    }

    // Compute class node sizes once (used for both sub-graph and outer-graph layouts).
    let node_sizes: Vec<(String, f64, f64)> = diag
        .class_order
        .iter()
        .filter_map(|id| {
            let cls = diag.classes.get(id)?;
            let (w, h) = class_box_size(cls);
            Some((id.clone(), w, h))
        })
        .collect();
    let class_size: std::collections::HashMap<&str, (f64, f64)> = node_sizes
        .iter()
        .map(|(id, w, h)| (id.as_str(), (*w, *h)))
        .collect();

    // ── Step 1: Pre-compute each namespace's inner layout (matches Mermaid's
    // "extract cluster contents into separate graph, replace with clusterNode"
    // pattern from dagre/index.js adjustClustersAndEdges).
    //
    // For each namespace, build a sub-graph containing its child classes plus any
    // intra-namespace edges. Run dagre on it. Record (sub_w, sub_h) and each child's
    // position relative to the sub-graph origin. Then in the outer graph we replace
    // the namespace with an opaque fixed-size node.
    //
    // Cross-boundary edges (edges with one endpoint inside the namespace, one outside)
    // are kept in the outer graph with the namespace ID as the endpoint.
    // CLUSTER_PAD must equal the CLUSTER_H_PAD/CLUSTER_V_PAD used at render time
    // (35) so the outer opaque-node size we send to dagre matches the visual cluster
    // rect we draw later. Mermaid achieves the same via updateNodeBounds reading the
    // rendered cluster element after recursiveRender.
    const CLUSTER_PAD: f64 = 35.0;
    const CLUSTER_TITLE_AREA: f64 = 0.0; // padding already includes title space
    let mut ns_layouts: std::collections::HashMap<String, NsLayout> =
        std::collections::HashMap::new();
    for (ns_name, ns_classes) in &diag.namespaces {
        let mut sg = Graph::with_options(false, true, true);
        // Sub-graph margins are 0 because CLUSTER_PAD (35) covers the visual padding
        // around the cluster contents; adding marginx/y here would double-count.
        sg.set_graph(GraphLabel {
            rankdir: Some(diag.direction.clone()),
            nodesep: Some(50.0),
            ranksep: Some(50.0),
            marginx: Some(0.0),
            marginy: Some(0.0),
            ..Default::default()
        });
        for child_id in ns_classes {
            if let Some(&(w, h)) = class_size.get(child_id.as_str()) {
                sg.set_node(
                    child_id,
                    NodeLabel {
                        width: w,
                        height: h,
                        ..Default::default()
                    },
                );
            }
        }
        // Intra-namespace edges only
        for (i, rel) in diag.relations.iter().enumerate() {
            let v_in = ns_classes.iter().any(|c| c == &rel.id1);
            let w_in = ns_classes.iter().any(|c| c == &rel.id2);
            if v_in && w_in {
                let (lbl_w, lbl_h) = if !rel.title.is_empty() {
                    let (tw, _) = measure(&rel.title, 16.0);
                    // Mermaid passes labelGroup.getBBox() to dagre, which includes the
                    // background rect with +2px padding on each side (createText.ts:178-183).
                    (tw + 4.0, 21.0)
                } else {
                    (0.0, 0.0)
                };
                sg.set_edge(
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
                    Some(&format!("e{}", i)),
                );
            }
        }
        layout(&mut sg);
        let sub_w = sg.graph().width.unwrap_or(50.0);
        let sub_h = sg.graph().height.unwrap_or(50.0);
        let mut child_positions = std::collections::HashMap::new();
        for child_id in ns_classes {
            if let Some(n) = sg.node_opt(child_id) {
                child_positions.insert(child_id.clone(), (n.x.unwrap_or(0.0), n.y.unwrap_or(0.0)));
            }
        }
        ns_layouts.insert(
            ns_name.clone(),
            NsLayout {
                outer_w: sub_w + 2.0 * CLUSTER_PAD,
                outer_h: sub_h + 2.0 * CLUSTER_PAD + CLUSTER_TITLE_AREA,
                child_positions,
                sub_origin_x: CLUSTER_PAD,
                sub_origin_y: CLUSTER_PAD + CLUSTER_TITLE_AREA,
            },
        );
    }

    // ── Step 2: Build the outer dagre graph (NO compound — clusters are now opaque).
    let mut g = Graph::with_options(false, true, true);
    g.set_graph(GraphLabel {
        rankdir: Some(diag.direction.clone()),
        nodesep: Some(50.0),
        ranksep: Some(50.0),
        marginx: Some(8.0),
        marginy: Some(8.0),
        ..Default::default()
    });

    // Namespaces FIRST as opaque fixed-size nodes (matches Mermaid's getData() order).
    for (ns_name, _) in &diag.namespaces {
        if let Some(nsl) = ns_layouts.get(ns_name) {
            g.set_node(
                ns_name,
                NodeLabel {
                    width: nsl.outer_w,
                    height: nsl.outer_h,
                    ..Default::default()
                },
            );
        }
    }

    // Classes that are NOT in any namespace go directly into outer graph.
    for (id, w, h) in &node_sizes {
        if class_ns.contains_key(id) {
            continue; // handled inside its namespace sub-graph
        }
        g.set_node(
            id,
            NodeLabel {
                width: *w,
                height: *h,
                ..Default::default()
            },
        );
    }

    // Note nodes, each followed by its note→class edge.
    for (ni, note) in diag.notes.iter().enumerate() {
        let note_id = format!("__note_{}", ni);
        let (tw, _) = measure(&note.text, 16.0);
        let note_w = (tw + 20.0).max(60.0);
        let note_h = 30.0_f64;
        g.set_node(
            &note_id,
            NodeLabel {
                width: note_w,
                height: note_h,
                ..Default::default()
            },
        );
        // If the target class is inside a namespace, route the edge to the namespace.
        let target = class_ns
            .get(&note.class_id)
            .cloned()
            .unwrap_or_else(|| note.class_id.clone());
        g.set_edge(
            &note_id,
            &target,
            EdgeLabel {
                minlen: Some(1),
                weight: Some(1.0),
                ..Default::default()
            },
            Some(&format!("note_edge_{}", ni)),
        );
    }

    // Relation edges — endpoints inside a namespace become the namespace ID.
    for (i, rel) in diag.relations.iter().enumerate() {
        // Skip intra-namespace edges (already handled in the sub-graph layout).
        let v_ns = class_ns.get(&rel.id1);
        let w_ns = class_ns.get(&rel.id2);
        if let (Some(a), Some(b)) = (v_ns, w_ns) {
            if a == b {
                continue;
            }
        }
        let src = v_ns.cloned().unwrap_or_else(|| rel.id1.clone());
        let dst = w_ns.cloned().unwrap_or_else(|| rel.id2.clone());
        if src == dst {
            continue;
        }
        let key = Some(format!("e{}", i));
        let (lbl_w, lbl_h) = if !rel.title.is_empty() {
            let (tw, _) = measure(&rel.title, 16.0);
            // Mermaid passes labelGroup.getBBox() to dagre, which includes the
            // background rect with +2px padding on each side (createText.ts:178-183).
            (tw + 4.0, 21.0)
        } else {
            (0.0, 0.0)
        };
        g.set_edge(
            &src,
            &dst,
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

    // ── Step 3: Top-align each namespace cluster with the tallest node in its rank.
    // Dagre center-aligns all nodes in a rank, so a 196-tall cluster next to a
    // 288-tall class node ends up with its top 46px below the class's top. Mermaid
    // instead top-aligns clusters with the rank's tallest member, which is what the
    // reference SVG shows. We replicate this by collecting nodes per rank (matched
    // by their y center) and shifting each cluster up by (max_h - cluster_h)/2.
    let mut rank_max_h: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();
    for id in &g.nodes() {
        if let Some(n) = g.node_opt(id) {
            // Skip namespace nodes themselves so they don't dominate their own rank
            if diag.namespaces.iter().any(|(k, _)| k == id) {
                continue;
            }
            let y = n.y.unwrap_or(0.0);
            let key = (y * 10.0).round() as i64; // 0.1px rank-bucketing
            let h = n.height;
            rank_max_h
                .entry(key)
                .and_modify(|m| {
                    if h > *m {
                        *m = h;
                    }
                })
                .or_insert(h);
        }
    }

    // Compose final positions for namespace children, applying the top-alignment shift.
    for (ns_name, ns_classes) in &diag.namespaces {
        if let (Some(nl), Some(nsl)) = (g.node_opt(ns_name), ns_layouts.get(ns_name)) {
            let ns_x = nl.x.unwrap_or(0.0);
            let ns_y = nl.y.unwrap_or(0.0);
            // Top-align the cluster with this rank's tallest non-cluster node.
            let rank_key = (ns_y * 10.0).round() as i64;
            let tallest_in_rank = rank_max_h.get(&rank_key).copied().unwrap_or(nl.height);
            let shift_up = (tallest_in_rank - nl.height) / 2.0;
            let ns_y_adj = ns_y - shift_up;
            let ns_left = ns_x - nsl.outer_w / 2.0;
            let ns_top = ns_y_adj - nsl.outer_h / 2.0;
            // Mutate the namespace node so cluster rendering downstream sees the shifted y.
            if let Some(nm) = g.node_opt_mut(ns_name) {
                nm.y = Some(ns_y_adj);
            }
            for child_id in ns_classes {
                if let (Some(&(w, h)), Some(&(cx, cy))) = (
                    class_size.get(child_id.as_str()),
                    nsl.child_positions.get(child_id),
                ) {
                    g.set_node(
                        child_id,
                        NodeLabel {
                            x: Some(ns_left + nsl.sub_origin_x + cx),
                            y: Some(ns_top + nsl.sub_origin_y + cy),
                            width: w,
                            height: h,
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    let graph_w_dagre = g.graph().width.unwrap_or(200.0);
    let graph_h = g.graph().height.unwrap_or(200.0);

    // Mermaid uses getBBox() on the full SVG, which includes cardinality terminal labels that
    // can extend beyond dagre's computed graph width. dagre_w already includes marginx; we only
    // expand if terminal labels push further right.
    let mut graph_w = graph_w_dagre;
    for (i, rel) in diag.relations.iter().enumerate() {
        let edge_key = format!("e{}", i);
        let e = dagre_dgl_rs::graph::Edge::named(&rel.id1, &rel.id2, &edge_key);
        if let Some(lbl_data) = g.edge(&e) {
            let pts = lbl_data.points.clone().unwrap_or_default();
            if pts.len() < 2 {
                continue;
            }
            if !rel.title1.is_empty() {
                let (cx, _) = calc_terminal_label_position(10.0, TerminalPos::StartRight, &pts);
                let w = measure_browser(&rel.title1, 11.0).0;
                // Start labels: foreignObject inside centered inner group → right = cx + w/2
                // +8 converts from content coords to viewBox coords (replicates getBBox + padding)
                graph_w = graph_w.max(cx + w / 2.0 + 8.0);
            }
            if !rel.title2.is_empty() {
                let (cx, _) = calc_terminal_label_position(10.0, TerminalPos::EndLeft, &pts);
                // End labels: foreignObject at group origin, extends cx + chars*9 (Mermaid's
                // setTerminalWidth uses value.length * 9 px). +8 for viewBox padding.
                let chars = rel.title2.chars().count() as f64;
                graph_w = graph_w.max(cx + chars * 9.0 + 8.0);
            }
        }
    }

    let svg_id = "mermaid-svg";

    let mut out = String::new();

    out.push_str(&svg_root(svg_id, &fmt(graph_w), &fmt(graph_h)));

    out.push_str("<g>");
    out.push_str(&build_markers(svg_id, vars.primary_color, vars.line_color));
    out.push_str("</g>");

    out.push_str(r#"<g class="root">"#);

    // Namespace clusters
    // From reference SVG: rect x=8,y=8,w=160,h=196; Cat center=(88,106), Cat half_h=63
    // → left/right pad = 8px, top/bottom total = 35px (label sits at y+8 inside box)
    const CLUSTER_H_PAD: f64 = 35.0;
    const CLUSTER_V_PAD: f64 = 35.0;
    out.push_str(r#"<g class="clusters">"#);
    for (ns_name, ns_classes) in &diag.namespaces {
        if let Some(nl) = g.node_opt(ns_name) {
            let cx = nl.x.unwrap_or(0.0);
            // Compute cluster bounds from actual child node positions
            let mut min_x = cx;
            let mut max_x = cx;
            let mut min_y = nl.y.unwrap_or(0.0);
            let mut max_y = min_y;
            for child_id in ns_classes {
                if let Some(cn) = g.node_opt(child_id) {
                    let ccx = cn.x.unwrap_or(0.0);
                    let ccy = cn.y.unwrap_or(0.0);
                    min_x = min_x.min(ccx - cn.width / 2.0);
                    max_x = max_x.max(ccx + cn.width / 2.0);
                    min_y = min_y.min(ccy - cn.height / 2.0);
                    max_y = max_y.max(ccy + cn.height / 2.0);
                }
            }
            let nw = (max_x - min_x) + CLUSTER_H_PAD * 2.0;
            let nh = (max_y - min_y) + CLUSTER_V_PAD * 2.0;
            let x = min_x - CLUSTER_H_PAD;
            let y = min_y - CLUSTER_V_PAD;
            out.push_str(&tmpl::namespace_group_open(&esc(ns_name)));
            out.push_str(&tmpl::namespace_rect(&fmt(x), &fmt(y), &fmt(nw), &fmt(nh)));
            // Label: 8px from box top, centered (matches ref translate(_, 8) + 16px text)
            let label_cx = x + nw / 2.0;
            out.push_str(&tmpl::namespace_label(
                FONT_SIZE as u32,
                vars.primary_text,
                &fmt(label_cx),
                &fmt(y + FONT_SIZE),
                &esc(ns_name),
                vars.font_family,
            ));
            out.push_str("</g>");
        }
    }
    out.push_str("</g>");

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
    // Note edges — dashed lines from note node to its class
    for (ni, note) in diag.notes.iter().enumerate() {
        let note_id = format!("__note_{}", ni);
        let edge_key = format!("note_edge_{}", ni);
        let e = dagre_dgl_rs::graph::Edge::named(&note_id, &note.class_id, &edge_key);
        if let Some(lbl) = g.edge(&e) {
            let pts = lbl.points.clone().unwrap_or_default();
            if pts.len() >= 2 {
                let path_d = edge_path(&pts);
                out.push_str(&tmpl::note_edge_path(&path_d, vars.line_color));
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
                // Edge labels render at 16px; use SVG-calibrated metrics (matching Mermaid's
                // calculateTextWidth via SVG getBBox) rather than backends::measure.
                let (text_w, _) = measure_browser(&rel.title, 16.0);
                // Background rect width = text width + 2*padding (Mermaid createText.ts:178-183).
                let rect_w = text_w + 4.0;
                out.push_str(&edge_label_text(
                    &fmt(mid.0),
                    &fmt(mid.1),
                    &fmt(-rect_w / 2.0),
                    &fmt(rect_w),
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
            // title1: source label — placed beside the source arrowhead (start_right position)
            // Mermaid always uses terminalMarkerSize=10 because arrowTypeStart is always a truthy
            // non-empty string ('none', 'aggregation', etc.) in JavaScript.
            if !rel.title1.is_empty() && apts.len() >= 2 {
                let (cx, cy) = calc_terminal_label_position(10.0, TerminalPos::StartRight, &apts);
                let w1 = measure_browser(&rel.title1, 11.0).0;
                out.push_str(&terminal_label_text(
                    &fmt(cx),
                    &fmt(cy),
                    vars.font_family,
                    vars.primary_text,
                    &esc(&rel.title1),
                    w1,
                ));
            }
            // title2: target label — placed beside the arrowhead tip (end_left position)
            if !rel.title2.is_empty() && apts.len() >= 2 {
                let (cx, cy) = calc_terminal_label_position(10.0, TerminalPos::EndLeft, &apts);
                let w2 = measure_browser(&rel.title2, 11.0).0;
                // Mermaid renders end-terminal labels 7px lower than source-terminal labels.
                // Source has `<g class="inner" transform="translate(0,-7)">` wrapping the text;
                // end's inner is empty so the -7 doesn't apply. Replicate by shifting cy down 7.
                out.push_str(&terminal_label_text(
                    &fmt(cx),
                    &fmt(cy + 7.0),
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
    // Note nodes — yellow sticky-note boxes
    for (ni, note) in diag.notes.iter().enumerate() {
        let note_id = format!("__note_{}", ni);
        if let Some(n) = g.node_opt(&note_id) {
            let cx = n.x.unwrap_or(0.0);
            let cy = n.y.unwrap_or(0.0);
            let nw = n.width;
            let nh = n.height;
            let x = cx - nw / 2.0;
            let y = cy - nh / 2.0;
            out.push_str(&tmpl::note_group_open(&fmt(cx), &fmt(cy)));
            out.push_str(&tmpl::note_rect(
                &fmt(-nw / 2.0),
                &fmt(-nh / 2.0),
                &fmt(nw),
                &fmt(nh),
            ));
            out.push_str(&tmpl::note_text(
                FONT_SIZE as u32,
                vars.primary_text,
                &esc(&note.text),
                vars.font_family,
            ));
            out.push_str("</g>");
            let _ = x;
            let _ = y;
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
/// Calibrated from ref class_basic Dog: 1-row section = 41 = 21 + 1*MEMBER_ROW_H.
/// (Mermaid: 10.5px pad top + n*row_h + 10.5px pad bottom = 21 + 20n.)
fn section_h_nonzero(rows: usize) -> f64 {
    rows as f64 * MEMBER_ROW_H + 21.0
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
    // Mermaid renders class box text in foreignObject HTML elements which inherit
    // the SVG root font-size (16px) rather than the g.classGroup text CSS (10px).
    // All width measurements must use 16px to match Mermaid's actual rendering.

    // ── Centred items: class name + annotations ──────────────────────────────
    let (name_w, _) = measure_bold(&cls.label, 16.0);

    let mut max_centred_w: f64 = name_w;
    for ann in &cls.annotations {
        // Use actual guillemet characters (U+00AB, U+00BB) that Mermaid displays —
        // these are narrower than ASCII "<<>>" and match the reference foreignObject widths.
        // Measure with actual Unicode chars; render with HTML entities to avoid encoding issues.
        let (raw_w, _) = measure(&format!("\u{00AB}{}\u{00BB}", ann), 16.0);
        max_centred_w = max_centred_w.max(raw_w);
    }

    let mut max_content_w: f64 = 0.0;
    for m in &cls.members {
        let (raw_w, _) = measure(&m.display_text(), 16.0);
        max_content_w = max_content_w.max(raw_w);
    }
    for m in &cls.methods {
        let (raw_w, _) = measure(&m.display_text(), 16.0);
        max_content_w = max_content_w.max(raw_w);
    }

    // ── shapeSvg bbox width ──────────────────────────────────────────────────
    let half_centred = max_centred_w / 2.0;
    let x_max = f64::max(half_centred, max_content_w);
    let bbox_w = x_max + half_centred;

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
        // Empty class: ref renders at h=77 = HEADER(41) + 18 + 18 (verified from ref class_basic
        // and class_multiplicity, where empty-class divider y=±19.5 with hh=38.5).
        (0, 0) => (18.0, 18.0),
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
        let ann_text = format!("\u{00AB}{}\u{00BB}", esc(ann));
        let row_centre_rel = i as f64 * ANNOTATION_H + ANNOTATION_H / 2.0;
        s.push_str(&tmpl::annotation_text(
            &fmt(row_centre_rel),
            16.0,
            pt,
            &ann_text,
            vars.font_family,
        ));
    }
    s.push_str("</g>");

    // ── Label group (class name) ──────────────────────────────────────────────────
    // Mermaid positions header text at a constant 20px below the header region top
    // (verified from ref class_associations label-group y=-18.5 vs box top=-38.5).
    // This differs from HEADER_H/2 = 21.5 which would put it at -17.
    let header_centre_y = ann_top_y + ann_rows as f64 * ANNOTATION_H + 20.0;
    let (name_fo_w, _) = measure_bold(&cls.label, 16.0);
    s.push_str(&tmpl::label_group_text(
        &fmt(-name_fo_w / 2.0),
        &fmt(header_centre_y),
        &fmt(name_fo_w / 2.0),
        16.0,
        pt,
        &esc(&cls.label),
        vars.font_family,
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
            16.0,
            pt,
            &esc(&text),
            vars.font_family,
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
            16.0,
            pt,
            &esc(&text),
            vars.font_family,
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
    // Match Mermaid's markerOffsets (utils/lineWithOffset.ts):
    //   aggregation/extension/composition = 17.25, dependency = 6, arrow_point = 4
    match end {
        EndType::Extension | EndType::Composition | EndType::Aggregation => 17.25,
        EndType::Arrow => 4.0,
        EndType::None => 0.0,
    }
}

fn start_trim(start: &EndType) -> f64 {
    match start {
        EndType::Extension | EndType::Composition | EndType::Aggregation => 17.25,
        EndType::Arrow => 4.0,
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
// terminalMarkerSize = 10 when an arrow marker is present, 0 otherwise.

#[derive(Clone, Copy)]
#[allow(dead_code)]
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
