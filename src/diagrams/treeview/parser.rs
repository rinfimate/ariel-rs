// Parser for the Mermaid treeView-beta diagram type.
//
// Grammar:
//   treeView-beta
//       "label1"
//           "child1"
//           "child2"
//               "grandchild"
//
// Each non-header line is a quoted or unquoted label at a given indent depth.
// Indentation is measured in raw characters (spaces/tabs), and the relative
// depth is inferred from the indent width of the first content line.

/// A single node in the tree.
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// Display label (quotes stripped).
    pub label: String,
    /// Child nodes in document order.
    pub children: Vec<TreeNode>,
}

/// Parsed treeView diagram.
#[derive(Debug)]
pub struct TreeViewDiagram {
    /// Top-level nodes (roots of the tree as written in the source).
    pub roots: Vec<TreeNode>,
}

/// Parse treeView-beta source text.
pub fn parse(input: &str) -> crate::error::ParseResult<TreeViewDiagram> {
    // Flat list of (indent_chars, label) for content lines.
    let mut items: Vec<(usize, String)> = Vec::new();
    let mut past_header = false;

    for raw in input.lines() {
        // Strip inline comments.
        let line = if let Some(pos) = raw.find("%%") {
            &raw[..pos]
        } else {
            raw
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Skip the header keyword line.
        if !past_header {
            let kw = trimmed.to_ascii_lowercase();
            if kw.starts_with("treeview-beta") || kw.starts_with("treeview") {
                past_header = true;
                continue;
            }
            // Not yet found header — keep scanning.
            continue;
        }

        // Skip accTitle / accDescr directives.
        if trimmed.starts_with("accTitle") || trimmed.starts_with("accDescr") {
            continue;
        }

        // Measure raw indent (spaces or tabs each count as 1 character).
        let indent = line.len() - line.trim_start().len();
        let label = strip_quotes(trimmed);
        if !label.is_empty() {
            items.push((indent, label));
        }
    }

    let roots = build_tree(&items);
    crate::error::ParseResult::ok(TreeViewDiagram { roots })
}

/// Strip surrounding double-quotes from a label.
fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Convert a flat (indent, label) list into a tree using a stack.
fn build_tree(items: &[(usize, String)]) -> Vec<TreeNode> {
    if items.is_empty() {
        return Vec::new();
    }

    // Each stack entry: (indent_of_that_node, index_into_nodes).
    let mut nodes: Vec<TreeNode> = Vec::new();
    let mut parents: Vec<Option<usize>> = Vec::new();
    let mut stack: Vec<(usize, usize)> = Vec::new(); // (indent, node_idx)

    for (indent, label) in items {
        let node = TreeNode {
            label: label.clone(),
            children: Vec::new(),
        };
        let idx = nodes.len();
        nodes.push(node);

        // Pop entries that are at the same or deeper indent.
        while let Some(&(top_indent, _)) = stack.last() {
            if top_indent >= *indent {
                stack.pop();
            } else {
                break;
            }
        }

        let parent = stack.last().map(|&(_, i)| i);
        parents.push(parent);
        stack.push((*indent, idx));
    }

    // Build children maps.
    let n = nodes.len();
    let mut children_map: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, &parent) in parents.iter().enumerate() {
        if let Some(p) = parent {
            children_map[p].push(i);
        }
    }

    let root_indices: Vec<usize> = (0..n).filter(|&i| parents[i].is_none()).collect();

    fn build(idx: usize, all: &[TreeNode], cm: &[Vec<usize>]) -> TreeNode {
        let src = &all[idx];
        let children = cm[idx].iter().map(|&ci| build(ci, all, cm)).collect();
        TreeNode {
            label: src.label.clone(),
            children,
        }
    }

    root_indices
        .iter()
        .map(|&ri| build(ri, &nodes, &children_map))
        .collect()
}
