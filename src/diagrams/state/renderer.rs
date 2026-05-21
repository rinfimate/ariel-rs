// State diagram renderer — faithful port of Mermaid stateRenderer-v3-unified.ts
// + dataFetcher.ts compound note pattern.
//
// Node layout follows dataFetcher.ts exactly:
//   - Regular state   → shape "rect", no parentId
//   - Note compound   → shape "noteGroup", isGroup=true, padding=16, no parentId
//   - Note node       → shape "note", parentId = "{stateId}----parent"
//   - State with note → no parentId (root-level, NOT inside compound)
//   - Note edge       → arrowhead="none", classes="transition note-edge"
//
// Sizes follow note.ts / shapes.js:
//   - State rect: (text_w + 2*padding).max(40) × 40  (padding=8)
//   - Note box: (text_w + 2*padding) × (text_h + 2*padding)
//   - NoteGroup: dagre computes from children (set to 0×0 initially)
//   - Fork/Join: 70×7 (outer) or 7×70 (inner LR)
//   - Start:  r=7 circle → 14×14
//   - End:    r=7+2 outer → 18×18
//   - Choice (diamond): 14×14

use super::constants::*;
use super::parser::{Edge, Node, Shape, StateDiagram};
use super::templates::{
    composite_cluster, composite_inner_group, drop_shadow_filter, edge_label_empty, edge_path,
    markers, node_choice, node_fork_join, node_note, node_rect, node_state_end, node_state_start,
    note_cluster, text_composite_label, text_edge_label, text_note_label, text_state_label,
};
use crate::svg::curve_basis_path;
use crate::text::measure;
use crate::theme::{Theme, ThemeVars};
use dagre_dgl_rs::graph::{EdgeLabel, Graph, GraphLabel, NodeLabel, Point};
use dagre_dgl_rs::layout::layout;

// ─── Public entry point ───────────────────────────────────────────────────────

pub fn render(diag: &StateDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    render_level(&diag.nodes, &diag.edges, &diag.direction, &vars, SVG_ID)
}

// ─── Inner layout for composite states ───────────────────────────────────────

/// Run dagre for the inner content of a composite state.
/// Returns (inner_width, inner_height, inner_svg_string).
fn run_inner_layout(
    nodes: &[Node],
    edges: &[Edge],
    direction: &str,
    vars: &ThemeVars,
    svg_id: &str,
    composite_label: &str,
    composite_dom_id: &str,
) -> (f64, f64, String) {
    let mut g = Graph::with_options(true, false, true);
    g.set_graph(GraphLabel {
        rankdir: Some(direction.to_string()),
        ranksep: Some(INNER_RANKSEP),
        nodesep: Some(INNER_NODESEP),
        marginx: Some(INNER_MARGINX),
        marginy: Some(INNER_MARGINY),
        ..Default::default()
    });

    for node in nodes {
        let (w, h) = node_size(node);
        let intersect = if node.shape == Shape::Choice {
            Some("diamond")
        } else {
            None
        };
        g.set_node(
            &node.id,
            NodeLabel {
                width: w,
                height: h,
                intersect_type: intersect,
                ..Default::default()
            },
        );
    }

    for edge in edges {
        if g.node_opt(&edge.start).is_none() || g.node_opt(&edge.end).is_none() {
            continue;
        }
        let is_note_edge = edge.classes.contains("note-edge");
        g.set_edge(
            &edge.start,
            &edge.end,
            EdgeLabel {
                minlen: Some(1),
                weight: if is_note_edge { Some(0.0) } else { Some(1.0) },
                width: Some(if edge.label.is_empty() { 0.0 } else { 1.0 }),
                height: Some(if edge.label.is_empty() { 0.0 } else { 24.0 }),
                labelpos: Some("c".to_string()),
                ..Default::default()
            },
            None,
        );
    }

    layout(&mut g);

    let graph_w = g.graph().width.unwrap_or(60.0);
    let graph_h = g.graph().height.unwrap_or(60.0);

    // Build inner SVG content (clusters, edges, nodes) without the <svg> wrapper
    let mut out = String::new();

    out.push_str("<g class=\"clusters\">");
    out.push_str(&composite_cluster(
        composite_dom_id,
        composite_label,
        graph_w,
        graph_h,
        0.0,
        vars,
    ));
    out.push_str("</g>");

    // Inner edge paths
    out.push_str("<g class=\"edgePaths\">");
    for (ei, edge) in edges.iter().enumerate() {
        let e = dagre_dgl_rs::graph::Edge::new(&edge.start, &edge.end);
        if let Some(lbl) = g.edge(&e) {
            if let Some(pts) = &lbl.points {
                if pts.len() >= 2 {
                    out.push_str(&render_edge(
                        edge,
                        pts,
                        svg_id,
                        1000 + ei,
                        &g,
                        nodes,
                        vars.state_transition_color,
                    ));
                }
            }
        }
    }
    out.push_str("</g>");

    // Inner edge labels
    out.push_str("<g class=\"edgeLabels\">");
    for edge in edges {
        out.push_str(edge_label_empty());
        let _ = edge;
    }
    out.push_str("</g>");

    // Inner nodes
    out.push_str("<g class=\"nodes\">");
    for node in nodes {
        if let Some(n) = g.node_opt(&node.id) {
            if let (Some(cx), Some(cy)) = (n.x, n.y) {
                out.push_str(&render_node(node, cx, cy, n.width, n.height, vars, svg_id));
            }
        }
    }
    out.push_str("</g>");

    // Return VISUAL size for outer dagre compound sizing.
    // -2*sp removes the dagre margins; -4 corrects the bottom margin difference
    // between dagre-dgl-rs (45.5) and dagre-d3-es compound border node behavior (41.5).
    (
        graph_w - 2.0 * CLUSTER_PADDING,
        graph_h - 2.0 * CLUSTER_PADDING - 4.0,
        out,
    )
}

// ─── Level renderer ───────────────────────────────────────────────────────────

fn render_level(
    nodes: &[Node],
    edges: &[Edge],
    direction: &str,
    vars: &ThemeVars,
    svg_id: &str,
) -> String {
    // Pre-compute inner layout for composite (group) nodes so we know their sizes
    // before running the outer dagre.  Mirrors how Mermaid recurses into composites.
    let mut composite_sizes: std::collections::HashMap<String, (f64, f64)> =
        std::collections::HashMap::new();
    let mut composite_inner_svgs: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    // Full (graph_w, graph_h) including dagre margins — used for inner group translate
    let mut composite_full_sizes: std::collections::HashMap<String, (f64, f64)> =
        std::collections::HashMap::new();

    for node in nodes {
        if node.is_group && node.shape == Shape::RoundedWithTitle {
            // Collect children of this composite
            let children: Vec<&Node> = nodes
                .iter()
                .filter(|n| n.parent_id.as_deref() == Some(&node.id))
                .collect();
            let child_edges: Vec<&Edge> = edges
                .iter()
                .filter(|e| {
                    children.iter().any(|n| n.id == e.start)
                        || children.iter().any(|n| n.id == e.end)
                })
                .collect();
            let child_nodes: Vec<Node> = children.iter().map(|n| (*n).clone()).collect();
            let child_edges: Vec<Edge> = child_edges.iter().map(|e| (*e).clone()).collect();
            if !child_nodes.is_empty() {
                let (visual_w, visual_h, inner_svg) = run_inner_layout(
                    &child_nodes,
                    &child_edges,
                    &node.dir,
                    vars,
                    svg_id,
                    &node.label,
                    &node.dom_id,
                );
                // Outer dagre sees visual size; inner group translate uses full size (+ 2*sp)
                let full_w = visual_w + 2.0 * CLUSTER_PADDING;
                let full_h = visual_h + 2.0 * CLUSTER_PADDING;
                composite_sizes.insert(node.id.clone(), (visual_w, visual_h));
                composite_full_sizes.insert(node.id.clone(), (full_w, full_h));
                composite_inner_svgs.insert(node.id.clone(), inner_svg);
            }
        }
    }

    // Build dagre graph with compound=true (matches dataFetcher.ts using graphlib compound:true)
    let mut g = Graph::with_options(true, false, true);
    g.set_graph(GraphLabel {
        rankdir: Some(direction.to_string()),
        ranksep: Some(RANKSEP),
        nodesep: Some(NODESEP),
        marginx: Some(MARGIN),
        marginy: Some(MARGIN),
        ..Default::default()
    });

    // Add all nodes to dagre, following dataFetcher.ts order:
    // groupData → noteData → nodeData
    for node in nodes {
        // Skip nodes that are children of a composite — they're in the inner layout
        if node
            .parent_id
            .as_ref()
            .map(|pid| {
                nodes
                    .iter()
                    .any(|n| n.id == *pid && n.is_group && n.shape == Shape::RoundedWithTitle)
            })
            .unwrap_or(false)
        {
            continue;
        }
        let (w, h) = if let Some(&(iw, ih)) = composite_sizes.get(&node.id) {
            (iw, ih) // composite uses inner layout size
        } else {
            node_size(node)
        };
        let intersect = if node.shape == Shape::Choice {
            Some("diamond")
        } else {
            None
        };
        g.set_node(
            &node.id,
            NodeLabel {
                width: w,
                height: h,
                intersect_type: intersect,
                ..Default::default()
            },
        );
    }

    // Set parent relationships (noteData.parentId = groupId)
    // Only for note compound children — composite children are handled in inner layout
    for node in nodes {
        if let Some(ref pid) = node.parent_id {
            // Only set parent if both the node AND the parent exist in the outer graph
            if g.node_opt(&node.id).is_some() && g.node_opt(pid).is_some() {
                g.set_parent(&node.id, Some(pid));
            }
        }
    }

    // Add edges between outer-level nodes only
    let mut edge_counter = 0usize;
    for edge in edges {
        // Skip edges between nodes that are not in the outer graph
        if g.node_opt(&edge.start).is_none() || g.node_opt(&edge.end).is_none() {
            continue;
        }
        let is_note_edge = edge.classes.contains("note-edge");
        g.set_edge(
            &edge.start,
            &edge.end,
            EdgeLabel {
                minlen: Some(1),
                weight: if is_note_edge { Some(0.0) } else { Some(1.0) },
                width: Some(if edge.label.is_empty() { 0.0 } else { 1.0 }),
                height: Some(if edge.label.is_empty() { 0.0 } else { 24.0 }),
                labelpos: Some("c".to_string()),
                ..Default::default()
            },
            None,
        );
        edge_counter += 1;
    }
    let _ = edge_counter;

    layout(&mut g);

    // Compute actual content bounding box from node positions (matches Mermaid's getBBox approach)
    let pad = MARGIN;
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for node in nodes {
        if let Some(n) = g.node_opt(&node.id) {
            if let (Some(cx), Some(cy)) = (n.x, n.y) {
                let hw = n.width / 2.0;
                let hh = n.height / 2.0;
                min_x = min_x.min(cx - hw);
                max_x = max_x.max(cx + hw);
                min_y = min_y.min(cy - hh);
                max_y = max_y.max(cy + hh);
            }
        }
    }
    // Include edge points and edge label extents in bounds.
    // Edge labels are centered at the path midpoint; their half-width must be included so
    // labels that overhang beyond node edges are fully contained in the viewBox.
    for edge in edges {
        let e = dagre_dgl_rs::graph::Edge::new(&edge.start, &edge.end);
        if let Some(lbl) = g.edge(&e) {
            if let Some(pts) = &lbl.points {
                for p in pts {
                    min_x = min_x.min(p.x);
                    max_x = max_x.max(p.x);
                    min_y = min_y.min(p.y);
                    max_y = max_y.max(p.y);
                }
                // If the edge has a visible label, include its horizontal extent.
                if !edge.label.is_empty() && pts.len() >= 2 {
                    let mid = midpoint(pts);
                    let (tw_raw, _) = measure(&edge.label, FONT_SIZE);
                    let tw = (tw_raw * LABEL_SCALE).max(20.0);
                    let half_tw = tw / 2.0;
                    min_x = min_x.min(mid.0 - half_tw);
                    max_x = max_x.max(mid.0 + half_tw);
                }
            }
        }
    }
    if min_x.is_infinite() {
        min_x = 0.0;
        min_y = 0.0;
        max_x = 100.0;
        max_y = 100.0;
    }
    let vb_x = min_x - pad;
    let vb_y = min_y - pad;
    let vb_w = (max_x - min_x) + 2.0 * pad;
    let vb_h = (max_y - min_y) + 2.0 * pad;

    // Build SVG
    let mut out = String::new();
    out.push_str(&super::templates::svg_root(svg_id, vb_x, vb_y, vb_w, vb_h));
    out.push_str("<g>");
    out.push_str(&markers(svg_id, vars.state_transition_color));
    out.push_str("</g>");
    out.push_str("<g class=\"root\">");

    // Clusters: noteGroup rects + composite (roundedWithTitle) groups
    out.push_str("<g class=\"clusters\">");
    for node in nodes {
        if node.shape == Shape::NoteGroup {
            if let Some(n) = g.node_opt(&node.id) {
                if let (Some(cx), Some(cy)) = (n.x, n.y) {
                    let w = n.width;
                    let h = n.height;
                    out.push_str(&note_cluster(
                        &node.dom_id,
                        cx - w / 2.0,
                        cy - h / 2.0,
                        w,
                        h,
                    ));
                }
            }
        }
    }
    out.push_str("</g>");

    // Helper: circle intersection — replace endpoint so it sits on the circle surface
    // instead of the bounding-box face (fixes gap on diagonal edges into start/end circles).
    let circle_intersect =
        |pts: &[Point], node_cx: f64, node_cy: f64, r: f64, is_target: bool| -> Vec<Point> {
            let mut v = pts.to_vec();
            if v.len() < 2 {
                return v;
            }
            let (inner, _outer) = if is_target {
                let n = v.len();
                (v[n - 2].clone(), v[n - 1].clone())
            } else {
                (v[1].clone(), v[0].clone())
            };
            // direction from inner waypoint toward node center
            let dx = node_cx - inner.x;
            let dy = node_cy - inner.y;
            let len = (dx * dx + dy * dy).sqrt();
            if len > 0.0 {
                let pt = Point {
                    x: node_cx - r * dx / len,
                    y: node_cy - r * dy / len,
                };
                if is_target {
                    *v.last_mut().unwrap() = pt;
                } else {
                    v[0] = pt;
                }
            }
            v
        };

    let node_shape_of =
        |id: &str| -> Option<Shape> { nodes.iter().find(|n| n.id == id).map(|n| n.shape.clone()) };
    let node_pos_of =
        |id: &str| -> Option<(f64, f64)> { g.node_opt(id).and_then(|n| n.x.zip(n.y)) };

    // Edge paths
    out.push_str("<g class=\"edgePaths\">");
    for (ei, edge) in edges.iter().enumerate() {
        let e = dagre_dgl_rs::graph::Edge::new(&edge.start, &edge.end);
        if let Some(lbl) = g.edge(&e) {
            if let Some(pts) = &lbl.points {
                if pts.len() >= 2 {
                    // Recompute endpoints for circular nodes
                    let mut pts2 = pts.clone();
                    if let (Some(shape), Some((cx, cy))) =
                        (node_shape_of(&edge.start), node_pos_of(&edge.start))
                    {
                        let r = match shape {
                            Shape::StateStart => START_R,
                            Shape::StateEnd => END_OUTER_R,
                            _ => 0.0,
                        };
                        if r > 0.0 {
                            pts2 = circle_intersect(&pts2, cx, cy, r, false);
                        }
                    }
                    if let (Some(shape), Some((cx, cy))) =
                        (node_shape_of(&edge.end), node_pos_of(&edge.end))
                    {
                        let r = match shape {
                            Shape::StateStart => START_R,
                            Shape::StateEnd => END_OUTER_R,
                            _ => 0.0,
                        };
                        if r > 0.0 {
                            pts2 = circle_intersect(&pts2, cx, cy, r, true);
                        }
                    }
                    out.push_str(&render_edge(
                        edge,
                        &pts2,
                        svg_id,
                        ei,
                        &g,
                        nodes,
                        vars.state_transition_color,
                    ));
                }
            }
        }
    }
    out.push_str("</g>");

    // Edge labels
    out.push_str("<g class=\"edgeLabels\">");
    for (ei, edge) in edges.iter().enumerate() {
        let e = dagre_dgl_rs::graph::Edge::new(&edge.start, &edge.end);
        if let Some(lbl) = g.edge(&e) {
            if let Some(pts) = &lbl.points {
                if !edge.label.is_empty() && pts.len() >= 2 {
                    let mid = midpoint(pts);
                    let (tw_raw, _) = measure(&edge.label, FONT_SIZE);
                    let tw = (tw_raw * LABEL_SCALE).max(20.0);
                    let edge_id = format!("{}-edge{}", svg_id, ei);
                    out.push_str(&text_edge_label(
                        mid.0,
                        mid.1,
                        -tw / 2.0,
                        -12.0,
                        tw,
                        &edge_id,
                        &edge.label,
                        vars.primary_text,
                        vars.edge_label_bg,
                    ));
                } else {
                    out.push_str(edge_label_empty());
                }
            }
        }
    }
    out.push_str("</g>");

    // Nodes
    out.push_str("<g class=\"nodes\">");
    for node in nodes {
        if node.shape == Shape::NoteGroup {
            continue; // rendered in clusters section
        }
        if let Some(n) = g.node_opt(&node.id) {
            if let (Some(cx), Some(cy)) = (n.x, n.y) {
                out.push_str(&render_node(node, cx, cy, n.width, n.height, vars, svg_id));
            }
        }
    }
    out.push_str("</g>");

    // Composite inner layouts — each rendered in a translated root group
    for node in nodes {
        if node.is_group && node.shape == Shape::RoundedWithTitle {
            if let (Some(inner_svg), Some(dagre_n)) =
                (composite_inner_svgs.get(&node.id), g.node_opt(&node.id))
            {
                if let (Some(cx), Some(cy)) = (dagre_n.x, dagre_n.y) {
                    // Use full size (visual + 2*sp) for the inner group translate
                    let (full_iw, full_ih) = composite_full_sizes
                        .get(&node.id)
                        .copied()
                        .unwrap_or((dagre_n.width, dagre_n.height));
                    let tx = cx - full_iw / 2.0;
                    let ty = cy - full_ih / 2.0;
                    out.push_str(&composite_inner_group(tx, ty, inner_svg));
                }
            }
        }
    }

    out.push_str("</g>"); // root
    out.push_str(&drop_shadow_filter(svg_id));
    out.push_str("</svg>");
    out
}

// ─── Node sizing (follows note.ts + shapes.js) ────────────────────────────────

fn node_size(node: &Node) -> (f64, f64) {
    match node.shape {
        Shape::StateStart => (START_R * 2.0, START_R * 2.0),
        Shape::StateEnd => (END_OUTER_R * 2.0, END_OUTER_R * 2.0),
        Shape::ForkJoin => (FORK_W, FORK_H),
        Shape::Choice => (CHOICE_R * 2.0, CHOICE_R * 2.0),
        Shape::NoteGroup => (0.0, 0.0), // dagre computes from children
        Shape::Note => {
            // note.ts: totalWidth = bbox.width + node.padding*2, node.padding=config.flowchart.padding=15
            let (tw, _) = measure(&node.label, FONT_SIZE);
            let w = (tw * LABEL_SCALE + 15.0 * 2.0).max(60.0);
            (w, NOTE_H)
        }
        // Composite groups: dagre computes size from children
        Shape::RoundedWithTitle if node.is_group => (0.0, 0.0),
        Shape::Rect | Shape::RectWithTitle | Shape::Divider | Shape::RoundedWithTitle => {
            let (tw, _) = measure(&node.label, FONT_SIZE);
            let w = (tw * LABEL_SCALE + NODE_PADDING * 2.0).max(40.0);
            (w, NODE_H)
        }
    }
}

// ─── Node SVG ────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn render_node(
    node: &Node,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    vars: &ThemeVars,
    _svg_id: &str,
) -> String {
    let dom_id = &node.dom_id;
    match node.shape {
        // stateStart.ts: rc.circle(0,0,14, solidStateFill(lineColor)) → fill=lineColor, r=7
        Shape::StateStart => node_state_start(dom_id, cx, cy, vars),
        // stateEnd.ts: outer rc.circle(0,0,14, stroke=lineColor,sw=2) fill=primary_color
        //              inner rc.circle(0,0,5, fill=stateBorder/primary_border) r=2.5
        Shape::StateEnd => node_state_end(dom_id, cx, cy, vars),
        Shape::ForkJoin => node_fork_join(dom_id, cx, cy, w, h, vars.line_color),
        Shape::Choice => node_choice(dom_id, cx, cy, vars),
        Shape::Note => {
            let label_html = text_note_label(&node.label, vars.note_text_color);
            node_note(dom_id, cx, cy, w, h, &node.label, &label_html, vars)
        }
        Shape::Rect | Shape::RectWithTitle | Shape::Divider => {
            let label_html = text_state_label(&node.label, vars.primary_text);
            node_rect(dom_id, cx, cy, w, h, vars, &label_html)
        }
        Shape::RoundedWithTitle => {
            // Composite state — simplified rect with title
            let hh = h / 2.0;
            let label_html = text_composite_label(&node.label, hh);
            node_rect(dom_id, cx, cy, w, h, vars, &label_html)
        }
        Shape::NoteGroup => String::new(), // rendered in clusters
    }
}

// ─── Edge SVG ─────────────────────────────────────────────────────────────────

fn render_edge(
    edge: &Edge,
    pts: &[Point],
    svg_id: &str,
    _idx: usize,
    _g: &Graph,
    _nodes: &[Node],
    line_color: &str,
) -> String {
    let pts_f: Vec<(f64, f64)> = pts.iter().map(|p| (p.x, p.y)).collect();
    let path_d = curve_basis_path(&pts_f);
    let edge_id = format!("{}-{}-{}", svg_id, edge.start, edge.end);
    let is_note = edge.classes.contains("note-edge");
    let dasharray = if is_note { "5" } else { "0" };
    let marker = if edge.arrowhead == "none" {
        String::new()
    } else if is_note {
        format!("url(#{svg_id}_stateDiagram-barbEnd)")
    } else {
        format!("url(#{svg_id}-dependencyEnd)")
    };
    edge_path(
        &path_d,
        &edge_id,
        &edge.classes,
        line_color,
        dasharray,
        &marker,
    )
}

// ─── Utilities ────────────────────────────────────────────────────────────────

fn midpoint(pts: &[Point]) -> (f64, f64) {
    let n = pts.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    let mid = n / 2;
    if n % 2 == 1 {
        (pts[mid].x, pts[mid].y)
    } else {
        (
            (pts[mid - 1].x + pts[mid].x) / 2.0,
            (pts[mid - 1].y + pts[mid].y) / 2.0,
        )
    }
}

// ─── Test snapshot ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn snapshot_default_theme() {
        let input = "stateDiagram-v2\n    Still --> Moving\n    Moving --> Still\n    Moving --> Crash\n    Crash --> [*]";
        let diag = super::super::parser::parse(input);
        let svg = render(&diag, Theme::Default);
        insta::assert_snapshot!(svg);
    }
}
