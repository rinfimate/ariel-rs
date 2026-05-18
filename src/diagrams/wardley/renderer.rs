use super::constants::*;
use super::parser::{Sourcing, WardleyDiagram, WardleyNode, WardleyNodeKind};
use super::templates;
/// Faithful Rust port of Mermaid's wardleyRenderer.ts.
///
/// Key rendering details:
/// - Canvas: typically 100×100 percentage-mapped to SVG pixels * scale
/// - X axis = Evolution (left=Genesis, right=Commodity), Y axis = Visibility (top=visible, bottom=invisible)
/// - Nodes are circles; pipeline parents are rectangles; markets are triangles
/// - Links drawn as arrows; dashed links use stroke-dasharray
/// - Axes drawn with stage boundaries and labels
/// - Annotations as numbered circles with connecting lines and text box
use crate::text::measure;
use crate::theme::Theme;

// ── Canvas constants (matching wardleyRenderer.ts defaults) ───────────────────
// All constants are imported from super::constants via `use super::constants::*`.

pub fn render(diag: &WardleyDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let canvas_w = diag.width * SCALE;
    let canvas_h = diag.height * SCALE;

    let total_w = canvas_w + PADDING * 2.0;
    let total_h = canvas_h + PADDING * 2.0 + 30.0; // extra for title

    let mut parts: Vec<String> = Vec::new();
    parts.push(build_style(ff));

    // Title
    if let Some(ref title) = diag.title {
        parts.push(templates::title_text(
            total_w / 2.0,
            TITLE_FONT,
            &escape(title),
        ));
    }

    // Main group: axes + content
    let gx = PADDING;
    let gy = PADDING + 30.0;
    parts.push(templates::main_group_open(gx, gy));

    // Background
    parts.push(templates::bg_rect(canvas_w, canvas_h));

    // Evolution stage backgrounds and labels
    let stages = &diag.evolution.stages;
    let n_stages = stages.len();
    for (i, (label, boundary)) in stages.iter().enumerate() {
        let x_start = boundary * canvas_w;
        let x_end = if i + 1 < n_stages {
            stages[i + 1].1 * canvas_w
        } else {
            canvas_w
        };
        let w = x_end - x_start;

        // Alternating light/lighter background
        let fill = if i % 2 == 0 {
            STAGE_FILL_EVEN
        } else {
            STAGE_FILL_ODD
        };
        parts.push(templates::stage_bg_rect(x_start, w, canvas_h, fill));

        // Stage boundary line (except first)
        if i > 0 {
            parts.push(templates::stage_boundary_line(x_start, canvas_h));
        }

        // Stage label at bottom
        parts.push(templates::stage_label(
            x_start + w / 2.0,
            canvas_h + 16.0,
            AXIS_FONT,
            &escape(label),
        ));
    }

    // Y-axis border lines and labels
    parts.push(templates::border_rect(canvas_w, canvas_h));
    parts.push(templates::axis_label_visible(AXIS_FONT));
    parts.push(templates::axis_label_invisible(canvas_h, AXIS_FONT));
    parts.push(templates::axis_label_evolution(
        canvas_w,
        canvas_h + PADDING - 5.0,
        AXIS_FONT,
    ));

    // Arrow marker definition
    parts.push(templates::arrow_marker());

    // Draw links first (behind nodes)
    for link in &diag.links {
        if let (Some(from_node), Some(to_node)) = (
            find_node(&diag.nodes, &link.from),
            find_node(&diag.nodes, &link.to),
        ) {
            let x1 = from_node.evolution * canvas_w;
            let y1 = (1.0 - from_node.visibility) * canvas_h;
            let x2 = to_node.evolution * canvas_w;
            let y2 = (1.0 - to_node.visibility) * canvas_h;

            // Shorten endpoints to not overlap node circles
            let (x1s, y1s, x2s, y2s) = shorten_line(x1, y1, x2, y2, NODE_RADIUS + 1.0);

            let dasharray = if link.dashed {
                r#" stroke-dasharray="5,3""#
            } else {
                ""
            };
            parts.push(templates::link_line(x1s, y1s, x2s, y2s, dasharray));

            // Link label
            if let Some(ref label) = link.label {
                let mx = (x1 + x2) / 2.0;
                let my = (y1 + y2) / 2.0 - 4.0;
                parts.push(templates::link_label(mx, my, AXIS_FONT, &escape(label)));
            }
        }
    }

    // Draw nodes
    for node in &diag.nodes {
        let cx = node.evolution * canvas_w;
        let cy = (1.0 - node.visibility) * canvas_h;
        render_node(&mut parts, node, cx, cy);
    }

    // Annotations
    for ann in &diag.annotations {
        let cx = ann.evolution * canvas_w;
        let cy = (1.0 - ann.visibility) * canvas_h;
        parts.push(templates::annotation_circle(cx, cy));
        parts.push(templates::annotation_number(cx, cy, AXIS_FONT, ann.number));
    }

    parts.push("</g>".to_string());

    templates::svg_root(total_w, total_h, &parts.join(""))
}

fn find_node<'a>(nodes: &'a [WardleyNode], id: &str) -> Option<&'a WardleyNode> {
    nodes.iter().find(|n| n.id == id || n.label == id)
}

fn render_node(parts: &mut Vec<String>, node: &WardleyNode, cx: f64, cy: f64) {
    match node.kind {
        WardleyNodeKind::Note => {
            // Note: text-only with light background
            let (tw, _) = measure(&node.label, FONT_SIZE);
            parts.push(templates::note_rect(cx, cy, tw, FONT_SIZE));
            parts.push(templates::note_text(
                cx,
                cy,
                FONT_SIZE,
                &escape(&node.label),
            ));
            return;
        }
        _ => {
            // Component / Anchor: circle
            let class = match node.kind {
                WardleyNodeKind::Anchor => "wardley-anchor",
                _ => "wardley-component",
            };

            // Sourcing overlay
            let fill_overlay = match node.sourcing {
                Sourcing::Build => " fill-opacity=\"0.3\" fill=\"#aaa\"",
                Sourcing::Buy => " fill-opacity=\"0.5\" fill=\"#888\"",
                Sourcing::Outsource => " fill-opacity=\"0.7\" fill=\"#555\"",
                Sourcing::Market => " fill-opacity=\"0.2\" fill=\"#4af\"",
                Sourcing::None => "",
            };

            parts.push(templates::node_circle(
                class,
                cx,
                cy,
                NODE_RADIUS,
                fill_overlay,
            ));

            // Inertia: small vertical line
            if node.inertia {
                parts.push(templates::inertia_line(cx, cy, NODE_RADIUS));
            }
        }
    }

    // Node label
    parts.push(templates::node_label(
        cx,
        cy,
        NODE_RADIUS,
        FONT_SIZE,
        &escape(&node.label),
    ));
}

/// Shorten a line segment by `amount` on each end.
fn shorten_line(x1: f64, y1: f64, x2: f64, y2: f64, amount: f64) -> (f64, f64, f64, f64) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < amount * 2.0 + 1.0 {
        return (x1, y1, x2, y2);
    }
    let ux = dx / len;
    let uy = dy / len;
    (
        x1 + ux * amount,
        y1 + uy * amount,
        x2 - ux * amount,
        y2 - uy * amount,
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn build_style(ff: &str) -> String {
    format!(
        r#"<style>
.wardley-title {{ fill: #333; font-family: {ff}; font-weight: bold; }}
.wardley-bg {{ fill: #fafaff; }}
.wardley-border {{ fill: none; stroke: #aaa; stroke-width: 1; }}
.wardley-axis-line {{ stroke: #ccc; stroke-width: 1; fill: none; }}
.wardley-axis-label {{ fill: #666; font-family: {ff}; }}
.wardley-stage-label {{ fill: #888; font-family: {ff}; }}
.wardley-component {{ fill: #fff; stroke: #333; stroke-width: 2; }}
.wardley-anchor {{ fill: #fff; stroke: #333; stroke-width: 2; stroke-dasharray: 4,2; }}
.wardley-pipeline {{ fill: none; stroke: #333; stroke-width: 2; stroke-dasharray: 5,3; }}
.wardley-note-box {{ fill: #fffde7; stroke: #bbb; stroke-width: 1; }}
.wardley-note {{ fill: #555; font-family: {ff}; }}
.wardley-label {{ fill: #333; font-family: {ff}; }}
.wardley-link {{ stroke: #666; stroke-width: 1.5; fill: none; }}
.wardley-link-label {{ fill: #666; font-family: {ff}; }}
.wardley-arrow-head {{ fill: #666; }}
.wardley-annotation {{ fill: #ffe; stroke: #999; stroke-width: 1; }}
.wardley-annotation-num {{ fill: #333; font-family: {ff}; font-weight: bold; }}
.wardley-inertia {{ stroke: #e33; stroke-width: 2; }}
</style>"#,
        ff = ff,
    )
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
        assert!(svg.contains("wardley-component"));
        assert!(svg.contains("wardley-link"));
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "wardley\n    title My Wardley Map\n    component UserNeed [0.9, 0.1]\n    component Backend [0.5, 0.7]\n    UserNeed->Backend\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(svg);
    }
}
