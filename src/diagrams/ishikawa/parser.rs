/// Parser for Mermaid Ishikawa (fishbone/cause-effect) diagram syntax.
///
/// Faithful port of ishikawaDb.ts addNode() logic.
///
/// Grammar:
///   fishbone|ishikawa
///   [title <text>]
///   <root-label>   (indented at level 0 relative to header)
///     Cause1       (level 1 = main bone)
///       SubCause1  (level 2 = sub bone)
///       SubCause2
///     Cause2
///       ...
///
/// The first non-header, non-title, non-comment line becomes the root (effect/head).
/// Indentation is relative to the first content line.

#[derive(Debug, Clone)]
pub struct IshikawaNode {
    pub text: String,
    pub children: Vec<IshikawaNode>,
}

impl IshikawaNode {
    pub fn new(text: impl Into<String>) -> Self {
        IshikawaNode {
            text: text.into(),
            children: Vec::new(),
        }
    }
}

pub struct IshikawaDiagram {
    pub title: Option<String>,
    /// The root node (fish head / effect). Its children are the main bones (causes).
    pub root: Option<IshikawaNode>,
}

/// Parse an Ishikawa diagram from Mermaid syntax.
/// Mirrors ishikawaDb.ts addNode() stack-based hierarchy building.
pub fn parse(input: &str) -> crate::error::ParseResult<IshikawaDiagram> {
    let mut title: Option<String> = None;
    let mut header_seen = false;

    // Collect (raw_indent, text) pairs
    let mut entries: Vec<(usize, String)> = Vec::new();

    for raw_line in input.lines() {
        let trimmed = raw_line.trim();

        if trimmed.is_empty() || trimmed.starts_with("%%") || trimmed.starts_with('%') {
            continue;
        }

        if !header_seen {
            let lower = trimmed.to_lowercase();
            if lower == "fishbone"
                || lower == "ishikawa"
                || lower.starts_with("fishbone ")
                || lower.starts_with("ishikawa ")
            {
                header_seen = true;
                continue;
            }
            continue;
        }

        // title line
        if trimmed.len() >= 5
            && trimmed[..5].eq_ignore_ascii_case("title")
            && (trimmed.len() == 5
                || trimmed.as_bytes()[5] == b' '
                || trimmed.as_bytes()[5] == b'\t')
        {
            if trimmed.len() > 5 {
                title = Some(trimmed[5..].trim().to_string());
            }
            continue;
        }

        let indent = raw_line.len() - raw_line.trim_start_matches([' ', '\t']).len();
        entries.push((indent, trimmed.to_string()));
    }

    if entries.is_empty() {
        return crate::error::ParseResult::ok(IshikawaDiagram { title, root: None });
    }

    // Faithful port of ishikawaDb.ts addNode(rawLevel, text):
    // - First node → root
    // - For each subsequent node:
    //   Pop stack while stack_top.level >= current level
    //   If stack is empty (or level <= root level) → child of root
    //   Else → child of stack top
    //   Push current node onto stack
    //
    // We build the tree using indices and a parent-index array.
    let n = entries.len();
    let mut nodes: Vec<IshikawaNode> = entries
        .iter()
        .map(|(_, t)| IshikawaNode::new(t.clone()))
        .collect();
    // parent_index[i] = j means nodes[i] is a child of nodes[j]; parent_index[0] = usize::MAX (root)
    let mut parent_index: Vec<usize> = vec![usize::MAX; n];

    // Stack of (indent_level, node_index)
    let mut stack: Vec<(usize, usize)> = Vec::new();

    for i in 0..n {
        let (level, _) = entries[i];
        if i == 0 {
            // Root
            stack.push((level, 0));
            continue;
        }

        // Pop while stack top level >= current level (but always keep root[0] conceptually)
        while stack.len() > 1 {
            let top_level = stack.last().unwrap().0;
            if top_level >= level {
                stack.pop();
            } else {
                break;
            }
        }

        // Parent is current stack top (or root if stack has only root or is effectively empty)
        let parent_idx = if stack.is_empty() {
            0 // parent = root
        } else {
            let top_level = stack.last().unwrap().0;
            if top_level >= level {
                // root level case
                0
            } else {
                stack.last().unwrap().1
            }
        };

        parent_index[i] = parent_idx;
        stack.push((level, i));
    }

    // Build tree from parent_index
    // We need to add children in order; collect children lists
    let mut children_lists: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, &p) in parent_index
        .iter()
        .enumerate()
        .skip(1)
        .take(n.saturating_sub(1))
    {
        children_lists[p].push(i);
    }

    // Recursive tree assembly
    fn assemble(idx: usize, nodes: &mut Vec<IshikawaNode>, children_lists: &[Vec<usize>]) {
        let child_indices = children_lists[idx].clone();
        for ci in child_indices {
            assemble(ci, nodes, children_lists);
            // Take the child out of nodes and add to parent
            // We can't borrow both mutably at once, so we swap with a placeholder
            let child = std::mem::replace(&mut nodes[ci], IshikawaNode::new(String::new()));
            nodes[idx].children.push(child);
        }
    }

    assemble(0, &mut nodes, &children_lists);

    crate::error::ParseResult::ok(IshikawaDiagram {
        title,
        root: Some(nodes.remove(0)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_fishbone() {
        // In Mermaid fishbone syntax, nodes at indent 8 (Worn parts, Calibration) are
        // both direct children of root (indent 4), same for Human error (indent 4 = root level).
        // The stack algorithm faithfully mirrors ishikawaDb.ts addNode():
        //   root[4] → Worn parts[8] (child of root), Bearings[12] (child of Worn parts),
        //   Calibration[8] (child of root), Human error[4] (child of root), Training[8] (child of HE)
        let input = "fishbone\n    Equipment failure\n        Worn parts\n            Bearings\n        Calibration\n    Human error\n        Training\n";
        let d = parse(input).diagram;
        let root = d.root.unwrap();
        assert_eq!(root.text, "Equipment failure");
        // root has 3 direct children: Worn parts, Calibration, Human error
        assert_eq!(root.children.len(), 3);
        assert_eq!(root.children[0].text, "Worn parts");
        assert_eq!(root.children[0].children.len(), 1);
        assert_eq!(root.children[0].children[0].text, "Bearings");
        assert_eq!(root.children[1].text, "Calibration");
        assert_eq!(root.children[2].text, "Human error");
        assert_eq!(root.children[2].children[0].text, "Training");
    }

    #[test]
    fn with_title() {
        let input = "ishikawa\n    title My Diagram\n    Effect\n        Cause A\n";
        let d = parse(input).diagram;
        assert_eq!(d.title.as_deref(), Some("My Diagram"));
        let root = d.root.unwrap();
        assert_eq!(root.text, "Effect");
        assert_eq!(root.children[0].text, "Cause A");
    }

    #[test]
    fn no_children() {
        let input = "fishbone\n    Effect only\n";
        let d = parse(input).diagram;
        let root = d.root.unwrap();
        assert_eq!(root.text, "Effect only");
        assert!(root.children.is_empty());
    }
}
