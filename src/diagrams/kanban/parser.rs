/// Parser for Mermaid kanban diagram syntax.
///
/// Faithful port of kanbanDb.ts.
///
/// Grammar:
///   kanban
///     columnId["Column Label"]
///       itemId["Item Label"]
///       itemId2["Item 2"]
///     columnId2
///       itemId3["Item 3"]
///
/// Columns are detected at indent level 1 (relative to `kanban`).
/// Items are detected at indent level 2+.
///
/// Node shapes (same as mindmap/kanbanDb.ts getType):
///   [text]    → Rect (default for kanban items)
///   (text)    → RoundedRect
///   ((text))  → Circle
///   )text(    → Cloud
///   ))text((  → Bang
///   {{text}}  → Hexagon
///   text      → Default (no border)
///
/// YAML metadata in node definitions is parsed for shape, icon, ticket, priority, etc.
/// This port handles the common cases faithfully.
/// Shape constants (mirrors kanbanDb.ts nodeType)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeShape {
    Default,     // 0/1 – plain text / no border
    RoundedRect, // 2 – (text)
    Rect,        // 3 – [text]
    Circle,      // 4 – ((text))
    Cloud,       // 5 – )text(
    Bang,        // 6 – ))text((
    Hexagon,     // 7 – {{text}}
}

/// A kanban column (section/group node in kanbanDb.ts getData).
#[derive(Debug, Clone)]
pub struct KanbanSection {
    pub id: String,
    pub label: String,
    pub items: Vec<KanbanItem>,
}

/// A kanban item (card within a column).
#[derive(Debug, Clone)]
pub struct KanbanItem {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
}

pub struct KanbanDiagram {
    pub sections: Vec<KanbanSection>,
}

/// Parse a kanban diagram from Mermaid syntax.
/// Mirrors the logic of kanbanDb.ts addNode + getData.
pub fn parse(input: &str) -> crate::error::ParseResult<KanbanDiagram> {
    let mut sections: Vec<KanbanSection> = Vec::new();

    // Strip YAML front-matter (--- ... ---) if present
    let body = strip_frontmatter(input);

    // Track base indentation
    let mut header_seen = false;
    let mut base_indent: Option<usize> = None;

    // We'll accumulate lines into sections using an indent-based approach.
    // Level 0 relative = column header; level 1+ = item.
    let mut current_section: Option<KanbanSection> = None;
    let mut item_counter: usize = 0;
    let mut section_counter: usize = 0;

    for raw_line in body.lines() {
        let trimmed_end = raw_line.trim_end();

        // Skip blank lines and comments
        let trimmed = trimmed_end.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        // Detect and skip the "kanban" header line
        if !header_seen {
            if trimmed.eq_ignore_ascii_case("kanban") {
                header_seen = true;
            }
            continue;
        }

        // Handle `title` line inside the diagram body — skip
        if trimmed.to_lowercase().starts_with("title ") {
            continue;
        }

        // Determine indentation
        let indent = raw_line.len() - raw_line.trim_start().len();

        // Establish base indentation from first non-header line
        if base_indent.is_none() {
            base_indent = Some(indent);
        }
        let base = base_indent.unwrap_or(0);

        // Relative indent level (0 = column, 1+ = item)
        let relative_level = if indent >= base {
            (indent - base) / 2
        } else {
            0
        };

        if relative_level == 0 {
            // This is a column/section header
            if let Some(sec) = current_section.take() {
                sections.push(sec);
            }

            let (id, label) = parse_node_id_and_label(trimmed, &mut section_counter);
            section_counter += 1;

            current_section = Some(KanbanSection {
                id,
                label,
                items: Vec::new(),
            });
        } else {
            // This is an item within the current section
            let (id, label, shape) = parse_item(trimmed, &mut item_counter);
            item_counter += 1;

            let item = KanbanItem { id, label, shape };

            if let Some(ref mut sec) = current_section {
                sec.items.push(item);
            }
        }
    }

    // Flush the last section
    if let Some(sec) = current_section.take() {
        sections.push(sec);
    }

    crate::error::ParseResult::ok(KanbanDiagram { sections })
}

/// Strip YAML front matter (--- ... ---) from input, returning the remainder.
fn strip_frontmatter(input: &str) -> &str {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("---") {
        return input;
    }
    // Find the closing ---
    let after_open = &trimmed[3..];
    if let Some(close_pos) = after_open.find("\n---") {
        let after_close = &after_open[close_pos + 4..];
        // Skip the newline after the closing ---
        return after_close.trim_start_matches('\n');
    }
    input
}

/// Parse a column/section line, extracting ID and label.
/// The line may be:
///   - plain identifier: `todo`  → id="todo", label="todo"
///   - with bracket label: `todo["To Do"]`  → id="todo", label="To Do"
///   - with quoted label: `todo[To Do]`  → id="todo", label="To Do"
fn parse_node_id_and_label(content: &str, counter: &mut usize) -> (String, String) {
    // Try to find bracket label: id[label] or id["label"]
    if let Some(bracket_pos) = content.find('[') {
        if content.ends_with(']') {
            let id = content[..bracket_pos].trim().to_string();
            let inner = &content[bracket_pos + 1..content.len() - 1];
            // Strip optional quotes
            let label = inner
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string();
            let id = if id.is_empty() {
                format!("section_{}", counter)
            } else {
                id
            };
            return (id, label);
        }
    }
    // Plain identifier — id and label are the same
    let id = content.trim().to_string();
    let label = id.clone();
    (id, label)
}

/// Parse a kanban item line, extracting (id, label, shape).
/// Mirrors kanbanDb.ts addNode() which handles YAML metadata @{ ... }.
///
/// Forms:
///   id["Label"]
///   id["Label"]@{ ticket: MC-2037, priority: Very High }
///   id[Label]
///   id
fn parse_item(content: &str, counter: &mut usize) -> (String, String, NodeShape) {
    // Strip YAML metadata @{ ... } if present
    let content_no_meta = if let Some(at_pos) = content.find("@{") {
        content[..at_pos].trim_end()
    } else {
        content
    };

    // Parse id + shape/label from content_no_meta
    parse_item_content(content_no_meta.trim(), counter)
}

/// Parse id + bracket-label + shape from content (without @{ } metadata).
fn parse_item_content(content: &str, counter: &mut usize) -> (String, String, NodeShape) {
    // Try double-bracket forms first (longer match wins)
    if let Some(pos) = content.find("((") {
        if content.ends_with("))") && content.len() > pos + 4 {
            let id = content[..pos].trim().to_string();
            let label = content[pos + 2..content.len() - 2]
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string();
            let id = if id.is_empty() {
                format!("item_{counter}")
            } else {
                id
            };
            return (id, label, NodeShape::Circle);
        }
    }
    if let Some(pos) = content.find("{{") {
        if content.ends_with("}}") && content.len() > pos + 4 {
            let id = content[..pos].trim().to_string();
            let label = content[pos + 2..content.len() - 2]
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string();
            let id = if id.is_empty() {
                format!("item_{counter}")
            } else {
                id
            };
            return (id, label, NodeShape::Hexagon);
        }
    }
    if let Some(pos) = content.find("))") {
        if content.ends_with("((") && content.len() > pos + 4 {
            let id = content[..pos].trim().to_string();
            let label = content[pos + 2..content.len() - 2]
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string();
            let id = if id.is_empty() {
                format!("item_{counter}")
            } else {
                id
            };
            return (id, label, NodeShape::Bang);
        }
    }
    // Single-bracket forms
    if let Some(pos) = content.find('[') {
        if content.ends_with(']') && content.len() > pos + 2 {
            let id = content[..pos].trim().to_string();
            let label = content[pos + 1..content.len() - 1]
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string();
            let id = if id.is_empty() {
                format!("item_{counter}")
            } else {
                id
            };
            return (id, label, NodeShape::Rect);
        }
    }
    // Rounded rect: id(label)
    if let Some(pos) = find_single_open_paren(content) {
        if content.ends_with(')') && !content.ends_with("))") && content.len() > pos + 2 {
            let id = content[..pos].trim().to_string();
            let label = content[pos + 1..content.len() - 1]
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string();
            let id = if id.is_empty() {
                format!("item_{counter}")
            } else {
                id
            };
            return (id, label, NodeShape::RoundedRect);
        }
    }
    // Cloud: id)label(
    if let Some(pos) = content.find(')') {
        if content.ends_with('(') && !content.ends_with("((") && content.len() > pos + 2 {
            let id = content[..pos].trim().to_string();
            let label = content[pos + 1..content.len() - 1]
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string();
            let id = if id.is_empty() {
                format!("item_{counter}")
            } else {
                id
            };
            return (id, label, NodeShape::Cloud);
        }
    }

    // Plain identifier — use it as both id and label
    let id = content.to_string();
    let label = id.clone();
    (id, label, NodeShape::Default)
}

/// Find a single '(' that is not part of '((' .
fn find_single_open_paren(content: &str) -> Option<usize> {
    let bytes = content.as_bytes();
    (0..bytes.len()).find(|&i| bytes[i] == b'(' && bytes.get(i + 1).copied() != Some(b'('))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_kanban() {
        let input = "kanban\n  todo\n    id1[Task 1]\n    id2[Task 2]\n  inProgress\n    id3[Task 3]\n  done\n    id4[Task 4]";
        let d = parse(input).diagram;
        assert_eq!(d.sections.len(), 3);
        assert_eq!(d.sections[0].id, "todo");
        assert_eq!(d.sections[0].label, "todo");
        assert_eq!(d.sections[0].items.len(), 2);
        assert_eq!(d.sections[0].items[0].label, "Task 1");
        assert_eq!(d.sections[0].items[1].label, "Task 2");
        assert_eq!(d.sections[1].id, "inProgress");
        assert_eq!(d.sections[1].items[0].label, "Task 3");
        assert_eq!(d.sections[2].id, "done");
        assert_eq!(d.sections[2].items[0].label, "Task 4");
    }

    #[test]
    fn section_with_bracket_label() {
        let input = "kanban\n  col1[\"To Do\"]\n    item1[\"Task A\"]\n";
        let d = parse(input).diagram;
        assert_eq!(d.sections[0].id, "col1");
        assert_eq!(d.sections[0].label, "To Do");
        assert_eq!(d.sections[0].items[0].label, "Task A");
    }

    #[test]
    fn item_shapes() {
        let input = "kanban\n  col\n    a[Rect]\n    b(Round)\n    c((Circle))\n";
        let d = parse(input).diagram;
        assert_eq!(d.sections[0].items[0].shape, NodeShape::Rect);
        assert_eq!(d.sections[0].items[1].shape, NodeShape::RoundedRect);
        assert_eq!(d.sections[0].items[2].shape, NodeShape::Circle);
    }

    #[test]
    fn yaml_metadata() {
        // Metadata fields are stripped but the item label and shape are still parsed.
        let input = "kanban\n  col\n    id1[Task]@{ ticket: MC-1, priority: High }\n";
        let d = parse(input).diagram;
        let item = &d.sections[0].items[0];
        assert_eq!(item.label, "Task");
        assert_eq!(item.shape, NodeShape::Rect);
    }

    #[test]
    fn frontmatter_stripped() {
        let input =
            "---\nconfig:\n  kanban:\n    sectionWidth: 150\n---\nkanban\n  col\n    id1[Task]\n";
        let d = parse(input).diagram;
        assert_eq!(d.sections.len(), 1);
        assert_eq!(d.sections[0].items[0].label, "Task");
    }
}
