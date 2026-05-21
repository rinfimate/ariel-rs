// Faithful Rust port of Mermaid's treeView renderer.
//
// Layout algorithm:
//   - Nodes are visited in depth-first pre-order (document order).
//   - Each node occupies one row; rows are spaced ROW_HEIGHT px apart.
//   - Node text is placed at x = depth * INDENT_STEP + LEFT_PAD,
//     y = (row_index + 0.5) * ROW_HEIGHT.
//   - Every node has a horizontal connector line:
//       x1 = text_x - INDENT_STEP, x2 = text_x - H_LINE_GAP, y = text_y.
//   - Every non-leaf node has a vertical connector line drawn after all text
//     and horizontal lines:
//       x = text_x, y1 = text_y + ROW_HEIGHT/2,
//       y2 = last_direct_child_y + V_LINE_BOTTOM_TWEAK.
//   - A synthetic root node "/" is always prepended at depth 0.
//
// A single virtual "/"-root wraps all parsed top-level nodes, matching the
// reference Mermaid output exactly.

use super::constants::*;
use super::parser::TreeViewDiagram;
use super::templates::{self, esc};
use crate::text_browser_metrics::measure_browser;
use crate::theme::Theme;

// ---------------------------------------------------------------------------
// Flat render record
// ---------------------------------------------------------------------------

/// A single rendered node (text + horizontal line info).
struct RenderedNode {
    label: String,
    text_x: f64,
    text_y: f64,
    /// Index of the last direct child in the flat list (None if leaf).
    last_child_idx: Option<usize>,
}

// ---------------------------------------------------------------------------
// Depth-first flattening
// ---------------------------------------------------------------------------

fn flatten(
    label: &str,
    children: &[super::parser::TreeNode],
    depth: usize,
    row: &mut usize,
    out: &mut Vec<RenderedNode>,
) {
    let text_x = depth as f64 * INDENT_STEP + LEFT_PAD;
    let text_y = (*row as f64 + 0.5) * ROW_HEIGHT;
    let my_idx = out.len();
    out.push(RenderedNode {
        label: label.to_string(),
        text_x,
        text_y,
        last_child_idx: None, // filled in below
    });
    *row += 1;

    if !children.is_empty() {
        let mut last_direct_child_idx = 0usize;
        for child in children {
            // Record the flat index of this direct child before recursing.
            last_direct_child_idx = out.len();
            flatten(&child.label, &child.children, depth + 1, row, out);
        }
        // last_direct_child_idx now holds the flat index of the last direct child.
        out[my_idx].last_child_idx = Some(last_direct_child_idx);
    }
}

// ---------------------------------------------------------------------------
// Main render
// ---------------------------------------------------------------------------

/// Render a treeView diagram to an SVG string.
pub fn render(diag: &TreeViewDiagram, _theme: Theme) -> String {
    // Build flat list: synthetic "/" root wraps all diagram roots.
    let root_label = "/";
    let root_children = &diag.roots;

    let mut row = 0usize;
    let mut nodes: Vec<RenderedNode> = Vec::new();

    // Synthetic root node at depth 0.
    let root_text_x = LEFT_PAD;
    let root_text_y = (row as f64 + 0.5) * ROW_HEIGHT;
    let root_my_idx = nodes.len();
    nodes.push(RenderedNode {
        label: root_label.to_string(),
        text_x: root_text_x,
        text_y: root_text_y,
        last_child_idx: None,
    });
    row += 1;

    let mut last_direct_child_of_root = 0usize;
    for child in root_children {
        last_direct_child_of_root = nodes.len();
        flatten(&child.label, &child.children, 1, &mut row, &mut nodes);
    }

    // Set last_child_idx for the synthetic root to its last direct child.
    if !root_children.is_empty() {
        nodes[root_my_idx].last_child_idx = Some(last_direct_child_of_root);
    }

    // Compute SVG dimensions.
    let total_rows = row;
    let vb_h = total_rows as f64 * ROW_HEIGHT;

    // Width: find the rightmost text edge.
    // Chrome getBBox() includes trailing advance (~4.4375px at 16px Arial).
    let max_right = nodes
        .iter()
        .map(|n| {
            let (tw, _) = measure_browser(&n.label, FONT_SIZE);
            n.text_x + tw + TRAILING_ADVANCE
        })
        .fold(0.0_f64, f64::max);
    let vb_w = max_right + VIEWBOX_RIGHT_PAD - VIEWBOX_X;

    let svg_id = "mermaid-svg-99";

    let mut out = String::new();

    // SVG root element.
    out.push_str(&templates::svg_root(
        svg_id,
        vb_w + VIEWBOX_X, // max-width = vb_x + vb_w
        VIEWBOX_X,
        VIEWBOX_Y,
        vb_w,
        vb_h,
    ));

    // Empty g (Mermaid boilerplate).
    out.push_str("<g></g>");

    // Main tree-view group.
    out.push_str(r#"<g class="tree-view">"#);

    // 1. Text labels and horizontal connector lines.
    for node in &nodes {
        // Text label.
        out.push_str(&templates::node_text(
            node.text_x,
            node.text_y,
            &esc(&node.label),
        ));

        // Horizontal connector line.
        let h_x1 = node.text_x - INDENT_STEP;
        let h_x2 = node.text_x - H_LINE_GAP;
        out.push_str(&templates::h_line(h_x1, node.text_y, h_x2, node.text_y));
    }

    // 2. Vertical connector lines (parent → last child), drawn last to match
    //    Mermaid's rendering order.
    for node in nodes.iter().rev() {
        if let Some(last_child_idx) = node.last_child_idx {
            let v_x = node.text_x;
            let v_y1 = node.text_y + ROW_HEIGHT / 2.0;
            let v_y2 = nodes[last_child_idx].text_y + V_LINE_BOTTOM_TWEAK;
            out.push_str(&templates::v_line(v_x, v_y1, v_x, v_y2));
        }
    }

    out.push_str("</g>");
    out.push_str("</svg>");

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::treeview::parser;

    #[test]
    fn basic_render_produces_svg() {
        let input = concat!(
            "treeView-beta\n",
            "    \"docs\"\n",
            "        \"build\"\n",
            "        \"make.bat\"\n",
        );
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "no <svg tag");
        assert!(svg.contains("docs"));
        assert!(svg.contains("build"));
        assert!(svg.contains("tree-view"));
        assert!(svg.contains("treeView-node-label"));
        assert!(svg.contains("treeView-node-line"));
    }

    #[test]
    fn synthetic_root_slash_is_present() {
        let input = "treeView-beta\n    \"docs\"\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains('>'), "svg should have content");
        // The '/' root should appear as text content.
        assert!(
            svg.contains(">/</text>") || svg.contains(">/</"),
            "slash root missing"
        );
    }

    #[test]
    fn viewbox_height_matches_row_count() {
        let input = concat!("treeView-beta\n", "    \"docs\"\n", "        \"build\"\n",);
        // 3 nodes (/ + docs + build) = 3 rows.
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        let expected_h = 3.0 * ROW_HEIGHT;
        assert!(
            svg.contains(&format!("{expected_h}")),
            "expected height {expected_h} not found in: {svg}"
        );
    }

    #[test]
    fn empty_diagram_has_only_root() {
        let input = "treeView-beta\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
        // 1 row: just the "/" root.
        let expected_h = ROW_HEIGHT;
        assert!(
            svg.contains(&format!("{expected_h}")),
            "expected height {expected_h} not found"
        );
    }
}
