// Faithful port of Mermaid stateDiagram-v2 parser (stateDb.ts + grammar)
// Handles stateDiagram-v2 and stateDiagram syntax.

use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[allow(dead_code)]
pub fn reset_counter() {
    COUNTER.store(0, Ordering::Relaxed);
}

fn fresh_id() -> u32 {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

// ─── Constants (from stateCommon.ts) ─────────────────────────────────────────

pub const DOMID_STATE: &str = "state";
#[allow(dead_code)]
pub const DOMID_TYPE_SPACER: &str = "----";
pub const NOTE_ID_SUFFIX: &str = "----note"; // NOTE_ID  = "----note"
pub const PARENT_ID_SUFFIX: &str = "----parent"; // PARENT_ID = "----parent"

// ─── Types ───────────────────────────────────────────────────────────────────

/// Maps to shape strings used in dataFetcher.ts
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// "rect" — normal state
    Rect,
    /// "rectWithTitle" — state with description lines
    RectWithTitle,
    /// "stateStart" — [*] as start
    StateStart,
    /// "stateEnd" — [*] as end
    StateEnd,
    /// "divider"
    #[allow(dead_code)]
    Divider,
    /// "roundedWithTitle" — composite group
    RoundedWithTitle,
    /// "note"
    Note,
    /// "noteGroup" — compound parent for note
    NoteGroup,
    /// "fork" / "join"
    ForkJoin,
    /// "choice"
    Choice,
}

/// A node in the layout graph — mirrors NodeData from dataFetcher.ts
#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub dom_id: String,
    pub shape: Shape,
    /// Display label / description
    pub label: String,
    /// For compound parent nodes: which node is the parent
    pub parent_id: Option<String>,
    /// padding (note compound uses 16, states use 8)
    #[allow(dead_code)]
    pub padding: f64,
    /// CSS classes string
    #[allow(dead_code)]
    pub css_classes: String,
    /// For composite states: sub-level nodes and edges
    pub is_group: bool,
    pub dir: String,
    /// For notes: position ("right of" / "left of")
    #[allow(dead_code)]
    pub position: Option<String>,
}

/// An edge in the layout graph — mirrors Edge from dataFetcher.ts
#[derive(Debug, Clone)]
pub struct Edge {
    #[allow(dead_code)]
    pub id: String,
    pub start: String,
    pub end: String,
    pub label: String,
    /// "none" for note edges, "normal" for state transitions
    pub arrowhead: String,
    /// CSS classes: "transition" or "transition note-edge"
    pub classes: String,
}

/// Top-level parse result
pub struct StateDiagram {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub direction: String,
}

// ─── Parser ──────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> StateDiagram {
    COUNTER.store(0, Ordering::Relaxed);
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut direction = "TB".to_string();

    // Strip YAML frontmatter
    let input = strip_frontmatter(input);
    let lines: Vec<&str> = input.lines().collect();

    let mut in_header = true;
    let mut i = 0;

    // Track which node ids have been seen (to avoid duplicates)
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    // Track composite parent context
    let mut composite_stack: Vec<String> = Vec::new();

    while i < lines.len() {
        let raw = lines[i];
        let line = strip_comment(raw).trim().to_string();
        i += 1;

        if line.is_empty() {
            continue;
        }

        // Wait for diagram header
        if in_header {
            if line.starts_with("stateDiagram") {
                in_header = false;
            }
            continue;
        }

        // direction
        if let Some(dir) = line.strip_prefix("direction ") {
            direction = dir.trim().to_string();
            continue;
        }

        // accTitle / accDescr — skip
        if line.starts_with("accTitle") || line.starts_with("accDescr") {
            continue;
        }

        // End of composite block
        if line == "}" {
            composite_stack.pop();
            continue;
        }

        // Note: "note right of X" / "note left of X"
        if line.starts_with("note ") {
            if let Some((note_nodes, note_edges)) =
                parse_note(&line, &lines, &mut i, &mut seen, &composite_stack)
            {
                for n in note_nodes {
                    nodes.push(n);
                }
                for e in note_edges {
                    edges.push(e);
                }
            }
            continue;
        }

        // Composite state: "state X {"
        if line.ends_with('{') {
            let id = line.trim_end_matches('{').trim().to_string();
            let id = if let Some(r) = id.strip_prefix("state ") {
                r.trim().to_string()
            } else {
                id
            };
            let parent_id = composite_stack.last().cloned();
            // Update existing node to be composite group (it may have been added as Rect)
            if let Some(n) = nodes.iter_mut().find(|n| n.id == id) {
                n.shape = Shape::RoundedWithTitle;
                n.is_group = true;
                n.dir = direction.clone();
            } else {
                ensure_node(
                    &id,
                    Shape::RoundedWithTitle,
                    &id,
                    parent_id,
                    8.0,
                    true,
                    &direction,
                    None,
                    &mut nodes,
                    &mut seen,
                );
            }
            composite_stack.push(id);
            continue;
        }

        // State with annotation: "state \"label\" as ID" or "state ID <<type>>"
        if line.starts_with("state ") {
            if let Some(rest) = line.strip_prefix("state ") {
                let parent_id = composite_stack.last().cloned();
                if let Some(id) = parse_state_declaration(rest, &mut nodes, &mut seen, parent_id) {
                    let _ = id;
                }
            }
            continue;
        }

        // Relation: "A --> B" or "A --> B : label"
        if line.contains("-->") {
            let (from_raw, to_raw, label) = parse_relation(&line);
            // docTranslator: [*] as first → "{parent}_start", as second → "{parent}_end"
            let parent_ctx = composite_stack
                .last()
                .cloned()
                .unwrap_or_else(|| "root".to_string());
            let from = if from_raw == "[*]" {
                format!("{}_start", parent_ctx)
            } else {
                from_raw.clone()
            };
            let to = if to_raw == "[*]" {
                format!("{}_end", parent_ctx)
            } else {
                to_raw.clone()
            };
            let from_shape = if from.ends_with("_start") {
                Shape::StateStart
            } else {
                node_shape(&from)
            };
            let to_shape = if to.ends_with("_end") {
                Shape::StateEnd
            } else {
                node_shape(&to)
            };
            let parent_id = if composite_stack.is_empty() {
                None
            } else {
                Some(parent_ctx)
            };
            ensure_node(
                &from,
                from_shape,
                &from,
                parent_id.clone(),
                8.0,
                false,
                &direction,
                None,
                &mut nodes,
                &mut seen,
            );
            ensure_node(
                &to, to_shape, &to, parent_id, 8.0, false, &direction, None, &mut nodes, &mut seen,
            );
            let edge_id = format!("edge{}", fresh_id());
            edges.push(Edge {
                id: edge_id,
                start: from,
                end: to,
                label,
                arrowhead: "normal".to_string(),
                classes: "transition".to_string(),
            });
            continue;
        }

        // "ID: description" or bare state declaration
        if !line.is_empty() {
            let parent_id = composite_stack.last().cloned();
            let (id, label) = if let Some(pos) = line.find(':') {
                let id = line[..pos].trim().to_string();
                let label = line[pos + 1..].trim().to_string();
                (id, label)
            } else {
                (line.clone(), line.clone())
            };
            // Update label if node already exists
            if let Some(n) = nodes.iter_mut().find(|n| n.id == id) {
                if !label.is_empty() && label != id {
                    n.label = label.clone();
                }
            } else {
                let shape = if label != id {
                    Shape::RectWithTitle
                } else {
                    node_shape(&id)
                };
                ensure_node(
                    &id, shape, &label, parent_id, 8.0, false, &direction, None, &mut nodes,
                    &mut seen,
                );
            }
        }
    }

    StateDiagram {
        nodes,
        edges,
        direction,
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn node_shape(id: &str) -> Shape {
    match id {
        "[*]" => Shape::StateStart, // direction resolved at edge-attach time
        _ => Shape::Rect,
    }
}

/// Ensure a node with given id exists; add it if not seen yet.
#[allow(clippy::too_many_arguments)]
fn ensure_node(
    id: &str,
    shape: Shape,
    label: &str,
    parent_id: Option<String>,
    padding: f64,
    is_group: bool,
    dir: &str,
    position: Option<String>,
    nodes: &mut Vec<Node>,
    seen: &mut std::collections::HashSet<String>,
) {
    if seen.contains(id) {
        return;
    }
    seen.insert(id.to_string());
    let count = fresh_id();
    let dom_id = format!("{}-{}-{}", DOMID_STATE, id, count);
    let css_classes = match &shape {
        Shape::Note => "statediagram-note".to_string(),
        Shape::NoteGroup => "statediagram-cluster".to_string(),
        Shape::StateStart | Shape::StateEnd => "statediagram-state".to_string(),
        _ => "statediagram-state".to_string(),
    };
    nodes.push(Node {
        id: id.to_string(),
        dom_id,
        shape,
        label: label.to_string(),
        parent_id,
        padding,
        css_classes,
        is_group,
        dir: dir.to_string(),
        position,
    });
}

/// Parse "note right of X\n  text\nend note"
/// Returns (nodes_to_add, edges_to_add)
fn parse_note(
    line: &str,
    lines: &[&str],
    i: &mut usize,
    seen: &mut std::collections::HashSet<String>,
    composite_stack: &[String],
) -> Option<(Vec<Node>, Vec<Edge>)> {
    let rest = line.strip_prefix("note ")?;
    let (position, state_id) = if let Some(r) = rest.strip_prefix("right of ") {
        ("right of", r.trim().to_string())
    } else if let Some(r) = rest.strip_prefix("left of ") {
        ("left of", r.trim().to_string())
    } else {
        return None;
    };

    // Collect note text until "end note"
    let mut text_lines: Vec<String> = Vec::new();
    while *i < lines.len() {
        let l = strip_comment(lines[*i]).trim().to_string();
        *i += 1;
        if l == "end note" {
            break;
        }
        text_lines.push(l);
    }
    let text = text_lines.join("\n");

    let count = fresh_id();

    // noteGroup (compound parent) — id = "{stateId}----parent"
    let group_id = format!("{}{}", state_id, PARENT_ID_SUFFIX);
    let group_dom_id = format!("{}-{}{}-{}", DOMID_STATE, state_id, PARENT_ID_SUFFIX, count);

    // note node — id = "{stateId}----note-{count}"
    let note_id = format!("{}{}-{}", state_id, NOTE_ID_SUFFIX, count);
    let note_dom_id = format!("{}-{}{}-{}", DOMID_STATE, state_id, NOTE_ID_SUFFIX, count);

    // edge id
    let edge_id = format!("{}-{}", state_id, note_id);

    let parent_from_stack = composite_stack.last().cloned();

    let mut new_nodes = Vec::new();
    let mut new_edges = Vec::new();

    // groupData (noteGroup) — no parentId, padding=16
    if !seen.contains(&group_id) {
        seen.insert(group_id.clone());
        new_nodes.push(Node {
            id: group_id.clone(),
            dom_id: group_dom_id,
            shape: Shape::NoteGroup,
            label: text.clone(),
            parent_id: None,
            padding: 16.0,
            css_classes: "statediagram-cluster".to_string(),
            is_group: true,
            dir: "TB".to_string(),
            position: Some(position.to_string()),
        });
    }

    // noteData — parentId = group_id
    new_nodes.push(Node {
        id: note_id.clone(),
        dom_id: note_dom_id,
        shape: Shape::Note,
        label: text.clone(),
        parent_id: Some(group_id),
        padding: 8.0,
        css_classes: "statediagram-note".to_string(),
        is_group: false,
        dir: "TB".to_string(),
        position: Some(position.to_string()),
    });

    // stateNode — no parentId (root level)
    if !seen.contains(&state_id) {
        seen.insert(state_id.clone());
        let state_count = fresh_id();
        let state_dom_id = format!("{}-{}-{}", DOMID_STATE, state_id, state_count);
        new_nodes.push(Node {
            id: state_id.clone(),
            dom_id: state_dom_id,
            shape: Shape::Rect,
            label: state_id.clone(),
            parent_id: parent_from_stack,
            padding: 8.0,
            css_classes: "statediagram-state".to_string(),
            is_group: false,
            dir: "TB".to_string(),
            position: None,
        });
    }

    // note edge — "right of": from=state, to=note; "left of": from=note, to=state
    let (from, to) = if position == "left of" {
        (note_id.clone(), state_id.clone())
    } else {
        (state_id.clone(), note_id.clone())
    };
    new_edges.push(Edge {
        id: edge_id,
        start: from,
        end: to,
        label: String::new(),
        arrowhead: "none".to_string(),
        classes: "transition note-edge".to_string(),
    });

    Some((new_nodes, new_edges))
}

/// Resolve [*] start/end based on whether it appears as source or target
#[allow(dead_code)]
pub fn resolve_start_end(nodes: &mut [Node], edges: &[Edge]) {
    for edge in edges {
        if edge.start == "[*]" {
            // [*] as source → StateStart
            if let Some(n) = nodes.iter_mut().find(|n| n.id == "[*]") {
                n.shape = Shape::StateStart;
            }
        }
        if edge.end == "[*]" {
            // [*] as target → StateEnd
            // If there's already a StateStart [*], we need a second node
            // Mermaid uses the same id but the shape changes based on context.
            // Simple heuristic: if this [*] has an incoming edge, it's an end.
            if let Some(n) = nodes.iter_mut().find(|n| n.id == "[*]") {
                if n.shape == Shape::StateStart {
                    // Need a separate end node — use a unique id
                    // This is handled by the renderer using the edge direction
                } else {
                    n.shape = Shape::StateEnd;
                }
            }
        }
    }
}

fn parse_relation(line: &str) -> (String, String, String) {
    let parts: Vec<&str> = line.splitn(2, "-->").collect();
    let from = parts[0].trim().to_string();
    let rest = parts.get(1).map(|s| s.trim()).unwrap_or("");
    let (to, label) = if let Some(pos) = rest.find(':') {
        (
            rest[..pos].trim().to_string(),
            rest[pos + 1..].trim().to_string(),
        )
    } else {
        (rest.to_string(), String::new())
    };
    (from, to, label)
}

fn parse_state_declaration(
    rest: &str,
    nodes: &mut Vec<Node>,
    seen: &mut std::collections::HashSet<String>,
    parent_id: Option<String>,
) -> Option<String> {
    // "\"label\" as ID"
    if let Some(stripped) = rest.strip_prefix('"') {
        let end_quote = stripped.find('"')? + 1;
        let label = stripped[..end_quote - 1].to_string();
        let after = stripped[end_quote..].trim();
        let id = after.strip_prefix("as ")?.trim().to_string();
        ensure_node(
            &id,
            Shape::Rect,
            &label,
            parent_id,
            8.0,
            false,
            "TB",
            None,
            nodes,
            seen,
        );
        return Some(id);
    }
    // "ID <<type>>"
    if let Some(pos) = rest.find("<<") {
        let id = rest[..pos].trim().to_string();
        let type_str = rest[pos + 2..].trim_end_matches(">>").trim();
        let shape = match type_str {
            "fork" | "join" => Shape::ForkJoin,
            "choice" => Shape::Choice,
            _ => Shape::Rect,
        };
        ensure_node(
            &id, shape, &id, parent_id, 8.0, false, "TB", None, nodes, seen,
        );
        return Some(id);
    }
    None
}

fn strip_comment(s: &str) -> &str {
    if let Some(p) = s.find("%%") {
        &s[..p]
    } else {
        s
    }
}

fn strip_frontmatter(input: &str) -> &str {
    let t = input.trim_start();
    if !t.starts_with("---") {
        return input;
    }
    let after = &t[3..];
    if let Some(end) = after.find("\n---") {
        &after[end + 4..]
    } else {
        input
    }
}
