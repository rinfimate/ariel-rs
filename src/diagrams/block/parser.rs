/// Parser for Mermaid block diagram syntax.
///
/// Faithful port of blockDB.ts.
///
/// Syntax:
///   block-beta
///       columns <N>
///       A["Label A"] B["Label B"] C["C"]
///       space              -- empty cell
///       space:2            -- spans 2 columns
///       A --> B            -- edge
///       A -- "label" --> B -- edge with label
///       block:id["label"]  -- nested block (sub-block)
///         ...
///       end

#[derive(Debug, Clone, PartialEq)]
pub enum BlockShape {
    Square,      // A["label"] or A["label"]:N  rect
    RoundedRect, // A("label")  rx/ry
    Cylinder,    // A[("label")]  cylinder
    Diamond,     // A{"label"}  diamond/rhombus
    Circle,      // A(("label"))
    Hexagon,     // A{{"label"}}
    BlockArrow,  // A<["label"]>(direction)  block arrow shape
    Default,     // bare A
}

#[derive(Debug, Clone)]
pub struct BlockNode {
    pub id: String,
    pub label: String,
    pub shape: BlockShape,
    pub col_span: usize,             // how many columns this occupies
    pub is_group: bool,              // true for nested block:id nodes
    pub group_children: Vec<String>, // ordered child node IDs (for nested blocks)
    pub style: Option<String>,       // user inline style from `style X fill:...,stroke:...`
}

#[derive(Debug, Clone)]
pub struct BlockEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BlockRow {
    /// Items in order: None=space, Some(id)=block node
    pub items: Vec<RowItem>,
}

#[derive(Debug, Clone)]
pub enum RowItem {
    Space(usize),        // span
    Node(String, usize), // id, span
}

#[derive(Debug, Default)]
pub struct BlockDiagram {
    pub columns: usize,
    pub nodes: indexmap::IndexMap<String, BlockNode>,
    pub edges: Vec<BlockEdge>,
    pub rows: Vec<BlockRow>, // row-by-row layout info
}

pub fn parse(input: &str) -> crate::error::ParseResult<BlockDiagram> {
    let mut diag = BlockDiagram {
        columns: 1,
        ..Default::default()
    };

    let mut current_row_items: Vec<RowItem> = Vec::new();
    let mut current_columns = 1usize;
    let mut in_block_stack: Vec<String> = Vec::new(); // nested block ids
    let mut in_yaml = false;

    let lines = input.lines().peekable();

    for raw_line in lines {
        let line = strip_comment(raw_line).trim().to_string();
        if line.is_empty() {
            continue;
        }

        // YAML frontmatter
        if line == "---" {
            in_yaml = !in_yaml;
            continue;
        }
        if in_yaml {
            continue;
        }

        // Diagram declaration
        if line == "block-beta" || line.starts_with("block-beta") {
            continue;
        }

        // accTitle / accDescr — skip
        if line.starts_with("accTitle") || line.starts_with("accDescr") {
            continue;
        }

        // columns N
        if let Some(rest) = line
            .strip_prefix("columns ")
            .or_else(|| line.strip_prefix("columns\t"))
        {
            let n: usize = rest.trim().parse().unwrap_or(1);
            if in_block_stack.is_empty() {
                diag.columns = n;
                current_columns = n;
            }
            continue;
        }

        // End of nested block — pop the block stack, restore columns
        if line == "end" {
            // Children were already added to group_children via the `continue` path above.
            // current_row_items still holds the group node itself — leave it for parent layout.
            in_block_stack.pop();
            current_columns = diag.columns;
            continue;
        }

        // Nested block open: block:id["label"]
        if line.starts_with("block:") || (line.starts_with("block") && line.contains(':')) {
            let rest = &line[5..]; // skip "block"
            let rest = rest.trim_start_matches(':').trim();
            let (id, label, shape) = parse_node_token_str(rest);
            if !current_row_items.is_empty() {
                diag.rows.push(BlockRow {
                    items: std::mem::take(&mut current_row_items),
                });
            }
            diag.nodes.insert(
                id.clone(),
                BlockNode {
                    id: id.clone(),
                    label,
                    shape,
                    col_span: 1,
                    is_group: true,
                    group_children: Vec::new(),
                    style: None,
                },
            );
            current_row_items.push(RowItem::Node(id.clone(), 1));
            in_block_stack.push(id);
            // Inside a nested block, items are added as group_children, not diag.rows
            current_columns = usize::MAX;
            continue;
        }

        // Style directive: style <id> <attrs> — parse and apply to node
        if line.starts_with("classDef ") || line.starts_with("class ") {
            continue;
        }
        if let Some(style_rest) = line.strip_prefix("style ") {
            let rest = style_rest.trim();
            // rest = "<id> <attrs>"
            if let Some(sp) = rest.find(|c: char| c.is_whitespace()) {
                let node_id = rest[..sp].trim().to_string();
                let attrs = rest[sp..].trim();
                // Convert comma-separated CSS props to semicolons: "fill:#969,stroke:#333" → "fill:#969;stroke:#333"
                let css = attrs.replace(',', ";");
                if let Some(node) = diag.nodes.get_mut(&node_id) {
                    node.style = Some(css);
                }
            }
            continue;
        }

        // Edge: A --> B, A -- "label" --> B, A -->|label| B
        if is_edge_line(&line) {
            if let Some(edge) = parse_edge(&line) {
                if !current_row_items.is_empty() {
                    diag.rows.push(BlockRow {
                        items: std::mem::take(&mut current_row_items),
                    });
                }
                // Ensure both nodes exist as default
                for id in [&edge.from, &edge.to] {
                    if !diag.nodes.contains_key(id.as_str()) {
                        diag.nodes.insert(
                            id.clone(),
                            BlockNode {
                                id: id.clone(),
                                label: id.clone(),
                                shape: BlockShape::Default,
                                col_span: 1,
                                is_group: false,
                                group_children: Vec::new(),
                                style: None,
                            },
                        );
                    }
                }
                diag.edges.push(edge);
                continue;
            }
        }

        // Row of nodes / spaces
        let items = parse_row_items(&line, &mut diag.nodes);
        if !items.is_empty() {
            // Inside a nested block: add nodes as group_children of the current group
            if let Some(group_id) = in_block_stack.last().cloned() {
                for item in &items {
                    if let RowItem::Node(child_id, _) = item {
                        if let Some(group) = diag.nodes.get_mut(&group_id) {
                            group.group_children.push(child_id.clone());
                        }
                    }
                }
                // Don't add to current_row_items or diag.rows
                continue;
            }

            // Top-level: check if we need a new row
            let cur_len: usize = current_row_items.iter().map(item_span).sum();
            let new_len: usize = items.iter().map(item_span).sum();
            // Flush current row if: overflow, OR columns=1 (each item on its own row)
            let should_flush = cur_len > 0
                && ((cur_len + new_len > current_columns && current_columns > 1)
                    || current_columns <= 1);
            if should_flush {
                diag.rows.push(BlockRow {
                    items: std::mem::take(&mut current_row_items),
                });
            }
            current_row_items.extend(items);

            // Auto-flush row when columns reached or columns=1
            let total: usize = current_row_items.iter().map(item_span).sum();
            if (total >= current_columns && current_columns > 1) || current_columns <= 1 {
                diag.rows.push(BlockRow {
                    items: std::mem::take(&mut current_row_items),
                });
            }
        }
    }

    // Flush remaining items
    if !current_row_items.is_empty() {
        diag.rows.push(BlockRow {
            items: current_row_items,
        });
    }

    crate::error::ParseResult::ok(diag)
}

fn item_span(item: &RowItem) -> usize {
    match item {
        RowItem::Space(n) => *n,
        RowItem::Node(_, n) => *n,
    }
}

fn is_edge_line(line: &str) -> bool {
    line.contains("-->") || line.contains("---") || line.contains("--")
}

fn parse_edge(line: &str) -> Option<BlockEdge> {
    // Patterns:
    //   A --> B
    //   A -- "label" --> B
    //   A -->|label| B
    let line = line.trim();

    // Try A --> B or A -->|label| B
    if let Some(pos) = line.find("-->") {
        let from = line[..pos].trim().to_string();
        let after = line[pos + 3..].trim();
        // Check for |label|
        if let Some(stripped) = after.strip_prefix('|') {
            let end = after.find('|').unwrap_or(after.len() - 1);
            if let Some(close) = stripped.find('|') {
                let label = stripped[..close].trim().to_string();
                let to_part = stripped[close + 1..].trim().to_string();
                return Some(BlockEdge {
                    from: clean_id(from),
                    to: clean_id(to_part),
                    label: if label.is_empty() { None } else { Some(label) },
                });
            }
            let _ = end;
        }
        // A --> B (possibly with whitespace)
        return Some(BlockEdge {
            from: clean_id(from),
            to: clean_id(after.to_string()),
            label: None,
        });
    }

    // Try A -- "label" --> B
    if let Some(dpos) = line.find(" -- ") {
        let from = line[..dpos].trim().to_string();
        let rest = &line[dpos + 4..];
        // Find the arrow
        if let Some(apos) = rest.find("-->") {
            let label_part = rest[..apos].trim().trim_matches('"').to_string();
            let to_part = rest[apos + 3..].trim().to_string();
            return Some(BlockEdge {
                from: clean_id(from),
                to: clean_id(to_part),
                label: if label_part.is_empty() {
                    None
                } else {
                    Some(label_part)
                },
            });
        }
    }

    None
}

fn clean_id(s: String) -> String {
    s.trim().trim_matches('"').to_string()
}

/// Parse a row line into RowItems, registering new nodes in the node map.
fn parse_row_items(line: &str, nodes: &mut indexmap::IndexMap<String, BlockNode>) -> Vec<RowItem> {
    let mut items = Vec::new();
    let line = line.trim();
    // Split by whitespace but respect quoted strings and brackets
    let tokens = tokenize_row(line);
    for tok in &tokens {
        if tok.is_empty() {
            continue;
        }
        // space or space:N
        if tok == "space" {
            items.push(RowItem::Space(1));
            continue;
        }
        if let Some(rest) = tok.strip_prefix("space:") {
            let n: usize = rest.parse().unwrap_or(1);
            items.push(RowItem::Space(n));
            continue;
        }
        // node token id["label"]:span or id["label"] or id
        let (id, label, shape) = parse_node_token_str(tok);
        if id.is_empty() {
            continue;
        }
        // Check for :N span suffix on the raw token
        let (id2, span) = extract_span_suffix(&id);
        let node_id = id2;
        if !nodes.contains_key(node_id.as_str()) {
            nodes.insert(
                node_id.clone(),
                BlockNode {
                    id: node_id.clone(),
                    label,
                    shape,
                    col_span: span,
                    is_group: false,
                    group_children: Vec::new(),
                    style: None,
                },
            );
        }
        items.push(RowItem::Node(node_id, span));
    }
    items
}

/// Extract ":N" span suffix from id if present.
fn extract_span_suffix(id: &str) -> (String, usize) {
    if let Some(pos) = id.rfind(':') {
        let suffix = &id[pos + 1..];
        if let Ok(n) = suffix.parse::<usize>() {
            return (id[..pos].to_string(), n);
        }
    }
    (id.to_string(), 1)
}

/// Tokenize a row line into separate tokens, respecting brackets/quotes.
fn tokenize_row(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    let mut in_quote = false;

    for c in line.chars() {
        match c {
            '"' => {
                in_quote = !in_quote;
                current.push(c);
            }
            '[' | '(' | '{' if !in_quote => {
                depth += 1;
                current.push(c);
            }
            ']' | ')' | '}' if !in_quote => {
                depth -= 1;
                current.push(c);
            }
            ' ' | '\t' if !in_quote && depth == 0 => {
                let t = current.trim().to_string();
                if !t.is_empty() {
                    tokens.push(t);
                }
                current = String::new();
            }
            _ => current.push(c),
        }
    }
    let t = current.trim().to_string();
    if !t.is_empty() {
        tokens.push(t);
    }
    tokens
}

/// Parse a single node token string into (id, label, shape).
pub fn parse_node_token_str(tok: &str) -> (String, String, BlockShape) {
    // Patterns:
    //   A               → id="A", label="A", shape=Default
    //   A["Label"]      → id="A", label="Label", shape=Square
    //   A("Label")      → id="A", label="Label", shape=RoundedRect
    //   A[("Label")]    → id="A", label="Label", shape=Cylinder
    //   A{"Label"}      → id="A", label="Label", shape=Diamond
    //   A(("Label"))    → id="A", label="Label", shape=Circle
    //   A{{"Label"}}    → id="A", label="Label", shape=Hexagon
    let tok = tok.trim();

    // Block arrow: id<["label"]>(direction)
    if let Some(lt_pos) = tok.find('<') {
        let id_part = tok[..lt_pos].trim().to_string();
        let rest = &tok[lt_pos..];
        // Extract label from <["..."]> and direction from (...)
        if let Some(lb) = rest.find("[\"").or_else(|| rest.find("['")) {
            let rb = rest.rfind(']').unwrap_or(rest.len());
            let inner = &rest[lb + 2..rb.min(rest.len()).saturating_sub(1)];
            return (id_part, inner.to_string(), BlockShape::BlockArrow);
        }
        // Simple <["label"]> without quotes
        if rest.starts_with("<[") {
            let inner = rest
                .trim_start_matches("<[")
                .trim_end_matches("]>")
                .trim_end_matches("](down)")
                .trim_end_matches("](up)")
                .trim_end_matches("](left)")
                .trim_end_matches("](right)");
            return (id_part, inner.to_string(), BlockShape::BlockArrow);
        }
    }

    // Find where the shape bracket starts
    let bracket_start = tok.find(['[', '(', '{']);

    if let Some(pos) = bracket_start {
        let id_part = tok[..pos].trim().to_string();
        let shape_part = &tok[pos..];

        let (shape, label) = if shape_part.starts_with("[((") {
            // Not standard, treat as square
            let inner = extract_inner(shape_part, '[', ']');
            (BlockShape::Square, unquote(inner))
        } else if shape_part.starts_with("[(") {
            let inner = extract_inner_multi(shape_part, "[(", ")]");
            (BlockShape::Cylinder, unquote(inner))
        } else if shape_part.starts_with("[[") {
            let inner = extract_inner_multi(shape_part, "[[", "]]");
            (BlockShape::Square, unquote(inner))
        } else if shape_part.starts_with('[') {
            let inner = extract_inner(shape_part, '[', ']');
            (BlockShape::Square, unquote(inner))
        } else if shape_part.starts_with("((") {
            let inner = extract_inner_multi(shape_part, "((", "))");
            (BlockShape::Circle, unquote(inner))
        } else if shape_part.starts_with('(') {
            let inner = extract_inner(shape_part, '(', ')');
            (BlockShape::RoundedRect, unquote(inner))
        } else if shape_part.starts_with("{{") {
            let inner = extract_inner_multi(shape_part, "{{", "}}");
            (BlockShape::Hexagon, unquote(inner))
        } else if shape_part.starts_with('{') {
            let inner = extract_inner(shape_part, '{', '}');
            (BlockShape::Diamond, unquote(inner))
        } else {
            (BlockShape::Default, id_part.clone())
        };

        let id = if id_part.is_empty() {
            // Anonymous — generate from label
            label
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect()
        } else {
            id_part
        };

        (id, label, shape)
    } else {
        // Bare identifier
        (tok.to_string(), tok.to_string(), BlockShape::Default)
    }
}

fn extract_inner(s: &str, open: char, close: char) -> &str {
    if let Some(start) = s.find(open) {
        let after = &s[start + 1..];
        if let Some(end) = after.rfind(close) {
            return &after[..end];
        }
    }
    s
}

fn extract_inner_multi<'a>(s: &'a str, open: &str, close: &str) -> &'a str {
    if let Some(start) = s.find(open) {
        let after = &s[start + open.len()..];
        if let Some(end) = after.find(close) {
            return &after[..end];
        }
        return after;
    }
    s
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn strip_comment(line: &str) -> &str {
    if let Some(pos) = line.find("%%") {
        &line[..pos]
    } else {
        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_block() {
        let input = "block-beta\n    columns 3\n    A[\"A\"] B[\"B\"] C[\"C\"]\n    space D[\"D\"] space\n    A --> D\n    B --> D";
        let diag = parse(input).diagram;
        assert_eq!(diag.columns, 3);
        assert!(diag.nodes.contains_key("A"));
        assert!(diag.nodes.contains_key("D"));
        assert_eq!(diag.edges.len(), 2);
    }

    #[test]
    fn parse_node_token_square() {
        let (id, label, shape) = parse_node_token_str(r#"A["Label A"]"#);
        assert_eq!(id, "A");
        assert_eq!(label, "Label A");
        assert_eq!(shape, BlockShape::Square);
    }

    #[test]
    fn parse_space() {
        let input = "block-beta\n    columns 3\n    space A[\"B\"] space\n";
        let diag = parse(input).diagram;
        assert_eq!(diag.columns, 3);
        assert!(diag.nodes.contains_key("A"));
    }
}
