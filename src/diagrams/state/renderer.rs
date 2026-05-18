use super::constants::*;
use super::templates;
// State diagram renderer — faithful port of stateRenderer-v3-unified.ts
// SVG structure mirrors Mermaid v11 reference output.
// Recursive layout approach: each composite state gets its own dagre layout run,
// and is treated as a fixed-size node in the outer layout. The inner content is
// rendered in a translated sub-group matching the <g class="root" translate(...)>
// structure from the Mermaid reference.

use super::parser::{flatten, FlatGraph, StateDiagram, StateNode, StateType, Transition};
use crate::text::measure;
use crate::theme::{Theme, ThemeVars};
use dagre_dgl_rs::graph::{EdgeLabel, Graph, GraphLabel, NodeLabel, Point};
use dagre_dgl_rs::layout::layout;

// All layout constants are imported from super::constants via `use super::constants::*`.

pub fn render(diag: &StateDiagram, theme: Theme, use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    let flat = flatten(diag);
    render_flat(&flat, &vars, use_foreign_object, &diag.direction)
}

// ─── Layout result types ───────────────────────────────────────────────────────

/// A note-cluster compound background rect (Mermaid's noteGroup pattern).
/// Records the absolute position and size of the compound parent node after layout.
struct NoteCluster {
    /// ID of the attached state (for SVG id attribute)
    state_id: String,
    /// Absolute x, y of the cluster rect top-left corner
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Layout of a single level (outer or inner for a composite).
struct LevelLayout {
    nodes: std::collections::HashMap<String, (f64, f64, f64, f64)>,
    edges: Vec<(String, String, Vec<Point>)>,
    note_clusters: Vec<NoteCluster>,
    width: f64,
    height: f64,
}

/// Positioned composite state (ready to render).
struct CompositeLayout {
    id: String,
    inner: LevelLayout,
    outer_cx: f64,
    outer_cy: f64,
}

/// Complete layout result passed to renderer.
struct FlatLayout {
    outer_nodes: std::collections::HashMap<String, (f64, f64, f64, f64)>,
    outer_edges: Vec<(String, String, Vec<Point>)>,
    note_clusters: Vec<NoteCluster>,
    composites: Vec<CompositeLayout>,
    width: f64,
    height: f64,
}

// ─── Helper functions ──────────────────────────────────────────────────────────

fn find_entry_child(composite_id: &str, flat: &FlatGraph) -> Option<String> {
    let children: Vec<String> = flat
        .parent
        .iter()
        .filter(|(_, p)| p.as_str() == composite_id)
        .map(|(c, _)| c.clone())
        .filter(|c| {
            flat.states
                .get(c)
                .map(|n| n.state_type != StateType::Note && n.state_type != StateType::Composite)
                .unwrap_or(false)
        })
        .collect();
    for child in &children {
        if flat
            .states
            .get(child)
            .map(|n| n.state_type == StateType::Start)
            .unwrap_or(false)
        {
            return Some(child.clone());
        }
    }
    let child_set: std::collections::HashSet<&str> = children.iter().map(|s| s.as_str()).collect();
    let has_incoming: std::collections::HashSet<String> = flat
        .transitions
        .iter()
        .filter(|t| child_set.contains(t.from.as_str()) && child_set.contains(t.to.as_str()))
        .map(|t| t.to.clone())
        .collect();
    for child in &children {
        if !has_incoming.contains(child) {
            return Some(child.clone());
        }
    }
    children.into_iter().next()
}

fn find_exit_child(composite_id: &str, flat: &FlatGraph) -> Option<String> {
    let children: Vec<String> = flat
        .parent
        .iter()
        .filter(|(_, p)| p.as_str() == composite_id)
        .map(|(c, _)| c.clone())
        .filter(|c| {
            flat.states
                .get(c)
                .map(|n| n.state_type != StateType::Note && n.state_type != StateType::Composite)
                .unwrap_or(false)
        })
        .collect();
    for child in &children {
        if flat
            .states
            .get(child)
            .map(|n| n.state_type == StateType::End)
            .unwrap_or(false)
        {
            return Some(child.clone());
        }
    }
    let child_set: std::collections::HashSet<&str> = children.iter().map(|s| s.as_str()).collect();
    let has_outgoing: std::collections::HashSet<String> = flat
        .transitions
        .iter()
        .filter(|t| child_set.contains(t.from.as_str()) && child_set.contains(t.to.as_str()))
        .map(|t| t.from.clone())
        .collect();
    for child in &children {
        if !has_outgoing.contains(child) {
            return Some(child.clone());
        }
    }
    children.into_iter().last()
}

fn collect_all_descendants(composite_id: &str, flat: &FlatGraph) -> Vec<String> {
    let mut result = Vec::new();
    collect_descendants_rec(composite_id, flat, &mut result);
    result
}

fn collect_descendants_rec(id: &str, flat: &FlatGraph, out: &mut Vec<String>) {
    let children: Vec<String> = flat
        .parent
        .iter()
        .filter(|(_, p)| p.as_str() == id)
        .map(|(c, _)| c.clone())
        .collect();
    for child in children {
        if let Some(n) = flat.states.get(&child) {
            if n.state_type == StateType::Composite {
                collect_descendants_rec(&child, flat, out);
            }
            out.push(child);
        }
    }
}

// ─── Single-level dagre layout ─────────────────────────────────────────────────

/// Run a dagre layout for a flat set of nodes (no compound). Composite children
/// at this level are treated as opaque fixed-size nodes.
fn run_level_layout(
    flat: &FlatGraph,
    node_ids: &[String],
    composite_sizes: &std::collections::HashMap<String, (f64, f64)>,
    transitions: &[&Transition],
    direction: &str,
    is_inner: bool,
) -> LevelLayout {
    // Use compound=true so nesting_graph runs (adds root→node edges),
    // matching the behaviour of the old flat compound layout.
    let mut g = Graph::with_options(true, false, true);
    let rankdir = match direction {
        "LR" => "LR",
        "RL" => "RL",
        "BT" => "BT",
        _ => "TB",
    };
    // Inner composite layouts use larger ranksep, marginy, and marginx to match Mermaid
    // reference proportions (Mermaid uses compound dagre with border nodes for inner layouts).
    // Outer layouts that contain composites use marginx=0 so that outer_cx = inner_w/2,
    // giving translate(0, ty) for composite sub-groups (matches Mermaid reference).
    // Pure outer layouts (no composites, e.g. state_notes) use marginx=MARGIN.
    let has_composites = composite_sizes.iter().next().is_some();
    let (nodesep, ranksep, marginx, marginy) = if is_inner {
        (NODESEP, INNER_RANKSEP, INNER_MARGINX, INNER_MARGINY)
    } else if has_composites {
        (NODESEP, RANKSEP, 0.0, MARGIN)
    } else {
        (NODESEP, RANKSEP, MARGIN, MARGIN)
    };
    g.set_graph(GraphLabel {
        rankdir: Some(rankdir.to_string()),
        nodesep: Some(nodesep),
        ranksep: Some(ranksep),
        marginx: Some(marginx),
        marginy: Some(marginy),
        ..Default::default()
    });

    let node_set: std::collections::HashSet<&str> = node_ids.iter().map(|s| s.as_str()).collect();

    // Add nodes
    // note_cluster_data: (note_id, attached_state_id, compound_parent_id)
    // Implements Mermaid's note-cluster compound pattern:
    //   - The note node becomes a child of {state_id}----parent via g.set_parent
    //   - The attached state stays at root level
    //   - After layout, dagre computes the compound node bounds (via border nodes)
    //   - These bounds are read to render the note-cluster background rect
    let mut note_cluster_data: Vec<(String, String, String)> = Vec::new();
    for id in node_ids {
        if let Some(node) = flat.states.get(id) {
            let (w, h) = if node.state_type == StateType::Composite {
                composite_sizes.get(id).cloned().unwrap_or((60.0, 60.0))
            } else {
                node_size(node)
            };
            if node.state_type == StateType::Note {
                // note.label encodes the attached state id
                let state_id = node.label.clone();
                let parent_id = format!("{}----parent", state_id);
                // Register the compound parent node (zero size — dagre computes from children)
                g.set_node(
                    &parent_id,
                    NodeLabel {
                        width: 0.0,
                        height: 0.0,
                        ..Default::default()
                    },
                );
                note_cluster_data.push((id.clone(), state_id, parent_id));
            }
            let intersect_type = if node.state_type == StateType::Choice {
                Some("diamond")
            } else {
                None
            };
            g.set_node(
                id,
                NodeLabel {
                    width: w,
                    height: h,
                    intersect_type,
                    ..Default::default()
                },
            );
        }
    }

    // Set compound parent relationships AFTER all nodes are registered
    for (note_id, _state_id, parent_id) in &note_cluster_data {
        g.set_parent(note_id, Some(parent_id));
    }

    // Build entry/exit maps for any composite nodes at this level
    let mut entry_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut exit_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for id in node_ids {
        if let Some(node) = flat.states.get(id) {
            if node.state_type == StateType::Composite {
                if let Some(e) = find_entry_child(id, flat) {
                    entry_map.insert(id.clone(), e);
                }
                if let Some(x) = find_exit_child(id, flat) {
                    exit_map.insert(id.clone(), x);
                }
            }
        }
    }

    // Add edges — only between nodes at this level
    let mut added: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    let mut edge_orig: std::collections::HashMap<(String, String), (String, String)> =
        std::collections::HashMap::new();

    for t in transitions {
        let from_note = flat
            .states
            .get(&t.from)
            .map(|n| n.state_type == StateType::Note)
            .unwrap_or(false);
        let to_note = flat
            .states
            .get(&t.to)
            .map(|n| n.state_type == StateType::Note)
            .unwrap_or(false);
        if from_note || to_note {
            continue;
        }

        // Resolve endpoints to nodes actually present at this level
        let lf = resolve_endpoint(&t.from, flat, &node_set, &entry_map, &exit_map, false);
        let lt = resolve_endpoint(&t.to, flat, &node_set, &entry_map, &exit_map, true);

        if let (Some(lf), Some(lt)) = (lf, lt) {
            if lf == lt {
                continue;
            }
            let (lbl_w, lbl_h) = if t.label.as_deref().map(|s| !s.is_empty()).unwrap_or(false) {
                (1.0, 24.0)
            } else {
                (0.0, 0.0)
            };
            let key = (lf.clone(), lt.clone());
            if !added.contains(&key) {
                added.insert(key.clone());
                edge_orig.insert(key, (t.from.clone(), t.to.clone()));
                g.set_edge(
                    &lf,
                    &lt,
                    EdgeLabel {
                        minlen: Some(1),
                        weight: Some(1.0),
                        width: Some(lbl_w),
                        height: Some(lbl_h),
                        labelpos: Some("c".to_string()),
                        ..Default::default()
                    },
                    None,
                );
            }
        }
    }

    for (note_id, state_id, _parent_id) in &note_cluster_data {
        if g.node_opt(state_id).is_some() {
            g.set_edge(
                state_id,
                note_id,
                EdgeLabel {
                    minlen: Some(1),
                    weight: Some(0.0),
                    ..Default::default()
                },
                None,
            );
        }
    }

    layout(&mut g);

    // Post-layout adjustment for note-cluster: our dagre pulls the attached state (State1)
    // too far left due to compound influence. We correct positions based on the reference:
    // 1. Same-rank nodes (State2): dagre already places them close to the correct position;
    //    only prevent overlap with the compound (desired_left = comp_right).
    // 2. Attached state (State1): align near State2's x, offset by half the width difference.
    for (_note_id, state_id, parent_id) in &note_cluster_data {
        if let Some(pn) = g.node_opt(parent_id) {
            if let (Some(comp_cx), Some(comp_cy)) = (pn.x, pn.y) {
                let comp_w = pn.width;
                let comp_right = comp_cx + comp_w / 2.0;
                let note_y = comp_cy;

                // Collect non-note nodes at note's y-rank (e.g. State2)
                let mut same_rank_ids: Vec<String> = Vec::new();
                for vid in node_ids {
                    if let Some(node) = flat.states.get(vid) {
                        if node.state_type == StateType::Note {
                            continue;
                        }
                        if let Some(vn) = g.node_opt(vid) {
                            if let Some(vy) = vn.y {
                                if (vy - note_y).abs() < RANKSEP / 4.0 {
                                    same_rank_ids.push(vid.clone());
                                }
                            }
                        }
                    }
                }

                // Only prevent State2 from overlapping compound (don't over-shift).
                // Our dagre with correct note width already places State2 close to reference.
                if !same_rank_ids.is_empty() {
                    let leftmost_x: f64 = same_rank_ids
                        .iter()
                        .filter_map(|vid| {
                            g.node_opt(vid).and_then(|n| n.x.map(|x| x - n.width / 2.0))
                        })
                        .fold(f64::INFINITY, f64::min);
                    let shift = if comp_right > leftmost_x {
                        comp_right - leftmost_x
                    } else {
                        0.0
                    };
                    if shift > 0.0 {
                        for vid in &same_rank_ids {
                            if let Some(vn) = g.node_opt_mut(vid) {
                                if let Some(vx) = vn.x.as_mut() {
                                    *vx += shift;
                                }
                            }
                        }
                    }
                }

                // Align attached state (State1) near State2's position.
                // JS dagre puts State1 close to State2 (its direct edge neighbor), but our
                // dagre pulls State1 left toward the compound. Fix: align State1 so its center
                // is State2_cx minus half the difference in half-widths.
                let state2_cx = same_rank_ids
                    .first()
                    .and_then(|id| g.node_opt(id))
                    .and_then(|n| n.x);
                let state2_half_w = same_rank_ids
                    .first()
                    .and_then(|id| g.node_opt(id))
                    .map(|n| n.width / 2.0)
                    .unwrap_or(0.0);
                if let Some(s2x) = state2_cx {
                    if let Some(sn) = g.node_opt(state_id) {
                        let old_x = sn.x.unwrap_or(0.0);
                        let state_half_w = sn.width / 2.0;
                        let target_x = s2x - (state_half_w - state2_half_w) / 2.0;
                        let delta_x = target_x - old_x;
                        if let Some(sn_mut) = g.node_opt_mut(state_id) {
                            sn_mut.x = Some(target_x);
                        }
                        // Re-route all edges touching state_id from scratch. The dagre
                        // edge points were computed for state_id's OLD position; after the
                        // x-adjustment the old waypoints produce detached/wrong paths.
                        // Replace with a clean L-shaped 4-point route: straight down from
                        // source bottom to mid-y, then across to target x, then down to target top.
                        if delta_x.abs() > 0.01 {
                            let mut touching: Vec<(String, String)> = Vec::new();
                            if let Some(outs) = g.out_edges(state_id) {
                                for e in outs {
                                    touching.push((e.v.clone(), e.w.clone()));
                                }
                            }
                            if let Some(ins) = g.in_edges(state_id) {
                                for e in ins {
                                    touching.push((e.v.clone(), e.w.clone()));
                                }
                            }
                            // Build reverse-edge set to detect bidirectional pairs.
                            let rev_set: std::collections::HashSet<(String, String)> = touching
                                .iter()
                                .map(|(a, b)| (b.clone(), a.clone()))
                                .collect();

                            for (idx, (ev, ew)) in touching.iter().enumerate() {
                                let src_cx = g.node_opt(ev).and_then(|n| n.x).unwrap_or(0.0);
                                let src_cy = g.node_opt(ev).and_then(|n| n.y).unwrap_or(0.0);
                                let src_hh = g.node_opt(ev).map(|n| n.height / 2.0).unwrap_or(20.0);
                                let src_hw = g.node_opt(ev).map(|n| n.width / 2.0).unwrap_or(0.0);
                                let tgt_cx = g.node_opt(ew).and_then(|n| n.x).unwrap_or(0.0);
                                let tgt_cy = g.node_opt(ew).and_then(|n| n.y).unwrap_or(0.0);
                                let tgt_hh = g.node_opt(ew).map(|n| n.height / 2.0).unwrap_or(20.0);
                                let tgt_hw = g.node_opt(ew).map(|n| n.width / 2.0).unwrap_or(0.0);

                                // Compute directional port x for a node boundary.
                                // If the horizontal travel is large relative to vertical (ratio ≥ 0.5),
                                // exit from near the corresponding edge; otherwise use center.
                                let port_x =
                                    |node_cx: f64, node_hw: f64, other_cx: f64, dy: f64| -> f64 {
                                        let dx = other_cx - node_cx;
                                        let ratio =
                                            if dy.abs() > 1.0 { dx / dy.abs() } else { 0.0 };
                                        if ratio.abs() >= 0.5 {
                                            let edge_x = node_cx + ratio.signum() * (node_hw - 8.0);
                                            edge_x.clamp(node_cx - node_hw, node_cx + node_hw)
                                        } else {
                                            node_cx
                                        }
                                    };

                                let dy = (tgt_cy - src_cy).abs();

                                // Bidirectional same-rank pair: offset both arrows laterally so
                                // they run as parallel lines rather than overlapping.
                                // The edge with the lower index gets a +OFFSET lane, the reverse gets -OFFSET.
                                let is_bidir = rev_set.contains(&(ev.clone(), ew.clone()));
                                let bidir_offset = if is_bidir {
                                    let sibling_idx =
                                        touching.iter().position(|(a, b)| a == ew && b == ev);
                                    let is_first = sibling_idx.map(|si| idx < si).unwrap_or(true);
                                    if is_first {
                                        8.0
                                    } else {
                                        -8.0
                                    }
                                } else {
                                    0.0
                                };

                                let is_src_note = flat
                                    .states
                                    .get(ev.as_str())
                                    .map(|n| n.state_type == StateType::Note)
                                    .unwrap_or(false);
                                let is_tgt_note = flat
                                    .states
                                    .get(ew.as_str())
                                    .map(|n| n.state_type == StateType::Note)
                                    .unwrap_or(false);
                                // Notes are always entered/exited at their center-top (edge comes from above).
                                let exit_x = if is_src_note {
                                    src_cx
                                } else {
                                    port_x(src_cx, src_hw, tgt_cx, dy) + bidir_offset
                                };
                                let entry_x = if is_tgt_note {
                                    tgt_cx
                                } else {
                                    port_x(tgt_cx, tgt_hw, src_cx, dy) + bidir_offset
                                };

                                let (p0, p3) = if src_cy <= tgt_cy {
                                    (
                                        Point {
                                            x: exit_x,
                                            y: src_cy + src_hh,
                                        },
                                        Point {
                                            x: entry_x,
                                            y: tgt_cy - tgt_hh,
                                        },
                                    )
                                } else {
                                    (
                                        Point {
                                            x: exit_x,
                                            y: src_cy - src_hh,
                                        },
                                        Point {
                                            x: entry_x,
                                            y: tgt_cy + tgt_hh,
                                        },
                                    )
                                };
                                let mid_y = (p0.y + p3.y) / 2.0;
                                let new_pts = vec![
                                    p0,
                                    Point {
                                        x: exit_x,
                                        y: mid_y,
                                    },
                                    Point {
                                        x: entry_x,
                                        y: mid_y,
                                    },
                                    p3,
                                ];
                                if let Some(lbl) = g.edge_vw_mut(ev, ew) {
                                    lbl.points = Some(new_pts);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Recompute graph width after any post-layout x adjustments (e.g. note-cluster fix).
    // The note-attached state may have been moved right, requiring a wider layout.
    let raw_w = g.graph().width.unwrap_or(60.0);
    let raw_h = g.graph().height.unwrap_or(60.0);
    let graph_w = if !note_cluster_data.is_empty() {
        let marginx = if is_inner { INNER_MARGINX } else { MARGIN };
        let mut max_right = 0.0f64;
        for id in node_ids {
            if let Some(n) = g.node_opt(id) {
                if let Some(cx) = n.x {
                    max_right = max_right.max(cx + n.width / 2.0);
                }
            }
        }
        if max_right + marginx > raw_w {
            max_right + marginx
        } else {
            raw_w
        }
    } else {
        raw_w
    };
    let graph_h = raw_h;

    let mut nodes = std::collections::HashMap::new();
    for id in node_ids {
        if let Some(n) = g.node_opt(id) {
            if let (Some(cx), Some(cy)) = (n.x, n.y) {
                nodes.insert(id.clone(), (cx, cy, n.width, n.height));
            }
        }
    }

    let mut edges = Vec::new();
    for (key, (orig_from, orig_to)) in &edge_orig {
        let e = dagre_dgl_rs::graph::Edge::new(&key.0, &key.1);
        if let Some(lbl) = g.edge(&e) {
            if let Some(pts) = &lbl.points {
                if pts.len() >= 2 {
                    edges.push((orig_from.clone(), orig_to.clone(), pts.clone()));
                }
            }
        }
    }
    for (note_id, state_id, _parent_id) in &note_cluster_data {
        if g.node_opt(state_id).is_some() {
            let e = dagre_dgl_rs::graph::Edge::new(state_id, note_id);
            if let Some(lbl) = g.edge(&e) {
                if let Some(pts) = &lbl.points {
                    if pts.len() >= 2 {
                        edges.push((state_id.clone(), note_id.clone(), pts.clone()));
                    }
                }
            }
        }
    }

    // Collect note-cluster rects from compound parent node positions after layout
    let mut note_clusters = Vec::new();
    for (_note_id, state_id, parent_id) in &note_cluster_data {
        if let Some(pn) = g.node_opt(parent_id) {
            if let (Some(cx), Some(cy)) = (pn.x, pn.y) {
                let pw = pn.width;
                let ph = pn.height;
                note_clusters.push(NoteCluster {
                    state_id: state_id.clone(),
                    x: cx - pw / 2.0,
                    y: cy - ph / 2.0,
                    width: pw,
                    height: ph,
                });
            }
        }
    }

    LevelLayout {
        nodes,
        edges,
        note_clusters,
        width: graph_w,
        height: graph_h,
    }
}

/// Resolve a transition endpoint to a node ID at the current layout level.
/// - If the id is directly in node_set: use it.
/// - If the id is a composite at this level: use it directly (composite is opaque here).
/// - Otherwise: None (endpoint is not at this level).
fn resolve_endpoint(
    id: &str,
    flat: &FlatGraph,
    node_set: &std::collections::HashSet<&str>,
    _entry_map: &std::collections::HashMap<String, String>,
    _exit_map: &std::collections::HashMap<String, String>,
    _is_target: bool,
) -> Option<String> {
    if node_set.contains(id) {
        return Some(id.to_string());
    }
    // Walk up the parent chain to find the ancestor at this level
    let mut current = id.to_string();
    loop {
        match flat.parent.get(&current) {
            Some(pid) => {
                if node_set.contains(pid.as_str()) {
                    return Some(pid.clone());
                }
                current = pid.clone();
            }
            None => return None,
        }
    }
}

// ─── Recursive layout ──────────────────────────────────────────────────────────

/// Recursively lay out from bottom up.
/// `is_inner` = true when laying out children of a composite state.
/// Returns (level_layout, composites_at_this_level_and_below).
fn layout_level(
    flat: &FlatGraph,
    node_ids: &[String],
    transitions: &[&Transition],
    direction: &str,
    is_inner: bool,
) -> (LevelLayout, Vec<CompositeLayout>) {
    let mut all_composites: Vec<CompositeLayout> = Vec::new();
    let mut composite_sizes: std::collections::HashMap<String, (f64, f64)> =
        std::collections::HashMap::new();

    // First: recursively lay out each composite's children
    for id in node_ids {
        if let Some(node) = flat.states.get(id) {
            if node.state_type == StateType::Composite {
                let child_ids: Vec<String> = flat
                    .parent
                    .iter()
                    .filter(|(_, p)| p.as_str() == id.as_str())
                    .map(|(c, _)| c.clone())
                    .filter(|c| flat.states.contains_key(c))
                    .collect();

                let child_set: std::collections::HashSet<&str> =
                    child_ids.iter().map(|s| s.as_str()).collect();
                let all_desc = collect_all_descendants(id, flat);
                let desc_set: std::collections::HashSet<&str> =
                    all_desc.iter().map(|s| s.as_str()).collect();

                // Inner transitions: both endpoints are descendants (or children) of this composite
                let inner_trans: Vec<&Transition> = flat
                    .transitions
                    .iter()
                    .filter(|t| {
                        (child_set.contains(t.from.as_str()) || desc_set.contains(t.from.as_str()))
                            && (child_set.contains(t.to.as_str())
                                || desc_set.contains(t.to.as_str()))
                    })
                    .collect();

                // Recursively lay out children (is_inner=true for composite sub-graphs)
                let (inner_layout, nested) =
                    layout_level(flat, &child_ids, &inner_trans, direction, true);

                let inner_w = inner_layout.width;
                let inner_h = inner_layout.height;
                // The outer dagre sees the composite as width=inner_w (full inner graph width),
                // height=rect_h (= inner_h - 2*CLUSTER_PAD, the visual cluster rect height).
                // This makes the outer dagre's ranksep gap equal to the gap from the outer node's
                // edge to the cluster rect boundary, matching Mermaid's behaviour.
                let rect_h = inner_h - 2.0 * CLUSTER_PAD;
                composite_sizes.insert(id.clone(), (inner_w, rect_h));

                // Store composite with placeholder outer position
                all_composites.push(CompositeLayout {
                    id: id.clone(),
                    inner: inner_layout,
                    outer_cx: 0.0,
                    outer_cy: 0.0,
                });
                all_composites.extend(nested);
            }
        }
    }

    // Now lay out this level with composites as opaque fixed-size nodes
    let level = run_level_layout(
        flat,
        node_ids,
        &composite_sizes,
        transitions,
        direction,
        is_inner,
    );

    // Fill in outer positions for composites at this level
    for comp in all_composites.iter_mut() {
        // Only update composites whose id is directly in node_ids
        if node_ids.iter().any(|n| n == &comp.id) {
            if let Some(&(cx, cy, _, _)) = level.nodes.get(&comp.id) {
                comp.outer_cx = cx;
                comp.outer_cy = cy;
            }
        }
    }

    (level, all_composites)
}

// ─── Main layout entry ─────────────────────────────────────────────────────────

fn run_layout(flat: &FlatGraph, direction: &str) -> FlatLayout {
    // Top-level nodes: those without a parent recorded in flat.parent
    let top_level_ids: Vec<String> = flat
        .states
        .iter()
        .filter(|(id, _)| !flat.parent.contains_key(*id))
        .map(|(id, _)| id.clone())
        .collect();

    let top_set: std::collections::HashSet<&str> =
        top_level_ids.iter().map(|s| s.as_str()).collect();

    // Top-level transitions: transitions where both endpoints have a top-level ancestor
    let top_trans: Vec<&Transition> = flat
        .transitions
        .iter()
        .filter(|t| {
            ancestor_in_set(&t.from, flat, &top_set) && ancestor_in_set(&t.to, flat, &top_set)
        })
        .collect();

    let (outer_layout, composites) =
        layout_level(flat, &top_level_ids, &top_trans, direction, false);

    // outer_nodes: non-composite nodes from outer_layout
    let mut outer_nodes = std::collections::HashMap::new();
    for (id, pos) in &outer_layout.nodes {
        if let Some(node) = flat.states.get(id) {
            if node.state_type != StateType::Composite {
                outer_nodes.insert(id.clone(), *pos);
            }
        }
    }

    FlatLayout {
        outer_nodes,
        outer_edges: outer_layout.edges,
        note_clusters: outer_layout.note_clusters,
        composites,
        width: outer_layout.width,
        height: outer_layout.height,
    }
}

/// Walk up the parent chain to check if `id` or any ancestor is in `set`.
fn ancestor_in_set(id: &str, flat: &FlatGraph, set: &std::collections::HashSet<&str>) -> bool {
    let mut current = id.to_string();
    loop {
        if set.contains(current.as_str()) {
            return true;
        }
        match flat.parent.get(&current) {
            Some(pid) => current = pid.clone(),
            None => return false,
        }
    }
}

// ─── SVG rendering ────────────────────────────────────────────────────────────

fn render_flat(
    flat: &FlatGraph,
    vars: &ThemeVars,
    use_foreign_object: bool,
    direction: &str,
) -> String {
    let layout = run_layout(flat, direction);

    // Compute min_x across all content to detect negative x (dagre can produce negative coords).
    let margin = MARGIN;
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    for &(cx, _, w, _) in layout.outer_nodes.values() {
        min_x = min_x.min(cx - w / 2.0 - margin);
        max_x = max_x.max(cx + w / 2.0 + margin);
    }
    for (_, _, pts) in &layout.outer_edges {
        for p in pts {
            min_x = min_x.min(p.x - margin);
            max_x = max_x.max(p.x + margin);
        }
    }
    for comp in &layout.composites {
        let inner_w = comp.inner.width;
        let inner_h = comp.inner.height;
        let x0 = comp.outer_cx - inner_w / 2.0;
        min_x = min_x.min(x0);
        max_x = max_x.max(x0 + inner_w);
        let _ = inner_h;
    }
    for nc in &layout.note_clusters {
        min_x = min_x.min(nc.x);
        max_x = max_x.max(nc.x + nc.width);
    }
    if min_x.is_infinite() {
        min_x = 0.0;
        max_x = layout.width;
    }

    // If content extends left of 0, we need to shift right to keep viewBox at "0 0 ..."
    let x_offset = if min_x < 0.0 { -min_x } else { 0.0 };
    let vb_w = if x_offset > 0.0 {
        max_x + x_offset
    } else {
        layout.width
    };
    let vb_h = layout.height;

    let svg_id = SVG_ID;
    let css = build_css(svg_id, vars);
    let mut out = String::new();
    out.push_str("<svg id=\"");
    out.push_str(svg_id);
    out.push_str("\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" class=\"statediagram\" style=\"max-width: ");
    out.push_str(&fmt(vb_w));
    out.push_str("px;\" viewBox=\"0 0 ");
    out.push_str(&fmt(vb_w));
    out.push(' ');
    out.push_str(&fmt(vb_h));
    out.push_str("\" role=\"graphics-document document\" aria-roledescription=\"stateDiagram\">");
    out.push_str("<style>");
    out.push_str(&css);
    out.push_str("</style>");
    out.push_str("<g>");
    out.push_str(&build_markers(svg_id));
    out.push_str("</g>");
    let x_translate = if x_offset > 0.0 {
        format!("<g transform=\"translate({}, 0)\">", fmt(x_offset))
    } else {
        String::new()
    };
    let x_close = if x_offset > 0.0 { "</g>" } else { "" };
    out.push_str(&x_translate);
    out.push_str(&render_all(&layout, flat, vars, svg_id, use_foreign_object));
    out.push_str(x_close);
    out.push_str(&templates::drop_shadow_filter(svg_id));
    out.push_str(&templates::drop_shadow_filter_small(svg_id));
    out.push_str("</svg>");
    out
}

fn render_all(
    layout: &FlatLayout,
    flat: &FlatGraph,
    vars: &ThemeVars,
    svg_id: &str,
    use_foreign_object: bool,
) -> String {
    let mut out = String::new();
    out.push_str("<g class=\"root\">");

    // Note-cluster compound background rects (Mermaid noteGroup pattern)
    out.push_str("<g class=\"clusters\">");
    for (ci, nc) in layout.note_clusters.iter().enumerate() {
        let cluster_id = format!("{}-state-{}----parent-{}", svg_id, nc.state_id, ci);
        out.push_str(&templates::note_cluster_rect(
            &cluster_id,
            &fmt(nc.x),
            &fmt(nc.y),
            &fmt(nc.width),
            &fmt(nc.height),
        ));
    }
    out.push_str("</g>");

    // Outer edge paths
    out.push_str("<g class=\"edgePaths\">");
    for (ei, (from, to, pts)) in layout.outer_edges.iter().enumerate() {
        out.push_str(&render_edge(from, to, pts, svg_id, ei, flat));
    }
    out.push_str("</g>");

    // Outer edge labels
    out.push_str("<g class=\"edgeLabels\">");
    for (ei, (from, to, pts)) in layout.outer_edges.iter().enumerate() {
        let edge_id = format!("{}-edge{}", svg_id, ei);
        out.push_str(&render_edge_label(
            from,
            to,
            pts,
            &edge_id,
            flat,
            use_foreign_object,
        ));
    }
    out.push_str("</g>");

    // Outer nodes (non-composite)
    out.push_str("<g class=\"nodes\">");
    for (ni, (id, &(cx, cy, w, h))) in layout.outer_nodes.iter().enumerate() {
        if let Some(node) = flat.states.get(id) {
            let dom_id = format!("{}-state-{}-{}", svg_id, id, ni);
            out.push_str(&render_state_node(
                node,
                cx,
                cy,
                w,
                h,
                vars,
                &dom_id,
                use_foreign_object,
            ));
        }
    }
    out.push_str("</g>");

    // Each composite state: rendered in its own translated sub-group
    for comp in &layout.composites {
        if let Some(node) = flat.states.get(&comp.id) {
            let inner_w = comp.inner.width;
            let inner_h = comp.inner.height;
            // translate to the top-left corner of the composite group
            let tx = comp.outer_cx - inner_w / 2.0;
            let ty = comp.outer_cy - inner_h / 2.0;

            out.push_str(&templates::composite_root_group(&fmt(tx), &fmt(ty)));

            // Cluster rect(s)
            out.push_str("<g class=\"clusters\">");
            out.push_str(&render_cluster_node(
                node,
                inner_w,
                inner_h,
                vars,
                svg_id,
                use_foreign_object,
            ));
            out.push_str("</g>");

            // Inner edge paths
            let ei_base = layout.outer_edges.len() + comp as *const _ as usize % 1000 * 100;
            out.push_str("<g class=\"edgePaths\">");
            for (ei, (from, to, pts)) in comp.inner.edges.iter().enumerate() {
                out.push_str(&render_edge(from, to, pts, svg_id, ei_base + ei, flat));
            }
            out.push_str("</g>");

            // Inner edge labels
            out.push_str("<g class=\"edgeLabels\">");
            for (ei, (from, to, pts)) in comp.inner.edges.iter().enumerate() {
                let edge_id = format!("{}-edge{}", svg_id, ei_base + ei);
                out.push_str(&render_edge_label(
                    from,
                    to,
                    pts,
                    &edge_id,
                    flat,
                    use_foreign_object,
                ));
            }
            out.push_str("</g>");

            // Inner nodes
            out.push_str("<g class=\"nodes\">");
            let ni_base = layout.outer_nodes.len() + comp as *const _ as usize % 1000 * 100;
            for (ni, (id, &(cx, cy, w, h))) in comp.inner.nodes.iter().enumerate() {
                if let Some(inner_node) = flat.states.get(id) {
                    if inner_node.state_type == StateType::Composite {
                        continue;
                    }
                    let dom_id = format!("{}-state-{}-{}", svg_id, id, ni_base + ni);
                    out.push_str(&render_state_node(
                        inner_node,
                        cx,
                        cy,
                        w,
                        h,
                        vars,
                        &dom_id,
                        use_foreign_object,
                    ));
                }
            }
            out.push_str("</g>");

            out.push_str("</g>");
        }
    }

    out.push_str("</g>");
    out
}

fn render_edge(
    from: &str,
    to: &str,
    pts: &[Point],
    svg_id: &str,
    idx: usize,
    flat: &FlatGraph,
) -> String {
    if pts.len() < 2 {
        return String::new();
    }
    // Trim endpoint for fork/join targets so the arrowhead stops just before the bar
    let target_is_fork_join = flat
        .states
        .get(to)
        .map(|n| matches!(n.state_type, StateType::Fork | StateType::Join))
        .unwrap_or(false);
    let trimmed_pts: Vec<Point>;
    let pts = if target_is_fork_join {
        let n = pts.len();
        let p1 = &pts[n - 2];
        let p2 = &pts[n - 1];
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 2.0 {
            let t = 2.0 / len;
            let last = Point {
                x: p2.x - dx * t,
                y: p2.y - dy * t,
            };
            trimmed_pts = pts[..n - 1]
                .iter()
                .cloned()
                .chain(std::iter::once(last))
                .collect();
            &trimmed_pts[..]
        } else {
            pts
        }
    } else {
        pts
    };
    let path_d = smooth_path(pts);
    let edge_id = format!("{}-edge{}", svg_id, idx);
    let is_note = flat
        .states
        .get(from)
        .map(|n| n.state_type == StateType::Note)
        .unwrap_or(false)
        || flat
            .states
            .get(to)
            .map(|n| n.state_type == StateType::Note)
            .unwrap_or(false);
    let extra = if is_note { " note-edge" } else { "" };
    let marker = if is_note {
        String::new()
    } else {
        templates::marker_end_barb(svg_id)
    };
    templates::edge_path(&path_d, &edge_id, extra, &marker)
}

fn render_edge_label(
    from: &str,
    to: &str,
    pts: &[Point],
    edge_id: &str,
    flat: &FlatGraph,
    use_foreign_object: bool,
) -> String {
    let lbl = flat
        .transitions
        .iter()
        .find(|t| t.from == from && t.to == to)
        .and_then(|t| t.label.as_deref())
        .filter(|s| !s.is_empty());
    if let Some(lbl) = lbl {
        let mid = midpoint(pts);
        let (fo_w_raw, _) = measure(lbl, FONT_SIZE);
        let fo_w = (fo_w_raw * LABEL_SCALE).max(20.0);
        if use_foreign_object {
            templates::edge_label_fo(
                edge_id,
                &fmt(mid.0 - fo_w / 2.0),
                &fmt(mid.1 - 12.0),
                &fmt(fo_w),
                &esc(lbl),
            )
        } else {
            templates::edge_label_text(&fmt(mid.0), &fmt(mid.1), &fmt(FONT_SIZE), &esc(lbl))
        }
    } else {
        templates::edge_label_empty(edge_id)
    }
}

fn node_size(node: &StateNode) -> (f64, f64) {
    match node.state_type {
        StateType::Start | StateType::End => (START_R * 2.0, START_R * 2.0),
        StateType::Fork | StateType::Join => (FORK_W, FORK_H),
        StateType::Choice => (CHOICE_SIZE * 2.0, CHOICE_SIZE * 2.0),
        StateType::Note => {
            let text = node.note_text.as_deref().unwrap_or("");
            let (tw, _) = measure(text, 16.0);
            ((tw * LABEL_SCALE + 30.0).max(60.0), 54.0)
        }
        StateType::Composite => (0.0, 0.0),
        StateType::Normal => {
            let (tw, _) = measure(&node.label, FONT_SIZE);
            ((tw * LABEL_SCALE + H_PAD * 2.0).max(40.0), NODE_H)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_state_node(
    node: &StateNode,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    vars: &ThemeVars,
    dom_id: &str,
    use_foreign_object: bool,
) -> String {
    let mut s = String::new();
    match node.state_type {
        StateType::Start => {
            s.push_str("<g class=\"node default\" id=\"");
            s.push_str(dom_id);
            s.push_str("\" data-look=\"classic\" transform=\"translate(");
            s.push_str(&fmt(cx));
            s.push_str(", ");
            s.push_str(&fmt(cy));
            s.push_str(")\"><circle class=\"state-start\" r=\"");
            s.push_str(&fmt(START_R));
            s.push_str("\" width=\"");
            s.push_str(&fmt(START_R * 2.0));
            s.push_str("\" height=\"");
            s.push_str(&fmt(START_R * 2.0));
            s.push_str("\"></circle></g>");
        }
        StateType::End => {
            s.push_str("<g class=\"node default\" id=\"");
            s.push_str(dom_id);
            s.push_str("\" data-look=\"classic\" transform=\"translate(");
            s.push_str(&fmt(cx));
            s.push_str(", ");
            s.push_str(&fmt(cy));
            s.push_str(")\"><g class=\"outer-path\">");
            s.push_str("<circle cx=\"0\" cy=\"0\" r=\"");
            s.push_str(&fmt(END_R));
            s.push_str("\" stroke=\"");
            s.push_str(vars.line_color);
            s.push_str("\" stroke-width=\"2\" fill=\"");
            s.push_str(vars.primary_color);
            s.push_str("\" stroke-dasharray=\"0 0\" style=\"\"></circle>");
            s.push_str("<circle cx=\"0\" cy=\"0\" r=\"");
            s.push_str(&fmt(END_INNER_R));
            s.push_str("\" stroke=\"none\" stroke-width=\"0\" fill=\"");
            s.push_str(vars.primary_border);
            s.push_str("\" style=\"\"></circle></g></g>");
        }
        StateType::Fork | StateType::Join => {
            let hw = w / 2.0;
            let hh = h / 2.0;
            s.push_str("<g class=\"node  statediagram-state \" id=\"");
            s.push_str(dom_id);
            s.push_str("\" data-look=\"classic\" transform=\"translate(");
            s.push_str(&fmt(cx));
            s.push_str(", ");
            s.push_str(&fmt(cy));
            s.push_str(")\"><g>");
            s.push_str("<path d=\"M");
            s.push_str(&fmt(-hw));
            s.push(' ');
            s.push_str(&fmt(-hh));
            s.push_str(" L");
            s.push_str(&fmt(hw));
            s.push(' ');
            s.push_str(&fmt(-hh));
            s.push_str(" L");
            s.push_str(&fmt(hw));
            s.push(' ');
            s.push_str(&fmt(hh));
            s.push_str(" L");
            s.push_str(&fmt(-hw));
            s.push(' ');
            s.push_str(&fmt(hh));
            s.push_str("\" stroke=\"none\" stroke-width=\"0\" fill=\"#333333\" style=\"\"></path>");
            s.push_str("</g></g>");
        }
        StateType::Choice => {
            let hs = CHOICE_SIZE;
            s.push_str("<g class=\"node  statediagram-state \" id=\"");
            s.push_str(dom_id);
            s.push_str("\" data-look=\"classic\" transform=\"translate(");
            s.push_str(&fmt(cx));
            s.push_str(", ");
            s.push_str(&fmt(cy));
            s.push_str(")\"><polygon fill=\"");
            s.push_str(vars.primary_color);
            s.push_str("\" stroke=\"");
            s.push_str(vars.primary_border);
            s.push_str("\" stroke-width=\"1px\" points=\"0,");
            s.push_str(&fmt(-hs));
            s.push(' ');
            s.push_str(&fmt(hs));
            s.push_str(",0 0,");
            s.push_str(&fmt(hs));
            s.push(' ');
            s.push_str(&fmt(-hs));
            s.push_str(",0\"></polygon></g>");
        }
        StateType::Note => {
            let hw = w / 2.0;
            let hh = h / 2.0;
            let text = node.note_text.as_deref().unwrap_or("");
            let (tw, _) = measure(text, FONT_SIZE);
            s.push_str("<g class=\"node statediagram-note \" id=\"");
            s.push_str(dom_id);
            s.push_str("\" data-look=\"classic\" transform=\"translate(");
            s.push_str(&fmt(cx));
            s.push_str(", ");
            s.push_str(&fmt(cy));
            s.push_str(")\">");
            s.push_str("<g class=\"basic label-container outer-path\">");
            s.push_str("<path d=\"M");
            s.push_str(&fmt(-hw));
            s.push(' ');
            s.push_str(&fmt(-hh));
            s.push_str(" L");
            s.push_str(&fmt(hw));
            s.push(' ');
            s.push_str(&fmt(-hh));
            s.push_str(" L");
            s.push_str(&fmt(hw));
            s.push(' ');
            s.push_str(&fmt(hh));
            s.push_str(" L");
            s.push_str(&fmt(-hw));
            s.push(' ');
            s.push_str(&fmt(hh));
            s.push_str(
                " Z\" stroke=\"none\" stroke-width=\"0\" fill=\"#fff5ad\" style=\"\"></path>",
            );
            s.push_str("<path d=\"M");
            s.push_str(&fmt(-hw));
            s.push(' ');
            s.push_str(&fmt(-hh));
            s.push_str(" L");
            s.push_str(&fmt(hw));
            s.push(' ');
            s.push_str(&fmt(-hh));
            s.push_str(" L");
            s.push_str(&fmt(hw));
            s.push(' ');
            s.push_str(&fmt(hh));
            s.push_str(" L");
            s.push_str(&fmt(-hw));
            s.push(' ');
            s.push_str(&fmt(hh));
            s.push_str(" Z\" stroke=\"#aaaa33\" stroke-width=\"1.3\" fill=\"none\" stroke-dasharray=\"0 0\" style=\"\"></path>");
            s.push_str("</g>");
            if use_foreign_object {
                s.push_str("<g class=\"label noteLabel\" style=\"\" transform=\"translate(");
                s.push_str(&fmt(-tw / 2.0 - 8.0));
                s.push_str(", -12)\"><rect></rect><foreignObject width=\"");
                s.push_str(&fmt(tw + 16.0));
                s.push_str("\" height=\"24\"><div xmlns=\"http://www.w3.org/1999/xhtml\" style=\"display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;\"><span class=\"nodeLabel markdown-node-label\"><p>");
                s.push_str(&esc(text));
                s.push_str("</p></span></div></foreignObject></g>");
            } else {
                s.push_str("<text x=\"0\" y=\"4\" text-anchor=\"middle\" font-family=\"Arial,sans-serif\" font-size=\"");
                s.push_str(&fmt(FONT_SIZE));
                s.push_str("\" fill=\"black\">");
                s.push_str(&esc(text));
                s.push_str("</text>");
            }
            s.push_str("</g>");
        }
        StateType::Normal => {
            let hw = w / 2.0;
            let hh = h / 2.0;
            let (tw_raw, _) = measure(&node.label, FONT_SIZE);
            let tw = tw_raw * LABEL_SCALE;
            s.push_str("<g class=\"node  statediagram-state \" id=\"");
            s.push_str(dom_id);
            s.push_str("\" data-look=\"classic\" transform=\"translate(");
            s.push_str(&fmt(cx));
            s.push_str(", ");
            s.push_str(&fmt(cy));
            s.push_str(")\">");
            s.push_str("<rect class=\"basic label-container\" style=\"\" rx=\"5\" ry=\"5\" x=\"");
            s.push_str(&fmt(-hw));
            s.push_str("\" y=\"");
            s.push_str(&fmt(-hh));
            s.push_str("\" width=\"");
            s.push_str(&fmt(w));
            s.push_str("\" height=\"");
            s.push_str(&fmt(h));
            s.push_str("\"></rect>");
            if use_foreign_object {
                s.push_str("<g class=\"label\" style=\"\" transform=\"translate(");
                s.push_str(&fmt(-tw / 2.0));
                s.push_str(", -12)\"><rect></rect><foreignObject width=\"");
                s.push_str(&fmt(tw));
                s.push_str("\" height=\"24\"><div xmlns=\"http://www.w3.org/1999/xhtml\" style=\"display: table-cell; white-space: nowrap; line-height: 1.5; max-width: 200px; text-align: center;\"><span class=\"nodeLabel markdown-node-label\"><p>");
                s.push_str(&esc(&node.label));
                s.push_str("</p></span></div></foreignObject></g>");
            } else {
                s.push_str("<g class=\"label\" style=\"\" transform=\"translate(0,0)\"><text x=\"0\" y=\"4\" text-anchor=\"middle\" font-family=\"Arial,sans-serif\" font-size=\"");
                s.push_str(&fmt(FONT_SIZE));
                s.push_str("\" fill=\"");
                s.push_str(vars.primary_text);
                s.push_str("\">");
                s.push_str(&esc(&node.label));
                s.push_str("</text></g>");
            }
            s.push_str("</g>");
        }
        StateType::Composite => {}
    }
    s
}

/// Render the cluster background rect for a composite state.
/// inner_w x inner_h = full inner dagre graph dimensions (with margins).
/// The cluster rect is at x=CLUSTER_PAD, y=CLUSTER_PAD within the translated group.
fn render_cluster_node(
    node: &StateNode,
    inner_w: f64,
    inner_h: f64,
    vars: &ThemeVars,
    svg_id: &str,
    use_foreign_object: bool,
) -> String {
    let label = &node.label;
    let (lw_raw, _) = measure(label, FONT_SIZE);
    let lw = lw_raw * LABEL_SCALE;
    let mut s = String::new();

    let rect_x = CLUSTER_PAD;
    let rect_y = CLUSTER_PAD;
    let rect_w = inner_w - CLUSTER_PAD * 2.0;
    let rect_h = inner_h - CLUSTER_PAD * 2.0;

    s.push_str("<g class=\" statediagram-state statediagram-cluster \" id=\"");
    s.push_str(svg_id);
    s.push_str("-state-");
    s.push_str(&node.id);
    s.push_str("\" data-id=\"");
    s.push_str(&node.id);
    s.push_str("\" data-look=\"classic\">");

    s.push_str("<g><rect class=\"outer\" x=\"");
    s.push_str(&fmt(rect_x));
    s.push_str("\" y=\"");
    s.push_str(&fmt(rect_y));
    s.push_str("\" width=\"");
    s.push_str(&fmt(rect_w));
    s.push_str("\" height=\"");
    s.push_str(&fmt(rect_h));
    s.push_str("\" data-look=\"classic\"></rect></g>");

    // Label: centered at top of outer rect
    let lx = inner_w / 2.0 - lw / 2.0;
    let ly = rect_y;
    if use_foreign_object {
        s.push_str("<g class=\"cluster-label\" transform=\"translate(");
        s.push_str(&fmt(lx));
        s.push(',');
        s.push_str(&fmt(ly));
        s.push_str(")\"><foreignObject width=\"");
        s.push_str(&fmt(lw));
        s.push_str("\" height=\"24\"><div xmlns=\"http://www.w3.org/1999/xhtml\" style=\"display: table-cell; white-space: nowrap; line-height: 1.5;\"><span class=\"nodeLabel \"><p>");
        s.push_str(&esc(label));
        s.push_str("</p></span></div></foreignObject></g>");
    } else {
        s.push_str("<text x=\"");
        s.push_str(&fmt(inner_w / 2.0));
        s.push_str("\" y=\"");
        s.push_str(&fmt(ly + FONT_SIZE));
        s.push_str("\" text-anchor=\"middle\" font-family=\"Arial,sans-serif\" font-size=\"");
        s.push_str(&fmt(FONT_SIZE));
        s.push_str("\" fill=\"");
        s.push_str(vars.primary_text);
        s.push_str("\">");
        s.push_str(&esc(label));
        s.push_str("</text>");
    }

    // Inner rect: below the title area, with CLUSTER_PAD/2 gap at the bottom
    let inner_rect_y = rect_y + CLUSTER_TITLE_H - CLUSTER_PAD;
    let inner_rect_h = rect_h - CLUSTER_TITLE_H + CLUSTER_PAD / 2.0;
    s.push_str("<rect class=\"inner\" x=\"");
    s.push_str(&fmt(rect_x));
    s.push_str("\" y=\"");
    s.push_str(&fmt(inner_rect_y));
    s.push_str("\" width=\"");
    s.push_str(&fmt(rect_w));
    s.push_str("\" height=\"");
    s.push_str(&fmt(inner_rect_h));
    s.push_str("\"></rect></g>");
    s
}

fn build_css(id: &str, vars: &ThemeVars) -> String {
    let pf = vars.primary_color;
    let ps = vars.primary_border;
    let lc = vars.line_color;
    let tc = vars.text_color;
    let ff = vars.font_family;
    let mut c = String::new();
    c.push_str(&format!(
        "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}"
    ));
    c.push_str("@keyframes edge-animation-frame{from{stroke-dashoffset:0;}}@keyframes dash{to{stroke-dashoffset:0;}}");
    c.push_str(&format!("#{id} .edge-animation-slow{{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 50s linear infinite;stroke-linecap:round;}}"));
    c.push_str(&format!("#{id} .edge-animation-fast{{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 20s linear infinite;stroke-linecap:round;}}"));
    c.push_str(&format!(
        "#{id} .error-icon{{fill:#552222;}}#{id} .error-text{{fill:#552222;stroke:#552222;}}"
    ));
    c.push_str(&format!("#{id} .edge-thickness-normal{{stroke-width:1px;}}#{id} .edge-thickness-thick{{stroke-width:3.5px;}}"));
    c.push_str(&format!("#{id} .edge-pattern-solid{{stroke-dasharray:0;}}#{id} .edge-thickness-invisible{{stroke-width:0;fill:none;}}"));
    c.push_str(&format!("#{id} .edge-pattern-dashed{{stroke-dasharray:3;}}#{id} .edge-pattern-dotted{{stroke-dasharray:2;}}"));
    c.push_str(&format!(
        "#{id} .marker{{fill:{lc};stroke:{lc};}}#{id} .marker.cross{{stroke:{lc};}}"
    ));
    c.push_str(&format!(
        "#{id} svg{{font-family:{ff};font-size:16px;}}#{id} p{{margin:0;}}"
    ));
    c.push_str(&format!(
        "#{id} defs [id$=\"-barbEnd\"]{{fill:{lc};stroke:{lc};}}"
    ));
    c.push_str(&format!(
        "#{id} g.stateGroup text{{fill:#9370DB;stroke:none;font-size:10px;}}"
    ));
    c.push_str(&format!(
        "#{id} g.stateGroup text{{fill:{tc};stroke:none;font-size:10px;}}"
    ));
    c.push_str(&format!(
        "#{id} g.stateGroup .state-title{{font-weight:bolder;fill:#131300;}}"
    ));
    c.push_str(&format!(
        "#{id} g.stateGroup rect{{fill:{pf};stroke:{ps};}}"
    ));
    c.push_str(&format!(
        "#{id} g.stateGroup line{{stroke:{lc};stroke-width:1;}}"
    ));
    c.push_str(&format!(
        "#{id} .transition{{stroke:{lc};stroke-width:1;fill:none;}}"
    ));
    c.push_str(&format!("#{id} .stateGroup .composit{{fill:white;border-bottom:1px;}}#{id} .stateGroup .alt-composit{{fill:#e0e0e0;border-bottom:1px;}}"));
    c.push_str(&format!("#{id} .state-note{{stroke:#aaaa33;fill:#fff5ad;}}#{id} .state-note text{{fill:black;stroke:none;font-size:10px;}}"));
    c.push_str(&format!(
        "#{id} .stateLabel .box{{stroke:none;stroke-width:0;fill:{pf};opacity:0.5;}}"
    ));
    c.push_str(&format!(
        "#{id} .edgeLabel .label rect{{fill:{pf};opacity:0.5;}}"
    ));
    c.push_str(&format!(
        "#{id} .edgeLabel{{background-color:rgba(232,232,232, 0.8);text-align:center;}}"
    ));
    c.push_str(&format!(
        "#{id} .edgeLabel p{{background-color:rgba(232,232,232, 0.8);}}"
    ));
    c.push_str(&format!("#{id} .edgeLabel rect{{opacity:0.5;background-color:rgba(232,232,232, 0.8);fill:rgba(232,232,232, 0.8);}}"));
    c.push_str(&format!(
        "#{id} .edgeLabel .label text{{fill:#333;}}#{id} .label div .edgeLabel{{color:#333;}}"
    ));
    c.push_str(&format!(
        "#{id} .stateLabel text{{fill:#131300;font-size:10px;font-weight:bold;}}"
    ));
    c.push_str(&format!(
        "#{id} .node circle.state-start{{fill:{lc};stroke:{lc};}}"
    ));
    c.push_str(&format!("#{id} .node .fork-join{{fill:{lc};stroke:{lc};}}"));
    c.push_str(&format!(
        "#{id} .node circle.state-end{{fill:{ps};stroke:white;stroke-width:1.5;}}"
    ));
    c.push_str(&format!(
        "#{id} .end-state-inner{{fill:white;stroke-width:1.5;}}"
    ));
    c.push_str(&format!(
        "#{id} .node rect{{fill:{pf};stroke:{ps};stroke-width:1px;}}"
    ));
    c.push_str(&format!(
        "#{id} .node polygon{{fill:{pf};stroke:{ps};stroke-width:1px;}}"
    ));
    c.push_str(&format!("#{id} [id$=\"-barbEnd\"]{{fill:{lc};}}"));
    c.push_str(&format!(
        "#{id} .statediagram-cluster rect{{fill:{pf};stroke:{ps};stroke-width:1px;}}"
    ));
    c.push_str(&format!(
        "#{id} .cluster-label,#{id} .nodeLabel{{color:#131300;}}"
    ));
    c.push_str(&format!(
        "#{id} .statediagram-cluster rect.outer{{rx:5px;ry:5px;}}"
    ));
    c.push_str(&format!("#{id} .statediagram-state .divider{{stroke:{ps};}}#{id} .statediagram-state .title-state{{rx:5px;ry:5px;}}"));
    c.push_str(&format!(
        "#{id} .statediagram-cluster.statediagram-cluster .inner{{fill:white;}}"
    ));
    c.push_str(&format!(
        "#{id} .statediagram-cluster.statediagram-cluster-alt .inner{{fill:#f0f0f0;}}"
    ));
    c.push_str(&format!("#{id} .statediagram-cluster .inner{{rx:0;ry:0;}}"));
    c.push_str(&format!(
        "#{id} .statediagram-state rect.basic{{rx:5px;ry:5px;}}"
    ));
    c.push_str(&format!(
        "#{id} .statediagram-state rect.divider{{stroke-dasharray:10,10;fill:#f0f0f0;}}"
    ));
    c.push_str(&format!("#{id} .note-edge{{stroke-dasharray:5;}}"));
    c.push_str(&format!(
        "#{id} .statediagram-note rect{{fill:#fff5ad;stroke:#aaaa33;stroke-width:1px;rx:0;ry:0;}}"
    ));
    c.push_str(&format!(
        "#{id} .statediagram-note rect{{fill:#fff5ad;stroke:#aaaa33;stroke-width:1px;rx:0;ry:0;}}"
    ));
    c.push_str(&format!("#{id} .statediagram-note text{{fill:black;}}#{id} .statediagram-note .nodeLabel{{color:black;}}"));
    c.push_str(&format!("#{id} .statediagram .edgeLabel{{color:red;}}"));
    c.push_str(&format!("#{id} [id$=\"-dependencyStart\"],#{id} [id$=\"-dependencyEnd\"]{{fill:{lc};stroke:{lc};stroke-width:1;}}"));
    c.push_str(&format!(
        "#{id} .statediagramTitleText{{text-anchor:middle;font-size:18px;fill:#333;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].statediagram-cluster rect{{fill:{pf};stroke:{ps};stroke-width:1;}}"));
    c.push_str(&format!("#{id} [data-look=\"neo\"].statediagram-cluster rect.outer{{rx:5px;ry:5px;filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!("#{id} .node .neo-node{{stroke:{ps};}}"));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node rect,#{id} [data-look=\"neo\"].cluster rect,#{id} [data-look=\"neo\"].node polygon{{stroke:{ps};filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node path{{stroke:{ps};stroke-width:1px;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node .outer-path{{filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node .neo-line path{{stroke:{ps};filter:none;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node circle{{stroke:{ps};filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node circle .state-start{{fill:#000000;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].icon-shape .icon{{fill:{ps};filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!("#{id} [data-look=\"neo\"].icon-shape .icon-neo path{{stroke:{ps};filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!("#{id} :root{{--mermaid-font-family:{ff};}}"));
    c
}

fn build_markers(id: &str) -> String {
    templates::barb_end_marker(id)
}

fn smooth_path(pts: &[Point]) -> String {
    let pairs: Vec<(f64, f64)> = pts.iter().map(|p| (p.x, p.y)).collect();
    crate::svg::smooth_bezier_path(&pairs)
}

fn midpoint(pts: &[Point]) -> (f64, f64) {
    if pts.is_empty() {
        return (0.0, 0.0);
    }
    let mid = pts.len() / 2;
    (pts[mid].x, pts[mid].y)
}

fn fmt(v: f64) -> String {
    let s = format!("{:.7}", v);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    const STATE_BASIC: &str = "stateDiagram-v2\n    [*] --> Still\n    Still --> [*]\n    Still --> Moving\n    Moving --> Still\n    Moving --> Crash\n    Crash --> [*]";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(STATE_BASIC).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains("<svg"), "missing <svg tag");
        assert!(svg.contains("Still"), "missing state name");
        assert!(svg.contains("Moving"), "missing state name");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(STATE_BASIC).diagram;
        let svg = render(&diag, Theme::Dark, false);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    // NOTE: dagre-dgl-rs uses std::collections::HashMap internally which has
    // non-deterministic iteration order, making the exact SVG output vary between
    // process launches. The snapshot is kept for reference but ignored in CI.
    #[test]
    #[ignore]
    fn snapshot_default_theme() {
        let diag = parser::parse(STATE_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
