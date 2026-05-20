/// Parser for Mermaid mindmap syntax.
///
/// Supported grammar (faithful port of mindmapDb.ts getType / addNode logic):
///
///   mindmap
///     root((Root text))
///       Child A
///         Sub A1
///       Child B[Rect]
///       Child C(Rounded)
///       Child D))Bang((
///       Child E{{Hexagon}}
///
/// Node shapes are determined by surrounding delimiters:
///   ((text))  → Circle
///   (text)    → Rounded rect
///   ))text((  → Bang (cloud-with-bang)
///   )text(    → Cloud
///   [text]    → Rect
///   {{text}}  → Hexagon
///   text      → Default (no border)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Default,     // 0 – no border / plain text
    RoundedRect, // 1 – (text)
    Rect,        // 2 – [text]
    Circle,      // 3 – ((text))
    Cloud,       // 4 – )text(
    Bang,        // 5 – ))text((
    Hexagon,     // 6 – {{text}}
}

#[derive(Debug, Clone)]
pub struct MindmapNode {
    pub id: usize,
    pub level: usize, // 0 = root
    pub descr: String,
    pub node_type: NodeType,
    pub children: Vec<MindmapNode>,
    pub padding: f64,
    pub section: Option<usize>,
    pub is_root: bool,
}

impl MindmapNode {
    pub fn new(
        id: usize,
        level: usize,
        descr: String,
        node_type: NodeType,
        is_root: bool,
        padding: f64,
    ) -> Self {
        MindmapNode {
            id,
            level,
            descr,
            node_type,
            children: Vec::new(),
            padding,
            section: None,
            is_root,
        }
    }
}

pub struct MindmapDiagram {
    pub root: Option<MindmapNode>,
}

/// Parse a mindmap diagram from Mermaid syntax.
/// Mirrors the logic of mindmapDb.ts addNode + getType.
pub fn parse(input: &str) -> crate::error::ParseResult<MindmapDiagram> {
    let mut counter = 0usize;
    let mut nodes_flat: Vec<(usize, MindmapNode)> = Vec::new(); // (indent_level, node)
    let mut base_indent: Option<usize> = None;

    for raw_line in input.lines() {
        let trimmed = raw_line.trim_end();

        // Skip blank lines, comments, and ::icon() / ::class() directives
        if trimmed.trim().is_empty()
            || trimmed.trim().starts_with("%%")
            || trimmed.trim().starts_with("::")
        {
            continue;
        }

        // Skip the "mindmap" keyword line itself
        if trimmed.trim() == "mindmap" {
            continue;
        }

        // Count leading spaces to determine indent level
        let indent = raw_line.len() - raw_line.trim_start().len();

        // Establish the base indentation from the first non-keyword line
        let relative_level = if let Some(base) = base_indent {
            // Each 2-space indent = 1 level; handle tabs as 2 spaces
            let delta = indent.saturating_sub(base);
            delta / 2
        } else {
            base_indent = Some(indent);
            0
        };

        let content = trimmed.trim();
        if content.is_empty() {
            continue;
        }

        // Parse the node shape from the content
        let (descr, node_type) = parse_node_content(content);

        // Compute padding (mirrors mindmapDb.ts addNode padding logic)
        let base_padding = 15.0; // mindmap.padding default
        let padding = match node_type {
            NodeType::RoundedRect | NodeType::Rect | NodeType::Hexagon => base_padding * 2.0,
            NodeType::Circle => 10.0,
            _ => base_padding,
        };

        let is_root = nodes_flat.is_empty();
        let node = MindmapNode::new(counter, relative_level, descr, node_type, is_root, padding);
        counter += 1;
        nodes_flat.push((relative_level, node));
    }

    if nodes_flat.is_empty() {
        return crate::error::ParseResult::ok(MindmapDiagram { root: None });
    }

    // Build the tree from the flat list, using a stack to track parents.
    // This mirrors mindmapDb.ts getParent() which finds the last node with level < current level.
    let mut built: Vec<MindmapNode> = Vec::new();

    for (level, node) in nodes_flat {
        if level == 0 {
            built.push(node);
        } else {
            // Find the last built node that should be a parent (level < this node's level)
            // We push into the tree recursively
            insert_node(&mut built, node, level);
        }
    }

    let mut root = built.into_iter().next();

    // Assign section numbers (mirrors mindmapDb.ts assignSections)
    if let Some(ref mut r) = root {
        assign_sections(r, None);
    }

    crate::error::ParseResult::ok(MindmapDiagram { root })
}

/// Recursively insert a node at the correct depth in the tree.
fn insert_node(siblings: &mut Vec<MindmapNode>, node: MindmapNode, target_level: usize) {
    if let Some(last) = siblings.last_mut() {
        if target_level > last.level + 1 {
            // Need to go deeper
            insert_node(&mut last.children, node, target_level);
        } else if target_level == last.level + 1 {
            last.children.push(node);
        } else {
            // Same level or higher — try the parent's sibling
            // This handles cases where indent jumps back
            siblings.push(node);
        }
    } else {
        siblings.push(node);
    }
}

/// Parse the text content of a node line and extract (description, NodeType).
/// Mirrors mindmapDb.ts getType() with its delimiter detection.
///
/// Handles two forms:
///  1. Pure shape: `((text))` — whole content is the shape notation
///  2. Prefixed: `nodeId((text))` — node id prefix before the shape notation
///     In this case the description is extracted from inside the delimiters.
///  3. Plain text: `Some text` — Default type, entire content is description
fn parse_node_content(content: &str) -> (String, NodeType) {
    // Helper: find a shape suffix in the content and return (inner_text, NodeType) if found.
    // We search for shape delimiters: (( )) {{ }} )) (( [ ] ( ) ) (
    // Each open/close pair defines the shape type.

    // Try double-bracket shapes first (longest match, to avoid partial matches)
    if let Some(pos) = content.find("((") {
        if content.ends_with("))") && content.len() > pos + 4 {
            let inner = &content[pos + 2..content.len() - 2];
            if !inner.is_empty() {
                return (inner.to_string(), NodeType::Circle);
            }
        }
    }
    if let Some(pos) = content.find("{{") {
        if content.ends_with("}}") && content.len() > pos + 4 {
            let inner = &content[pos + 2..content.len() - 2];
            if !inner.is_empty() {
                return (inner.to_string(), NodeType::Hexagon);
            }
        }
    }
    if let Some(pos) = content.find("))") {
        if content.ends_with("((") && content.len() > pos + 4 {
            let inner = &content[pos + 2..content.len() - 2];
            if !inner.is_empty() {
                return (inner.to_string(), NodeType::Bang);
            }
        }
    }
    // Single-bracket shapes (must check they aren't part of double-bracket)
    if let Some(pos) = content.find('[') {
        if content.ends_with(']') && content.len() > pos + 2 {
            let inner = &content[pos + 1..content.len() - 1];
            if !inner.is_empty() {
                return (inner.to_string(), NodeType::Rect);
            }
        }
    }
    // Rounded rect / cloud — look for a single '(' that isn't part of '(('
    // We find the LAST '(' in the content that starts a shape block
    if let Some(pos) = find_shape_open_paren(content) {
        if content.ends_with(')') && !content.ends_with("))") && content.len() > pos + 2 {
            let inner = &content[pos + 1..content.len() - 1];
            if !inner.is_empty() {
                return (inner.to_string(), NodeType::RoundedRect);
            }
        }
    }
    if let Some(pos) = content.find(')') {
        if content.ends_with('(') && !content.ends_with("((") && content.len() > pos + 2 {
            let inner = &content[pos + 1..content.len() - 1];
            if !inner.is_empty() {
                return (inner.to_string(), NodeType::Cloud);
            }
        }
    }

    // Default: plain text, no border
    (content.to_string(), NodeType::Default)
}

/// Find the position of a single '(' that starts a shape block.
/// We want a '(' that is NOT part of '((' (double paren).
fn find_shape_open_paren(content: &str) -> Option<usize> {
    let bytes = content.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'(' {
            // Not part of '((' — check next char
            let next = bytes.get(i + 1).copied();
            if next != Some(b'(') {
                // This is a single '(' — check it's part of a valid shape:
                // the content should end with ')' (not '))')
                return Some(i);
            }
        }
    }
    None
}

/// Assign section numbers to nodes, mirroring mindmapDb.ts assignSections.
/// Root: section = None.
/// Root's children: section = index % (MAX_SECTIONS - 1).
/// Deeper nodes: inherit parent's section.
fn assign_sections(node: &mut MindmapNode, section: Option<usize>) {
    const MAX_SECTIONS: usize = 12;

    if node.level == 0 {
        node.section = None;
    } else {
        node.section = section;
    }

    let n_children = node.children.len();
    for i in 0..n_children {
        let child_section = if node.level == 0 {
            Some(i % (MAX_SECTIONS - 1))
        } else {
            section
        };
        assign_sections(&mut node.children[i], child_section);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_mindmap() {
        let input = "mindmap\n  root((Root))\n    Topic A\n      Sub A1\n    Topic B";
        let d = parse(input).diagram;
        let root = d.root.as_ref().unwrap();
        assert_eq!(root.descr, "Root");
        assert_eq!(root.node_type, NodeType::Circle);
        assert_eq!(root.children.len(), 2);
        assert_eq!(root.children[0].descr, "Topic A");
        assert_eq!(root.children[0].children.len(), 1);
        assert_eq!(root.children[0].children[0].descr, "Sub A1");
        assert_eq!(root.children[1].descr, "Topic B");
    }

    #[test]
    fn node_shapes() {
        let input = "mindmap\n  root\n    A[Rect]\n    B(Rounded)\n    C((Circle))\n    D{{Hex}}";
        let d = parse(input).diagram;
        let root = d.root.as_ref().unwrap();
        assert_eq!(root.children[0].node_type, NodeType::Rect);
        assert_eq!(root.children[1].node_type, NodeType::RoundedRect);
        assert_eq!(root.children[2].node_type, NodeType::Circle);
        assert_eq!(root.children[3].node_type, NodeType::Hexagon);
    }

    #[test]
    fn section_assignment() {
        let input = "mindmap\n  root((R))\n    A\n    B\n    C";
        let d = parse(input).diagram;
        let root = d.root.as_ref().unwrap();
        assert_eq!(root.section, None);
        assert_eq!(root.children[0].section, Some(0));
        assert_eq!(root.children[1].section, Some(1));
        assert_eq!(root.children[2].section, Some(2));
    }
}
