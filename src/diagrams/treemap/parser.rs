// Faithful Rust port of mermaid/src/diagrams/treemap/parser.ts + db.ts + utils.ts
//
// Grammar (treemap-beta):
//   treemap-beta
//   [title <text>]
//   [classDef <name> <styles>]
//   nodes indented by spaces/tabs; a leaf has an optional numeric value:
//     Section Name
//       Child One
//       Child Two: 42
//     ...

#[derive(Debug, Clone)]
pub struct TreemapNode {
    pub name: String,
    pub value: Option<f64>,
    pub children: Vec<TreemapNode>,
    /// CSS class selector applied to this node
    pub class_selector: Option<String>,
}

#[derive(Debug)]
pub struct TreemapDiagram {
    pub title: Option<String>,
    /// Top-level nodes (direct children of the virtual root)
    pub roots: Vec<TreemapNode>,
}

// ─── parser ─────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<TreemapDiagram> {
    let mut title: Option<String> = None;
    let mut items: Vec<(usize, String, Option<f64>, Option<String>)> = Vec::new(); // (indent, name, value, class)

    let mut in_header = true; // haven't started collecting nodes yet

    for raw in input.lines() {
        // Strip inline comments
        let line = if let Some(pos) = raw.find("%%") {
            &raw[..pos]
        } else {
            raw
        };

        // Compute indent depth (each leading space or tab counts as 1)
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Header keyword: treemap-beta
        if in_header && (trimmed == "treemap-beta" || trimmed == "treemap") {
            in_header = false;
            continue;
        }
        if in_header {
            // Haven't seen the header yet – still look for it
            if trimmed.starts_with("treemap") {
                in_header = false;
            }
            continue;
        }

        // title
        if let Some(rest) = trimmed
            .strip_prefix("title ")
            .or_else(|| trimmed.strip_prefix("title\t"))
        {
            title = Some(rest.trim().to_string());
            continue;
        }
        if trimmed == "title" {
            title = Some(String::new());
            continue;
        }

        // classDef – skip (no visual rendering of class colours in this port)
        if trimmed.starts_with("classDef ") {
            continue;
        }

        // accTitle / accDescr – skip
        if trimmed.starts_with("accTitle") || trimmed.starts_with("accDescr") {
            continue;
        }

        // Node line: "Name" or "Name: value" or "Name:::class"
        let (name_part, value, class_selector) = parse_node_line(trimmed);
        if !name_part.is_empty() {
            items.push((indent, name_part, value, class_selector));
        }
    }

    let roots = build_hierarchy(&items);
    crate::error::ParseResult::ok(TreemapDiagram { title, roots })
}

/// Strip surrounding double-quotes from a name string.
fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Parse a node line of the form:
///   Name
///   Name: 42
///   Name:::className
///   Name: 42:::className
fn parse_node_line(line: &str) -> (String, Option<f64>, Option<String>) {
    // Split off class selector (:::className)
    let (body, class_sel) = if let Some(pos) = line.find(":::") {
        (&line[..pos], Some(line[pos + 3..].trim().to_string()))
    } else {
        (line, None)
    };

    // Split name and value at ':'
    if let Some(colon) = body.find(':') {
        let name = strip_quotes(body[..colon].trim());
        let val_str = body[colon + 1..].trim();
        let value = val_str.parse::<f64>().ok();
        (name, value, class_sel)
    } else {
        (strip_quotes(body.trim()), None, class_sel)
    }
}

/// Port of buildHierarchy from utils.ts – converts flat (indent, name, value) list
/// into a tree.  Nodes with no children registered become leaves; otherwise sections.
fn build_hierarchy(items: &[(usize, String, Option<f64>, Option<String>)]) -> Vec<TreemapNode> {
    if items.is_empty() {
        return Vec::new();
    }

    // We don't know which items are leaves vs sections until we see children.
    // Two-pass approach: first mark which indices have children.

    // Build a stack-based hierarchy.
    // Stack entry: (indent, index-into-nodes vec)
    let mut nodes: Vec<TreemapNode> = Vec::new();
    // Map from node index → parent index (None = root)
    let mut parents: Vec<Option<usize>> = Vec::new();

    // Stack holds (indent, node_index)
    let mut stack: Vec<(usize, usize)> = Vec::new();

    for (indent, name, value, class_sel) in items {
        let node = TreemapNode {
            name: name.clone(),
            value: *value,
            children: Vec::new(),
            class_selector: class_sel.clone(),
        };
        let idx = nodes.len();
        nodes.push(node);

        // Pop stack entries at same or deeper indent
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

    // Assemble tree bottom-up: work in reverse index order so children are
    // already fully constructed when we attach them to parents.
    // Since we can't move out of a Vec easily, we'll use indices and reconstruct.
    let n = nodes.len();
    let mut children_map: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, &parent) in parents.iter().enumerate().take(n) {
        if let Some(p) = parent {
            children_map[p].push(i);
        }
    }

    // Drain nodes into a flat array, then build recursively.
    // We'll build from the root indices.
    let root_indices: Vec<usize> = (0..n).filter(|&i| parents[i].is_none()).collect();

    fn build_node(
        idx: usize,
        all_nodes: &[TreemapNode],
        children_map: &[Vec<usize>],
    ) -> TreemapNode {
        let src = &all_nodes[idx];
        let children: Vec<TreemapNode> = children_map[idx]
            .iter()
            .map(|&ci| build_node(ci, all_nodes, children_map))
            .collect();
        TreemapNode {
            name: src.name.clone(),
            value: src.value,
            children,
            class_selector: src.class_selector.clone(),
        }
    }

    root_indices
        .iter()
        .map(|&ri| build_node(ri, &nodes, &children_map))
        .collect()
}
