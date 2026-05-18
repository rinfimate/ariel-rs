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
use super::templates;
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

    out.push_str(&format!(
        "<svg id=\"{}\" width=\"100%\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" style=\"max-width: {}px;\" viewBox=\"{} {} {} {}\" role=\"graphics-document document\" aria-roledescription=\"block\">",
        svg_id,
        fmt_px(vb_w),
        fmt(vb_x), fmt(vb_y), fmt(vb_w), fmt(vb_h),
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
    s.push_str(&format!(
        "<g class=\"node default default flowchart-label\" id=\"mermaid-block-{}\" transform=\"translate({}, {})\">",
        esc(&node.id), fmt(cx), fmt(cy),
    ));

    match node.shape {
        BlockShape::Square | BlockShape::Default => {
            s.push_str(&format!(
                "<rect class=\"basic label-container\" style=\"\" rx=\"0\" ry=\"0\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"></rect>",
                fmt(-hw), fmt(-hh), fmt(w), fmt(h),
            ));
        }
        BlockShape::RoundedRect => {
            s.push_str(&format!(
                "<rect class=\"basic label-container\" style=\"\" rx=\"8\" ry=\"8\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"></rect>",
                fmt(-hw), fmt(-hh), fmt(w), fmt(h),
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
            s.push_str(&format!(
                "<polygon points=\"{}\" class=\"basic label-container\" style=\"\"></polygon>",
                pts,
            ));
        }
        BlockShape::Circle => {
            let r = hw.min(hh);
            s.push_str(&format!(
                "<circle cx=\"0\" cy=\"0\" r=\"{}\" class=\"basic label-container\" style=\"\"></circle>",
                fmt(r),
            ));
        }
        BlockShape::Cylinder => {
            let ry_val = 7.0_f64;
            let body_h = h - ry_val;
            let fill = "#ECECFF";
            let stroke = "#9370DB";
            s.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"fill:{};stroke:{};stroke-width:1;\"></rect>",
                fmt(-hw), fmt(-hh + ry_val), fmt(w), fmt(body_h), fill, stroke,
            ));
            s.push_str(&format!(
                "<ellipse cx=\"0\" cy=\"{}\" rx=\"{}\" ry=\"{}\" style=\"fill:{};stroke:{};stroke-width:1;\"></ellipse>",
                fmt(-hh + ry_val), fmt(hw), fmt(ry_val), fill, stroke,
            ));
            s.push_str(&format!(
                "<ellipse cx=\"0\" cy=\"{}\" rx=\"{}\" ry=\"{}\" style=\"fill:{};stroke:{};stroke-width:1;\"></ellipse>",
                fmt(-hh + ry_val + body_h), fmt(hw), fmt(ry_val), fill, stroke,
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
            s.push_str(&format!(
                "<polygon points=\"{}\" class=\"basic label-container\" style=\"\"></polygon>",
                pts,
            ));
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

    s.push_str(&format!(
        "<g class=\"label\" style=\"\" transform=\"translate({}, {})\">",
        fmt(-(tw / 2.0)),
        fmt(label_ty),
    ));
    s.push_str("<rect></rect>");
    s.push_str(&format!(
        "<foreignObject width=\"{}\" height=\"{}\">",
        fmt(tw),
        fmt(fo_h),
    ));
    s.push_str("<div xmlns=\"http://www.w3.org/1999/xhtml\" style=\"display: table-cell; white-space: nowrap; line-height: 1.5;\">");
    s.push_str(&format!(
        "<span class=\"nodeLabel \"><p>{}</p></span>",
        esc(&node.label)
    ));
    s.push_str("</div>");
    s.push_str("</foreignObject>");
    s.push_str("</g>");

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
    s.push_str(&format!(
        "<path d=\"{}\" id=\"{}\" class=\" edge-thickness-normal edge-pattern-solid edge-thickness-normal edge-pattern-solid flowchart-link LS-{} LE-{}\" marker-end=\"{}\"></path>",
        path, edge_id,
        edge.from.to_lowercase(), edge.to.to_lowercase(),
        marker_end,
    ));

    // Edge label if present
    if let Some(ref label) = edge.label {
        if !label.is_empty() {
            let mid_y = (sy + ey) / 2.0;
            s.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"12\" font-family=\"Arial, sans-serif\" fill=\"#333\">{}</text>",
                fmt(mid_x), fmt(mid_y - 5.0), esc(label),
            ));
        }
    }

    s
}

/// Build the CSS style block (matches Mermaid's block diagram style exactly).
fn build_style(id: &str, ff: &str) -> String {
    let mut c = String::new();
    c.push_str(&format!(
        "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}"
    ));
    c.push_str("@keyframes edge-animation-frame{from{stroke-dashoffset:0;}}");
    c.push_str("@keyframes dash{to{stroke-dashoffset:0;}}");
    c.push_str(&format!("#{id} .edge-animation-slow{{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 50s linear infinite;stroke-linecap:round;}}"));
    c.push_str(&format!("#{id} .edge-animation-fast{{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 20s linear infinite;stroke-linecap:round;}}"));
    c.push_str(&format!("#{id} .error-icon{{fill:#552222;}}"));
    c.push_str(&format!(
        "#{id} .error-text{{fill:#552222;stroke:#552222;}}"
    ));
    c.push_str(&format!(
        "#{id} .edge-thickness-normal{{stroke-width:1px;}}"
    ));
    c.push_str(&format!(
        "#{id} .edge-thickness-thick{{stroke-width:3.5px;}}"
    ));
    c.push_str(&format!("#{id} .edge-pattern-solid{{stroke-dasharray:0;}}"));
    c.push_str(&format!(
        "#{id} .edge-thickness-invisible{{stroke-width:0;fill:none;}}"
    ));
    c.push_str(&format!(
        "#{id} .edge-pattern-dashed{{stroke-dasharray:3;}}"
    ));
    c.push_str(&format!(
        "#{id} .edge-pattern-dotted{{stroke-dasharray:2;}}"
    ));
    c.push_str(&format!("#{id} .marker{{fill:#333333;stroke:#333333;}}"));
    c.push_str(&format!("#{id} .marker.cross{{stroke:#333333;}}"));
    c.push_str(&format!("#{id} svg{{font-family:{ff};font-size:16px;}}"));
    c.push_str(&format!("#{id} p{{margin:0;}}"));
    c.push_str(&format!("#{id} .label{{font-family:{ff};color:#333;}}"));
    c.push_str(&format!("#{id} .cluster-label text{{fill:#333;}}"));
    c.push_str(&format!("#{id} .cluster-label span,#{id} p{{color:#333;}}"));
    c.push_str(&format!(
        "#{id} .label text,#{id} span,#{id} p{{fill:#333;color:#333;}}"
    ));
    c.push_str(&format!("#{id} .node rect,#{id} .node circle,#{id} .node ellipse,#{id} .node polygon,#{id} .node path{{fill:#ECECFF;stroke:#9370DB;stroke-width:1px;}}"));
    c.push_str(&format!(
        "#{id} .flowchart-label text{{text-anchor:middle;}}"
    ));
    c.push_str(&format!("#{id} .node .label{{text-align:center;}}"));
    c.push_str(&format!("#{id} .node.clickable{{cursor:pointer;}}"));
    c.push_str(&format!("#{id} .arrowheadPath{{fill:#333333;}}"));
    c.push_str(&format!(
        "#{id} .edgePath .path{{stroke:#333333;stroke-width:2.0px;}}"
    ));
    c.push_str(&format!(
        "#{id} .flowchart-link{{stroke:#333333;fill:none;}}"
    ));
    c.push_str(&format!(
        "#{id} .edgeLabel{{background-color:rgba(232,232,232, 0.8);text-align:center;}}"
    ));
    c.push_str(&format!(
        "#{id} .edgeLabel p{{margin:0;padding:0;display:inline;}}"
    ));
    c.push_str(&format!("#{id} .edgeLabel rect{{opacity:0.5;background-color:rgba(232,232,232, 0.8);fill:rgba(232,232,232, 0.8);}}"));
    c.push_str(&format!(
        "#{id} .labelBkg{{background-color:rgba(232,232,232, 0.8);}}"
    ));
    c.push_str(&format!("#{id} .node .cluster{{fill:rgba(255, 255, 222, 0.5);stroke:rgba(170, 170, 51, 0.2);box-shadow:rgba(50, 50, 93, 0.25) 0px 13px 27px -5px,rgba(0, 0, 0, 0.3) 0px 8px 16px -8px;stroke-width:1px;}}"));
    c.push_str(&format!("#{id} .cluster text{{fill:#333;}}"));
    c.push_str(&format!("#{id} .cluster span,#{id} p{{color:#333;}}"));
    c.push_str(&format!("#{id} div.mermaidTooltip{{position:absolute;text-align:center;max-width:200px;padding:2px;font-family:{ff};font-size:12px;background:hsl(80, 100%, 96.2745098039%);border:1px solid #aaaa33;border-radius:2px;pointer-events:none;z-index:100;}}"));
    c.push_str(&format!(
        "#{id} .flowchartTitleText{{text-anchor:middle;font-size:18px;fill:#333;}}"
    ));
    c.push_str(&format!("#{id} .label-icon{{display:inline-block;height:1em;overflow:visible;vertical-align:-0.125em;}}"));
    c.push_str(&format!(
        "#{id} .node .label-icon path{{fill:currentColor;stroke:revert;stroke-width:revert;}}"
    ));
    c.push_str(&format!("#{id} .node .neo-node{{stroke:#9370DB;}}"));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node rect,#{id} [data-look=\"neo\"].cluster rect,#{id} [data-look=\"neo\"].node polygon{{stroke:#9370DB;filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node path{{stroke:#9370DB;stroke-width:1px;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node .outer-path{{filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node .neo-line path{{stroke:#9370DB;filter:none;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].node circle{{stroke:#9370DB;filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!(
        "#{id} [data-look=\"neo\"].node circle .state-start{{fill:#000000;}}"
    ));
    c.push_str(&format!("#{id} [data-look=\"neo\"].icon-shape .icon{{fill:#9370DB;filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!("#{id} [data-look=\"neo\"].icon-shape .icon-neo path{{stroke:#9370DB;filter:drop-shadow(1px 2px 2px rgba(185, 185, 185, 1));}}"));
    c.push_str(&format!("#{id} :root{{--mermaid-font-family:{ff};}}"));
    c
}

/// Build marker definitions matching Mermaid's block diagram markers.
fn build_markers(svg_id: &str) -> String {
    let mut m = String::new();

    m.push_str(&format!(
        "<marker id=\"{svg_id}_block-pointEnd\" class=\"marker block\" viewBox=\"0 0 10 10\" refX=\"6\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"12\" markerHeight=\"12\" orient=\"auto\"><path d=\"M 0 0 L 10 5 L 0 10 z\" class=\"arrowMarkerPath\" style=\"stroke-width: 1; stroke-dasharray: 1, 0;\"></path></marker>"
    ));
    m.push_str(&format!(
        "<marker id=\"{svg_id}_block-pointStart\" class=\"marker block\" viewBox=\"0 0 10 10\" refX=\"4.5\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"12\" markerHeight=\"12\" orient=\"auto\"><path d=\"M 0 5 L 10 10 L 10 0 z\" class=\"arrowMarkerPath\" style=\"stroke-width: 1; stroke-dasharray: 1, 0;\"></path></marker>"
    ));
    m.push_str(&format!(
        "<marker id=\"{svg_id}_block-circleEnd\" class=\"marker block\" viewBox=\"0 0 10 10\" refX=\"11\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"11\" markerHeight=\"11\" orient=\"auto\"><circle cx=\"5\" cy=\"5\" r=\"5\" class=\"arrowMarkerPath\" style=\"stroke-width: 1; stroke-dasharray: 1, 0;\"></circle></marker>"
    ));
    m.push_str(&format!(
        "<marker id=\"{svg_id}_block-circleStart\" class=\"marker block\" viewBox=\"0 0 10 10\" refX=\"-1\" refY=\"5\" markerUnits=\"userSpaceOnUse\" markerWidth=\"11\" markerHeight=\"11\" orient=\"auto\"><circle cx=\"5\" cy=\"5\" r=\"5\" class=\"arrowMarkerPath\" style=\"stroke-width: 1; stroke-dasharray: 1, 0;\"></circle></marker>"
    ));
    m.push_str(&format!(
        "<marker id=\"{svg_id}_block-crossEnd\" class=\"marker cross block\" viewBox=\"0 0 11 11\" refX=\"12\" refY=\"5.2\" markerUnits=\"userSpaceOnUse\" markerWidth=\"11\" markerHeight=\"11\" orient=\"auto\"><path d=\"M 1,1 l 9,9 M 10,1 l -9,9\" class=\"arrowMarkerPath\" style=\"stroke-width: 2; stroke-dasharray: 1, 0;\"></path></marker>"
    ));
    m.push_str(&format!(
        "<marker id=\"{svg_id}_block-crossStart\" class=\"marker cross block\" viewBox=\"0 0 11 11\" refX=\"-1\" refY=\"5.2\" markerUnits=\"userSpaceOnUse\" markerWidth=\"11\" markerHeight=\"11\" orient=\"auto\"><path d=\"M 1,1 l 9,9 M 10,1 l -9,9\" class=\"arrowMarkerPath\" style=\"stroke-width: 2; stroke-dasharray: 1, 0;\"></path></marker>"
    ));

    m
}

/// Format a float for SVG attributes — drop trailing zeros, max 3 decimal places.
fn fmt(v: f64) -> String {
    let s = format!("{:.3}", v);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

/// Format for pixel-based max-width (Mermaid uses fractional pixels).
fn fmt_px(v: f64) -> String {
    let s = format!("{:.6}", v);
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
    #[ignore = "platform-specific float precision — run locally"]
    fn snapshot_default_theme() {
        let input = "block-beta\n    columns 3\n    A[\"A\"]:1\n    B[\"B\"]:1\n    C[\"C\"]:1\n    space:1\n    D[\"D\"]:1\n    space:1";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(svg);
    }
}
