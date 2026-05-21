/// Faithful Rust port of Mermaid's wardleyRenderer.ts.
///
/// Key rendering details:
/// - Default canvas: 900×600 px with 48px padding on all sides
/// - X axis = Evolution (left=Genesis, right=Commodity)
/// - Y axis = Visibility (top=visible=100, bottom=invisible=0)
/// - Nodes store coordinates as 0-100 percentage values
/// - projectX(v) = padding + v/100 * chartWidth
/// - projectY(v) = height - padding - v/100 * chartHeight
/// - Anchors: text-only (no circle), bold, centered
/// - Components: circle r=6 with label
/// - Trends: red dashed arrows showing future evolution positions
use super::constants::*;
use super::parser::{Sourcing, WardleyDiagram, WardleyNode, WardleyNodeKind};
use super::templates::{self, esc, fmt_f};
use crate::theme::Theme;

pub fn render(diag: &WardleyDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let line_color = vars.line_color;
    let text_color = vars.text_color;
    // Wardley chart background: dark theme uses #333 (lighter than page bg), others use white
    let bg = if matches!(theme, crate::theme::Theme::Dark) {
        "#333333"
    } else {
        vars.background
    };
    // Trend/evolve arrow color: dark theme uses a lighter red (#ff6b6b), others use standard red
    let trend_color = if matches!(theme, crate::theme::Theme::Dark) {
        "#ff6b6b"
    } else {
        "#dc3545"
    };
    let width = diag.width;
    let height = diag.height;
    let chart_width = width - PADDING * 2.0;
    let chart_height = height - PADDING * 2.0;

    // SVG ID for marker references
    let svg_id = "wardley-svg";

    let mut out = String::new();

    // ── SVG root ──────────────────────────────────────────────────────────────
    out.push_str(&templates::svg_root(width, height));

    // ── <defs> with arrow markers ─────────────────────────────────────────────
    out.push_str(&templates::defs_block(svg_id, line_color, trend_color));

    // ── wardley-map group ─────────────────────────────────────────────────────
    out.push_str("<g class=\"wardley-map\">");

    // ── Background rect ───────────────────────────────────────────────────────
    out.push_str(&templates::background_rect(width, height, bg));

    // ── Title ─────────────────────────────────────────────────────────────────
    if let Some(ref title) = diag.title {
        out.push_str(&templates::title_text(
            width / 2.0,
            PADDING / 2.0,
            TITLE_FONT_SIZE,
            &esc(title),
            text_color,
        ));
    }

    // ── Axes ──────────────────────────────────────────────────────────────────
    out.push_str("<g class=\"wardley-axes\">");
    // X axis (bottom)
    out.push_str(&templates::axis_line(
        PADDING,
        width - PADDING,
        height - PADDING,
        height - PADDING,
        line_color,
    ));
    // Y axis (left)
    out.push_str(&templates::axis_line(
        PADDING,
        PADDING,
        PADDING,
        height - PADDING,
        line_color,
    ));
    // X axis label "Evolution"
    out.push_str(&templates::axis_label_x(
        PADDING + chart_width / 2.0,
        height - PADDING / 4.0,
        AXIS_FONT_SIZE,
        text_color,
    ));
    // Y axis label "Visibility" (rotated)
    let ry = PADDING + chart_height / 2.0;
    let rx = PADDING / 3.0;
    out.push_str(&templates::axis_label_y(rx, ry, AXIS_FONT_SIZE, text_color));
    out.push_str("</g>"); // wardley-axes

    // ── Evolution stages ──────────────────────────────────────────────────────
    let stages = &diag.evolution.stages;
    if !stages.is_empty() {
        out.push_str("<g class=\"wardley-stages\">");

        // Build stage positions as (start_0_100, end_0_100)
        let stage_positions: Vec<(f64, f64)> = stages
            .iter()
            .enumerate()
            .map(|(i, (_, start))| {
                let end = if i + 1 < stages.len() {
                    stages[i + 1].1
                } else {
                    100.0
                };
                (*start, end)
            })
            .collect();

        for (i, (stage_label, _)) in stages.iter().enumerate() {
            let (start_pct, end_pct) = stage_positions[i];
            let start_x = PADDING + start_pct / 100.0 * chart_width;
            let end_x = PADDING + end_pct / 100.0 * chart_width;
            let center_x = (start_x + end_x) / 2.0;

            // Stage boundary vertical dashed line (not for first stage)
            if i > 0 {
                out.push_str(&templates::stage_line(start_x, PADDING, height - PADDING));
            }

            // Stage label below x axis
            out.push_str(&templates::stage_label(
                center_x,
                height - PADDING / 1.5,
                STAGE_FONT_SIZE,
                &esc(stage_label),
                text_color,
            ));
        }

        out.push_str("</g>"); // wardley-stages
    }

    // ── Compute node positions ────────────────────────────────────────────────
    // positions[i] = (svgX, svgY) for diag.nodes[i]
    let positions: Vec<(f64, f64)> = diag
        .nodes
        .iter()
        .map(|n| {
            (
                project_x(n.evolution, PADDING, chart_width),
                project_y(n.visibility, height, PADDING, chart_height),
            )
        })
        .collect();

    let find_pos = |id: &str| -> Option<(f64, f64)> {
        diag.nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.id == id || n.label == id)
            .map(|(i, _)| positions[i])
    };

    // ── Links ─────────────────────────────────────────────────────────────────
    out.push_str("<g class=\"wardley-links\">");
    for link in &diag.links {
        let src_pos = find_pos(&link.from);
        let tgt_pos = find_pos(&link.to);
        if let (Some((x1, y1)), Some((x2, y2))) = (src_pos, tgt_pos) {
            let (sx1, sy1, sx2, sy2) = shorten_line(x1, y1, x2, y2, NODE_RADIUS, NODE_RADIUS);
            let dash_attr = if link.dashed {
                " stroke-dasharray=\"6 6\"".to_string()
            } else {
                String::new()
            };

            out.push_str(&templates::link_line(
                &fmt_f(sx1),
                &fmt_f(sy1),
                &fmt_f(sx2),
                &fmt_f(sy2),
                &dash_attr,
                line_color,
            ));

            // Link label
            if let Some(ref label) = link.label {
                let mx = (x1 + x2) / 2.0;
                let my = (y1 + y2) / 2.0;
                out.push_str(&templates::link_label(
                    &fmt_f(mx),
                    &fmt_f(my - 4.0),
                    LABEL_FONT_SIZE,
                    &esc(label),
                    text_color,
                ));
            }
        }
    }
    out.push_str("</g>"); // wardley-links

    // ── Trends (evolve arrows) ────────────────────────────────────────────────
    out.push_str("<g class=\"wardley-trends\">");
    for trend in &diag.trends {
        if let Some((ox, oy)) = find_pos(&trend.node_id) {
            let target_x = project_x(trend.target_x, PADDING, chart_width);
            let target_y = project_y(trend.target_y, height, PADDING, chart_height);
            let dx = target_x - ox;
            let dy = target_y - oy;
            let dist = (dx * dx + dy * dy).sqrt();
            let shorten_by = NODE_RADIUS + 2.0;
            let (ax2, ay2) = if dist > shorten_by {
                (
                    target_x - dx / dist * shorten_by,
                    target_y - dy / dist * shorten_by,
                )
            } else {
                (target_x, target_y)
            };
            out.push_str(&templates::trend_arrow(
                &fmt_f(ox),
                &fmt_f(oy),
                &fmt_f(ax2),
                &fmt_f(ay2),
                svg_id,
                trend_color,
            ));
        }
    }
    out.push_str("</g>"); // wardley-trends

    // ── Nodes ─────────────────────────────────────────────────────────────────
    out.push_str("<g class=\"wardley-nodes\">");
    for (i, node) in diag.nodes.iter().enumerate() {
        let (cx, cy) = positions[i];
        render_node(&mut out, node, cx, cy, line_color, text_color, bg);
    }
    out.push_str("</g>"); // wardley-nodes

    // ── Annotations ───────────────────────────────────────────────────────────
    if !diag.annotations.is_empty() {
        out.push_str("<g class=\"wardley-annotations\">");
        for ann in &diag.annotations {
            let ax = project_x(ann.evolution, PADDING, chart_width);
            let ay = project_y(ann.visibility, height, PADDING, chart_height);
            out.push_str(&templates::annotation(
                &fmt_f(ax),
                &fmt_f(ay),
                ann.number,
                line_color,
                bg,
                text_color,
            ));
        }
        out.push_str("</g>"); // wardley-annotations
    }

    out.push_str("</g>"); // wardley-map
    out.push_str("</svg>");
    out
}

/// Map evolution percentage (0-100) to SVG x coordinate.
fn project_x(value: f64, padding: f64, chart_width: f64) -> f64 {
    padding + value / 100.0 * chart_width
}

/// Map visibility percentage (0-100) to SVG y coordinate.
/// High visibility → top of chart (small y); low visibility → bottom (large y).
fn project_y(value: f64, height: f64, padding: f64, chart_height: f64) -> f64 {
    height - padding - value / 100.0 * chart_height
}

fn render_node(
    out: &mut String,
    node: &WardleyNode,
    cx: f64,
    cy: f64,
    line_color: &str,
    text_color: &str,
    bg: &str,
) {
    let class_suffix = match node.kind {
        WardleyNodeKind::Anchor => "anchor",
        WardleyNodeKind::Note => "note",
        WardleyNodeKind::Component => "component",
    };
    out.push_str(&templates::node_group_open(class_suffix));

    match node.kind {
        WardleyNodeKind::Anchor => {
            // Anchors: text only, bold, centered
            let lx = node.label_offset_x.map(|dx| cx + dx).unwrap_or(cx);
            let ly = node.label_offset_y.map(|dy| cy + dy).unwrap_or(cy - 3.0);
            out.push_str(&templates::anchor_label(
                &fmt_f(lx),
                &fmt_f(ly),
                LABEL_FONT_SIZE,
                &esc(&node.label),
                text_color,
            ));
        }
        WardleyNodeKind::Note => {
            // Notes: text only
            out.push_str(&templates::note_text(
                &fmt_f(cx),
                &fmt_f(cy),
                &esc(&node.label),
                text_color,
            ));
        }
        WardleyNodeKind::Component => {
            // Sourcing overlays
            match node.sourcing {
                Sourcing::Outsource => {
                    out.push_str(&templates::sourcing_overlay_circle(
                        "wardley-outsource-overlay",
                        &fmt_f(cx),
                        &fmt_f(cy),
                        NODE_RADIUS * 2.0,
                        "#666",
                        line_color,
                    ));
                }
                Sourcing::Buy => {
                    out.push_str(&templates::sourcing_overlay_circle(
                        "wardley-buy-overlay",
                        &fmt_f(cx),
                        &fmt_f(cy),
                        NODE_RADIUS * 2.0,
                        "#ccc",
                        line_color,
                    ));
                }
                Sourcing::Build => {
                    out.push_str(&templates::sourcing_overlay_circle(
                        "wardley-build-overlay",
                        &fmt_f(cx),
                        &fmt_f(cy),
                        NODE_RADIUS * 2.0,
                        "#eee",
                        "#000",
                    ));
                }
                _ => {}
            }

            // Component circle
            out.push_str(&templates::component_circle(
                &fmt_f(cx),
                &fmt_f(cy),
                NODE_RADIUS,
                line_color,
                bg,
            ));

            // Inertia: vertical line to the right of the node
            if node.inertia {
                let line_x = cx + NODE_RADIUS + 15.0;
                let half_h = NODE_RADIUS;
                out.push_str(&templates::inertia_line(
                    &fmt_f(line_x),
                    &fmt_f(cy - half_h),
                    &fmt_f(cy + half_h),
                    line_color,
                ));
            }

            // Component label
            let lx = cx + node.label_offset_x.unwrap_or(NODE_LABEL_OFFSET);
            let ly = cy + node.label_offset_y.unwrap_or(-NODE_LABEL_OFFSET);
            out.push_str(&templates::component_label(
                &fmt_f(lx),
                &fmt_f(ly),
                LABEL_FONT_SIZE,
                &esc(&node.label),
                text_color,
            ));
        }
    }

    out.push_str("</g>");
}

/// Shorten a line segment: start end moves `r1` toward target, end moves `r2` toward source.
fn shorten_line(x1: f64, y1: f64, x2: f64, y2: f64, r1: f64, r2: f64) -> (f64, f64, f64, f64) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < r1 + r2 + 1.0 {
        return (x1, y1, x2, y2);
    }
    let ux = dx / len;
    let uy = dy / len;
    (x1 + ux * r1, y1 + uy * r1, x2 - ux * r2, y2 - uy * r2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::wardley::parser;

    #[test]
    fn render_produces_svg() {
        let input = "wardley\n    title My Wardley Map\n    component UserNeed [0.9, 0.1]\n    component Backend [0.5, 0.7]\n    UserNeed->Backend\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("wardley-node--component"));
        assert!(svg.contains("wardley-link"));
    }

    #[test]
    fn render_wardley_beta_corpus() {
        let input = "wardley-beta\ntitle Tea Shop\nanchor Business [0.95, 0.63]\nanchor Public [0.95, 0.78]\ncomponent Cup of Tea [0.79, 0.61] label [19, -4]\ncomponent Cup [0.73, 0.78]\ncomponent Tea [0.63, 0.81]\ncomponent Hot Water [0.52, 0.80]\ncomponent Water [0.38, 0.82]\ncomponent Kettle [0.43, 0.35] label [-57, 4]\ncomponent Power [0.1, 0.7] label [-27, 20]\nBusiness -> Cup of Tea\nPublic -> Cup of Tea\nCup of Tea -> Cup\nCup of Tea -> Tea\nCup of Tea -> Hot Water\nHot Water -> Water\nHot Water -> Kettle\nKettle -> Power\nevolve Kettle 0.62\nevolve Power 0.89\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(
            svg.contains("viewBox=\"0 0 900 600\""),
            "expected 900x600 viewBox, got: {}",
            &svg[..200]
        );
        assert!(svg.contains("Tea Shop"));
        assert!(svg.contains("wardley-trend"));
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "wardley\n    title My Wardley Map\n    component UserNeed [0.9, 0.1]\n    component Backend [0.5, 0.7]\n    UserNeed->Backend\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
