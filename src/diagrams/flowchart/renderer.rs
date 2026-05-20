// Mermaid flowchart renderer — faithful port of flowRenderer-v3-unified.ts
// SVG structure mirrors the reference exactly:
//   <svg> <style/> <g>{markers}</g> <g class="root"> clusters / edgePaths / edgeLabels / nodes </g> </svg>
//
// Node geometry follows Mermaid v11 padding rules (calibrated from reference SVG).
//
// Cluster rendering matches clusters.js `rect()` shape:
//   - Clusters WITHOUT external connections → clusterNode=true → rendered as nested
//     <g class="root" transform="translate(ox,oy)"> in the parent's <g class="nodes">
//   - Clusters WITH external connections → rendered as <g class="cluster " id="{svgId}-{sgId}"
//     data-look="classic"> in the parent's <g class="clusters">
//
// Reference SVG cluster element:
//   <g class="cluster " id="{svgId}-{sgId}" data-look="classic">
//     <rect style="" x="{x}" y="{y}" width="{w}" height="{h}">
//     <g class="cluster-label " transform="translate({cx-lw/2},{y_top})">
//       <foreignObject width="{lw}" height="24">
//         <div ...><span class="nodeLabel "><p>{label}</p></span></div>
//       </foreignObject>
//     </g>
//   </g>

use super::constants::*;
use super::parser::{EdgeStyle, FlowchartDiagram, NodeShape, NodeStyle, Subgraph};
use super::templates::{self, build_css, esc, fmt};
use crate::icons::parse_fa_label;
use crate::svg::SvgWriter;
use crate::text::measure;
use crate::theme::{Theme, ThemeVars};
use dagre_dgl_rs::graph::{EdgeLabel, Graph, GraphLabel, NodeLabel, Point};
use dagre_dgl_rs::layout::layout;
use std::collections::{HashMap, HashSet};

// --- Public entry point -------------------------------------------------------

/// Render a flowchart diagram to SVG.
/// `use_foreign_object`: when true, node/edge labels use <foreignObject> HTML (matching Mermaid);
///                       when false, plain SVG <text> elements are used.
pub fn render(diag: &FlowchartDiagram, theme: Theme, use_foreign_object: bool) -> String {
    let vars = theme.resolve();

    let subgraph_ids: HashSet<String> = diag.subgraphs.iter().map(|sg| sg.id.clone()).collect();

    // Build a map from subgraph_id -> parent subgraph_id (if nested)
    let mut sg_parent: HashMap<String, String> = HashMap::new();
    for sg in &diag.subgraphs {
        for member in &sg.members {
            if subgraph_ids.contains(member) {
                sg_parent.insert(member.clone(), sg.id.clone());
            }
        }
    }

    // Determine which subgraphs have "external connections" (edges that cross their boundary)
    // External = at least one edge has one endpoint inside the cluster and one outside
    // This mirrors Mermaid's `adjustClustersAndEdges` externalConnections logic.
    let sg_descendants: HashMap<String, HashSet<String>> = diag
        .subgraphs
        .iter()
        .map(|sg| {
            (
                sg.id.clone(),
                collect_descendants(&sg.id, &subgraph_ids, diag),
            )
        })
        .collect();

    // Determine which subgraphs have "external connections".
    // Mirrors Mermaid's adjustClustersAndEdges logic:
    //   for each cluster, check if any ORIGINAL edge has exactly one endpoint
    //   that is a DESCENDANT of the cluster (the cluster node itself is NOT a descendant).
    //   Edges from/to the cluster node id itself do NOT count because the cluster id
    //   is not in its own descendants set — this is intentional in Mermaid.
    let mut sg_external: HashSet<String> = HashSet::new();
    for edge in &diag.edges {
        for sg in &diag.subgraphs {
            let descs = sg_descendants.get(&sg.id).unwrap();
            // isDescendant checks if the id is in the descendants set
            // The cluster's own id is NEVER in its own descendants → cluster-level edges won't trigger
            let v_in = descs.contains(&edge.from);
            let w_in = descs.contains(&edge.to);
            if v_in ^ w_in {
                sg_external.insert(sg.id.clone());
            }
        }
    }

    // Determine the outer rankdir
    let rankdir = match diag.direction.as_str() {
        "LR" => "LR",
        "RL" => "RL",
        "BT" => "BT",
        _ => "TB",
    };

    // --- Recursive layout approach (mirrors Mermaid's recursiveRender) ---
    //
    // Mermaid distinguishes:
    //   "internal" (clusterNode) subgraphs: fully enclosed, no external connections
    //       → laid out recursively with a DIFFERENT inner direction (flip outer or use explicit)
    //       → treated as opaque nodes in the outer layout
    //   "external" subgraphs: have at least one edge crossing their boundary
    //       → their leaf nodes participate directly in the outer layout
    //       → the cluster rect is just a visual overlay drawn around those nodes
    //
    // For TOP-LEVEL subgraphs only (nested subgraphs are handled recursively).

    // Step 1: Identify top-level subgraphs (no parent sg)
    let top_sg_ids: Vec<String> = diag
        .subgraphs
        .iter()
        .filter(|sg| !sg_parent.contains_key(&sg.id))
        .map(|sg| sg.id.clone())
        .collect();

    // Step 2: For INTERNAL top-level subgraphs, compute recursive layouts (bottom-up)
    // Outer ranksep = 50 (standard dagre default for flowcharts)
    let outer_ranksep = OUTER_RANKSEP;
    // sg_layouts maps EVERY internal subgraph id → its SgLayout (top-level and nested).
    // This allows composite_sg_layout to recurse into nested subgraphs correctly.
    let mut sg_layouts: HashMap<String, SgLayout> = HashMap::new();
    for sg_id in &top_sg_ids {
        if !sg_external.contains(sg_id) {
            // Internal → compute recursive layout (also collects all nested layouts)
            let layout_result = compute_sg_layout(
                sg_id,
                diag,
                &subgraph_ids,
                rankdir,
                outer_ranksep,
                &mut sg_layouts,
            );
            sg_layouts.insert(sg_id.clone(), layout_result);
        }
    }

    // Step 3: Build outer dagre graph — compound graph so that external subgraphs
    // get proper bounding boxes from dagre's nesting_graph / border nodes.
    //   - Orphan leaf nodes (no subgraph parent)
    //   - Leaf nodes inside EXTERNAL top-level subgraphs (as children of their compound node)
    //   - External top-level subgraphs as compound nodes (set_parent for their leaves)
    //   - Internal top-level subgraphs as opaque pre-sized nodes (no set_parent)
    let mut g = Graph::with_options(true, true, true); // compound outer layout
    g.set_graph(GraphLabel {
        rankdir: Some(rankdir.to_string()),
        nodesep: Some(NODE_SEP),
        ranksep: Some(outer_ranksep),
        marginx: Some(GRAPH_MARGIN),
        marginy: Some(GRAPH_MARGIN),
        ..Default::default()
    });

    // Add external top-level subgraphs as compound parent nodes (zero initial size)
    for sg_id in &top_sg_ids {
        if sg_external.contains(sg_id) {
            g.set_node(
                sg_id,
                NodeLabel {
                    width: 0.0,
                    height: 0.0,
                    ..Default::default()
                },
            );
        }
    }

    // Add all nodes that belong at the outer level:
    // - Orphan leaf nodes (no subgraph parent) → added directly
    // - Leaf descendants of EXTERNAL top-level subgraphs → added with set_parent to their top-level external ancestor
    for (id, node) in &diag.nodes {
        if subgraph_ids.contains(id) {
            continue;
        }

        // Find top-level ancestor subgraph (if any)
        let top_level_ancestor = if let Some(direct_parent) = diag.node_subgraph.get(id) {
            let mut cur = direct_parent.clone();
            loop {
                match sg_parent.get(&cur) {
                    None => break,
                    Some(p) => cur = p.clone(),
                }
            }
            Some(cur)
        } else {
            None
        };

        let in_outer = match &top_level_ancestor {
            Some(ancestor) => sg_external.contains(ancestor),
            None => true,
        };

        if in_outer {
            let (w, h) = node_size(&node.label, &node.shape);
            let intersect_type = shape_intersect_type(&node.shape);
            g.set_node(
                id,
                NodeLabel {
                    width: w,
                    height: h,
                    intersect_type,
                    ..Default::default()
                },
            );
            // If this node is inside an external compound, set its parent
            if let Some(ancestor) = &top_level_ancestor {
                if sg_external.contains(ancestor) {
                    g.set_parent(id, Some(ancestor));
                }
            }
        }
    }

    // Add INTERNAL top-level subgraphs as opaque nodes (not compound in outer layout).
    // sg_layout.width/height are the compound cluster dimensions (the rect w/h from
    // removeBorderNodes, e.g. 305.6 × 124 for 'one'). These are used directly as the
    // opaque node dimensions in the outer dagre layout, matching Mermaid's approach where
    // updateNodeBounds(node, elem) sets node.width/height from the getBBox() of the
    // recursively-rendered sub-graph SVG element, which returns the compound cluster rect size.
    // The "full size" for the SVG <g class="root"> translate = (width + 16, height + 16).
    let sg_layout_margin = SG_LAYOUT_MARGIN;
    for sg_id in &top_sg_ids {
        if !sg_external.contains(sg_id) {
            if let Some(sg_layout) = sg_layouts.get(sg_id) {
                g.set_node(
                    sg_id,
                    NodeLabel {
                        width: sg_layout.width.max(1.0),
                        height: sg_layout.height.max(1.0),
                        ..Default::default()
                    },
                );
            }
        }
    }

    // Step 4: Add edges to the outer layout
    // For each original edge, map its endpoints to outer-layout nodes:
    //   - If endpoint is a leaf in an EXTERNAL subgraph → use the leaf node ID
    //   - If endpoint is inside an INTERNAL subgraph → use the top-level subgraph ID (opaque node)
    //   - If endpoint is an orphan leaf → use the leaf node ID
    //   - If endpoint is a subgraph ID → resolve to top-level representative
    for edge in &diag.edges {
        let from_outer = map_to_outer_node(
            &edge.from,
            &top_sg_ids,
            &subgraph_ids,
            &sg_external,
            &sg_parent,
            &sg_descendants,
            diag,
        );
        let to_outer = map_to_outer_node(
            &edge.to,
            &top_sg_ids,
            &subgraph_ids,
            &sg_external,
            &sg_parent,
            &sg_descendants,
            diag,
        );
        if let (Some(from_n), Some(to_n)) = (from_outer, to_outer) {
            if from_n != to_n && g.has_node(&from_n) && g.has_node(&to_n) {
                // Pass edge label dimensions to dagre so it reserves space for the label
                // proxy node (see inject_edge_label_proxies in dagre layout).
                let (lbl_w, lbl_h) = if let Some(lbl) = edge.label.as_deref() {
                    if !lbl.is_empty() {
                        let (tw, _) = measure(lbl, FONT_SIZE);
                        (tw, LABEL_FO_HEIGHT)
                    } else {
                        (0.0, 0.0)
                    }
                } else {
                    (0.0, 0.0)
                };
                g.set_edge(
                    &from_n,
                    &to_n,
                    EdgeLabel {
                        minlen: Some(1),
                        weight: Some(1.0),
                        width: Some(lbl_w),
                        height: Some(lbl_h),
                        ..Default::default()
                    },
                    None,
                );
            }
        }
    }

    layout(&mut g);

    // Step 5: Composite all positions into global coordinates
    let mut node_global: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();
    let mut sg_global_bounds_map: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();
    // Collect inner-subgraph edges in global coordinates for addition to g_full.
    let mut edge_global: Vec<(String, String, Vec<Point>)> = Vec::new();

    // Collect the full sg_layout dimensions for every internal subgraph at all nesting levels.
    // sg_layout.width/height = compound cluster rect dimensions (e.g. 305.6 × 124).
    // Full dimensions for SVG rendering = (width + 16, height + 16) = (321.6 × 140).
    // child_sg_full_sizes stores (full_w, full_h) = (width+16, height+16) for each child.
    let mut all_sg_full_sizes: HashMap<String, (f64, f64)> = HashMap::new();
    fn collect_full_sizes(
        sg_id: &str,
        sg_layout: &SgLayout,
        all_sg_layouts: &HashMap<String, SgLayout>,
        margin2: f64,
        out: &mut HashMap<String, (f64, f64)>,
    ) {
        // sg_layout.width/height are compound cluster rect dims; full = rect + 16.
        out.insert(
            sg_id.to_string(),
            (sg_layout.width + margin2, sg_layout.height + margin2),
        );
        // child_sg_full_sizes already contains full dims (rect+16) for each child.
        for (child_id, &(full_w, full_h)) in &sg_layout.child_sg_full_sizes {
            out.insert(child_id.clone(), (full_w, full_h));
            if let Some(child_layout) = all_sg_layouts.get(child_id) {
                collect_full_sizes(child_id, child_layout, all_sg_layouts, margin2, out);
            }
        }
    }
    for (sg_id, sg_layout) in &sg_layouts {
        collect_full_sizes(
            sg_id,
            sg_layout,
            &sg_layouts,
            sg_layout_margin,
            &mut all_sg_full_sizes,
        );
    }

    // Outer layout nodes → global positions
    for v in g.nodes() {
        if let Some(n) = g.node_opt(&v) {
            if let (Some(cx), Some(cy)) = (n.x, n.y) {
                if sg_external.contains(&v) {
                    // External subgraph: dagre compound layout computed x/y/width/height
                    // via border nodes (nesting_graph + add_border_segments).
                    sg_global_bounds_map.insert(v.clone(), (cx, cy, n.width, n.height));
                } else if subgraph_ids.contains(&v) {
                    // Internal subgraph opaque node. The dagre node was sized with the compound
                    // cluster rect dimensions (sg_layout.width/height). The full size for the
                    // SVG <g class="root"> translate is (rect_w + 16, rect_h + 16).
                    let (full_w, full_h) = all_sg_full_sizes
                        .get(&v)
                        .copied()
                        .unwrap_or((n.width + sg_layout_margin, n.height + sg_layout_margin));
                    sg_global_bounds_map.insert(v.clone(), (cx, cy, full_w, full_h));
                    // Recursively composite its children using full dimensions for origin.
                    if let Some(sg_layout) = sg_layouts.get(&v) {
                        let sg_ox = cx - full_w / 2.0;
                        let sg_oy = cy - full_h / 2.0;
                        composite_sg_layout(
                            &v,
                            sg_layout,
                            &sg_layouts,
                            &subgraph_ids,
                            diag,
                            sg_ox,
                            sg_oy,
                            &mut node_global,
                            &mut sg_global_bounds_map,
                            &all_sg_full_sizes,
                            sg_layout_margin,
                            &mut edge_global,
                        );
                    }
                } else {
                    // Outer leaf node (may be inside external compound)
                    node_global.insert(v.clone(), (cx, cy, n.width, n.height));
                }
            }
        }
    }

    // Build the sg_bounds map for SVG rendering (format: center x,y and w,h)
    let sg_bounds: HashMap<String, (f64, f64, f64, f64)> = sg_global_bounds_map.clone();

    // Build the unified graph for SVG rendering (edge routing + node positions)
    let mut g_full = Graph::with_options(true, true, false);

    // Add all nodes with global positions
    for (id, &(cx, cy, w, h)) in &node_global {
        let mut nl = NodeLabel {
            width: w,
            height: h,
            ..Default::default()
        };
        nl.x = Some(cx);
        nl.y = Some(cy);
        g_full.set_node(id, nl);
    }
    for (sg_id, &(cx, cy, w, h)) in &sg_global_bounds_map {
        if !g_full.has_node(sg_id) {
            let mut nl = NodeLabel {
                width: w,
                height: h,
                ..Default::default()
            };
            nl.x = Some(cx);
            nl.y = Some(cy);
            g_full.set_node(sg_id, nl);
        }
    }

    // Add outer-level edges (with routing points) to g_full
    for e in g.edges() {
        if let Some(lbl) = g.edge(&e) {
            g_full.set_edge_obj(&e, lbl.clone());
        }
    }
    // Add inner-subgraph edges (composited to global coordinates) to g_full.
    for (from, to, pts) in edge_global {
        let _e = dagre_dgl_rs::graph::Edge::new(&from, &to);
        if !g_full.has_edge(&from, &to) {
            g_full.set_edge(
                &from,
                &to,
                EdgeLabel {
                    points: Some(pts),
                    ..Default::default()
                },
                None,
            );
        } else if let Some(lbl) = g_full.edge_vw_mut(&from, &to) {
            lbl.points = Some(pts);
        }
    }

    // Compute total bounding box
    let (graph_w, graph_h) = {
        let margin_x = GRAPH_MARGIN;
        let margin_y = GRAPH_MARGIN;
        let mut max_x = 0.0_f64;
        let mut max_y = 0.0_f64;
        for &(cx, cy, w, h) in node_global.values() {
            max_x = max_x.max(cx + w / 2.0);
            max_y = max_y.max(cy + h / 2.0);
        }
        for (sg_id, &(cx, cy, w, h)) in &sg_global_bounds_map {
            // w/h are always full dimensions (rect+16) for internal subgraphs,
            // and compound layout bounds for external subgraphs.
            // Use rect size (full - 16) for the bounding box of internal subgraphs
            // since the outer layout sized them by their rect dims.
            let (bw, bh) = if sg_external.contains(sg_id) {
                (w, h)
            } else {
                (
                    (w - sg_layout_margin).max(1.0),
                    (h - sg_layout_margin).max(1.0),
                )
            };
            max_x = max_x.max(cx + bw / 2.0);
            max_y = max_y.max(cy + bh / 2.0);
        }
        (max_x + margin_x, max_y + margin_y)
    };

    let g = g_full; // use g_full for SVG rendering

    let svg_id = "mermaid-svg";
    let css = build_css(svg_id, &vars);

    let mut w = SvgWriter::with_capacity(32_768);

    w.raw(&templates::svg_root(svg_id, graph_w, graph_w, graph_h));

    w.raw("<style>").raw(&css).raw("</style>");

    // Markers in a bare <g> before <g class="root">
    w.raw("<g>")
        .raw(&templates::all_markers(svg_id))
        .raw("</g>");

    w.raw(r#"<g class="root">"#);

    // Render the top-level root group (recurse into the compound structure)
    // Top-level subgraphs = those with no parent
    let top_level_sgs: Vec<&Subgraph> = diag
        .subgraphs
        .iter()
        .filter(|sg| !sg_parent.contains_key(&sg.id))
        .collect();

    // render_root_group appends into a &mut String; give it the writer's buffer.
    let out_buf = w.finish();
    let mut out = out_buf;

    render_root_group(
        &mut out,
        svg_id,
        diag,
        &g,
        &subgraph_ids,
        &sg_parent,
        &diag.node_subgraph,
        &sg_bounds,
        &sg_external,
        &sg_descendants,
        &top_level_sgs,
        None, // self_cluster: None at top level
        None, // no parent offset
        use_foreign_object,
        &vars,
    );

    let mut w2 = SvgWriter::with_capacity(out.len() + 4096);
    w2.raw(&out);
    w2.raw("</g>"); // root
    w2.raw(&templates::drop_shadow_filter(svg_id));
    w2.raw(&templates::drop_shadow_filter_small(svg_id));
    w2.raw("</svg>");
    w2.finish()
}

/// Render the content of one "root group" — clusters / edgePaths / edgeLabels / nodes.
///
/// Parameters:
/// - `context_sgs`: subgraphs that are direct children at this rendering level
/// - `self_cluster`: if Some(sg), this root group IS the compound for `sg`, so render sg's
///   OWN background rect as the first item in `<g class="clusters">`
/// - `parent_offset`: absolute coordinate origin of this group (subtract to get local coords)
/// - `node_subgraph`: leaf node id → direct parent subgraph id (from diag.node_subgraph)
#[allow(clippy::too_many_arguments)]
#[allow(clippy::only_used_in_recursion)]
fn render_root_group(
    out: &mut String,
    svg_id: &str,
    diag: &FlowchartDiagram,
    g: &Graph,
    subgraph_ids: &HashSet<String>,
    sg_parent: &HashMap<String, String>,
    node_subgraph: &HashMap<String, String>,
    sg_bounds: &HashMap<String, (f64, f64, f64, f64)>,
    sg_external: &HashSet<String>,
    sg_descendants: &HashMap<String, HashSet<String>>,
    context_sgs: &[&Subgraph],
    self_cluster: Option<&Subgraph>,
    parent_offset: Option<(f64, f64)>,
    use_foreign_object: bool,
    vars: &ThemeVars,
) {
    let (off_x, off_y) = parent_offset.unwrap_or((0.0, 0.0));
    let ff = vars.font_family;

    // IDs of subgraphs at THIS level
    let external_here: Vec<&Subgraph> = context_sgs
        .iter()
        .copied()
        .filter(|sg| sg_external.contains(&sg.id))
        .collect();
    let internal_here: Vec<&Subgraph> = context_sgs
        .iter()
        .copied()
        .filter(|sg| !sg_external.contains(&sg.id))
        .collect();

    // --- <g class="clusters"> ---
    out.push_str(r#"<g class="clusters">"#);

    // If this root group IS the compound for self_cluster, render the compound's own rect.
    // The rect in local coords: x=MARGIN_X, y=MARGIN_Y, w=comp_w-2*MX, h=comp_h-2*MY
    if let Some(sc) = self_cluster {
        if let Some(&(_, _, comp_w, comp_h)) = sg_bounds.get(&sc.id) {
            let rect_x = GRAPH_MARGIN;
            let rect_y = GRAPH_MARGIN;
            let rect_w = comp_w - SG_LAYOUT_MARGIN;
            let rect_h = comp_h - SG_LAYOUT_MARGIN;
            let local_cx = comp_w / 2.0;
            out.push_str(&render_cluster_rect(
                svg_id,
                sc,
                rect_x,
                rect_y,
                local_cx,
                rect_w,
                rect_h,
                use_foreign_object,
            ));
        }
    }

    // External child clusters at this level
    for sg in &external_here {
        if let Some(&(abs_cx, abs_cy, w, h)) = sg_bounds.get(&sg.id) {
            let local_cx = abs_cx - off_x;
            let local_cy = abs_cy - off_y;
            let lx = local_cx - w / 2.0;
            let ly = local_cy - h / 2.0;
            out.push_str(&render_cluster_rect(
                svg_id,
                sg,
                lx,
                ly,
                local_cx,
                w,
                h,
                use_foreign_object,
            ));
        }
    }
    out.push_str("</g>");

    // --- <g class="edgePaths"> ---
    out.push_str(r#"<g class="edgePaths">"#);
    for (i, edge) in diag.edges.iter().enumerate() {
        // Resolve each endpoint to the dagre node key for THIS render level.
        // Returns None if the endpoint is not at this level (edge should be skipped).
        let from_key = resolve_edge_endpoint(
            &edge.from,
            subgraph_ids,
            diag,
            g,
            context_sgs,
            node_subgraph,
            sg_parent,
            sg_external,
            self_cluster,
        );
        let to_key = resolve_edge_endpoint(
            &edge.to,
            subgraph_ids,
            diag,
            g,
            context_sgs,
            node_subgraph,
            sg_parent,
            sg_external,
            self_cluster,
        );
        let (from_key, to_key) = match (from_key, to_key) {
            (Some(f), Some(t)) if f != t => (f, t),
            _ => continue,
        };

        let e = dagre_dgl_rs::graph::Edge::new(&from_key, &to_key);
        if let Some(lbl) = g.edge(&e) {
            let pts = lbl.points.clone().unwrap_or_default();
            if pts.len() >= 2 {
                let mut local_pts: Vec<Point> = pts
                    .iter()
                    .map(|p| Point {
                        x: p.x - off_x,
                        y: p.y - off_y,
                    })
                    .collect();
                // If the original endpoint is an external subgraph, strip all points
                // that lie inside the cluster and replace with the boundary exit point.
                if subgraph_ids.contains(&edge.from) && sg_external.contains(&edge.from) {
                    if let Some(&(cx, cy, w, h)) = sg_bounds.get(&edge.from) {
                        let lcx = cx - off_x;
                        let lcy = cy - off_y;
                        local_pts = clip_path_from_cluster(&local_pts, lcx, lcy, w, h, true);
                    }
                }
                if subgraph_ids.contains(&edge.to) && sg_external.contains(&edge.to) {
                    if let Some(&(cx, cy, w, h)) = sg_bounds.get(&edge.to) {
                        let lcx = cx - off_x;
                        let lcy = cy - off_y;
                        local_pts = clip_path_from_cluster(&local_pts, lcx, lcy, w, h, false);
                    }
                }
                let edge_id = format!("{}-L_{}_{}_{}", svg_id, edge.from, edge.to, i);
                let is_thick = matches!(edge.style, EdgeStyle::ThickArrow);
                let is_dashed = matches!(edge.style, EdgeStyle::DotArrow | EdgeStyle::DotLine);
                let has_arrow = !matches!(edge.style, EdgeStyle::Line | EdgeStyle::DotLine);
                let is_cross = matches!(edge.style, EdgeStyle::CrossArrow);
                let is_open = matches!(edge.style, EdgeStyle::OpenArrow);
                // Trim end so arrowhead tip lands on node boundary, not inside the fill.
                let end_trim = if has_arrow && !is_cross && !is_open {
                    POINT_END_TRIM
                } else {
                    0.0
                };
                let trimmed = trim_path_end(&local_pts, end_trim);
                let path_d = edge_path(&trimmed);
                let classes = if is_thick {
                    " edge-thickness-thick edge-pattern-solid flowchart-link"
                } else if is_dashed {
                    " edge-thickness-normal edge-pattern-dashed flowchart-link"
                } else {
                    " edge-thickness-normal edge-pattern-solid edge-thickness-normal edge-pattern-solid flowchart-link"
                };
                let marker_end = if is_cross {
                    templates::marker_end_cross(svg_id)
                } else if is_open {
                    templates::marker_end_circle(svg_id)
                } else if has_arrow {
                    templates::marker_end_point(svg_id)
                } else {
                    String::new()
                };
                out.push_str(&templates::edge_path(
                    &path_d,
                    &edge_id,
                    classes,
                    &marker_end,
                ));
            }
        }
    }
    out.push_str("</g>");

    // --- <g class="edgeLabels"> ---
    out.push_str(r#"<g class="edgeLabels">"#);
    for (i, edge) in diag.edges.iter().enumerate() {
        let from_key = resolve_edge_endpoint(
            &edge.from,
            subgraph_ids,
            diag,
            g,
            context_sgs,
            node_subgraph,
            sg_parent,
            sg_external,
            self_cluster,
        );
        let to_key = resolve_edge_endpoint(
            &edge.to,
            subgraph_ids,
            diag,
            g,
            context_sgs,
            node_subgraph,
            sg_parent,
            sg_external,
            self_cluster,
        );
        let (from_key, to_key) = match (from_key, to_key) {
            (Some(f), Some(t)) if f != t => (f, t),
            _ => continue,
        };

        let e = dagre_dgl_rs::graph::Edge::new(&from_key, &to_key);
        if let Some(lbl_data) = g.edge(&e) {
            let pts = lbl_data.points.clone().unwrap_or_default();
            let edge_id = format!("{}-L_{}_{}_{}", svg_id, edge.from, edge.to, i);
            match edge.label.as_deref() {
                Some(lbl_text) if !lbl_text.is_empty() => {
                    let mid_abs = midpoint(&pts);
                    let mx = mid_abs.0 - off_x;
                    let my = mid_abs.1 - off_y;
                    let (fo_w_raw, _) = measure(lbl_text, FONT_SIZE);
                    let fo_w = fo_w_raw * TEXT_SCALE;
                    if use_foreign_object {
                        out.push_str(&templates::edge_label_fo(
                            &fmt(mx),
                            &fmt(my),
                            &edge_id,
                            &fmt(-fo_w / 2.0),
                            &fmt(fo_w),
                            LABEL_FO_HEIGHT,
                            LABEL_Y_OFFSET,
                            &esc(lbl_text),
                        ));
                    } else {
                        out.push_str(&templates::edge_label_text(
                            &fmt(mx),
                            &fmt(my),
                            &fmt(-fo_w / 2.0),
                            &fmt(fo_w),
                            LABEL_FO_HEIGHT,
                            LABEL_Y_OFFSET,
                            TEXT_LABEL_Y,
                            ff,
                            FONT_SIZE,
                            &esc(lbl_text),
                        ));
                    }
                }
                _ => {
                    out.push_str(&templates::edge_label_empty(&edge_id));
                }
            }
        }
    }
    out.push_str("</g>");

    // --- <g class="nodes"> ---
    out.push_str(r#"<g class="nodes">"#);

    // Internal subgraphs → compound <g class="root" transform="...">
    for sg in &internal_here {
        if let Some(&(abs_cx, abs_cy, comp_w, comp_h)) = sg_bounds.get(&sg.id) {
            let abs_ox = abs_cx - comp_w / 2.0;
            let abs_oy = abs_cy - comp_h / 2.0;
            let local_ox = abs_ox - off_x;
            let local_oy = abs_oy - off_y;

            out.push_str(&templates::subgraph_root_group(
                &fmt(local_ox),
                &fmt(local_oy),
            ));

            let child_sgs: Vec<&Subgraph> = diag
                .subgraphs
                .iter()
                .filter(|child| {
                    sg_parent
                        .get(&child.id)
                        .map(|p| p == &sg.id)
                        .unwrap_or(false)
                })
                .collect();

            render_root_group(
                out,
                svg_id,
                diag,
                g,
                subgraph_ids,
                sg_parent,
                node_subgraph,
                sg_bounds,
                sg_external,
                sg_descendants,
                &child_sgs,
                Some(sg), // self_cluster = render sg's own rect in its clusters section
                Some((abs_ox, abs_oy)),
                use_foreign_object,
                vars,
            );

            out.push_str("</g>"); // root
        }
    }

    // Leaf nodes at this level
    let mut node_idx = 0usize;
    for (id, flow_node) in &diag.nodes {
        if subgraph_ids.contains(id) {
            node_idx += 1;
            continue;
        }

        if !node_at_level(
            id,
            context_sgs,
            node_subgraph,
            sg_parent,
            sg_external,
            self_cluster,
        ) {
            node_idx += 1;
            continue;
        }

        if let Some(n) = g.node_opt(id) {
            let lcx = n.x.unwrap_or(0.0) - off_x;
            let lcy = n.y.unwrap_or(0.0) - off_y;
            let w = n.width;
            let h = n.height;
            let node_style = diag.node_styles.get(id);
            let node_dom_id = format!("{}-flowchart-{}-{}", svg_id, id, node_idx);
            out.push_str(&render_node(
                flow_node,
                lcx,
                lcy,
                w,
                h,
                vars,
                node_style,
                &node_dom_id,
                use_foreign_object,
            ));
        }
        node_idx += 1;
    }

    out.push_str("</g>"); // nodes
}

/// Render a cluster background rect + label.
/// This is the `<g class="cluster ">` element rendered into `<g class="clusters">`.
/// Coords are LOCAL to the current group's origin.
#[allow(clippy::too_many_arguments)]
fn render_cluster_rect(
    svg_id: &str,
    sg: &Subgraph,
    x: f64,        // local x of top-left
    y: f64,        // local y of top-left
    local_cx: f64, // local center x (for label centering)
    w: f64,
    h: f64,
    use_foreign_object: bool,
) -> String {
    let label = sg.label.as_deref().unwrap_or(&sg.id);
    let (lw_raw, _) = measure(label, FONT_SIZE);
    let lw = lw_raw * TEXT_SCALE;

    let label_html = if use_foreign_object {
        templates::cluster_label_fo(
            &fmt(local_cx - lw / 2.0),
            &fmt(y),
            &fmt(lw),
            LABEL_FO_HEIGHT,
            &esc(label),
        )
    } else {
        templates::cluster_label_text(
            &fmt(local_cx),
            &fmt(y + CLUSTER_LABEL_TEXT_DY),
            "Arial, sans-serif",
            FONT_SIZE,
            &esc(label),
        )
    };

    templates::cluster_group(
        svg_id,
        &sg.id,
        &fmt(x),
        &fmt(y),
        &fmt(w),
        &fmt(h),
        &label_html,
    )
}

// --- Recursive sub-graph layout -----------------------------------------------

/// Flip the main graph direction for use as the default inner direction,
/// matching Mermaid's mermaid-graphlib.js logic:
///   let dir = graphSettings.rankdir === 'TB' ? 'LR' : 'TB';
fn flip_dir(outer_rankdir: &str) -> &'static str {
    match outer_rankdir {
        "TB" => "LR",
        "BT" => "LR",
        _ => "TB",
    }
}

/// Result of laying out a single subgraph in isolation.
///  - `node_positions`: leaf-node id → (center_x, center_y, width, height) relative to subgraph origin (0,0)
///  - `child_sg_origins`: child-subgraph id → (origin_x, origin_y) relative to subgraph origin
///  - `child_sg_sizes`: child-subgraph id → (width, height)
///  - `width`, `height`: total bounding box of the subgraph including margins
///  - `inner_graph`: the dagre graph used for this level (for edge routing)
#[derive(Clone)]
struct SgLayout {
    node_positions: HashMap<String, (f64, f64, f64, f64)>,
    /// Edge waypoints in this subgraph's local coordinate system.
    /// Stored as (from_id, to_id, points) for use in g_full after global compositing.
    edges: Vec<(String, String, Vec<Point>)>,
    /// Top-left origins of child subgraphs in this layout's coordinate system,
    /// based on the rect sizes used in the inner dagre layout.
    child_sg_origins: HashMap<String, (f64, f64)>,
    /// Rect sizes (full_size - 16) that were used as opaque node dimensions in
    /// the inner dagre layout.
    child_sg_sizes: HashMap<String, (f64, f64)>,
    /// Full sizes (sg_layout.width, sg_layout.height) of child subgraphs for use
    /// in the renderer's translate calculation.
    child_sg_full_sizes: HashMap<String, (f64, f64)>,
    width: f64,
    height: f64,
}

/// Recursively compute the layout of a subgraph and all its children.
/// Returns `SgLayout` with positions relative to the subgraph's own (0,0) origin.
///
/// `parent_rankdir`: the rankdir of the parent level (used to derive default inner dir)
/// `parent_ranksep`: the ranksep of the parent level (inner ranksep = parent + 25)
fn compute_sg_layout(
    sg_id: &str,
    diag: &FlowchartDiagram,
    subgraph_ids: &HashSet<String>,
    parent_rankdir: &str,
    parent_ranksep: f64,
    all_layouts_out: &mut HashMap<String, SgLayout>,
) -> SgLayout {
    let sg = diag.subgraphs.iter().find(|s| s.id == sg_id).unwrap();

    // Determine this subgraph's effective direction
    let inner_rankdir: &str = if let Some(d) = sg.direction.as_deref() {
        match d {
            "LR" => "LR",
            "RL" => "RL",
            "BT" => "BT",
            "TB" => "TB",
            _ => flip_dir(parent_rankdir),
        }
    } else {
        flip_dir(parent_rankdir)
    };

    // Mermaid adds RANKSEP_INCREMENT to ranksep at each recursive level
    let inner_ranksep = parent_ranksep + RANKSEP_INCREMENT;

    // Recursively compute child subgraph layouts first (bottom-up).
    // Also insert each child's layout into all_layouts_out for use in composite_sg_layout.
    let mut child_layouts: HashMap<String, SgLayout> = HashMap::new();
    for member in &sg.members {
        if subgraph_ids.contains(member) {
            let child_layout = compute_sg_layout(
                member,
                diag,
                subgraph_ids,
                inner_rankdir,
                inner_ranksep,
                all_layouts_out,
            );
            // Also add to the flat all_layouts_out map for composite_sg_layout recursion.
            all_layouts_out.insert(member.clone(), child_layout.clone());
            child_layouts.insert(member.clone(), child_layout);
        }
    }

    // Build a COMPOUND dagre graph for this subgraph level, mirroring Mermaid's
    // recursiveRender approach: the subgraph itself is the compound parent node,
    // and all its members (leaf nodes and child subgraphs) are compound children.
    // This produces exact position matching with the reference SVG.
    //
    // - Leaf members are added as real nodes with their computed sizes.
    // - Child subgraphs are added as opaque nodes using their compound cluster
    //   dimensions (child_layout.width × child_layout.height = removeBorderNodes output).
    //   This matches Mermaid's updateNodeBounds(node, elem) → getBBox() which returns
    //   the compound cluster rect dimensions without the +16 margin adjustment.
    let mut g = Graph::with_options(true, true, true); // compound = true
    g.set_graph(GraphLabel {
        rankdir: Some(inner_rankdir.to_string()),
        nodesep: Some(NODE_SEP),
        ranksep: Some(inner_ranksep),
        marginx: Some(GRAPH_MARGIN),
        marginy: Some(GRAPH_MARGIN),
        ..Default::default()
    });

    // Add the subgraph itself as the compound parent (zero initial size)
    g.set_node(
        sg_id,
        NodeLabel {
            width: 0.0,
            height: 0.0,
            ..Default::default()
        },
    );

    // Add all members as compound children of sg_id
    for member in &sg.members {
        if !subgraph_ids.contains(member) {
            if let Some(node) = diag.nodes.get(member) {
                let (w, h) = node_size(&node.label, &node.shape);
                let intersect_type = shape_intersect_type(&node.shape);
                g.set_node(
                    member,
                    NodeLabel {
                        width: w,
                        height: h,
                        intersect_type,
                        ..Default::default()
                    },
                );
                g.set_parent(member, Some(sg_id));
            }
        } else {
            // Child subgraph as opaque node using its compound cluster dimensions.
            // child_layout.width/height are the compound removeBorderNodes output
            // (the rect w/h from the child's own compound layout).
            let child = &child_layouts[member];
            g.set_node(
                member,
                NodeLabel {
                    width: child.width.max(1.0),
                    height: child.height.max(1.0),
                    ..Default::default()
                },
            );
            g.set_parent(member, Some(sg_id));
        }
    }

    // Add edges between direct members of this subgraph
    let members_set: HashSet<&str> = sg.members.iter().map(|s| s.as_str()).collect();

    for edge in &diag.edges {
        let from_member = resolve_to_member(&edge.from, &members_set, subgraph_ids, diag);
        let to_member = resolve_to_member(&edge.to, &members_set, subgraph_ids, diag);
        if let (Some(from_m), Some(to_m)) = (from_member, to_member) {
            if from_m != to_m {
                if !g.has_node(from_m) || !g.has_node(to_m) {
                    continue;
                }
                g.set_edge(
                    from_m,
                    to_m,
                    EdgeLabel {
                        minlen: Some(1),
                        weight: Some(1.0),
                        ..Default::default()
                    },
                    None,
                );
            }
        }
    }

    // Run compound layout — this gives exact positions matching Mermaid's compound dagre.
    layout(&mut g);

    // Read cluster bounds from the sg node (set by removeBorderNodes after layout).
    // These are the compound rect dimensions: width = |r.x - l.x|, height = |b.y - t.y|.
    let (cluster_w, cluster_h) = if let Some(sg_n) = g.node_opt(sg_id) {
        (sg_n.width, sg_n.height)
    } else {
        (100.0, 100.0)
    };

    // Read the compound parent's center from the inner layout (set by removeBorderNodes).
    // In Mermaid's compound dagre, member nodes are positioned so their group is
    // vertically centered within the cluster for LR/RL layouts, and horizontally
    // centered for TB/BT.  Our compound layout produces the correct cluster bounds
    // but member nodes may be offset relative to the cluster center.
    // We read the compound center and apply a compensating shift.
    let _sg_inner_cx = g.node_opt(sg_id).and_then(|n| n.x).unwrap_or(0.0);
    let sg_inner_cy = g.node_opt(sg_id).and_then(|n| n.y).unwrap_or(0.0);

    let mut node_positions: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();
    let mut child_sg_origins: HashMap<String, (f64, f64)> = HashMap::new();
    let mut child_sg_sizes: HashMap<String, (f64, f64)> = HashMap::new();
    let mut child_sg_full_sizes: HashMap<String, (f64, f64)> = HashMap::new();
    // Capture edge waypoints from the inner layout for later compositing into g_full.
    let mut edges: Vec<(String, String, Vec<Point>)> = Vec::new();
    for e in g.edges() {
        if let Some(lbl) = g.edge(&e) {
            if let Some(pts) = &lbl.points {
                if pts.len() >= 2 {
                    edges.push((e.v.clone(), e.w.clone(), pts.clone()));
                }
            }
        }
    }

    for member in &sg.members {
        if !subgraph_ids.contains(member) {
            if let Some(n) = g.node_opt(member) {
                let cx = n.x.unwrap_or(0.0);
                let cy = n.y.unwrap_or(0.0);
                node_positions.insert(member.clone(), (cx, cy, n.width, n.height));
            }
        } else {
            if let Some(n) = g.node_opt(member) {
                let cx = n.x.unwrap_or(0.0);
                let cy = n.y.unwrap_or(0.0);
                // n.width/height = child compound cluster rect dimensions.
                let child_rect_w = n.width;
                let child_rect_h = n.height;
                // Full size for SVG <g class="root"> translate = rect + SG_LAYOUT_MARGIN.
                let child_full_w = child_rect_w + SG_LAYOUT_MARGIN;
                let child_full_h = child_rect_h + SG_LAYOUT_MARGIN;
                // Store the top-left origin based on full dimensions (matching the renderer translate).
                child_sg_origins.insert(
                    member.clone(),
                    (cx - child_full_w / 2.0, cy - child_full_h / 2.0),
                );
                // child_sg_sizes stores the rect dims for center calculation in composite_sg_layout.
                child_sg_sizes.insert(member.clone(), (child_rect_w, child_rect_h));
                // child_sg_full_sizes for the renderer's translate offset.
                child_sg_full_sizes.insert(member.clone(), (child_full_w, child_full_h));
            }
        }
    }

    // Apply centering offset for LR/RL inner layouts:
    // The compound dagre places member nodes at the correct X positions but their Y positions
    // are at the top of the content area rather than vertically centered in the cluster.
    // Mermaid centers member nodes vertically within the cluster bounding box.
    // Fix: compute the current center-of-mass of member Y positions and shift to match sg_inner_cy.
    // This aligns single-row content (all at same Y) to the cluster center,
    // and preserves relative positioning for multi-row content.
    if inner_rankdir == "LR" || inner_rankdir == "RL" {
        let all_cy_values: Vec<f64> = node_positions
            .values()
            .map(|&(_, cy, _, _)| cy)
            .chain(child_sg_origins.values().map(|&(_, oy)| oy))
            .collect();
        if !all_cy_values.is_empty() {
            let min_cy = all_cy_values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_cy = all_cy_values
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max);
            let member_cy_center = (min_cy + max_cy) / 2.0;
            let cy_offset = sg_inner_cy - member_cy_center;
            if cy_offset.abs() > 0.01 {
                for (_, pos) in node_positions.iter_mut() {
                    pos.1 += cy_offset;
                }
                for (_, origin) in child_sg_origins.iter_mut() {
                    origin.1 += cy_offset;
                }
                for (_, _, pts) in edges.iter_mut() {
                    for p in pts.iter_mut() {
                        p.y += cy_offset;
                    }
                }
            }
        }
    }

    // Similarly for BT/TB layouts, apply X centering if needed:
    // (Currently handled by bt_layer_offset for TB with child subgraphs.)

    SgLayout {
        node_positions,
        edges,
        child_sg_origins,
        child_sg_sizes,
        child_sg_full_sizes,
        width: cluster_w,
        height: cluster_h,
    }
}

/// Map an edge endpoint to the appropriate node in the outer layout.
///
/// Returns:
/// - The leaf node ID if it's in an external top-level subgraph or orphan
/// - The top-level internal subgraph ID if it's inside an internal subgraph
/// - None if no matching outer node
#[allow(clippy::too_many_arguments)]
fn map_to_outer_node(
    node_id: &str,
    top_sg_ids: &[String],
    subgraph_ids: &HashSet<String>,
    sg_external: &HashSet<String>,
    _sg_parent: &HashMap<String, String>,
    sg_descendants: &HashMap<String, HashSet<String>>,
    diag: &FlowchartDiagram,
) -> Option<String> {
    // If the node_id is itself a top-level subgraph
    if top_sg_ids.contains(&node_id.to_string()) {
        if sg_external.contains(node_id) {
            // External subgraph: find a representative leaf node
            return collect_leaf_ids(node_id, subgraph_ids, diag)
                .into_iter()
                .next();
        } else {
            // Internal subgraph: return the subgraph ID (opaque node)
            return Some(node_id.to_string());
        }
    }

    // If node_id is a leaf: find its top-level subgraph ancestor
    for sg_id in top_sg_ids {
        if let Some(descs) = sg_descendants.get(sg_id) {
            if descs.contains(node_id) {
                // This node is inside sg_id
                if sg_external.contains(sg_id) {
                    // External top-level sg → node is directly in outer layout
                    return Some(node_id.to_string());
                } else {
                    // Internal top-level sg → return the opaque subgraph node
                    return Some(sg_id.clone());
                }
            }
        }
    }

    // No subgraph contains this node → orphan outer node
    Some(node_id.to_string())
}

/// Get all descendants of a subgraph (for edge resolution).
fn collect_descendants_direct(
    sg_id: &str,
    subgraph_ids: &HashSet<String>,
    diag: &FlowchartDiagram,
) -> HashSet<String> {
    let mut result = HashSet::new();
    if let Some(sg) = diag.subgraphs.iter().find(|s| s.id == sg_id) {
        for member in &sg.members {
            result.insert(member.clone());
            if subgraph_ids.contains(member) {
                result.extend(collect_descendants_direct(member, subgraph_ids, diag));
            }
        }
    }
    result
}

/// Find which direct member of the set `members_set` contains node `id`.
/// Returns `Some(member_id)` if found, or `None` if not under any member.
fn resolve_to_member<'a>(
    id: &str,
    members_set: &HashSet<&'a str>,
    subgraph_ids: &HashSet<String>,
    diag: &FlowchartDiagram,
) -> Option<&'a str> {
    // Is id itself a direct member?
    if members_set.contains(id) {
        return members_set.get(id).copied();
    }
    // Is id inside a subgraph member?
    for &member in members_set {
        if subgraph_ids.contains(member) {
            let desc = collect_descendants_direct(member, subgraph_ids, diag);
            if desc.contains(id) {
                return Some(member);
            }
        }
    }
    None
}

/// Compute global positions by compositing recursive layouts.
/// Returns:
/// - `node_global`: leaf_id -> (cx, cy, w, h) in global absolute coords
/// - `sg_global_bounds`: sg_id -> (cx, cy, w, h) in global absolute coords
#[allow(dead_code)]
#[allow(clippy::only_used_in_recursion)]
#[allow(clippy::too_many_arguments)]
fn composite_layouts(
    top_g: &Graph,
    _top_level_sgs: &[String],
    sg_layouts: &HashMap<String, SgLayout>,
    subgraph_ids: &HashSet<String>,
    diag: &FlowchartDiagram,
    offset_x: f64,
    offset_y: f64,
    node_global: &mut HashMap<String, (f64, f64, f64, f64)>,
    sg_global_bounds: &mut HashMap<String, (f64, f64, f64, f64)>,
    all_sg_full_sizes: &HashMap<String, (f64, f64)>,
    margin2: f64,
) {
    // Process nodes in top_g (leaf nodes and top-level subgraph opaque nodes)
    for v in top_g.nodes() {
        if subgraph_ids.contains(&v) {
            // This is a top-level subgraph in top_g
            if let Some(n) = top_g.node_opt(&v) {
                let cx = n.x.unwrap_or(0.0) + offset_x;
                let cy = n.y.unwrap_or(0.0) + offset_y;
                let (full_w, full_h) = all_sg_full_sizes
                    .get(&v)
                    .copied()
                    .unwrap_or((n.width + margin2, n.height + margin2));
                sg_global_bounds.insert(v.clone(), (cx, cy, full_w, full_h));

                // Now recursively composite child layout
                if let Some(sg_layout) = sg_layouts.get(&v) {
                    let sg_ox = cx - full_w / 2.0;
                    let sg_oy = cy - full_h / 2.0;
                    composite_sg_layout(
                        &v,
                        sg_layout,
                        sg_layouts,
                        subgraph_ids,
                        diag,
                        sg_ox,
                        sg_oy,
                        node_global,
                        sg_global_bounds,
                        all_sg_full_sizes,
                        margin2,
                        &mut Vec::new(),
                    );
                }
            }
        } else {
            // Leaf node
            if let Some(n) = top_g.node_opt(&v) {
                let cx = n.x.unwrap_or(0.0) + offset_x;
                let cy = n.y.unwrap_or(0.0) + offset_y;
                node_global.insert(v.clone(), (cx, cy, n.width, n.height));
            }
        }
    }
}

/// Recursively composite a subgraph's layout into global coordinates.
///
/// `all_sg_full_sizes`: map of sg_id → full (width, height) for every internal subgraph.
/// `margin2`: 2 * margin (= 16.0) used to recover full size when not in the map.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::only_used_in_recursion)]
fn composite_sg_layout(
    _sg_id: &str,
    sg_layout: &SgLayout,
    all_sg_layouts: &HashMap<String, SgLayout>,
    subgraph_ids: &HashSet<String>,
    diag: &FlowchartDiagram,
    sg_ox: f64, // top-left origin of this sg in global coords (based on full dimensions)
    sg_oy: f64,
    node_global: &mut HashMap<String, (f64, f64, f64, f64)>,
    sg_global_bounds: &mut HashMap<String, (f64, f64, f64, f64)>,
    all_sg_full_sizes: &HashMap<String, (f64, f64)>,
    margin2: f64,
    edge_global: &mut Vec<(String, String, Vec<Point>)>,
) {
    // Direct leaf nodes of this sg
    for (id, &(cx, cy, w, h)) in &sg_layout.node_positions {
        node_global.insert(id.clone(), (cx + sg_ox, cy + sg_oy, w, h));
    }

    // Edges at this sg level — offset to global coordinates
    for (from, to, pts) in &sg_layout.edges {
        let global_pts: Vec<Point> = pts
            .iter()
            .map(|p| Point {
                x: p.x + sg_ox,
                y: p.y + sg_oy,
            })
            .collect();
        if global_pts.len() >= 2 {
            edge_global.push((from.clone(), to.clone(), global_pts));
        }
    }

    // Child subgraphs
    for (child_sg_id, &(origin_x, origin_y)) in &sg_layout.child_sg_origins {
        // origin_x/y = top-left of the child's full group (rect + 16).
        // child_sg_full_sizes stores the full dimensions (rect + 16).
        let (full_w, full_h) = all_sg_full_sizes
            .get(child_sg_id)
            .or_else(|| sg_layout.child_sg_full_sizes.get(child_sg_id))
            .copied()
            .unwrap_or_else(|| {
                let (rw, rh) = sg_layout
                    .child_sg_sizes
                    .get(child_sg_id)
                    .copied()
                    .unwrap_or((100.0, 100.0));
                (rw + margin2, rh + margin2)
            });
        // Global center = origin (global) + full_w/2, full_h/2
        let global_cx = origin_x + sg_ox + full_w / 2.0;
        let global_cy = origin_y + sg_oy + full_h / 2.0;
        sg_global_bounds.insert(child_sg_id.clone(), (global_cx, global_cy, full_w, full_h));

        if let Some(child_layout) = all_sg_layouts.get(child_sg_id) {
            let child_global_ox = global_cx - full_w / 2.0;
            let child_global_oy = global_cy - full_h / 2.0;
            composite_sg_layout(
                child_sg_id,
                child_layout,
                all_sg_layouts,
                subgraph_ids,
                diag,
                child_global_ox,
                child_global_oy,
                node_global,
                sg_global_bounds,
                all_sg_full_sizes,
                margin2,
                edge_global,
            );
        }
    }
}

// --- Node sizing -------------------------------------------------------------

fn shape_intersect_type(shape: &NodeShape) -> Option<&'static str> {
    match shape {
        NodeShape::Diamond | NodeShape::Hexagon => Some("diamond"),
        NodeShape::Circle => Some("circle"),
        _ => None,
    }
}

fn node_size(label: &str, shape: &NodeShape) -> (f64, f64) {
    // Measure the display text, not the raw label — strip fa:fa-xxx prefix if present.
    let (_, display_text) = crate::icons::parse_fa_label(label);
    let measure_text = if display_text.is_empty() {
        label
    } else {
        display_text
    };
    let (raw_tw, _) = measure(measure_text, FONT_SIZE);
    let tw = raw_tw * TEXT_SCALE;
    match shape {
        NodeShape::Rectangle | NodeShape::Default => ((tw + H_PAD * 2.0).max(50.0), RECT_H),
        NodeShape::Subroutine => {
            // Subroutine uses smaller padding than Rectangle to match reference.
            // Total width = text + inner_pad(≈11 each side) + line_offset(8 each side).
            ((tw + SUBROUTINE_H_PAD).max(50.0), COMPACT_H)
        }
        NodeShape::RoundedRect => ((tw + SMALL_PAD * 2.0).max(40.0), RECT_H),
        NodeShape::Cylinder => {
            // Cylinder (database) uses smaller horizontal padding than Rectangle.
            // Height is proportional: CYLINDER_BODY_H + 2*ry where ry = (w/2)*CYLINDER_RY_FACTOR.
            let w = (tw + CYLINDER_H_PAD).max(40.0);
            let ry = (w / 2.0 * CYLINDER_RY_FACTOR).max(CYLINDER_MIN_RY);
            let h = CYLINDER_BODY_H + 2.0 * ry;
            (w, h)
        }
        NodeShape::Stadium => ((tw + SMALL_PAD * 2.0).max(40.0), COMPACT_H),
        NodeShape::Diamond | NodeShape::Hexagon => {
            let dim = (tw + DIAMOND_PAD * 2.0).max(60.0);
            (dim, dim)
        }
        NodeShape::Circle => {
            // The reference uses r = tw/2 + CIRCLE_LABEL_PAD, not DIAMOND_PAD.
            // Minimum radius = CIRCLE_MIN_RADIUS to ensure there's room for short labels.
            let r = (tw / 2.0 + CIRCLE_LABEL_PAD).max(CIRCLE_MIN_RADIUS);
            (r * 2.0, r * 2.0)
        }
        NodeShape::Asymmetric => {
            // lean_right: width = text + padding, height = COMPACT_H.
            // The parallelogram extends ±h/2 beyond the text box horizontally,
            // so dagre sees the full width correctly.
            ((tw + ASYMMETRIC_BASE_PAD).max(40.0), COMPACT_H)
        }
    }
}

// --- Node rendering ----------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn render_node(
    node: &super::parser::FlowNode,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    vars: &ThemeVars,
    style: Option<&NodeStyle>,
    dom_id: &str,
    use_foreign_object: bool,
) -> String {
    let (raw_tw, _) = measure(&node.label, FONT_SIZE);
    let tw = raw_tw * TEXT_SCALE;
    let ff = vars.font_family;

    // Build inline style override from NodeStyle (Mermaid applies these after layout).
    // Mermaid appends " !important" to each property so user styles win over CSS classes.
    // `color` is applied to the <g class="label"> element, NOT the shape, so that it
    // inherits into the foreignObject HTML content.
    let (inline_style, label_color_style, div_color_style) = if let Some(s) = style {
        let mut shape_parts = Vec::new();
        if let Some(f) = s.fill.as_deref() {
            shape_parts.push(format!("fill:{} !important", f));
        }
        if let Some(st) = s.stroke.as_deref() {
            shape_parts.push(format!("stroke:{} !important", st));
        }
        if let Some(sw) = s.stroke_width.as_deref() {
            shape_parts.push(format!("stroke-width:{} !important", sw));
        }
        let lc = s
            .color
            .as_deref()
            .map(|c| format!("color:{} !important", c))
            .unwrap_or_default();
        let dc = s
            .color
            .as_deref()
            .map(|c| format!("color: {} !important; ", c))
            .unwrap_or_default();
        (shape_parts.join(";"), lc, dc)
    } else {
        (String::new(), String::new(), String::new())
    };

    let mut s = String::new();
    s.push_str(&templates::node_group(dom_id, &fmt(cx), &fmt(cy)));

    // Shape
    match node.shape {
        NodeShape::Rectangle | NodeShape::Default => {
            s.push_str(&templates::node_rect(
                &fmt(-w / 2.0),
                &fmt(w),
                &inline_style,
            ));
        }
        NodeShape::RoundedRect => {
            s.push_str(&templates::node_rounded_rect(
                &fmt(-w / 2.0),
                &fmt(w),
                &inline_style,
            ));
        }
        NodeShape::Stadium => {
            let half_h = COMPACT_H / 2.0;
            let rx = half_h;
            s.push_str(&templates::node_stadium_rect(
                &fmt(rx),
                &fmt(-w / 2.0),
                &fmt(-half_h),
                &fmt(w),
                &fmt(COMPACT_H),
                &inline_style,
            ));
        }
        NodeShape::Diamond => {
            let hw = w / 2.0;
            let hh = h / 2.0;
            s.push_str(&templates::node_diamond(
                &fmt(hw),
                &fmt(w),
                &fmt(-hh),
                &fmt(-h),
                &fmt(-hw),
                &fmt(hh),
                &inline_style,
            ));
        }
        NodeShape::Circle => {
            s.push_str(&templates::node_circle(&fmt(w / 2.0), &inline_style));
        }
        NodeShape::Asymmetric => {
            // rect_left_inv_arrow — faithful port of Mermaid rectLeftInvArrow.ts
            //   notch = y/2 = -h/4  (left corners extend h/4 beyond bbox left)
            //   polygon shifted right by -notch/2 = h/8
            // Final points (node center = 0,0):
            let hw = w / 2.0;
            let hh = h / 2.0;
            let shift = hh / 4.0; // h/8 — rightward translate from Mermaid
            let pts = format!(
                "{},{} {},{} {},{} {},{} {},{}",
                fmt(-(hw + shift)),
                fmt(-hh), // top-left outer
                fmt(-hw + shift),
                fmt(0.0), // left notch tip (V-point)
                fmt(-(hw + shift)),
                fmt(hh), // bottom-left outer
                fmt(hw + shift),
                fmt(hh), // bottom-right
                fmt(hw + shift),
                fmt(-hh), // top-right
            );
            s.push_str(&templates::node_asymmetric(&pts, &inline_style));
        }
        NodeShape::Cylinder => {
            let hw = w / 2.0;
            let hh = h / 2.0;
            // Use proportional ry matching Mermaid's cylinder shape formula.
            let ry = (hw * CYLINDER_RY_FACTOR).max(CYLINDER_MIN_RY);
            let body_half = hh - ry; // half of the body height (excluding ellipses)
            let stroke = style
                .and_then(|s| s.stroke.as_deref())
                .unwrap_or(vars.primary_border);
            // Path: start at top-ellipse center, trace top ellipse, go down body, trace bottom ellipse, close
            let d = format!(
                "M {},{} a {},{} 0,0,0 {},0 a {},{} 0,0,0 {},0 l0,{} a {},{} 0,0,0 {},0 l0,{}",
                fmt(-hw),
                fmt(-body_half),
                fmt(hw),
                fmt(ry),
                fmt(w),
                fmt(hw),
                fmt(ry),
                fmt(-w),
                fmt(2.0 * body_half),
                fmt(hw),
                fmt(ry),
                fmt(w),
                fmt(-2.0 * body_half),
            );
            s.push_str(&templates::node_cylinder_body(&d, &inline_style));
            let top_d = format!(
                "M {},{} a {},{} 0,0,0 {},0",
                fmt(-hw),
                fmt(-body_half),
                fmt(hw),
                fmt(ry),
                fmt(w),
            );
            s.push_str(&templates::node_cylinder_top(&top_d, stroke));
        }
        NodeShape::Subroutine => {
            let half_h = COMPACT_H / 2.0;
            let stroke = style
                .and_then(|s| s.stroke.as_deref())
                .unwrap_or(vars.primary_border);
            s.push_str(&templates::node_subroutine_rect(
                &fmt(-w / 2.0),
                &fmt(-half_h),
                &fmt(w),
                &fmt(COMPACT_H),
                &inline_style,
            ));
            s.push_str(&templates::node_subroutine_line(
                &fmt(-w / 2.0 + SUBROUTINE_LINE_INSET),
                &fmt(-half_h),
                &fmt(half_h),
                stroke,
            ));
            s.push_str(&templates::node_subroutine_line(
                &fmt(w / 2.0 - SUBROUTINE_LINE_INSET),
                &fmt(-half_h),
                &fmt(half_h),
                stroke,
            ));
        }
        NodeShape::Hexagon => {
            let hw = w / 2.0;
            let hh = h / 2.0;
            let indent = hh * 0.5;
            let pts = format!(
                "{},{} {},{} {},{} {},{} {},{} {},{}",
                fmt(-hw + indent),
                fmt(-hh),
                fmt(hw - indent),
                fmt(-hh),
                fmt(hw),
                0.0,
                fmt(hw - indent),
                fmt(hh),
                fmt(-hw + indent),
                fmt(hh),
                fmt(-hw),
                0.0,
            );
            s.push_str(&templates::node_hexagon(&pts, &inline_style));
        }
    }

    // Label — apply color to the <g>, the div, the span, AND the <p> so that
    // the CSS rule "#id span { color:#333 }" cannot override via direct targeting.
    let span_color = if !div_color_style.is_empty() {
        format!(
            " style=\"{}\"",
            div_color_style.trim_end_matches(' ').trim_end_matches(';')
        )
    } else {
        String::new()
    };

    // Resolve FA icon syntax: "fa:fa-ban forbidden" → icon char + remaining text.
    let (fa_char, fa_text) = parse_fa_label(&node.label);

    if use_foreign_object {
        // Cylinder labels sit lower to center in the body, not at the top ellipse.
        let label_ty = if node.shape == NodeShape::Cylinder {
            CYLINDER_LABEL_Y_OFFSET
        } else {
            LABEL_Y_OFFSET
        };
        // For the FO path the label goes inside an HTML <p> element. When a Font
        // Awesome icon is present, wrap the glyph in a <span> with the FA font
        // so the icon renders when FA is loaded and degrades to a box otherwise.
        // Show only the text portion of FA labels — the unicode glyph is omitted
        // because Font Awesome is not guaranteed to be loaded in all contexts.
        let fo_label = if fa_char.is_some() {
            esc(fa_text)
        } else {
            esc(&node.label)
        };
        // Asymmetric shape is shifted right by h/8 — label center follows.
        let label_tx = if node.shape == NodeShape::Asymmetric {
            fmt(-tw / 2.0 + h / 8.0)
        } else {
            fmt(-tw / 2.0)
        };
        s.push_str(&templates::node_label_fo(
            &label_color_style,
            &label_tx,
            label_ty,
            &fmt(tw),
            LABEL_FO_HEIGHT,
            &div_color_style,
            &span_color,
            &fo_label,
        ));
    } else {
        let text_fill = if !label_color_style.is_empty() {
            style
                .and_then(|s| s.color.as_deref())
                .unwrap_or(vars.primary_text)
        } else {
            vars.primary_text
        };
        // For the plain SVG text path, show only the text portion of any FA label.
        // We don't embed the FA glyph here because Font Awesome may not be loaded
        // in static contexts (resvg, PNG export), which would show a broken box.
        let svg_label = if fa_char.is_some() {
            if fa_text.is_empty() {
                String::new()
            } else {
                esc(fa_text)
            }
        } else {
            esc(&node.label)
        };
        s.push_str(&templates::node_label_text(
            &label_color_style,
            TEXT_LABEL_Y,
            ff,
            FONT_SIZE,
            text_fill,
            &svg_label,
        ));
    }

    s.push_str("</g>");
    s
}

// --- Graph helpers -----------------------------------------------------------

/// Collect all descendant node IDs of a subgraph (recursively).
fn collect_descendants(
    sg_id: &str,
    subgraph_ids: &HashSet<String>,
    diag: &FlowchartDiagram,
) -> HashSet<String> {
    let mut result = HashSet::new();
    if let Some(sg) = diag.subgraphs.iter().find(|sg| sg.id == sg_id) {
        for member in &sg.members {
            result.insert(member.clone());
            if subgraph_ids.contains(member) {
                let inner = collect_descendants(member, subgraph_ids, diag);
                result.extend(inner);
            }
        }
    }
    result
}

/// Collect all LEAF (non-subgraph) descendants of a subgraph.
fn collect_leaf_ids(
    sg_id: &str,
    subgraph_ids: &HashSet<String>,
    diag: &FlowchartDiagram,
) -> Vec<String> {
    let mut result = Vec::new();
    if let Some(sg) = diag.subgraphs.iter().find(|sg| sg.id == sg_id) {
        for member in &sg.members {
            if subgraph_ids.contains(member) {
                result.extend(collect_leaf_ids(member, subgraph_ids, diag));
            } else {
                result.push(member.clone());
            }
        }
    }
    result
}

/// Resolve a possibly-subgraph node ID to its first leaf node.
/// Returns true if subgraph `sg_id` is a direct member of the current render context.
/// At the top level (self_cluster=None): true for top-level subgraphs (no parent).
/// At an inner level (self_cluster=Some(sc)): true if sg's direct parent is sc.
fn is_sg_at_level(
    sg_id: &str,
    sg_parent: &HashMap<String, String>,
    self_cluster: Option<&Subgraph>,
) -> bool {
    match self_cluster {
        None => sg_parent.get(sg_id).is_none(),
        Some(sc) => sg_parent.get(sg_id).map(|p| p == &sc.id).unwrap_or(false),
    }
}

/// Resolve an edge endpoint to the dagre node key for the current render level.
/// Returns None if this endpoint does not belong at the current level (edge should be skipped).
///
/// For subgraph IDs (e.g. "TOP"): returns Some("TOP") if the subgraph is at this level
/// and exists as a node in g.  For leaf nodes: returns Some(id) if node_at_level holds.
#[allow(clippy::too_many_arguments)]
fn resolve_edge_endpoint<'a>(
    id: &str,
    subgraph_ids: &HashSet<String>,
    diag: &'a FlowchartDiagram,
    g: &Graph,
    context_sgs: &[&'a Subgraph],
    node_subgraph: &HashMap<String, String>,
    sg_parent: &HashMap<String, String>,
    sg_external: &HashSet<String>,
    self_cluster: Option<&'a Subgraph>,
) -> Option<String> {
    if subgraph_ids.contains(id) {
        if sg_external.contains(id) {
            // External subgraph: edge was stored with its leaf representative.
            // Return the leaf so the edge can be found in dagre; boundary clipping
            // will move the rendered path to the cluster rect boundary.
            let leaf = resolve_to_leaf(id, subgraph_ids, diag)?;
            if g.has_node(&leaf) {
                return Some(leaf);
            }
            return None;
        }
        // Internal (opaque) subgraph: edge stored with compound node ID.
        if is_sg_at_level(id, sg_parent, self_cluster) && g.has_node(id) {
            return Some(id.to_string());
        }
        None
    } else {
        // Leaf node: use node_at_level as before
        if node_at_level(
            id,
            context_sgs,
            node_subgraph,
            sg_parent,
            sg_external,
            self_cluster,
        ) {
            Some(id.to_string())
        } else {
            None
        }
    }
}

fn resolve_to_leaf(
    id: &str,
    subgraph_ids: &HashSet<String>,
    diag: &FlowchartDiagram,
) -> Option<String> {
    if !subgraph_ids.contains(id) {
        return Some(id.to_string());
    }
    if let Some(sg) = diag.subgraphs.iter().find(|sg| sg.id == id) {
        for member in &sg.members {
            if let Some(leaf) = resolve_to_leaf(member, subgraph_ids, diag) {
                return Some(leaf);
            }
        }
    }
    None
}

/// Determine if a leaf node belongs at the given rendering level.
///
/// A node belongs at this level if:
///
/// Case A — This is a top-level group (no self_cluster):
///   - Node has no direct parent subgraph (orphan), OR
///   - Node's direct parent is in context_sgs AND that parent is EXTERNAL
///     (external clusters don't create their own level — children render at parent level)
///   - Node's direct parent is in context_sgs AND it's INTERNAL → belongs INSIDE the compound, NOT here
///
/// Case B — This is a compound group (self_cluster = Some(sg)):
///   - Node's direct parent is `sg` (the compound we're inside)
///   - NOTE: child subgraphs that are internal will recurse further; their nodes don't belong here
///     at this level if the child subgraph is internal — but since child_sgs are passed as context_sgs,
///     node_at_level is called recursively for the child level.
fn node_at_level(
    node_id: &str,
    context_sgs: &[&Subgraph],
    node_subgraph: &HashMap<String, String>,
    sg_parent: &HashMap<String, String>,
    sg_external: &HashSet<String>,
    self_cluster: Option<&Subgraph>,
) -> bool {
    match self_cluster {
        None => {
            // Top-level case: node belongs here if direct parent is None or external
            match node_subgraph.get(node_id) {
                None => {
                    // No parent subgraph — orphan node at top level
                    // BUT only render at top level if context is the top-level set
                    // (i.e., none of context_sgs has a parent — they're all top-level)
                    context_sgs.iter().all(|sg| !sg_parent.contains_key(&sg.id))
                }
                Some(direct_parent) => {
                    // Node is directly inside direct_parent
                    // Check if direct_parent is in context_sgs
                    let in_context = context_sgs.iter().any(|sg| sg.id == *direct_parent);
                    if in_context {
                        // Belongs here if the parent is external (external clusters don't recurse)
                        sg_external.contains(direct_parent)
                    } else {
                        // direct_parent is not in context_sgs — node is inside a nested subgraph
                        // Check if the ultimate top-level ancestor of this node is external
                        // and in context_sgs
                        let mut cur = direct_parent.clone();
                        loop {
                            match sg_parent.get(&cur) {
                                None => return false, // ancestor has no parent in sg_parent → not at top level context
                                Some(anc) => {
                                    if context_sgs.iter().any(|sg| sg.id == *anc) {
                                        return sg_external.contains(anc.as_str());
                                    }
                                    cur = anc.clone();
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(sc) => {
            // Compound case: node belongs here if its direct parent is `sc`
            // AND sc is not an internal child compound (internal children will recurse)
            match node_subgraph.get(node_id) {
                Some(direct_parent) if direct_parent == &sc.id => {
                    // Direct child of this compound — render here UNLESS
                    // it's inside an internal child subgraph (but then node_subgraph would point to that child)
                    true
                }
                Some(direct_parent)
                    // Node's direct parent is a child subgraph of sc
                    // If that child subgraph is EXTERNAL, the node renders at this level
                    // If INTERNAL, the node renders inside that child's compound → NOT here
                    if context_sgs.iter().any(|sg| sg.id == *direct_parent) =>
                {
                    sg_external.contains(direct_parent)
                }
                Some(_) => false, // direct parent is not a known context subgraph
                None => false, // orphan node — shouldn't be inside a compound
            }
        }
    }
}

// --- Subgraph bounding box ---------------------------------------------------

#[allow(dead_code)]
fn compute_subgraph_bbox(leaves: &[String], g: &Graph) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    for member in leaves {
        if let Some(n) = g.node_opt(member) {
            let x = n.x.unwrap_or(0.0);
            let y = n.y.unwrap_or(0.0);
            let hw = n.width / 2.0;
            let hh = n.height / 2.0;
            min_x = min_x.min(x - hw);
            max_x = max_x.max(x + hw);
            min_y = min_y.min(y - hh);
            max_y = max_y.max(y + hh);
        }
    }
    if min_x == f64::MAX {
        return None;
    }
    Some((
        min_x - SG_PAD_H,
        min_y - SG_PAD_T,
        max_x - min_x + SG_PAD_H * 2.0,
        max_y - min_y + SG_PAD_T + SG_PAD_B,
    ))
}

// --- Edge path ---------------------------------------------------------------

fn trim_path_end(pts: &[Point], amount: f64) -> Vec<Point> {
    if amount <= 0.0 || pts.len() < 2 {
        return pts.to_vec();
    }
    let mut r = pts.to_vec();
    let n = r.len();
    let last = r[n - 1].clone();
    let prev = r[n - 2].clone();
    let dx = last.x - prev.x;
    let dy = last.y - prev.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= amount {
        r.truncate(n - 1);
    } else {
        let frac = (len - amount) / len;
        r[n - 1] = Point {
            x: prev.x + dx * frac,
            y: prev.y + dy * frac,
        };
    }
    r
}

#[allow(dead_code)]
fn trim_path_start(pts: &[Point], amount: f64) -> Vec<Point> {
    if amount <= 0.0 || pts.len() < 2 {
        return pts.to_vec();
    }
    let mut r = pts.to_vec();
    let first = r[0].clone();
    let next = r[1].clone();
    let dx = next.x - first.x;
    let dy = next.y - first.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= amount {
        r.remove(0);
    } else {
        let frac = amount / len;
        r[0] = Point {
            x: first.x + dx * frac,
            y: first.y + dy * frac,
        };
    }
    r
}

/// D3 curveBasis path — matches Mermaid JS reference edge curves.
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

// --- Helpers -----------------------------------------------------------------

fn pt_in_rect(p: &Point, cx: f64, cy: f64, w: f64, h: f64) -> bool {
    p.x >= cx - w / 2.0 - 1.0
        && p.x <= cx + w / 2.0 + 1.0
        && p.y >= cy - h / 2.0 - 1.0
        && p.y <= cy + h / 2.0 + 1.0
}

fn rect_exit(p0: &Point, p1: &Point, cx: f64, cy: f64, w: f64, h: f64) -> Option<Point> {
    let left = cx - w / 2.0;
    let right = cx + w / 2.0;
    let top = cy - h / 2.0;
    let bot = cy + h / 2.0;
    let dx = p1.x - p0.x;
    let dy = p1.y - p0.y;
    let mut best_t = f64::INFINITY;
    for &(val, is_y) in &[(left, false), (right, false), (top, true), (bot, true)] {
        let (denom, numer, pmin, pmax) = if is_y {
            (dy, val - p0.y, left, right)
        } else {
            (dx, val - p0.x, top, bot)
        };
        if denom.abs() < 1e-9 {
            continue;
        }
        let t = numer / denom;
        if !(1e-6..=1.0 + 1e-6).contains(&t) {
            continue;
        }
        let perp = if is_y { p0.x + dx * t } else { p0.y + dy * t };
        if perp < pmin - 1.0 || perp > pmax + 1.0 {
            continue;
        }
        if t < best_t {
            best_t = t;
        }
    }
    if best_t.is_finite() {
        Some(Point {
            x: p0.x + dx * best_t,
            y: p0.y + dy * best_t,
        })
    } else {
        None
    }
}

/// Strip all initial (or final) points inside the cluster rect and replace with
/// the boundary exit (or entry) point so the arrow originates from the cluster edge.
fn clip_path_from_cluster(
    pts: &[Point],
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    from_start: bool,
) -> Vec<Point> {
    if pts.len() < 2 {
        return pts.to_vec();
    }
    if from_start {
        // Find last consecutive inside point from the front
        let mut last_in = 0usize;
        for (i, pt) in pts.iter().enumerate().take(pts.len() - 1) {
            if pt_in_rect(pt, cx, cy, w, h) {
                last_in = i;
            } else {
                break;
            }
        }
        if last_in == 0 && !pt_in_rect(&pts[0], cx, cy, w, h) {
            return pts.to_vec(); // first point already outside
        }
        // Compute exit from last_in → last_in+1
        if let Some(exit) = rect_exit(&pts[last_in], &pts[last_in + 1], cx, cy, w, h) {
            let mut result = vec![exit];
            result.extend_from_slice(&pts[last_in + 1..]);
            return result;
        }
        pts.to_vec()
    } else {
        // Mirror: strip trailing inside points
        let n = pts.len();
        let mut last_in = n - 1;
        for i in (1..n).rev() {
            if pt_in_rect(&pts[i], cx, cy, w, h) {
                last_in = i;
            } else {
                break;
            }
        }
        if last_in == n - 1 && !pt_in_rect(&pts[n - 1], cx, cy, w, h) {
            return pts.to_vec();
        }
        if let Some(entry) = rect_exit(&pts[last_in], &pts[last_in - 1], cx, cy, w, h) {
            let mut result = pts[..last_in].to_vec();
            result.push(entry);
            return result;
        }
        pts.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    const FLOWCHART_BASIC: &str = "flowchart TD\n    A[Christmas] -->|Get money| B(Go shopping)\n    B --> C{Let me think}\n    C -->|One| D[Laptop]\n    C -->|Two| E[iPhone]\n    C -->|Three| F[Car]";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(FLOWCHART_BASIC).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains("<svg"), "missing <svg tag");
        assert!(svg.contains("Christmas"), "missing node label");
        assert!(svg.contains("Go shopping"), "missing node label");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(FLOWCHART_BASIC).diagram;
        let svg = render(&diag, Theme::Dark, false);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn snapshot_default_theme() {
        let diag = parser::parse(FLOWCHART_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
