use super::constants::*;
/// Faithful Rust port of Mermaid's blockRenderer.ts.
///
/// Layout algorithm (matches Mermaid blockRenderer.ts):
/// - Node widths are computed from text measurement + horizontal padding.
/// - All single-span nodes get the same width (max across all nodes).
/// - Elements in each row placed left-to-right with H_GAP between nodes.
/// - space tokens leave empty cells (same width as a regular column).
/// - Rows stacked vertically with V_GAP between them.
/// - Entire diagram is vertically centered at y=0 (viewBox uses negative coords).
/// - Edges drawn as cubic bezier paths from node right-edge to node left-edge.
use super::parser::{BlockDiagram, BlockEdge, BlockNode, BlockShape, RowItem};
#[allow(unused_imports)]
use super::templates::{
    self, build_markers, edge_label_text, edge_path, esc, fmt, fmt_px, node_circle,
    node_cylinder_ellipse, node_cylinder_rect, node_diamond, node_group, node_hexagon,
    node_label_fo, node_rect_rounded, node_rect_square, svg_root,
};
use crate::text::measure;
use crate::theme::Theme;

/// Decode basic HTML entities to their text equivalents for measurement.
fn decode_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

/// Compute text width scaled to browser metrics.
fn text_width(label: &str) -> f64 {
    let decoded = decode_entities(label);
    let (tw, _) = measure(&decoded, FONT_SIZE);
    tw * TEXT_SCALE
}

pub fn render(diag: &BlockDiagram, theme: Theme, _use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    let primary_color = vars.primary_color;
    let primary_border = vars.primary_border;
    // Dark theme block text: Chrome color-picker on reference gives #F9FFFE
    // (from CSS `.cluster-label span, p { color:#F9FFFE }` which wins via specificity)
    let primary_text = match theme {
        Theme::Dark => "#F9FFFE",
        _ => vars.primary_text,
    };
    let secondary_color = vars.secondary_color;
    let line_color = vars.line_color;
    let cluster_bg = vars.block_container_fill;
    let cluster_border = vars.block_container_stroke;
    let svg_id = "mermaid-block";

    let cols = if diag.columns < 1 { 1 } else { diag.columns };

    // ── Pass 1: compute widths bottom-up (Mermaid setBlockSizes algorithm) ──────
    // For each group node, compute its width from children's max text width.
    // Then col_w for top-level = max width of all top-level items (including groups).
    fn group_width(node_id: &str, nodes: &indexmap::IndexMap<String, BlockNode>) -> f64 {
        let node = match nodes.get(node_id) {
            Some(n) => n,
            None => return 0.0,
        };
        if node.is_group && !node.group_children.is_empty() {
            let max_child_w = node
                .group_children
                .iter()
                .filter_map(|cid| nodes.get(cid.as_str()))
                .map(|cn| text_width(&cn.label) + H_PAD * 2.0)
                .fold(0.0_f64, f64::max);
            let n = node.group_children.len();
            n as f64 * (max_child_w + H_GAP) + H_GAP
        } else {
            text_width(&node.label) + H_PAD * 2.0
        }
    }

    // col_w = max effective width of all top-level row items
    // row_h = max effective height (groups are taller: NODE_H + 2*H_GAP; leaves = NODE_H)
    let (col_w, row_h) = {
        let mut max_w = 0.0_f64;
        let mut max_h = NODE_H;
        for row in &diag.rows {
            for item in &row.items {
                if let RowItem::Node(id, _) = item {
                    let w = group_width(id, &diag.nodes);
                    if w > max_w {
                        max_w = w;
                    }
                    // Group height = child_height + 2*padding (Mermaid setBlockSizes)
                    if let Some(n) = diag.nodes.get(id.as_str()) {
                        let h = if n.is_group && !n.group_children.is_empty() {
                            NODE_H + 2.0 * H_GAP
                        } else {
                            NODE_H
                        };
                        if h > max_h {
                            max_h = h;
                        }
                    }
                }
            }
        }
        (max_w.max(20.0), max_h)
    };

    // ── Pass 2: compute layout positions ─────────────────────────────────────
    // node_pos maps id → (cx, cy, w, h).
    // Positions are relative to content origin (top-left); we'll center later.
    let mut node_pos: std::collections::HashMap<String, (f64, f64, f64, f64)> =
        std::collections::HashMap::new();

    let mut cur_y = 0.0f64; // current row top

    for row in &diag.rows {
        let mut col_offset = 0usize;
        let mut cur_x = 0.0f64;

        // All rows use the uniform max height (Mermaid makes all siblings same height)
        let cy = cur_y + row_h / 2.0;

        for item in &row.items {
            match item {
                RowItem::Space(span) => {
                    // Advance x by span columns worth of space
                    for i in 0..*span {
                        if i > 0 {
                            cur_x += H_GAP;
                        }
                        cur_x += col_w;
                    }
                    col_offset += span;
                    // Add gap after this item unless at end of row
                    if col_offset < cols {
                        cur_x += H_GAP;
                    }
                }
                RowItem::Node(id, span) => {
                    if let Some(node) = diag.nodes.get(id.as_str()) {
                        // Mermaid setBlockSizes: within a group, all children get the SAME width
                        // (the max of the siblings' text widths). Group width = n*(maxChild+padding)+padding.
                        let (w, group_child_w) = if node.is_group && !node.group_children.is_empty()
                        {
                            let max_child_w = node
                                .group_children
                                .iter()
                                .filter_map(|cid| diag.nodes.get(cid.as_str()))
                                .map(|cn| text_width(&cn.label) + H_PAD * 2.0)
                                .fold(0.0_f64, f64::max);
                            let n = node.group_children.len();
                            let total = n as f64 * (max_child_w + H_GAP) + H_GAP;
                            (total, max_child_w)
                            // height is row_h (set by the layout loop above)
                        } else if *span <= 1 {
                            (col_w, col_w)
                        } else {
                            (
                                col_w * (*span as f64) + H_GAP * (span.saturating_sub(1) as f64),
                                col_w,
                            )
                        };

                        let cx = cur_x + w / 2.0;
                        // All siblings get the same uniform row_h (Mermaid setBlockSizes)
                        node_pos.insert(id.clone(), (cx, cy, w, row_h));

                        // Layout group children uniformly (all same width = group_child_w)
                        if node.is_group && !node.group_children.is_empty() {
                            let mut child_x = cur_x + H_GAP;
                            let child_y = cy;
                            for child_id in &node.group_children.clone() {
                                let child_cx = child_x + group_child_w / 2.0;
                                node_pos.insert(
                                    child_id.clone(),
                                    (child_cx, child_y, group_child_w, NODE_H),
                                );
                                child_x += group_child_w + H_GAP;
                            }
                        }

                        cur_x += w;
                        col_offset += span;
                        // Add gap after this item unless at end of row
                        if col_offset < cols {
                            cur_x += H_GAP;
                        }
                    }
                }
            }
            if col_offset >= cols {
                break;
            }
        }

        cur_y += row_h + V_GAP;
    }

    // Fallback: nodes not placed by rows
    {
        let mut col = 0usize;
        let mut fallback_y = cur_y;
        for (id, _node) in &diag.nodes {
            if node_pos.contains_key(id.as_str()) {
                continue;
            }
            let w = col_w;
            let cx = col as f64 * (w + H_GAP) + w / 2.0;
            let cy = fallback_y + NODE_H / 2.0;
            node_pos.insert(id.clone(), (cx, cy, w, NODE_H));
            col += 1;
            if col >= cols {
                col = 0;
                fallback_y += NODE_H + V_GAP;
            }
        }
        if col > 0 || !diag.rows.is_empty() {
            // don't need to update cur_y here
        }
    }

    // ── Compute content bounding box (before centering) ───────────────────────
    let mut min_x = f64::MAX;
    let mut raw_min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut raw_max_y = f64::MIN;

    for &(cx, cy, w, h) in node_pos.values() {
        min_x = min_x.min(cx - w / 2.0);
        raw_min_y = raw_min_y.min(cy - h / 2.0);
        max_x = max_x.max(cx + w / 2.0);
        raw_max_y = raw_max_y.max(cy + h / 2.0);
    }

    if node_pos.is_empty() {
        min_x = 0.0;
        raw_min_y = 0.0;
        max_x = 100.0;
        raw_max_y = 50.0;
    }

    // ── Center the diagram vertically at y=0 ─────────────────────────────────
    // Mermaid vertically centers the block diagram so the midpoint of the
    // content is at y=0, giving the viewBox negative y coordinates.
    let content_h = raw_max_y - raw_min_y;
    let center_y = raw_min_y + content_h / 2.0;

    // Shift all node y coords so center_y becomes 0
    let y_shift = -center_y;
    let mut final_pos: std::collections::HashMap<String, (f64, f64, f64, f64)> =
        std::collections::HashMap::new();
    for (id, &(cx, cy, w, h)) in &node_pos {
        final_pos.insert(id.clone(), (cx, cy + y_shift, w, h));
    }

    // Recompute bounding box after centering
    let min_y = raw_min_y + y_shift;
    let _max_y = raw_max_y + y_shift;
    let content_w = max_x - min_x;

    // viewBox: add MARGIN on each side
    let vb_x = min_x - MARGIN;
    let vb_y = min_y - MARGIN;
    let vb_w = content_w + MARGIN * 2.0;
    let vb_h = content_h + MARGIN * 2.0;

    let mut out = String::new();

    out.push_str(&svg_root(
        svg_id,
        &fmt_px(vb_w),
        &fmt(vb_x),
        &fmt(vb_y),
        &fmt(vb_w),
        &fmt(vb_h),
    ));

    // Empty <g> for compatibility
    out.push_str("<g></g>");

    // Arrow markers
    out.push_str(&build_markers(svg_id, line_color));

    // Main block group
    out.push_str("<g class=\"block\">");

    // Render nodes in insertion order
    for (id, node) in &diag.nodes {
        if let Some(&(cx, cy, w, h)) = final_pos.get(id.as_str()) {
            out.push_str(&render_node(
                node,
                cx,
                cy,
                w,
                h,
                primary_color,
                primary_border,
                primary_text,
                secondary_color,
                cluster_bg,
                cluster_border,
            ));
        }
    }

    // Render edges
    for edge in &diag.edges {
        if let (Some(&(fx, fy, fw, fh)), Some(&(tx, ty, tw, th))) =
            (final_pos.get(&edge.from), final_pos.get(&edge.to))
        {
            out.push_str(&render_edge(
                edge,
                fx,
                fy,
                fw,
                fh,
                tx,
                ty,
                tw,
                th,
                svg_id,
                line_color,
                primary_text,
            ));
        }
    }

    out.push_str("</g>");
    out.push_str("</svg>");
    out
}

/// Render a single node with foreignObject label (matches Mermaid structure).
#[allow(clippy::too_many_arguments)]
fn render_node(
    node: &BlockNode,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    fill: &str,
    stroke: &str,
    text_color: &str,
    _secondary_color: &str,
    cluster_bg: &str,
    cluster_border: &str,
) -> String {
    let hw = w / 2.0;
    let hh = h / 2.0;

    // Build inline style: user override takes priority over theme defaults.
    let node_style = match node.style {
        Some(ref s) => s.clone(),
        None => format!("fill:{fill};stroke:{stroke};stroke-width:1px;"),
    };

    let mut s = String::new();
    s.push_str(&node_group(
        "mermaid-block",
        &esc(&node.id),
        &fmt(cx),
        &fmt(cy),
    ));

    // Group/container nodes use cluster fill+border (`.cluster rect` CSS in Mermaid).
    if node.is_group {
        s.push_str(&format!(
            r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" style="fill:{cluster_bg};stroke:{cluster_border};stroke-width:1px;" rx="0" ry="0"></rect>"##,
            x = fmt(-hw), y = fmt(-hh), w = fmt(w), h = fmt(h),
            cluster_bg = cluster_bg, cluster_border = cluster_border,
        ));
        s.push_str("</g>");
        return s;
    }

    match node.shape {
        BlockShape::Square | BlockShape::Default => {
            s.push_str(&node_rect_square(
                &fmt(-hw),
                &fmt(-hh),
                &fmt(w),
                &fmt(h),
                &node_style,
            ));
        }
        BlockShape::RoundedRect => {
            s.push_str(&node_rect_rounded(
                &fmt(-hw),
                &fmt(-hh),
                &fmt(w),
                &fmt(h),
                &node_style,
            ));
        }
        BlockShape::Diamond => {
            let pts = format!(
                "{},{} {},{} {},{} {},{}",
                fmt(0.0),
                fmt(-hh),
                fmt(hw),
                fmt(0.0),
                fmt(0.0),
                fmt(hh),
                fmt(-hw),
                fmt(0.0),
            );
            s.push_str(&node_diamond(&pts, &node_style));
        }
        BlockShape::Circle => {
            // Circle uses its natural radius based on label width (matches Mermaid's block_arrow formula).
            // Mermaid: r = bbox.width/2 + halfPadding, where halfPadding = node.padding/2 = 4.
            let label_w = text_width(&node.label);
            let r = label_w / 2.0 + 4.0;
            s.push_str(&node_circle(&fmt(r), &node_style));
        }
        BlockShape::Cylinder => {
            let ry_val = 7.0_f64;
            let body_h = h - ry_val;
            s.push_str(&node_cylinder_rect(
                &fmt(-hw),
                &fmt(-hh + ry_val),
                &fmt(w),
                &fmt(body_h),
                fill,
                stroke,
            ));
            s.push_str(&node_cylinder_ellipse(
                "0",
                &fmt(-hh + ry_val),
                &fmt(hw),
                &fmt(ry_val),
                fill,
                stroke,
            ));
            s.push_str(&node_cylinder_ellipse(
                "0",
                &fmt(-hh + ry_val + body_h),
                &fmt(hw),
                &fmt(ry_val),
                fill,
                stroke,
            ));
        }
        BlockShape::BlockArrow => {
            // Mermaid's block_arrow "down" direction polygon (faithful port of blockArrowHelper.ts).
            // padding = 8, h = fo_h + 2*padding = 24 + 16 = 40
            // m = h/2 = 20 (midpoint), p2 = padding/2 = 4
            // w = label_width + 2*m + padding = label_width + 48
            let label_w = text_width(&node.label);
            let pad = 8.0_f64;
            let arrow_h = FONT_SIZE * 1.5 + 2.0 * pad; // 40
            let m = arrow_h / 2.0; // 20
            let arrow_w = label_w + 2.0 * m + pad; // label_w + 48
            let arrow_hw = arrow_w / 2.0;
            let arrow_hh = arrow_h / 2.0; // 20
            let p2 = pad / 2.0; // 4
                                // 7-point "down" polygon in local coords (centered at 0,0, y-down positive):
            let pts = format!(
                "{},{} {},{} {},{} {},{} {},{} {},{} {},{}",
                fmt(0.0),
                fmt(arrow_hh), // bottom tip
                fmt(-arrow_hw),
                fmt(arrow_hh - p2), // left outer wing
                fmt(-arrow_hw + m),
                fmt(arrow_hh - p2), // left inner body bottom
                fmt(-arrow_hw + m),
                fmt(-arrow_hh + p2), // left inner body top
                fmt(arrow_hw - m),
                fmt(-arrow_hh + p2), // right inner body top
                fmt(arrow_hw - m),
                fmt(arrow_hh - p2), // right inner body bottom
                fmt(arrow_hw),
                fmt(arrow_hh - p2), // right outer wing
            );
            s.push_str(&format!(
                r##"<polygon points="{pts}" class="label-container" style="{node_style}"></polygon>"##,
                pts = pts, node_style = node_style,
            ));
        }
        BlockShape::Hexagon => {
            let indent = hh * 0.5;
            let pts = format!(
                "{},{} {},{} {},{} {},{} {},{} {},{}",
                fmt(-hw + indent),
                fmt(-hh),
                fmt(hw - indent),
                fmt(-hh),
                fmt(hw),
                fmt(0.0),
                fmt(hw - indent),
                fmt(hh),
                fmt(-hw + indent),
                fmt(hh),
                fmt(-hw),
                fmt(0.0),
            );
            s.push_str(&node_hexagon(&pts, &node_style));
        }
    }

    // Label — decode HTML entities before escaping for SVG
    let decoded_label = decode_entities(&node.label);
    let tw = text_width(&node.label);
    let fo_h = (FONT_SIZE * 1.5).round(); // 24.0
    let pad_v = (h - fo_h) / 2.0;
    let label_ty = -(hh - pad_v);

    s.push_str(&node_label_fo(
        "0",
        &fmt(label_ty),
        &fmt(tw),
        &fmt(fo_h),
        &esc(&decoded_label),
        text_color,
    ));

    s.push_str("</g>");
    s
}

/// Faithful port of Mermaid's intersect-rect.js.
/// Returns the point on the rectangle boundary (cx, cy, hw, hh) along the line toward (px, py).
fn intersect_rect(cx: f64, cy: f64, hw: f64, hh: f64, px: f64, py: f64) -> (f64, f64) {
    let dx = px - cx;
    let dy = py - cy;
    if dx.abs() < 1e-9 && dy.abs() < 1e-9 {
        return (cx, cy);
    }
    let (sx, sy) = if dy.abs() * hw > dx.abs() * hh {
        // Exits through top or bottom edge
        let h = if dy < 0.0 { -hh } else { hh };
        let sx = if dy.abs() > 1e-12 { h * dx / dy } else { 0.0 };
        (sx, h)
    } else {
        // Exits through left or right edge
        let w = if dx < 0.0 { -hw } else { hw };
        let sy = if dx.abs() > 1e-12 { w * dy / dx } else { 0.0 };
        (w, sy)
    };
    (cx + sx, cy + sy)
}

/// Render an edge as a path with arrowhead (matches Mermaid block diagram structure).
/// Uses rect-boundary intersection so edges land at the correct node boundary point.
#[allow(clippy::too_many_arguments)]
fn render_edge(
    edge: &BlockEdge,
    fx: f64,
    fy: f64,
    fw: f64,
    fh: f64,
    tx: f64,
    ty: f64,
    tw: f64,
    th: f64,
    svg_id: &str,
    line_color: &str,
    text_color: &str,
) -> String {
    // Start point: intersection on source boundary toward target center
    let (sx, sy) = intersect_rect(fx, fy, fw / 2.0, fh / 2.0, tx, ty);

    // End point (raw): intersection on target boundary toward source center
    let (ex_raw, ey_raw) = intersect_rect(tx, ty, tw / 2.0, th / 2.0, fx, fy);

    // Trim endpoint 4 px toward source so arrowhead tip lands exactly on boundary
    let (ex, ey) = {
        let dx = sx - ex_raw;
        let dy = sy - ey_raw;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > 1e-9 {
            let trim = 4.0_f64;
            (ex_raw + dx / dist * trim, ey_raw + dy / dist * trim)
        } else {
            (ex_raw, ey_raw)
        }
    };

    // Path: Mermaid-style catmull-rom through [start, midpoint, end].
    // Midpoint of CENTER-to-CENTER segment (before boundary intersection).
    let mid_x = (fx + tx) / 2.0;
    let mid_y = (fy + ty) / 2.0;

    // Convert catmull-rom [s, m, e] to piecewise cubics (standard approach).
    // Phantom endpoint mirrors: p_before_s = 2*s - m, p_after_e = 2*e - m.
    // Segment s→m: ctrl1 = s + (m-p_before)/6 = s + (m-s)/3
    //              ctrl2 = m - (p_after-s)/6  = m - (e-s)/6
    // Segment m→e: ctrl1 = m + (e-p_before)/6 = m + (e-s)/6
    //              ctrl2 = e - (p_after-m)/6  = e - (e-m)/3
    let step = (mid_x - sx) / 3.0;
    let vstep = (mid_y - sy) / 3.0;
    // Path: catmull-rom through [start, midpoint, end].
    // The cubic bezier's final tangent at (ex,ey) gives orient="auto" the correct
    // arrowhead direction — no trailing L needed (zero-length L corrupts orientation).
    let path = format!(
        "M{},{} L{},{} C{},{},{},{},{},{} C{},{},{},{},{},{}",
        fmt(sx),
        fmt(sy),
        fmt(sx + step),
        fmt(sy + vstep),
        fmt(sx + step * 2.0),
        fmt(sy + vstep * 2.0),
        fmt(mid_x - step),
        fmt(mid_y - vstep),
        fmt(mid_x),
        fmt(mid_y),
        fmt(mid_x + (ex - mid_x) / 3.0),
        fmt(mid_y + (ey - mid_y) / 3.0),
        fmt(ex - (ex - mid_x) / 3.0),
        fmt(ey - (ey - mid_y) / 3.0),
        fmt(ex),
        fmt(ey),
    );

    let edge_id = format!("{}-1-{}-{}", svg_id, edge.from, edge.to);
    let marker_end = format!("url(#{}_block-pointEnd)", svg_id);

    let mut s = String::new();
    s.push_str(&edge_path(
        &path,
        &edge_id,
        &edge.from.to_lowercase(),
        &edge.to.to_lowercase(),
        &marker_end,
        line_color,
    ));

    // Edge label if present
    if let Some(ref label) = edge.label {
        if !label.is_empty() {
            s.push_str(&edge_label_text(
                &fmt(mid_x),
                &fmt(mid_y - 5.0),
                &esc(label),
                text_color,
            ));
        }
    }

    s
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    #[test]
    fn basic_render_produces_svg() {
        let input = "block-beta\n    columns 3\n    A[\"A\"] B[\"B\"] C[\"C\"]\n    space D[\"D\"] space\n    A --> D\n    B --> D";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains("<svg"), "no <svg element");
        assert!(svg.contains("flowchart-label"), "no nodes");
    }

    #[test]
    fn renders_edges() {
        let input = "block-beta\n    A[\"A\"]\n    B[\"B\"]\n    A --> B";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains("<path"), "no edge path");
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "block-beta\n    columns 3\n    A[\"A\"]:1\n    B[\"B\"]:1\n    C[\"C\"]:1\n    space:1\n    D[\"D\"]:1\n    space:1";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
