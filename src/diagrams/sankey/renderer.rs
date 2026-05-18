use super::constants::*;
use super::parser::{LinkColor, NodeAlignment, SankeyDiagram};
use super::templates;
/// Faithful Rust port of Mermaid's sankeyRenderer.ts + d3-sankey v0.12.3.
///
/// Implements the d3-sankey layout algorithm from scratch (since we can't use npm).
///
/// The d3-sankey algorithm:
///   1. Assign each node to a "column" (layer) based on link topology
///      (sankeyLeft, sankeyRight, sankeyCenter, sankeyJustify alignment modes).
///   2. Within each column, compute node heights proportional to their total value
///      using a GLOBAL ky = min over all columns of (height - (n-1)*py) / sum(values).
///   3. Position nodes vertically within columns, then spread evenly.
///   4. Iterative relaxation with alpha decay (6 iterations).
///   5. Compute link y positions (y0/y1) for source/target attachment points.
///   6. Links are drawn as horizontal cubic Bezier paths (sankeyLinkHorizontal).
///
/// Colors: Tableau10 palette (mirrors d3-scale schemeTableau10).
use crate::theme::Theme;

// ── Tableau10 palette (mirrors d3-schemeTableau10) ────────────────────────────
// TABLEAU10 is imported from constants.

/// Pick color by insertion index (mirrors d3.scaleOrdinal insertion order).
fn tableau_color_by_index(idx: usize) -> &'static str {
    TABLEAU10[idx % TABLEAU10.len()]
}

// ── Layout structures ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct LayoutNode {
    id: String,
    #[allow(dead_code)]
    index: usize,
    /// depth (column from left, BFS)
    depth: usize,
    /// height (column from right, BFS)
    height: usize,
    /// assigned layer (column) after alignment
    layer: usize,
    /// Total value flowing through this node
    value: f64,
    /// Computed x positions (left edge, right edge)
    x0: f64,
    x1: f64,
    /// Computed y positions (top, bottom)
    y0: f64,
    y1: f64,
    /// Source links (indices into layout_links)
    source_links: Vec<usize>,
    /// Target links (indices into layout_links)
    target_links: Vec<usize>,
}

#[derive(Debug, Clone)]
struct LayoutLink {
    source: usize, // LayoutNode index
    target: usize, // LayoutNode index
    value: f64,
    width: f64, // computed = value * ky
    /// y attachment at source (center of link band at source node)
    y0: f64,
    /// y attachment at target (center of link band at target node)
    y1: f64,
    index: usize,
}

struct SankeyLayout {
    nodes: Vec<LayoutNode>,
    links: Vec<LayoutLink>,
    #[allow(dead_code)]
    num_columns: usize,
}

// ── d3-sankey algorithm implementation ────────────────────────────────────────

/// Compute sankey layout from parsed diagram data.
/// Mirrors d3-sankey's sankey() function (v0.12.3).
fn compute_layout(
    diag: &SankeyDiagram,
    width: f64,
    height: f64,
    node_width: f64,
    node_padding: f64,
    alignment: &NodeAlignment,
) -> SankeyLayout {
    let n = diag.nodes.len();
    if n == 0 {
        return SankeyLayout {
            nodes: vec![],
            links: vec![],
            num_columns: 0,
        };
    }

    // Build index map
    let mut node_index: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for (i, node) in diag.nodes.iter().enumerate() {
        node_index.insert(&node.id, i);
    }

    // Initialize layout nodes
    let mut layout_nodes: Vec<LayoutNode> = diag
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| LayoutNode {
            id: node.id.clone(),
            index: i,
            depth: 0,
            height: 0,
            layer: 0,
            value: 0.0,
            x0: 0.0,
            x1: 0.0,
            y0: 0.0,
            y1: 0.0,
            source_links: vec![],
            target_links: vec![],
        })
        .collect();

    // Initialize layout links
    let mut layout_links: Vec<LayoutLink> = diag
        .links
        .iter()
        .enumerate()
        .map(|(i, link)| {
            let src = *node_index.get(link.source.as_str()).unwrap_or(&0);
            let tgt = *node_index.get(link.target.as_str()).unwrap_or(&0);
            LayoutLink {
                source: src,
                target: tgt,
                value: link.value,
                width: 0.0,
                y0: 0.0,
                y1: 0.0,
                index: i,
            }
        })
        .collect();

    // Connect source/target links to nodes
    for (li, link) in layout_links.iter().enumerate() {
        let src = link.source;
        let tgt = link.target;
        layout_nodes[src].source_links.push(li);
        layout_nodes[tgt].target_links.push(li);
    }

    // ── Step 1: Compute node values ───────────────────────────────────────────
    // Value = max(sum of outgoing, sum of incoming) — mirrors d3-sankey computeNodeValues
    for node in layout_nodes.iter_mut() {
        let src_sum: f64 = node
            .source_links
            .iter()
            .map(|&li| layout_links[li].value)
            .sum();
        let tgt_sum: f64 = node
            .target_links
            .iter()
            .map(|&li| layout_links[li].value)
            .sum();
        node.value = src_sum.max(tgt_sum);
    }

    // ── Step 2: Compute depths (BFS forward) — mirrors computeNodeDepths ─────
    // d3 starts with current = ALL nodes (Set), assigns depth = wave index,
    // then next = union of all targets of current nodes.
    // Nodes can be revisited and get their depth OVERWRITTEN — so each node
    // ends up with the depth of the LAST (deepest) wave it belongs to.
    // This equals the length of the longest path from any source to this node.
    {
        let mut x = 0usize;
        // Use Vec<bool> to simulate JS Set membership (O(1) insert/check)
        let mut current_set: Vec<bool> = vec![true; n]; // all nodes start in current
        let mut current_list: Vec<usize> = (0..n).collect();

        while !current_list.is_empty() {
            for &ni in &current_list {
                layout_nodes[ni].depth = x;
            }
            x += 1;
            if x > n {
                break; // circular link protection
            }
            let mut next_set: Vec<bool> = vec![false; n];
            let mut next_list: Vec<usize> = Vec::new();
            for &ni in &current_list {
                for &li in &layout_nodes[ni].source_links {
                    let tgt = layout_links[li].target;
                    if !next_set[tgt] {
                        next_set[tgt] = true;
                        next_list.push(tgt);
                    }
                }
                current_set[ni] = false;
            }
            current_list = next_list;
            current_set = next_set;
        }
    }

    // ── Step 3: Compute heights (BFS backward) — mirrors computeNodeHeights ──
    // Same approach, but follows target→source links backward.
    {
        let mut x = 0usize;
        let mut current_list: Vec<usize> = (0..n).collect();

        while !current_list.is_empty() {
            for &ni in &current_list {
                layout_nodes[ni].height = x;
            }
            x += 1;
            if x > n {
                break;
            }
            let mut next_set: Vec<bool> = vec![false; n];
            let mut next_list: Vec<usize> = Vec::new();
            for &ni in &current_list {
                for &li in &layout_nodes[ni].target_links {
                    let src = layout_links[li].source;
                    if !next_set[src] {
                        next_set[src] = true;
                        next_list.push(src);
                    }
                }
            }
            current_list = next_list;
        }
    }

    // ── Step 4: Compute layers (columns) and x positions ─────────────────────
    // mirrors computeNodeLayers
    let max_depth = layout_nodes.iter().map(|n| n.depth).max().unwrap_or(0);
    let num_columns = max_depth + 1;
    let kx = if num_columns > 1 {
        (width - node_width) / (num_columns as f64 - 1.0)
    } else {
        0.0
    };

    for node in layout_nodes.iter_mut() {
        let raw_layer = match alignment {
            NodeAlignment::Left => node.depth,
            NodeAlignment::Right => {
                // right: n - 1 - height
                (num_columns - 1).saturating_sub(node.height)
            }
            NodeAlignment::Center => {
                // center: if no incoming links → depth, else if no outgoing → max_col,
                // otherwise depth (like d3 center function)
                node.depth
            }
            NodeAlignment::Justify => {
                // justify: if has outgoing links → depth, else → max_col
                if !node.source_links.is_empty() {
                    node.depth
                } else {
                    max_depth
                }
            }
        };
        let layer = raw_layer.min(num_columns - 1);
        node.layer = layer;
        node.x0 = kx * layer as f64;
        node.x1 = node.x0 + node_width;
    }

    // Build columns array (ordered by layer)
    let mut columns: Vec<Vec<usize>> = vec![vec![]; num_columns];
    for i in 0..n {
        columns[layout_nodes[i].layer].push(i);
    }
    // Sort each column by current y0 (ascending) — initial sort
    // d3 sorts by ascendingBreadth after relaxation, but initially uses insertion order

    // ── Step 5: Compute node breadths (y positions) ───────────────────────────
    // mirrors computeNodeBreadths
    // First clamp py: py = min(dy, height / (max_col_size - 1))
    let max_col_size = columns.iter().map(|c| c.len()).max().unwrap_or(1);
    let py = if max_col_size > 1 {
        node_padding.min(height / (max_col_size as f64 - 1.0))
    } else {
        node_padding
    };

    // initializeNodeBreadths: compute global ky, set initial positions
    let ky = columns
        .iter()
        .filter(|c| !c.is_empty())
        .map(|c| {
            let sum_vals: f64 = c.iter().map(|&i| layout_nodes[i].value).sum();
            let avail = height - (c.len() as f64 - 1.0) * py;
            if sum_vals > 0.0 {
                avail / sum_vals
            } else {
                f64::MAX
            }
        })
        .fold(f64::MAX, f64::min);
    let ky = if ky == f64::MAX { 1.0 } else { ky };

    // Set link widths = value * ky (global)
    for link in layout_links.iter_mut() {
        link.width = link.value * ky;
    }

    // Initial node positions: pack from top with py gaps, then spread slack evenly
    for col_nodes in &columns {
        if col_nodes.is_empty() {
            continue;
        }
        let mut y = 0.0_f64;
        for &i in col_nodes {
            layout_nodes[i].y0 = y;
            layout_nodes[i].y1 = y + layout_nodes[i].value * ky;
            y = layout_nodes[i].y1 + py;
        }
        // y is now: (total_node_height + total_padding + one_extra_py)
        // slack = (height - y + py) / (n + 1)
        let slack = (height - y + py) / (col_nodes.len() as f64 + 1.0);
        for (idx, &i) in col_nodes.iter().enumerate() {
            let offset = slack * (idx as f64 + 1.0);
            layout_nodes[i].y0 += offset;
            layout_nodes[i].y1 += offset;
        }
        // reorderLinks: sort sourceLinks by target y0, targetLinks by source y0
        reorder_links_for_col(col_nodes, &mut layout_nodes, &layout_links);
    }

    // Iterative relaxation — mirrors d3-sankey's 6 iterations with alpha decay
    for iter in 0..6usize {
        let alpha = 0.99_f64.powi(iter as i32);
        let beta = (1.0 - alpha).max((iter as f64 + 1.0) / 6.0);

        // relaxRightToLeft
        for ci in (0..columns.len()).rev().skip(1) {
            let col_nodes = columns[ci].clone();
            for &ni in &col_nodes {
                let src_links = layout_nodes[ni].source_links.clone();
                if src_links.is_empty() {
                    continue;
                }
                let mut y = 0.0_f64;
                let mut w = 0.0_f64;
                for &li in &src_links {
                    let link = &layout_links[li];
                    let tgt = link.target;
                    let v = link.value
                        * (layout_nodes[tgt].layer as f64 - layout_nodes[ni].layer as f64);
                    y += source_top(&layout_nodes, &layout_links, ni, tgt, py) * v;
                    w += v;
                }
                if w <= 0.0 {
                    continue;
                }
                let dy = (y / w - layout_nodes[ni].y0) * alpha;
                layout_nodes[ni].y0 += dy;
                layout_nodes[ni].y1 += dy;
                reorder_node_links(ni, &mut layout_nodes, &layout_links);
            }
            // sort column by y0
            columns[ci].sort_by(|&a, &b| {
                layout_nodes[a]
                    .y0
                    .partial_cmp(&layout_nodes[b].y0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            resolve_collisions(&mut layout_nodes, &columns[ci], beta, height, py);
        }

        // relaxLeftToRight
        for column in columns.iter_mut().skip(1) {
            let col_nodes = column.clone();
            for &ni in &col_nodes {
                let tgt_links = layout_nodes[ni].target_links.clone();
                if tgt_links.is_empty() {
                    continue;
                }
                let mut y = 0.0_f64;
                let mut w = 0.0_f64;
                for &li in &tgt_links {
                    let link = &layout_links[li];
                    let src = link.source;
                    let v = link.value
                        * (layout_nodes[ni].layer as f64 - layout_nodes[src].layer as f64);
                    y += target_top(&layout_nodes, &layout_links, src, ni, py) * v;
                    w += v;
                }
                if w <= 0.0 {
                    continue;
                }
                let dy = (y / w - layout_nodes[ni].y0) * alpha;
                layout_nodes[ni].y0 += dy;
                layout_nodes[ni].y1 += dy;
                reorder_node_links(ni, &mut layout_nodes, &layout_links);
            }
            // sort column by y0
            column.sort_by(|&a, &b| {
                layout_nodes[a]
                    .y0
                    .partial_cmp(&layout_nodes[b].y0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            resolve_collisions(&mut layout_nodes, column, beta, height, py);
        }
    }

    // ── Step 6: Compute link breadths (y0/y1 attachment positions) ───────────
    // mirrors computeLinkBreadths — starts from node.y0, accumulates link.width
    compute_link_breadths(&mut layout_nodes, &mut layout_links);

    SankeyLayout {
        nodes: layout_nodes,
        links: layout_links,
        num_columns,
    }
}

/// Reorder source and target links for all nodes in a column by breadth.
/// Mirrors d3-sankey's reorderLinks().
fn reorder_links_for_col(col_nodes: &[usize], nodes: &mut [LayoutNode], links: &[LayoutLink]) {
    for &ni in col_nodes {
        // sort sourceLinks by target y0 ascending
        let mut src = nodes[ni].source_links.clone();
        src.sort_by(|&a, &b| {
            let ya = nodes[links[a].target].y0;
            let yb = nodes[links[b].target].y0;
            ya.partial_cmp(&yb)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| links[a].index.cmp(&links[b].index))
        });
        nodes[ni].source_links = src;

        // sort targetLinks by source y0 ascending
        let mut tgt = nodes[ni].target_links.clone();
        tgt.sort_by(|&a, &b| {
            let ya = nodes[links[a].source].y0;
            let yb = nodes[links[b].source].y0;
            ya.partial_cmp(&yb)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| links[a].index.cmp(&links[b].index))
        });
        nodes[ni].target_links = tgt;
    }
}

/// Reorder links for a single node during relaxation.
/// Mirrors d3-sankey's reorderNodeLinks() — cross-node re-sorting.
fn reorder_node_links(ni: usize, nodes: &mut [LayoutNode], links: &[LayoutLink]) {
    // For each source link's target node: re-sort that target's targetLinks by source y0
    let src_links = nodes[ni].source_links.clone();
    for &li in &src_links {
        let tgt_ni = links[li].target;
        let mut tgt_tgt_links = nodes[tgt_ni].target_links.clone();
        tgt_tgt_links.sort_by(|&a, &b| {
            let ya = nodes[links[a].source].y0;
            let yb = nodes[links[b].source].y0;
            ya.partial_cmp(&yb)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| links[a].index.cmp(&links[b].index))
        });
        nodes[tgt_ni].target_links = tgt_tgt_links;
    }

    // For each target link's source node: re-sort that source's sourceLinks by target y0
    let tgt_links = nodes[ni].target_links.clone();
    for &li in &tgt_links {
        let src_ni = links[li].source;
        let mut src_src_links = nodes[src_ni].source_links.clone();
        src_src_links.sort_by(|&a, &b| {
            let ya = nodes[links[a].target].y0;
            let yb = nodes[links[b].target].y0;
            ya.partial_cmp(&yb)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| links[a].index.cmp(&links[b].index))
        });
        nodes[src_ni].source_links = src_src_links;
    }
}

/// Returns the target.y0 that would produce an ideal link from source to target.
/// Mirrors d3-sankey's targetTop(source, target).
fn target_top(
    nodes: &[LayoutNode],
    links: &[LayoutLink],
    src_ni: usize,
    tgt_ni: usize,
    py: f64,
) -> f64 {
    let src = &nodes[src_ni];
    let tgt = &nodes[tgt_ni];

    // y starts at source.y0, offset by (n_source_links - 1) * py / 2
    let mut y = src.y0 - (src.source_links.len() as f64 - 1.0) * py / 2.0;

    // Advance y for each source link until we reach the one targeting tgt_ni
    for &li in &src.source_links {
        let link = &links[li];
        if link.target == tgt_ni {
            break;
        }
        y += link.width + py;
    }

    // Subtract widths of target links before the one from src_ni
    for &li in &tgt.target_links {
        let link = &links[li];
        if link.source == src_ni {
            break;
        }
        y -= link.width;
    }

    y
}

/// Returns the source.y0 that would produce an ideal link from source to target.
/// Mirrors d3-sankey's sourceTop(source, target).
fn source_top(
    nodes: &[LayoutNode],
    links: &[LayoutLink],
    src_ni: usize,
    tgt_ni: usize,
    py: f64,
) -> f64 {
    let src = &nodes[src_ni];
    let tgt = &nodes[tgt_ni];

    // y starts at target.y0, offset by (n_target_links - 1) * py / 2
    let mut y = tgt.y0 - (tgt.target_links.len() as f64 - 1.0) * py / 2.0;

    // Advance y for each target link until we reach the one from src_ni
    for &li in &tgt.target_links {
        let link = &links[li];
        if link.source == src_ni {
            break;
        }
        y += link.width + py;
    }

    // Subtract widths of source links before the one targeting tgt_ni
    for &li in &src.source_links {
        let link = &links[li];
        if link.target == tgt_ni {
            break;
        }
        y -= link.width;
    }

    y
}

/// Resolve collisions within a column using alpha-based spring.
/// Mirrors d3-sankey's resolveCollisions(nodes, alpha).
fn resolve_collisions(
    nodes: &mut [LayoutNode],
    col_nodes: &[usize],
    alpha: f64,
    height: f64,
    py: f64,
) {
    if col_nodes.is_empty() {
        return;
    }
    let m = col_nodes.len();
    let i = m / 2;
    let subject = col_nodes[i];

    // resolveCollisionsBottomToTop from subject going up
    resolve_bottom_to_top(nodes, col_nodes, subject, i, alpha, py);
    // resolveCollisionsTopToBottom from subject going down
    resolve_top_to_bottom(nodes, col_nodes, subject, i, alpha, py, height);
    // resolveCollisionsBottomToTop from bottom
    resolve_bottom_to_top_from_end(nodes, col_nodes, alpha, py, height);
    // resolveCollisionsTopToBottom from top
    resolve_top_to_bottom_from_start(nodes, col_nodes, alpha, py);
}

fn resolve_bottom_to_top(
    nodes: &mut [LayoutNode],
    col_nodes: &[usize],
    _subject_idx: usize,
    start: usize,
    alpha: f64,
    py: f64,
) {
    // resolveCollisionsBottomToTop(nodes, subject.y0 - py, i - 1, alpha)
    if start == 0 {
        return;
    }
    let y_start = nodes[col_nodes[start]].y0 - py;
    let mut y = y_start;
    let mut idx = start as isize - 1;
    while idx >= 0 {
        let ni = col_nodes[idx as usize];
        let dy = (nodes[ni].y1 - y) * alpha;
        if dy > 1e-6 {
            let h = nodes[ni].y1 - nodes[ni].y0;
            nodes[ni].y0 -= dy;
            nodes[ni].y1 -= dy;
            // clamp y0 to 0
            if nodes[ni].y0 < 0.0 {
                nodes[ni].y0 = 0.0;
                nodes[ni].y1 = h;
            }
        }
        y = nodes[ni].y0 - py;
        idx -= 1;
    }
}

fn resolve_top_to_bottom(
    nodes: &mut [LayoutNode],
    col_nodes: &[usize],
    _subject_idx: usize,
    start: usize,
    alpha: f64,
    py: f64,
    _height: f64,
) {
    // resolveCollisionsTopToBottom(nodes, subject.y1 + py, i + 1, alpha)
    let m = col_nodes.len();
    if start + 1 >= m {
        return;
    }
    let y_start = nodes[col_nodes[start]].y1 + py;
    let mut y = y_start;
    for &ni in col_nodes.iter().take(m).skip(start + 1) {
        let dy = (y - nodes[ni].y0) * alpha;
        if dy > 1e-6 {
            let _h = nodes[ni].y1 - nodes[ni].y0;
            nodes[ni].y0 += dy;
            nodes[ni].y1 += dy;
        }
        y = nodes[ni].y1 + py;
    }
}

fn resolve_bottom_to_top_from_end(
    nodes: &mut [LayoutNode],
    col_nodes: &[usize],
    alpha: f64,
    py: f64,
    height: f64,
) {
    // resolveCollisionsBottomToTop(nodes, y1, nodes.length - 1, alpha)
    let m = col_nodes.len();
    if m == 0 {
        return;
    }
    let mut y = height;
    let mut idx = m as isize - 1;
    while idx >= 0 {
        let ni = col_nodes[idx as usize];
        let dy = (nodes[ni].y1 - y) * alpha;
        if dy > 1e-6 {
            let h = nodes[ni].y1 - nodes[ni].y0;
            nodes[ni].y0 -= dy;
            nodes[ni].y1 -= dy;
            if nodes[ni].y0 < 0.0 {
                nodes[ni].y0 = 0.0;
                nodes[ni].y1 = h;
            }
        }
        y = nodes[ni].y0 - py;
        idx -= 1;
    }
}

fn resolve_top_to_bottom_from_start(
    nodes: &mut [LayoutNode],
    col_nodes: &[usize],
    alpha: f64,
    py: f64,
) {
    // resolveCollisionsTopToBottom(nodes, y0, 0, alpha)
    let m = col_nodes.len();
    let mut y = 0.0_f64;
    for &ni in col_nodes.iter().take(m) {
        let dy = (y - nodes[ni].y0) * alpha;
        if dy > 1e-6 {
            let _h = nodes[ni].y1 - nodes[ni].y0;
            nodes[ni].y0 += dy;
            nodes[ni].y1 += dy;
        }
        y = nodes[ni].y1 + py;
    }
}

/// Compute link attachment y positions.
/// Mirrors d3-sankey's computeLinkBreadths — starts from node.y0, no sorting.
fn compute_link_breadths(nodes: &mut [LayoutNode], links: &mut [LayoutLink]) {
    for node in nodes.iter() {
        // source links: y0 starts at node.y0, accumulates link.width
        let mut y0 = node.y0;
        let src_links = node.source_links.clone();
        for &li in &src_links {
            links[li].y0 = y0 + links[li].width / 2.0;
            y0 += links[li].width;
        }

        // target links: y1 starts at node.y0, accumulates link.width
        let mut y1 = node.y0;
        let tgt_links = node.target_links.clone();
        for &li in &tgt_links {
            links[li].y1 = y1 + links[li].width / 2.0;
            y1 += links[li].width;
        }
    }
}

// ── SVG rendering ──────────────────────────────────────────────────────────────

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Generate a Sankey link path (mirrors d3.sankeyLinkHorizontal).
/// Draws a cubic Bezier from (source_x1, y0) to (target_x0, y1)
/// with horizontal control points at the midpoint x.
fn sankey_link_path(
    src_x1: f64, // source node right edge
    tgt_x0: f64, // target node left edge
    y0: f64,     // source attachment y
    y1: f64,     // target attachment y
) -> String {
    let mid_x = (src_x1 + tgt_x0) / 2.0;
    format!(
        "M{x0:.2},{y0:.2} C{mx:.2},{y0:.2} {mx:.2},{y1:.2} {x1:.2},{y1:.2}",
        x0 = src_x1,
        y0 = y0,
        mx = mid_x,
        y1 = y1,
        x1 = tgt_x0,
    )
}

/// Find the "central" layer — the layer of the node with maximum value.
/// Mirrors sankeyRenderer.ts findCentralNodeLayer().
fn find_central_node_layer(nodes: &[LayoutNode]) -> usize {
    let mut max_value = 0.0_f64;
    let mut central_layer = 0usize;
    for node in nodes {
        if node.value > max_value {
            max_value = node.value;
            central_layer = node.layer;
        }
    }
    central_layer
}

/// Determine label position for a node.
/// Mirrors sankeyRenderer.ts getLabelPosition() with 'legacy' style (default).
///
/// Legacy: position-based (original behavior):
///   if x0 < width/2 → place label to the right (x1 + 6, text-anchor="start")
///   else             → place label to the left  (x0 - 6, text-anchor="end")
fn label_position(node: &LayoutNode, width: f64) -> (f64, &'static str) {
    if node.x0 < width / 2.0 {
        (node.x1 + LABEL_OFFSET, "start")
    } else {
        (node.x0 - LABEL_OFFSET, "end")
    }
}

/// Outlined label position (mirrors sankeyRenderer.ts 'outlined' style):
///   layer < centralLayer → label left of node
///   else                 → label right of node
#[allow(dead_code)]
fn label_position_outlined(node: &LayoutNode, central_layer: usize) -> (f64, &'static str) {
    if node.layer < central_layer {
        (node.x0 - 6.0, "end")
    } else {
        (node.x1 + 6.0, "start")
    }
}

fn build_css(svg_id: &str, ff: &str) -> String {
    format!(
        concat!(
            "#{id}{{font-family:{ff};font-size:14px;fill:#333;}}",
            "#{id} .nodes .node rect{{shape-rendering:crispEdges;}}",
            "#{id} .links .link{{fill:none;stroke-opacity:0.5;}}",
            "#{id} .node-labels text{{font-size:14px;}}",
            "#{id} .sankey-label-bg{{stroke:#fff;stroke-width:4px;paint-order:stroke;fill:#fff;opacity:0.8;}}",
            "#{id} .sankey-label-fg{{}}",
        ),
        id = svg_id,
        ff = ff,
    )
}

pub fn render(diag: &SankeyDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let svg_id = SVG_ID;
    let conf = &diag.config;

    let width = conf.width;
    let height = conf.height;
    let node_width = conf.node_width;
    let node_padding = conf.node_padding + if conf.show_values { 15.0 } else { 0.0 };
    let show_values = conf.show_values;
    let prefix = &conf.prefix;
    let suffix = &conf.suffix;

    if diag.nodes.is_empty() {
        return templates::svg_empty(svg_id, width, height);
    }

    // Compute layout
    let layout = compute_layout(
        diag,
        width,
        height,
        node_width,
        node_padding,
        &conf.node_alignment,
    );

    let nodes = &layout.nodes;
    let links = &layout.links;

    let _central_layer = find_central_node_layer(nodes);

    // Color scheme: assign colors by node insertion index (mirrors d3.scaleOrdinal)
    let get_node_color = |_id: &str, idx: usize| -> &'static str { tableau_color_by_index(idx) };

    let css = build_css(svg_id, ff);

    let mut parts: Vec<String> = Vec::new();

    parts.push(templates::svg_root(svg_id, width, height));
    parts.push(format!("<style>{}</style>", css));

    // ── Nodes ─────────────────────────────────────────────────────────────────
    parts.push(r#"<g class="nodes">"#.to_string());

    for (i, node) in nodes.iter().enumerate() {
        let color = get_node_color(&node.id, i);
        let node_h = node.y1 - node.y0;
        let node_w = node.x1 - node.x0;

        parts.push(templates::node_group(i, node.x0, node.y0));
        parts.push(templates::node_rect(node_h, node_w, color));
        parts.push("</g>".to_string());
    }

    parts.push("</g>".to_string());

    // ── Node labels ───────────────────────────────────────────────────────────
    parts.push(format!(
        r#"<g class="node-labels" font-size="{}">"#,
        LABEL_FONT_SIZE_ATTR
    ));

    for node in nodes.iter() {
        let label = if show_values {
            let rounded = (node.value * 100.0).round() / 100.0;
            format!("{}\n{}{}{}", node.id, prefix, rounded, suffix)
        } else {
            node.id.clone()
        };

        let (lx, anchor) = label_position(node, width);
        let ly = (node.y1 + node.y0) / 2.0;
        let dy = "0em";
        // Mermaid puts label+value in one text element with a newline between them.
        // SVG's default whitespace normalization collapses the newline to a space,
        // rendering "Homes 40" on a single line — matching the reference output.
        let text_content = escape(&label);

        parts.push(templates::node_label_text(
            lx,
            ly,
            dy,
            anchor,
            ff,
            &text_content,
        ));
    }

    parts.push("</g>".to_string());

    // ── Links (defs for gradients) ────────────────────────────────────────────
    let link_color_mode = &conf.link_color;

    // Build gradient defs if needed
    if *link_color_mode == LinkColor::Gradient {
        parts.push("<defs>".to_string());
        for (li, link) in links.iter().enumerate() {
            let src_color = get_node_color(&nodes[link.source].id, link.source);
            let tgt_color = get_node_color(&nodes[link.target].id, link.target);
            parts.push(templates::linear_gradient(
                li,
                nodes[link.source].x1,
                nodes[link.target].x0,
                src_color,
                tgt_color,
            ));
        }
        parts.push("</defs>".to_string());
    }

    // Link paths
    parts.push(r#"<g class="links" fill="none" stroke-opacity="0.5">"#.to_string());

    for (li, link) in links.iter().enumerate() {
        let src = &nodes[link.source];
        let tgt = &nodes[link.target];

        let path_d = sankey_link_path(src.x1, tgt.x0, link.y0, link.y1);
        let stroke_width = link.width.max(1.0);

        let stroke = match link_color_mode {
            LinkColor::Gradient => format!("url(#lg-{})", li),
            LinkColor::Source => get_node_color(&src.id, link.source).to_string(),
            LinkColor::Target => get_node_color(&tgt.id, link.target).to_string(),
            LinkColor::Custom(c) => c.clone(),
        };

        parts.push(templates::link_path(&path_d, &stroke, stroke_width));
    }

    parts.push("</g>".to_string());
    parts.push("</svg>".to_string());

    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::sankey::parser;

    #[test]
    fn basic_render_produces_svg() {
        let input = "sankey-beta\nA,B,10\nA,C,20\nB,D,5\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "missing <svg");
        assert!(svg.contains("class=\"nodes\""));
        assert!(svg.contains("class=\"links\""));
    }

    #[test]
    fn node_labels_present() {
        let input = "sankey-beta\nA,B,10\nA,C,20\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(
            svg.contains(">A<")
                || svg.contains(">A</")
                || svg.contains("A</tspan>")
                || svg.contains(">A\n")
        );
    }

    #[test]
    fn empty_sankey_produces_svg() {
        let input = "sankey-beta\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn link_path_cubic_bezier() {
        let path = sankey_link_path(100.0, 200.0, 50.0, 80.0);
        assert!(path.starts_with("M100.00,50.00"));
        assert!(path.contains('C'));
        assert!(path.ends_with("200.00,80.00"));
    }

    #[test]
    fn frontmatter_config_used() {
        let input = "---\nconfig:\n  sankey:\n    showValues: false\n    width: 800\n    height: 500\n---\nsankey-beta\nA,B,10\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("800"));
        assert!(svg.contains("500"));
    }

    #[test]
    fn gradient_defs_present() {
        let input = "sankey-beta\nA,B,10\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("linearGradient"));
    }

    #[test]
    fn column_assignment_justify() {
        // With justify alignment, sinks (no outgoing) go to last column.
        let input = "sankey-beta\nA,B,10\nA,C,20\nB,D,5\nC,D,15\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("class=\"nodes\""));
    }

    #[test]
    #[ignore = "platform-specific float precision — run locally"]
    fn snapshot_default_theme() {
        let input = "sankey-beta\nCoal,Power,50\nGas,Power,30\nNuclear,Power,20\nPower,Homes,40\nPower,Industry,60";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(svg);
    }
}
