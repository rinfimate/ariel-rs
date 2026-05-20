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
    self, build_markers, build_style, edge_label_text, edge_path, esc, fmt, fmt_px, node_circle,
    node_cylinder_ellipse, node_cylinder_rect, node_diamond, node_group, node_hexagon,
    node_label_fo, node_rect_rounded, node_rect_square, svg_root,
};
use crate::text::measure;
use crate::theme::Theme;

/// Compute text width scaled to browser metrics.
fn text_width(label: &str) -> f64 {
    let (tw, _) = measure(label, FONT_SIZE);
    tw * TEXT_SCALE
}

pub fn render(diag: &BlockDiagram, theme: Theme, _use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let svg_id = "mermaid-block";

    let cols = if diag.columns < 1 { 1 } else { diag.columns };

    // ── Pass 1: compute uniform column width ─────────────────────────────────
    // Mermaid uses the max text width across all single-span nodes + 2×H_PAD
    // as the uniform column width for all cells.
    let max_text_w = diag
        .nodes
        .values()
        .filter(|n| n.col_span <= 1)
        .map(|n| text_width(&n.label))
        .fold(0.0f64, f64::max);

    // If there are multi-span nodes only, fall back to something reasonable
    let col_w = if max_text_w > 0.0 {
        max_text_w + H_PAD * 2.0
    } else {
        // Try multi-span nodes
        let max_tw = diag
            .nodes
            .values()
            .map(|n| text_width(&n.label))
            .fold(0.0f64, f64::max);
        (max_tw + H_PAD * 2.0).max(20.0)
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

        let row_h = NODE_H;
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
                    if let Some(_node) = diag.nodes.get(id.as_str()) {
                        // Width for a multi-span node spans multiple columns + gaps
                        let w = if *span <= 1 {
                            col_w
                        } else {
                            col_w * (*span as f64) + H_GAP * (span.saturating_sub(1) as f64)
                        };

                        let cx = cur_x + w / 2.0;
                        node_pos.insert(id.clone(), (cx, cy, w, NODE_H));

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

    // CSS style
    out.push_str("<style>");
    out.push_str(&build_style(svg_id, ff));
    out.push_str("</style>");

    // Empty <g> for compatibility
    out.push_str("<g></g>");

    // Arrow markers
    out.push_str(&build_markers(svg_id));

    // Main block group
    out.push_str("<g class=\"block\">");

    // Render nodes in insertion order
    for (id, node) in &diag.nodes {
        if let Some(&(cx, cy, w, h)) = final_pos.get(id.as_str()) {
            out.push_str(&render_node(node, cx, cy, w, h));
        }
    }

    // Render edges
    for edge in &diag.edges {
        if let (Some(&(fx, fy, fw, _fh)), Some(&(tx, ty, tw, _th))) =
            (final_pos.get(&edge.from), final_pos.get(&edge.to))
        {
            out.push_str(&render_edge(edge, fx, fy, fw, tx, ty, tw, svg_id));
        }
    }

    out.push_str("</g>");
    out.push_str("</svg>");
    out
}

/// Render a single node with foreignObject label (matches Mermaid structure).
fn render_node(node: &BlockNode, cx: f64, cy: f64, w: f64, h: f64) -> String {
    let hw = w / 2.0;
    let hh = h / 2.0;

    let mut s = String::new();
    // Mermaid uses the SVG id as prefix for node ids
    s.push_str(&node_group(
        "mermaid-block",
        &esc(&node.id),
        &fmt(cx),
        &fmt(cy),
    ));

    match node.shape {
        BlockShape::Square | BlockShape::Default => {
            s.push_str(&node_rect_square(&fmt(-hw), &fmt(-hh), &fmt(w), &fmt(h)));
        }
        BlockShape::RoundedRect => {
            s.push_str(&node_rect_rounded(&fmt(-hw), &fmt(-hh), &fmt(w), &fmt(h)));
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
            s.push_str(&node_diamond(&pts));
        }
        BlockShape::Circle => {
            let r = hw.min(hh);
            s.push_str(&node_circle(&fmt(r)));
        }
        BlockShape::Cylinder => {
            let ry_val = 7.0_f64;
            let body_h = h - ry_val;
            let fill = "#ECECFF";
            let stroke = "#9370DB";
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
            s.push_str(&node_hexagon(&pts));
        }
    }

    // Label using foreignObject — use browser-scaled width to avoid text clipping.
    let tw = text_width(&node.label);
    // Mermaid places label group at: translate(-(tw/2), -(h/2-pad_top))
    // where pad_top makes the label appear centered within the node.
    // From reference: label y-translate = -(hh - (h - fo_h)/2) = -(hh - pad_v)
    // With h=32, fo_h=24: pad_v = (32-24)/2 = 4. So label_ty = -(16-4) = -12. ✓
    let fo_h = (FONT_SIZE * 1.5).round(); // 24.0
    let pad_v = (h - fo_h) / 2.0;
    let label_ty = -(hh - pad_v); // = -hh + pad_v = -(h/2) + (h-fo_h)/2

    s.push_str(&node_label_fo(
        &fmt(-(tw / 2.0)),
        &fmt(label_ty),
        &fmt(tw),
        &fmt(fo_h),
        &esc(&node.label),
    ));

    s.push_str("</g>");
    s
}

/// Render an edge as a cubic bezier path with arrowhead (matches Mermaid structure).
#[allow(clippy::too_many_arguments)]
fn render_edge(
    edge: &BlockEdge,
    fx: f64,
    fy: f64,
    fw: f64,
    tx: f64,
    ty: f64,
    tw: f64,
    svg_id: &str,
) -> String {
    // Edge goes from right edge of source to left edge of target
    // (assuming source is to the left of target in typical layouts)
    let sx = fx + fw / 2.0; // source right edge
    let sy = fy;
    // Trim 4px before target left edge so arrowhead tip touches (not enters) the block.
    // block-pointEnd: viewBox=10, refX=6, markerWidth=12 → tip overhang=(10-6)*1.2=4.8px;
    // empirically 4px trim produces 0.8px overlap matching the reference.
    let ex = tx - tw / 2.0 - 4.0;
    let ey = ty;

    let mid_x = (sx + ex) / 2.0;

    // Mermaid uses a multi-segment cubic bezier similar to dagre's edge routing.
    // For a horizontal edge: M sx,sy L sx+step,sy C ... L ex,ey
    let dx = ex - sx;
    let step = (dx * 0.15).abs().min(dx.abs() / 2.0);

    let path = if (sy - ey).abs() < 0.01 {
        // Horizontal edge — use Mermaid's style: M pt L pt C pt pt pt pt L pt
        format!(
            "M{},{} L{},{} C{},{},{},{},{},{} L{},{}",
            fmt(sx),
            fmt(sy),
            fmt(sx + step),
            fmt(sy),
            fmt(mid_x - step),
            fmt(sy),
            fmt(mid_x + step),
            fmt(ey),
            fmt(ex - step),
            fmt(ey),
            fmt(ex),
            fmt(ey),
        )
    } else {
        // Non-horizontal: simple bezier
        format!(
            "M{},{} C{},{},{},{},{},{}",
            fmt(sx),
            fmt(sy),
            fmt(mid_x),
            fmt(sy),
            fmt(mid_x),
            fmt(ey),
            fmt(ex),
            fmt(ey),
        )
    };

    let edge_id = format!("{}-1-{}-{}", svg_id, edge.from, edge.to);
    let marker_end = format!("url(#{}_block-pointEnd)", svg_id);

    let mut s = String::new();
    s.push_str(&edge_path(
        &path,
        &edge_id,
        &edge.from.to_lowercase(),
        &edge.to.to_lowercase(),
        &marker_end,
    ));

    // Edge label if present
    if let Some(ref label) = edge.label {
        if !label.is_empty() {
            let mid_y = (sy + ey) / 2.0;
            s.push_str(&edge_label_text(
                &fmt(mid_x),
                &fmt(mid_y - 5.0),
                &esc(label),
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
