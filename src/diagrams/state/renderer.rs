/// Faithful port of Mermaid's dagre/index.js + mermaid-graphlib.js + clusters.js
///
/// Pipeline:
///   render() → build compound graph → adjustClustersAndEdges → extractor →
///   layout main graph → apply title offset → recursiveRender → SVG wrapper
use super::constants::*;
use super::parser::{Edge as PEdge, Node as PNode, Shape, StateDiagram};
use super::templates::{self, esc};
use crate::backends::layout;
use crate::backends::measure;
use crate::theme::{Theme, ThemeVars};
use dagre_dgl_rs::graph::{Edge, EdgeLabel, Graph, GraphLabel, NodeLabel};
use std::collections::{HashMap, HashSet};

pub fn render(diag: &StateDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    do_render(&diag.nodes, &diag.edges, &diag.direction, &vars)
}

// ─── Main render ─────────────────────────────────────────────────────────────

fn do_render(pnodes: &[PNode], pedges: &[PEdge], direction: &str, vars: &ThemeVars) -> String {
    let mut g = Graph::with_options(true, true, true);
    g.set_graph(GraphLabel {
        rankdir: Some(direction.into()),
        nodesep: Some(NODE_SEP),
        ranksep: Some(RANK_SEP),
        marginx: Some(MARGIN),
        marginy: Some(MARGIN),
        ..Default::default()
    });

    for pn in pnodes {
        let (w, h) = node_size(pn);
        g.set_node(
            &pn.id,
            NodeLabel {
                width: w,
                height: h,
                ..Default::default()
            },
        );
        if let Some(ref pid) = pn.parent_id {
            if g.has_node(pid) {
                g.set_parent(&pn.id, Some(pid));
            }
        }
    }

    for (i, pe) in pedges.iter().enumerate() {
        if !g.has_node(&pe.start) || !g.has_node(&pe.end) {
            continue;
        }
        let lw = if pe.label.is_empty() {
            0.0
        } else {
            label_w(&pe.label)
        };
        // Edge label height: ref state_choice shows choice→False distance 101.5 with
        // edge label vs our 104.5 = 3 px less in ref. Mermaid uses font_size*1.3125 = 21
        // (line-height for 16px text in browser) not 24.
        let lh = if pe.label.is_empty() { 0.0 } else { 21.0 };
        g.set_edge(
            &pe.start,
            &pe.end,
            EdgeLabel {
                minlen: Some(1),
                weight: Some(1.0),
                width: Some(lw),
                height: Some(lh),
                labelpos: Some("c".into()),
                ..Default::default()
            },
            Some(&format!("e{}", i)),
        );
    }

    let mut cluster_db: HashMap<String, ClusterEntry> = HashMap::new();
    adjust_clusters_and_edges(&mut g, &mut cluster_db);

    let mut sub_graphs: HashMap<String, SubGraph> = HashMap::new();
    extractor(
        &mut g,
        direction,
        &mut cluster_db,
        pnodes,
        &mut sub_graphs,
        0,
    );

    layout(&mut g);
    apply_title_offset(&mut g, &cluster_db, pnodes);

    let mut body = String::new();
    recursive_render(
        &g,
        pedges,
        pnodes,
        &sub_graphs,
        &cluster_db,
        vars,
        &mut body,
    );

    let (vx, vy, vw, vh) = viewbox(&g, pedges, pnodes, &sub_graphs);
    templates::svg_root(
        vx,
        vy,
        vw,
        vh,
        &css(vars),
        &templates::arrow_marker(vars.state_transition_color),
        &body,
    )
}

// ─── Structures ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct ClusterEntry {
    anchor: String,
    external: bool,
}

struct SubGraph {
    graph: Graph,
    #[allow(dead_code)]
    dir: String,
}

// ─── adjustClustersAndEdges ───────────────────────────────────────────────────

fn adjust_clusters_and_edges(g: &mut Graph, cluster_db: &mut HashMap<String, ClusterEntry>) {
    for id in g.nodes() {
        if !g.children(&id).is_empty() {
            let anchor = find_non_cluster_child(&id, g).unwrap_or(id.clone());
            cluster_db.insert(
                id.clone(),
                ClusterEntry {
                    anchor,
                    external: false,
                },
            );
        }
    }
    let mut descendants: HashMap<String, HashSet<String>> = HashMap::new();
    for id in cluster_db.keys() {
        descendants.insert(id.clone(), extract_descendants(id, g));
    }
    for e in g.edges() {
        for (cid, descs) in &descendants {
            let d1 = descs.contains(&e.v);
            let d2 = descs.contains(&e.w);
            if d1 ^ d2 {
                if let Some(entry) = cluster_db.get_mut(cid) {
                    entry.external = true;
                }
            }
        }
    }
    let edges: Vec<Edge> = g.edges();
    let mut removes: Vec<Edge> = Vec::new();
    let mut adds: Vec<(String, String, EdgeLabel, String)> = Vec::new();
    for e in edges {
        let vc = cluster_db.contains_key(&e.v);
        let wc = cluster_db.contains_key(&e.w);
        if !vc && !wc {
            continue;
        }
        let new_v = if vc {
            cluster_db[&e.v].anchor.clone()
        } else {
            e.v.clone()
        };
        let new_w = if wc {
            cluster_db[&e.w].anchor.clone()
        } else {
            e.w.clone()
        };
        if new_v != e.v {
            if let Some(par) = g.parent(&new_v).map(|s| s.to_string()) {
                if let Some(en) = cluster_db.get_mut(&par) {
                    en.external = true;
                }
            }
        }
        if new_w != e.w {
            if let Some(par) = g.parent(&new_w).map(|s| s.to_string()) {
                if let Some(en) = cluster_db.get_mut(&par) {
                    en.external = true;
                }
            }
        }
        if let Some(lbl) = g.edge(&e) {
            removes.push(e.clone());
            adds.push((
                new_v,
                new_w,
                lbl.clone(),
                e.name.clone().unwrap_or_default(),
            ));
        }
    }
    for e in removes {
        g.remove_edge_obj(&e);
    }
    for (v, w, lbl, name) in adds {
        if g.has_node(&v) && g.has_node(&w) {
            g.set_edge(&v, &w, lbl, Some(&name));
        }
    }
}

fn extract_descendants(id: &str, g: &Graph) -> HashSet<String> {
    let mut res = HashSet::new();
    for c in g.children(id) {
        res.insert(c.clone());
        res.extend(extract_descendants(&c, g));
    }
    res
}

fn find_non_cluster_child(id: &str, g: &Graph) -> Option<String> {
    let children = g.children(id);
    if children.is_empty() {
        return Some(id.to_string());
    }
    for c in &children {
        if g.children(c).is_empty() {
            return Some(c.clone());
        }
        if let Some(r) = find_non_cluster_child(c, g) {
            return Some(r);
        }
    }
    None
}

// ─── extractor ────────────────────────────────────────────────────────────────
//
// Direction logic:
//   - explicit dir → use it
//   - concurrent region (all children are dividers) → SAME dir as parent
//     (keeps dividers in same rank → side-by-side in TB or LR)
//   - otherwise → flip TB↔LR

#[allow(clippy::only_used_in_recursion)]
fn extractor(
    g: &mut Graph,
    parent_dir: &str,
    cluster_db: &mut HashMap<String, ClusterEntry>,
    pnodes: &[PNode],
    sub_graphs: &mut HashMap<String, SubGraph>,
    depth: usize,
) {
    if depth > 10 {
        return;
    }
    let nodes: Vec<String> = g.nodes();
    if !nodes.iter().any(|id| !g.children(id).is_empty()) {
        return;
    }

    for id in &nodes {
        if g.children(id).is_empty() {
            continue;
        }
        let pn = pnodes.iter().find(|n| n.id == *id);
        // Note groups are never extracted
        let is_note_grp = pn.map(|n| n.shape == Shape::NoteGroup).unwrap_or(false);
        if is_note_grp {
            continue;
        }

        let pn_dir = pn.map(|n| n.dir.as_str()).unwrap_or("");
        let all_children_are_dividers = !g.children(id).is_empty()
            && g.children(id).iter().all(|cid| {
                pnodes
                    .iter()
                    .find(|n| n.id == *cid)
                    .map(|n| n.shape == Shape::Divider)
                    .unwrap_or(false)
            });

        let parent_ranksep = g.graph().ranksep.unwrap_or(RANK_SEP);
        let sub_ranksep = if all_children_are_dividers {
            SUB_RANK_SEP
        } else {
            parent_ranksep + 25.0
        };

        let sub_dir = if !pn_dir.is_empty() {
            pn_dir.to_string()
        } else if all_children_are_dividers {
            parent_dir.to_string()
        } else {
            "TB".to_string()
        };

        let mut sg = Graph::with_options(true, true, true);
        // Concurrent regions use larger marginy to create top/bottom padding inside inner rect
        let (sg_marginx, sg_marginy) = if all_children_are_dividers {
            (CONCURRENT_MARGINX, CONCURRENT_MARGINY)
        } else {
            // Composite states (explicit dir) use larger margins to match reference
            (COMPOSITE_MARGINX, COMPOSITE_MARGINY)
        };
        sg.set_graph(GraphLabel {
            rankdir: Some(sub_dir.clone()),
            nodesep: Some(SUB_NODE_SEP),
            ranksep: Some(sub_ranksep),
            marginx: Some(sg_marginx),
            marginy: Some(sg_marginy),
            ..Default::default()
        });

        let child_ids = collect_all_descendants(id, g);
        for cid in &child_ids {
            if let Some(n) = g.node_opt(cid) {
                sg.set_node(cid, n.clone());
                if let Some(par) = g.parent(cid) {
                    if par != id && child_ids.contains(&par.to_string()) {
                        sg.set_parent(cid, Some(par));
                    }
                }
            }
        }
        for e in g.edges() {
            if child_ids.contains(&e.v) && child_ids.contains(&e.w) {
                if let Some(lbl) = g.edge(&e) {
                    sg.set_edge(&e.v, &e.w, lbl.clone(), e.name.as_deref());
                }
            }
        }

        // Recursively extract nested clusters (but NOT for concurrent regions —
        // dividers stay as compound nodes so they layout side-by-side).
        if !all_children_are_dividers {
            extractor(&mut sg, &sub_dir, cluster_db, pnodes, sub_graphs, depth + 1);
        }

        layout(&mut sg);

        if all_children_are_dividers {
            correct_divider_layout(&mut sg, pnodes);
        }

        // Composite correction: dagre's translate_graph includes edge-label proxies in its
        // min_y scan, so real nodes land ~8px lower than COMPOSITE_MARGINY. Shift them up
        // so the inner-rect top pad matches the reference (COMPOSITE_MARGINY + 3.0 = 19.5px).
        if !all_children_are_dividers && !pn.map(|p| p.shape == Shape::Divider).unwrap_or(false) {
            let target_top = COMPOSITE_MARGINY + 3.0;
            let nids: Vec<String> = sg.nodes();
            let min_top = nids
                .iter()
                .filter_map(|nid| {
                    sg.node_opt(nid)
                        .and_then(|n| n.y.map(|y| y - n.height / 2.0))
                })
                .fold(f64::MAX, f64::min);
            if min_top < f64::MAX {
                let dy = target_top - min_top;
                // Only shift UP (dy < 0). When there are no labeled edges, dagre places content
                // at margin_y (16.5) which is already above target_top — no correction needed.
                // A positive dy would push the bottom node outside the box.
                if dy < -0.1 {
                    for nid in &nids {
                        if let Some(n) = sg.node_opt_mut(nid) {
                            if let Some(ref mut y) = n.y {
                                *y += dy;
                            }
                        }
                    }
                    let edges: Vec<_> = sg.edges();
                    for e in &edges {
                        if let Some(lbl) = sg.edge_mut(e) {
                            if let Some(ref mut pts) = lbl.points {
                                for p in pts.iter_mut() {
                                    p.y += dy;
                                }
                            }
                            if let Some(ref mut ly) = lbl.y {
                                *ly += dy;
                            }
                        }
                    }
                }
            }
        }

        // Size the cluster in the main graph from its sub-graph dimensions.
        // Dividers have no title; composite states add CLUSTER_LABEL_H.
        let is_divider_node = pn.map(|n| n.shape == Shape::Divider).unwrap_or(false);
        // Labeled edges create proxy nodes at lh=18; reference DOM measures ~24px → 12px height deficit
        let has_labeled_edges = sg.edges().iter().any(|e| {
            sg.edge(e)
                .and_then(|l| l.height)
                .map(|h| h > 0.0)
                .unwrap_or(false)
        });
        let sgh = if is_divider_node {
            sg.graph().height.unwrap_or(40.0) + 2.0 * MARGIN
        } else if all_children_are_dividers {
            // correct_divider_layout sets sg.w exactly; sg.h is from dagre TB layout of leaf divider nodes
            sg.graph().height.unwrap_or(40.0)
                + 2.0 * MARGIN
                + CLUSTER_TITLE_AREA
                + CONCURRENT_HEIGHT_ADJUST
        } else {
            // Composite: outer rect = full dagre extent so edges touch
            let label_adjust = if has_labeled_edges { 12.0 } else { 0.0 };
            sg.graph().height.unwrap_or(40.0)
                + CLUSTER_TITLE_AREA
                + 4.0
                + label_adjust
                + COMPOSITE_BOTTOM_EXT
        };
        let sgw = if is_divider_node {
            sg.graph().width.unwrap_or(40.0) + 2.0 * MARGIN
        } else {
            // Concurrent: correct_divider_layout already set sg.w to the exact outer rect width
            // Composite: sg.w is already the correct full width
            sg.graph().width.unwrap_or(40.0)
        };
        if let Some(n) = g.node_opt_mut(id) {
            n.width = sgw;
            n.height = sgh;
        }

        // Re-create border-crossing edges as cluster↔outside
        let mut to_add: Vec<(String, String, EdgeLabel, String)> = Vec::new();
        for e in g.edges() {
            let v_in = child_ids.contains(&e.v);
            let w_in = child_ids.contains(&e.w);
            if v_in == w_in {
                continue;
            }
            if let Some(lbl) = g.edge(&e) {
                let (new_v, new_w) = if v_in {
                    (id.to_string(), e.w.clone())
                } else {
                    (e.v.clone(), id.to_string())
                };
                to_add.push((
                    new_v,
                    new_w,
                    lbl.clone(),
                    e.name.clone().unwrap_or_default(),
                ));
            }
        }
        for (v, w, lbl, name) in to_add {
            if g.has_node(&v) && g.has_node(&w) {
                g.set_edge(&v, &w, lbl, Some(&name));
            }
        }

        for cid in &child_ids {
            g.remove_node(cid);
        }
        sub_graphs.insert(
            id.clone(),
            SubGraph {
                graph: sg,
                dir: sub_dir,
            },
        );
    }
    extractor(g, parent_dir, cluster_db, pnodes, sub_graphs, depth + 1);
}

// ─── Concurrent divider corrections ──────────────────────────────────────────

fn correct_divider_layout(sg: &mut Graph, pnodes: &[PNode]) {
    let mut div_nodes: Vec<(String, f64, f64, f64)> = sg
        .nodes()
        .iter()
        .filter(|nid| {
            pnodes
                .iter()
                .find(|p| p.id.as_str() == nid.as_str())
                .map(|p| p.is_group)
                .unwrap_or(false)
        })
        .filter_map(|nid| {
            sg.node_opt(nid)
                .map(|n| (nid.clone(), n.x.unwrap_or(0.0), n.width, n.height))
        })
        .collect();
    div_nodes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    if div_nodes.len() >= 2 {
        let mut cursor_x = div_nodes[0].1;
        for i in 1..div_nodes.len() {
            let prev_w = div_nodes[i - 1].2;
            let cur_w = div_nodes[i].2;
            let target_cx = cursor_x + prev_w / 2.0 + CONCURRENT_DIV_GAP + cur_w / 2.0;
            let delta = target_cx - div_nodes[i].1;
            let nid = div_nodes[i].0.clone();
            shift_subtree_x(sg, &nid, delta);
            let descendants = collect_all_descendants(&nid, sg);
            let mut shifted = vec![nid.clone()];
            shifted.extend(descendants);
            for e in sg.edges() {
                if !shifted.contains(&e.v) && !shifted.contains(&e.w) {
                    continue;
                }
                if let Some(lbl) = sg.edge_mut(&e) {
                    if let Some(ref mut pts) = lbl.points {
                        for p in pts.iter_mut() {
                            p.x += delta;
                        }
                    }
                    if let Some(ref mut lx) = lbl.x {
                        *lx += delta;
                    }
                }
            }
            div_nodes[i].1 = target_cx;
            cursor_x = target_cx;
        }
        let last = &div_nodes[div_nodes.len() - 1];
        sg.graph_mut().width = Some(last.1 + last.2 / 2.0 + CONCURRENT_MARGINX);
        // Shift dividers so top pad = reference (11.5px from inner rect top).
        // dagre places div center at CONCURRENT_MARGINY + div_h/2; we want top edge = MARGIN+3.5.
        // y_shift = (target_top - CONCURRENT_MARGINY) = 11.5 - 19.5 = -8 = -MARGIN.
        let y_shift = -MARGIN;
        for (div_id, _, _, _) in &div_nodes {
            if let Some(n) = sg.node_opt_mut(div_id) {
                if let Some(ref mut cy) = n.y {
                    *cy += y_shift;
                }
            }
        }
    }

    // Y-centering: visually center each divider's leaf children within the rect.
    // rect spans [cy - h/2, cy + h/2]. rect_center = cy_compound (actual y after y_shift).
    let vis_half = |h: f64| -> f64 { h / 2.0 };
    for (div_id, _cx, _div_w, _div_h) in &div_nodes {
        // Read actual div center y after y_shift was applied
        let rect_center_y = sg.node_opt(div_id).and_then(|n| n.y).unwrap_or(0.0);
        // Leaf children: nodes in sg that are NOT groups in pnodes
        let leaves: Vec<String> = sg
            .nodes()
            .into_iter()
            .filter(|nid| {
                let pn = pnodes.iter().find(|p| p.id == *nid);
                // Must be a child of this divider (has divider as parent) and not a group
                pn.map(|p| p.parent_id.as_deref() == Some(div_id.as_str()) && !p.is_group)
                    .unwrap_or(false)
            })
            .collect();
        if leaves.is_empty() {
            continue;
        }
        let ct = leaves
            .iter()
            .filter_map(|c| {
                sg.node_opt(c)
                    .map(|n| n.y.unwrap_or(0.0) - vis_half(n.height))
            })
            .fold(f64::MAX, f64::min);
        let cb = leaves
            .iter()
            .filter_map(|c| {
                sg.node_opt(c)
                    .map(|n| n.y.unwrap_or(0.0) + vis_half(n.height))
            })
            .fold(f64::MIN, f64::max);
        if ct == f64::MAX || cb == f64::MIN {
            continue;
        }
        let delta_y = rect_center_y - (ct + cb) / 2.0;
        if delta_y.abs() < 0.01 {
            continue;
        }
        for cid in &leaves {
            if let Some(n) = sg.node_opt_mut(cid) {
                if let Some(ref mut y) = n.y {
                    *y += delta_y;
                }
            }
        }
        for e in sg.edges() {
            if !leaves.contains(&e.v) && !leaves.contains(&e.w) {
                continue;
            }
            if let Some(lbl) = sg.edge_mut(&e) {
                if let Some(ref mut pts) = lbl.points {
                    for p in pts.iter_mut() {
                        p.y += delta_y;
                    }
                }
                if let Some(ref mut ly) = lbl.y {
                    *ly += delta_y;
                }
            }
        }
    }
}

fn shift_subtree_x(g: &mut Graph, id: &str, delta: f64) {
    if let Some(n) = g.node_opt_mut(id) {
        if let Some(ref mut x) = n.x {
            *x += delta;
        }
    }
    for child in g.children(id) {
        shift_subtree_x(g, &child, delta);
    }
}

fn collect_all_descendants(id: &str, g: &Graph) -> Vec<String> {
    let mut res = Vec::new();
    for c in g.children(id) {
        res.push(c.clone());
        res.extend(collect_all_descendants(&c, g));
    }
    res
}

// ─── apply_title_offset ───────────────────────────────────────────────────────

fn apply_title_offset(g: &mut Graph, cluster_db: &HashMap<String, ClusterEntry>, pnodes: &[PNode]) {
    // For compound nodes that stay in the main graph (not extracted into sub_graphs),
    // shift the cluster upward and children downward so the title area is at the top.
    // NoteGroups have NO title area — skip them.
    for cid in cluster_db.keys() {
        let children = g.children(cid);
        if children.is_empty() {
            continue;
        } // extracted sub-graph or leaf
          // Skip note groups — they have no title, just an invisible container
        let is_note_grp = pnodes
            .iter()
            .find(|p| p.id == *cid)
            .map(|p| p.shape == Shape::NoteGroup)
            .unwrap_or(false);
        if is_note_grp {
            continue;
        }
        // Expand cluster and shift it up to make room for title
        if let Some(n) = g.node_opt_mut(cid) {
            n.height += CLUSTER_LABEL_H;
            if let Some(ref mut cy) = n.y {
                *cy -= CLUSTER_LABEL_H / 2.0;
            }
        }
        // Shift children DOWN by CLUSTER_LABEL_H so they appear below the title,
        // and add 25px per rank gap to match Mermaid's dagre-d3-es spacing.
        // Sort children top→bottom, then add i*25 extra (i=0 for topmost child).
        let mut children_with_y: Vec<(String, f64)> = children
            .iter()
            .filter_map(|c| g.node_opt(c).and_then(|n| n.y).map(|y| (c.clone(), y)))
            .collect();
        children_with_y.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let extra_total = if children_with_y.len() > 1 {
            (children_with_y.len() as f64 - 1.0) * 25.0
        } else {
            0.0
        };
        for (i, (child, _)) in children_with_y.iter().enumerate() {
            let extra = i as f64 * 25.0;
            if let Some(n) = g.node_opt_mut(child) {
                if let Some(ref mut y) = n.y {
                    *y += CLUSTER_LABEL_H + extra;
                }
            }
        }
        // Expand cluster bottom by extra_total (children pushed down, top unchanged)
        if let Some(n) = g.node_opt_mut(cid) {
            n.height += extra_total;
            if let Some(ref mut cy) = n.y {
                *cy += extra_total / 2.0;
            }
        }
    }
}

// ─── recursiveRender ─────────────────────────────────────────────────────────

fn recursive_render(
    g: &Graph,
    pedges: &[PEdge],
    pnodes: &[PNode],
    sub_graphs: &HashMap<String, SubGraph>,
    _cluster_db: &HashMap<String, ClusterEntry>,
    vars: &ThemeVars,
    out: &mut String,
) {
    out.push_str("<g class=\"root\">");
    out.push_str("<g class=\"clusters\">");
    for id in &g.nodes() {
        let has_children = !g.children(id).is_empty();
        let in_sub = sub_graphs.contains_key(id);
        if let Some(n) = g.node_opt(id) {
            let cx = n.x.unwrap_or(0.0);
            let cy = n.y.unwrap_or(0.0);
            let w = n.width;
            let h = n.height;
            let dom_id = format!("mermaid-svg-{}", esc(id));
            let pn = pnodes.iter().find(|p| p.id == *id);
            let label = pn.map(|p| p.label.as_str()).unwrap_or(id.as_str());
            let is_note_group = pn.map(|p| p.shape == Shape::NoteGroup).unwrap_or(false);
            let is_divider = pn.map(|p| p.shape == Shape::Divider).unwrap_or(false);

            if is_note_group {
                out.push_str(&templates::note_group_placeholder(&dom_id));
            } else if in_sub {
                // Extracted sub-graph. Content starts at inner rect (after title area).
                // inner_rect.y = cy - h/2 + CLUSTER_PAD + CLUSTER_LABEL_H
                // ty = inner_rect.y - MARGIN = cy - h/2 + CLUSTER_TITLE_AREA
                out.push_str(&render_cluster(
                    &dom_id, cx, cy, w, h, label, is_divider, vars,
                ));
                // tx at outer rect left; sub-graph's own marginx provides internal padding
                let tx = cx - w / 2.0;
                let ty = cy - h / 2.0 + CLUSTER_TITLE_AREA;
                out.push_str(&templates::translate_group_open(tx, ty));
                render_sub_graph(&sub_graphs[id].graph, pnodes, pedges, vars, out);
                out.push_str("</g>");
            } else if has_children {
                // Compound node still in main graph (external connections, not extracted)
                out.push_str(&render_cluster(
                    &dom_id, cx, cy, w, h, label, is_divider, vars,
                ));
            }
        }
    }
    out.push_str("</g>");

    out.push_str("<g class=\"edgePaths\">");
    for (i, pe) in pedges.iter().enumerate() {
        let e = Edge::named(&pe.start, &pe.end, &format!("e{}", i));
        if let Some(lbl) = g.edge(&e) {
            let mut lbl_clone = lbl.clone();
            // Fix diamond gap: project edge endpoints onto actual diamond boundary for choice nodes
            let start_choice = pnodes
                .iter()
                .find(|p| p.id == pe.start)
                .map(|p| p.shape == Shape::Choice)
                .unwrap_or(false);
            let end_choice = pnodes
                .iter()
                .find(|p| p.id == pe.end)
                .map(|p| p.shape == Shape::Choice)
                .unwrap_or(false);
            if start_choice || end_choice {
                if let Some(ref mut pts) = lbl_clone.points {
                    if start_choice {
                        if let Some(n) = g.node_opt(&pe.start) {
                            let cx = n.x.unwrap_or(0.0);
                            let cy = n.y.unwrap_or(0.0);
                            if pts.len() >= 2 {
                                let (p1x, p1y) = (pts[1].x, pts[1].y);
                                clip_to_diamond(&mut pts[0], cx, cy, CHOICE_SIZE, p1x, p1y);
                            }
                        }
                    }
                    if end_choice {
                        if let Some(n) = g.node_opt(&pe.end) {
                            let cx = n.x.unwrap_or(0.0);
                            let cy = n.y.unwrap_or(0.0);
                            let last = pts.len() - 1;
                            if last >= 1 {
                                let prev_x = pts[last - 1].x;
                                let prev_y = pts[last - 1].y;
                                clip_to_diamond(
                                    &mut pts[last],
                                    cx,
                                    cy,
                                    CHOICE_SIZE,
                                    prev_x,
                                    prev_y,
                                );
                            }
                        }
                    }
                }
            }
            render_edge_pts(&lbl_clone, &pe.classes, vars, out);
        }
    }
    out.push_str("</g>");

    out.push_str("<g class=\"edgeLabels\">");
    for (i, pe) in pedges.iter().enumerate() {
        if pe.label.is_empty() {
            continue;
        }
        let e = Edge::named(&pe.start, &pe.end, &format!("e{}", i));
        if let Some(lbl) = g.edge(&e) {
            if let (Some(x), Some(y)) = (lbl.x, lbl.y) {
                out.push_str(&edge_label(x, y, &pe.label, vars));
            }
        }
    }
    out.push_str("</g>");

    out.push_str("<g class=\"nodes\">");
    for id in &g.nodes() {
        let pn_is_group = pnodes
            .iter()
            .find(|p| p.id == *id)
            .map(|p| p.is_group)
            .unwrap_or(false);
        if pn_is_group {
            continue;
        }
        if sub_graphs.contains_key(id) {
            continue;
        }
        // Skip children of extracted sub-graphs (they're rendered inside the sub-graph)
        let parent_in_sub = g
            .parent(id)
            .map(|par| sub_graphs.contains_key(par))
            .unwrap_or(false);
        if parent_in_sub {
            continue;
        }
        if let Some(n) = g.node_opt(id) {
            let cx = n.x.unwrap_or(0.0);
            let cy = n.y.unwrap_or(0.0);
            let dom_id = format!("mermaid-svg-{}", esc(id));
            if let Some(pn) = pnodes.iter().find(|p| p.id == *id) {
                out.push_str(&render_node(pn, &dom_id, cx, cy, n.width, n.height, vars));
            }
        }
    }
    out.push_str("</g>");
    out.push_str("</g>");
}

fn render_sub_graph(
    sg: &Graph,
    pnodes: &[PNode],
    pedges: &[PEdge],
    vars: &ThemeVars,
    out: &mut String,
) {
    // Use pnodes.is_group to detect clusters — dagre may not preserve compound
    // structure after layout, so sg.children() is unreliable here.
    let is_cluster = |id: &str| {
        pnodes
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.is_group)
            .unwrap_or(false)
    };

    out.push_str("<g class=\"clusters\">");
    for id in &sg.nodes() {
        if !is_cluster(id) {
            continue;
        }
        if let Some(n) = sg.node_opt(id) {
            let cx = n.x.unwrap_or(0.0);
            let cy = n.y.unwrap_or(0.0);
            let dom_id = format!("mermaid-svg-{}", esc(id));
            let pn = pnodes.iter().find(|p| p.id == *id);
            let label = pn.map(|p| p.label.as_str()).unwrap_or("");
            let is_divider = pn.map(|p| p.shape == Shape::Divider).unwrap_or(false);
            out.push_str(&render_cluster(
                &dom_id, cx, cy, n.width, n.height, label, is_divider, vars,
            ));
        }
    }
    out.push_str("</g>");

    out.push_str("<g class=\"edgePaths\">");
    for e in sg.edges() {
        if let Some(lbl) = sg.edge(&e) {
            let pe = pedges.iter().find(|p| p.start == e.v && p.end == e.w);
            render_edge_pts(
                lbl,
                pe.map(|p| p.classes.as_str()).unwrap_or("transition"),
                vars,
                out,
            );
        }
    }
    out.push_str("</g>");

    out.push_str("<g class=\"edgeLabels\">");
    for e in sg.edges() {
        if let Some(lbl) = sg.edge(&e) {
            if let (Some(x), Some(y)) = (lbl.x, lbl.y) {
                if let Some(pe) = pedges.iter().find(|p| p.start == e.v && p.end == e.w) {
                    if !pe.label.is_empty() {
                        out.push_str(&edge_label(x, y, &pe.label, vars));
                    }
                }
            }
        }
    }
    out.push_str("</g>");

    out.push_str("<g class=\"nodes\">");
    for id in &sg.nodes() {
        if is_cluster(id) {
            continue;
        }
        if let Some(n) = sg.node_opt(id) {
            let cx = n.x.unwrap_or(0.0);
            let cy = n.y.unwrap_or(0.0);
            let dom_id = format!("mermaid-svg-{}", esc(id));
            if let Some(pn) = pnodes.iter().find(|p| p.id == *id) {
                out.push_str(&render_node(pn, &dom_id, cx, cy, n.width, n.height, vars));
            }
        }
    }
    out.push_str("</g>");
}

// ─── Cluster rendering (clusters.js) ─────────────────────────────────────────
//
// divider: rect at full node.x-w/2, node.y-h/2 (no inset, no title)
// roundedWithTitle: outer rect + title text + inner content rect

#[allow(clippy::too_many_arguments)]
fn render_cluster(
    dom_id: &str,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    label: &str,
    is_divider: bool,
    vars: &ThemeVars,
) -> String {
    let x = cx - w / 2.0;
    let y = cy - h / 2.0;
    if is_divider {
        templates::cluster_divider(dom_id, x, y, w, h, vars.state_end_bg)
    } else {
        // Outer rect at full node boundary so edge endpoints touch it (no CLUSTER_PAD inset).
        let inner_y = y + CLUSTER_TITLE_AREA;
        let inner_h = h - CLUSTER_TITLE_AREA - 4.0;
        let tcy = y + CLUSTER_TITLE_AREA / 2.0;
        templates::cluster_compound(
            dom_id,
            x,
            y,
            w,
            h,
            cx,
            tcy,
            inner_y,
            inner_h,
            vars.primary_color,
            vars.state_end_bg,
            FONT_SIZE as u32,
            vars.primary_text,
            &esc(label),
            vars.state_composit_bg,
        )
    }
}

// ─── Node rendering ───────────────────────────────────────────────────────────

fn render_node(
    pn: &PNode,
    dom_id: &str,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    vars: &ThemeVars,
) -> String {
    match pn.shape {
        Shape::Start => templates::node_start(dom_id, cx, cy, vars),
        Shape::End => templates::node_end(dom_id, cx, cy, vars),
        Shape::Fork | Shape::Join => templates::node_fork_join(dom_id, cx, cy, vars),
        Shape::Choice => templates::node_choice(dom_id, cx, cy, vars),
        Shape::Note => templates::node_note(dom_id, cx, cy, w, h, &pn.label, vars),
        _ => templates::node_rect(dom_id, cx, cy, w, h, &pn.label, vars),
    }
}

// ─── Edge rendering ───────────────────────────────────────────────────────────

/// Project point p onto the diamond boundary, approaching from direction of (from_x, from_y).
/// Diamond: center (cx, cy), half-size s. Uses ray-diamond intersection.
fn clip_to_diamond(
    p: &mut dagre_dgl_rs::graph::Point,
    cx: f64,
    cy: f64,
    s: f64,
    from_x: f64,
    from_y: f64,
) {
    let dx = from_x - cx;
    let dy = from_y - cy;
    if dx.abs() < 0.001 && dy.abs() < 0.001 {
        return;
    }
    // Ray from center toward (from_x, from_y): intersect with diamond |x/s| + |y/s| = 1
    let t = s / (dx.abs() + dy.abs());
    let mut x = cx + dx * t;
    let mut y = cy + dy * t;
    // Replicate Mermaid's intersect-line.js integer-rounding quirk applied to floats:
    // for each diamond clip, the result is shifted by 0.5 away from zero in world coords.
    // The quirk: offset = |denom/2| is added/subtracted to the numerator before division,
    // intended for integer-rounding but mis-applied to floats. Net effect for choice
    // diamonds: result shifts by sign(coord) * 0.5.
    if x >= 0.0 {
        x += 0.5;
    } else {
        x -= 0.5;
    }
    if y >= 0.0 {
        y += 0.5;
    } else {
        y -= 0.5;
    }
    p.x = x;
    p.y = y;
}

fn render_edge_pts(lbl: &EdgeLabel, classes: &str, vars: &ThemeVars, out: &mut String) {
    if let Some(ref pts) = lbl.points {
        if pts.len() < 2 {
            return;
        }
        let d = crate::svg::curve_basis_path(&pts.iter().map(|p| (p.x, p.y)).collect::<Vec<_>>());
        let is_note = classes.contains("note-edge");
        let dash = if is_note { "stroke-dasharray:5;" } else { "" };
        let me = if is_note {
            ""
        } else {
            " marker-end=\"url(#state-barbEnd)\""
        };
        out.push_str(&templates::transition_path(
            &d,
            vars.state_transition_color,
            dash,
            me,
        ));
    }
}

fn edge_label(x: f64, y: f64, label: &str, vars: &ThemeVars) -> String {
    // Mermaid: .edgeLabel .label rect{fill:#ECECFF;opacity:0.5;} — light tint behind label.
    // Background rect width matches Mermaid's labelGroup bbox (text width + 2*padding).
    let text_w = crate::text_browser_metrics::measure_browser(label, FONT_SIZE).0;
    let rect_w = text_w + 4.0;
    let rect_h = 21.0;
    // Mermaid translates the inner label group by -10.5 px (= -rect_h/2) for 16px edge labels,
    // placing the rect's top at the group's top. label_tspan's -font_size/1.882 = -8.5 is wrong here.
    let lbl_g = crate::diagrams::util::label_tspan_raw(
        0.0,
        -10.5,
        &esc(label),
        FONT_SIZE,
        vars.primary_text,
        "middle",
        "",
        vars.font_family,
    );
    templates::edge_label_group(x, y, rect_w, rect_h, vars.primary_color, &lbl_g)
}

// ─── Sizing ───────────────────────────────────────────────────────────────────

fn node_size(pn: &PNode) -> (f64, f64) {
    match pn.shape {
        // stateStart/stateEnd: default width=height=14 (Mermaid stateStart.ts)
        Shape::Start | Shape::End => {
            let d = START_RADIUS * 2.0;
            (d, d)
        }
        Shape::Fork | Shape::Join => (FORK_JOIN_WIDTH, FORK_JOIN_HEIGHT),
        // choice diamond: dagre size matches start/end (14px default in Mermaid)
        Shape::Choice => (CHOICE_SIZE * 2.0, CHOICE_SIZE * 2.0),
        Shape::Note => {
            let (tw, _) = measure(&pn.label, FONT_SIZE);
            ((tw + NOTE_PADDING * 2.0).max(NOTE_MIN_WIDTH), NOTE_HEIGHT)
        }
        Shape::Group | Shape::Divider | Shape::NoteGroup => (1.0, 1.0),
        _ => {
            let (tw, _) = measure(&pn.label, FONT_SIZE);
            // 33 = FONT_SIZE*1.1 + 15.4 padding (Mermaid lineHeight=1.1, matches ref state rect h=33).
            ((tw + NODE_PADDING * 2.0).max(1.0), 33.0)
        }
    }
}

fn label_w(label: &str) -> f64 {
    measure(label, FONT_SIZE).0
}

// ─── ViewBox ──────────────────────────────────────────────────────────────────

fn viewbox(
    g: &Graph,
    pedges: &[PEdge],
    _pnodes: &[PNode],
    sub_graphs: &HashMap<String, SubGraph>,
) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for id in &g.nodes() {
        if let Some(n) = g.node_opt(id) {
            let cx = n.x.unwrap_or(0.0);
            let cy = n.y.unwrap_or(0.0);
            min_x = min_x.min(cx - n.width / 2.0);
            min_y = min_y.min(cy - n.height / 2.0);
            max_x = max_x.max(cx + n.width / 2.0);
            max_y = max_y.max(cy + n.height / 2.0);
        }
    }
    for (i, pe) in pedges.iter().enumerate() {
        let e = Edge::named(&pe.start, &pe.end, &format!("e{}", i));
        if let Some(lbl) = g.edge(&e) {
            if let Some(ref pts) = lbl.points {
                for p in pts {
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }
            }
            if !pe.label.is_empty() {
                if let Some(x) = lbl.x {
                    let hw = label_w(&pe.label) / 2.0;
                    min_x = min_x.min(x - hw);
                    max_x = max_x.max(x + hw);
                }
            }
        }
    }
    for (cid, sg) in sub_graphs {
        if let Some(cn) = g.node_opt(cid) {
            let tx = cn.x.unwrap_or(0.0) - cn.width / 2.0;
            let ty = cn.y.unwrap_or(0.0) - cn.height / 2.0 + CLUSTER_LABEL_H;
            for nid in &sg.graph.nodes() {
                if let Some(n) = sg.graph.node_opt(nid) {
                    let scx = tx + n.x.unwrap_or(0.0);
                    let scy = ty + n.y.unwrap_or(0.0);
                    min_x = min_x.min(scx - n.width / 2.0);
                    min_y = min_y.min(scy - n.height / 2.0);
                    max_x = max_x.max(scx + n.width / 2.0);
                    max_y = max_y.max(scy + n.height / 2.0);
                }
            }
        }
    }
    if min_x == f64::MAX {
        (0.0, 0.0, 200.0, 200.0)
    } else {
        (
            min_x - MARGIN,
            min_y - MARGIN,
            (max_x - min_x) + 2.0 * MARGIN,
            (max_y - min_y) + 2.0 * MARGIN,
        )
    }
}

// ─── CSS ──────────────────────────────────────────────────────────────────────

fn css(vars: &ThemeVars) -> String {
    templates::css(
        vars.primary_color,
        vars.state_end_bg,
        vars.state_start_fill,
        vars.state_transition_color,
        vars.note_bg,
        vars.note_border,
        FONT_SIZE as u32,
        vars.font_family,
    )
}
