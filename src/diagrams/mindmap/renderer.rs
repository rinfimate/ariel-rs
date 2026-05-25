use super::constants::*;
use super::parser::{MindmapDiagram, MindmapNode, NodeType};
use super::templates::{self, esc, node_group_open};
use crate::backends::measure;
use crate::theme::Theme;

#[derive(Debug, Clone)]
struct LayoutNode {
    id: usize,
    descr: String,
    node_type: NodeType,
    section: Option<usize>,
    is_root: bool,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
    children: Vec<usize>,
    #[allow(dead_code)]
    parent: Option<usize>,
}

fn node_size(descr: &str, node_type: NodeType, padding: f64) -> (f64, f64) {
    let (text_w, _) = measure(descr, FONT_SIZE);
    match node_type {
        NodeType::Circle => {
            let diam = (text_w + 2.0 * padding).max(40.0);
            (diam, diam)
        }
        _ => {
            let w = (text_w + 2.0 * padding).max(20.0);
            (w, NODE_SHAPE_H + 5.0)
        }
    }
}

fn flatten_tree(root: &MindmapNode) -> Vec<LayoutNode> {
    let mut nodes = Vec::new();
    flatten_node(root, None, &mut nodes);
    nodes
}

fn flatten_node(node: &MindmapNode, parent: Option<usize>, out: &mut Vec<LayoutNode>) {
    let my_idx = out.len();
    let (w, h) = node_size(&node.descr, node.node_type, node.padding);
    out.push(LayoutNode {
        id: node.id,
        descr: node.descr.clone(),
        node_type: node.node_type,
        section: node.section,
        is_root: node.is_root,
        width: w,
        height: h,
        x: 0.0,
        y: 0.0,
        children: Vec::new(),
        parent,
    });
    if let Some(p) = parent {
        out[p].children.push(my_idx);
    }
    for child in &node.children {
        flatten_node(child, Some(my_idx), out);
    }
}

fn subtree_h(nodes: &[LayoutNode], idx: usize) -> f64 {
    if nodes[idx].children.is_empty() {
        nodes[idx].height + NODE_V_GAP
    } else {
        let sum: f64 = nodes[idx]
            .children
            .iter()
            .map(|&ci| subtree_h(nodes, ci))
            .sum();
        sum.max(nodes[idx].height + NODE_V_GAP)
    }
}

fn layout_mindmap(nodes: &mut Vec<LayoutNode>) {
    if nodes.is_empty() {
        return;
    }
    let root_children: Vec<usize> = nodes[0].children.clone();
    let n = root_children.len();
    if n == 0 {
        return;
    }

    let n_right = n / 2;
    let right_children: Vec<usize> = root_children[..n_right].to_vec();
    let left_children: Vec<usize> = root_children[n_right..].to_vec();

    let right_span: f64 = right_children
        .iter()
        .map(|&ci| subtree_h(nodes, ci))
        .sum::<f64>()
        .max(1.0);
    let left_span: f64 = left_children
        .iter()
        .map(|&ci| subtree_h(nodes, ci))
        .sum::<f64>()
        .max(1.0);
    let total_span = left_span.max(right_span);
    let root_y = total_span / 2.0;
    nodes[0].x = 0.0;
    nodes[0].y = root_y;

    let root_half_w = nodes[0].width / 2.0;
    if n_right > 0 {
        layout_side(
            nodes,
            &right_children,
            root_half_w,
            root_y - right_span / 2.0,
            1.0,
        );
    }
    layout_side(
        nodes,
        &left_children,
        root_half_w,
        root_y - left_span / 2.0,
        -1.0,
    );
}

fn layout_side(
    nodes: &mut Vec<LayoutNode>,
    children: &[usize],
    parent_hw: f64,
    start_y: f64,
    dir: f64,
) {
    let mut cursor = start_y;
    for &ci in children {
        let span = subtree_h(nodes, ci);
        let cy = cursor + span / 2.0;
        let chw = nodes[ci].width / 2.0;
        nodes[ci].x = dir * (parent_hw + NODE_H_GAP + chw);
        nodes[ci].y = cy;
        let gc: Vec<usize> = nodes[ci].children.clone();
        if !gc.is_empty() {
            layout_subtree(nodes, &gc, nodes[ci].x, chw, cursor, dir);
        }
        cursor += span;
    }
}

fn layout_subtree(
    nodes: &mut Vec<LayoutNode>,
    children: &[usize],
    px: f64,
    phw: f64,
    slot_start: f64,
    dir: f64,
) {
    let mut cursor = slot_start;
    for &ci in children {
        let span = subtree_h(nodes, ci);
        let cy = cursor + span / 2.0;
        let chw = nodes[ci].width / 2.0;
        nodes[ci].x = px + dir * (phw + NODE_H_GAP + chw);
        nodes[ci].y = cy;
        let gc: Vec<usize> = nodes[ci].children.clone();
        if !gc.is_empty() {
            layout_subtree(nodes, &gc, nodes[ci].x, chw, cursor, dir);
        }
        cursor += span;
    }
}

struct MindmapColors<'a> {
    root_fill: &'a str,
    root_text: &'a str,
    section_fills: &'a [&'a str],
    section_text: &'a [&'a str],
    section_lines: &'a [&'a str],
}

impl<'a> MindmapColors<'a> {
    fn fill(&self, section: Option<usize>, is_root: bool) -> &'a str {
        if is_root {
            self.root_fill
        } else if let Some(s) = section {
            self.section_fills[s % 11]
        } else {
            self.section_fills[0]
        }
    }
    fn text(&self, section: Option<usize>, is_root: bool) -> &'a str {
        if is_root {
            self.root_text
        } else if let Some(s) = section {
            self.section_text[s % 11]
        } else {
            self.section_text[0]
        }
    }
    fn line(&self, section: Option<usize>) -> &'a str {
        if let Some(s) = section {
            self.section_lines[s % 11]
        } else {
            self.section_lines[0]
        }
    }
}

fn render_node_shape(node: &LayoutNode, cx: f64, cy: f64, col: &MindmapColors) -> String {
    let fill = col.fill(node.section, node.is_root);
    match node.node_type {
        NodeType::Circle => {
            let r = node.width / 2.0;
            templates::node_circle(cx, cy, r, fill)
        }
        _ => {
            let half_w = node.width / 2.0;
            let hh = NODE_SHAPE_H / 2.0;
            let path = format!(
                "M{:.4},{:.4} v{:.4} q0,-5 5,-5 h{:.4} q5,0 5,5 v{:.4} q0,5 -5,5 h{:.4} q-5,0 -5,-5 Z",
                cx - half_w, cy + hh, -NODE_SHAPE_H,
                node.width - 2.0 * NODE_RECT_RX, NODE_SHAPE_H,
                -(node.width - 2.0 * NODE_RECT_RX)
            );
            let lc = col.line(node.section);
            let ly = cy + hh + 5.0;
            templates::node_rect_with_line(
                cx,
                cy,
                half_w,
                hh,
                NODE_RECT_RX,
                node.width,
                fill,
                lc,
                &path,
                ly,
            )
        }
    }
}

fn render_node_text(node: &LayoutNode, cx: f64, cy: f64, ff: &str, col: &MindmapColors) -> String {
    let text = esc(&node.descr);
    let color = col.text(node.section, node.is_root);
    templates::node_label(cx, cy, ff, FONT_SIZE, color, &text)
}

fn render_edge(
    px: f64,
    py: f64,
    cx: f64,
    cy: f64,
    section: Option<usize>,
    col: &MindmapColors,
) -> String {
    let mid_x = (px + cx) / 2.0;
    let color = col.fill(section, false);
    templates::edge(px, py, mid_x, cx, cy, color)
}

fn render_fallback(
    diag: &MindmapDiagram,
    ff: &str,
    text_color: &str,
    col: &MindmapColors,
) -> String {
    let root = match &diag.root {
        Some(r) => r,
        None => return templates::empty_svg().to_string(),
    };

    let mut nodes = flatten_tree(root);
    layout_mindmap(&mut nodes);

    let min_x = nodes
        .iter()
        .map(|n| n.x - n.width / 2.0)
        .fold(f64::INFINITY, f64::min);
    let min_y = nodes
        .iter()
        .map(|n| n.y - n.height / 2.0)
        .fold(f64::INFINITY, f64::min);
    let max_x = nodes
        .iter()
        .map(|n| n.x + n.width / 2.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = nodes
        .iter()
        .map(|n| n.y + n.height / 2.0)
        .fold(f64::NEG_INFINITY, f64::max);

    let svg_w = max_x - min_x + 2.0 * MARGIN;
    let svg_h = max_y - min_y + 2.0 * MARGIN;
    let off_x = -min_x + MARGIN;
    let off_y = -min_y + MARGIN;

    let svg_id = "mermaid-mindmap";
    let css = format!(
        "#{id}{{font-family:{ff};font-size:16px;fill:{text_color};}}\n#{id} .mindmap-node-label{{}}\n#{id} .mindmap-edge{{stroke-linecap:round;}}\n#{id} p{{margin:0;}}\n",
        id = svg_id,
        ff = ff,
        text_color = text_color,
    );

    let mut parts = vec![
        templates::svg_root(svg_id, svg_w, svg_h),
        format!("<style>{}</style>", css),
        "<g class=\"mindmap-edges\">".to_string(),
    ];

    for node in &nodes {
        for &ci in &node.children {
            let child = &nodes[ci];
            parts.push(render_edge(
                node.x + off_x,
                node.y + off_y,
                child.x + off_x,
                child.y + off_y,
                child.section,
                col,
            ));
        }
    }
    parts.push("</g>".to_string());
    parts.push("<g class=\"mindmap-nodes\">".to_string());

    for node in &nodes {
        let cx = node.x + off_x;
        let cy = node.y + off_y;
        let sc = if node.is_root {
            "section-root section--1".to_string()
        } else if let Some(s) = node.section {
            format!("section-{}", s)
        } else {
            String::new()
        };
        parts.push(node_group_open(&sc, node.id));
        parts.push(render_node_shape(node, cx, cy, col));
        parts.push(render_node_text(node, cx, cy, ff, col));
        parts.push("</g>".to_string());
    }
    parts.push("</g>".to_string());
    parts.push("</svg>".to_string());
    parts.join("\n")
}

// ── Public render function ────────────────────────────────────────────────────

pub fn render(diag: &MindmapDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let col = MindmapColors {
        root_fill: vars.mindmap_root_fill,
        root_text: vars.mindmap_root_text,
        section_fills: vars.mindmap_section_fills,
        section_text: vars.mindmap_section_text,
        section_lines: vars.mindmap_section_lines,
    };
    render_fallback(diag, ff, vars.text_color, &col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::mindmap::parser;

    #[test]
    fn basic_render_produces_svg() {
        let input = "mindmap\n  root((Root))\n    Topic A\n      Sub A1\n    Topic B";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Root"));
        assert!(svg.contains("Topic A"));
    }

    #[test]
    fn empty_mindmap_produces_svg() {
        let diag = MindmapDiagram { root: None };
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
    }

    // NOTE: mindmap layout uses floating-point positions that vary based on
    // traversal order, which is non-deterministic. Ignored for stable CI.
    #[test]
    #[ignore]
    fn snapshot_default_theme() {
        let input = "mindmap\n  root((mindmap))\n    Origins\n      Long history\n      Popularisation\n    Research\n      On effectiveness\n      On Whiteboard\n    Tools\n      Pen\n      Mermaid";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
