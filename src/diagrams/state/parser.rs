// State diagram parser — faithful port of Mermaid stateDb.ts
// Supports stateDiagram-v2 and stateDiagram syntax.

use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn fresh_id(prefix: &str) -> String {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}{}", prefix, n)
}

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum StateType {
    /// Normal state
    Normal,
    /// [*] — used as both start and end depending on position
    Start,
    End,
    /// <<fork>>
    Fork,
    /// <<join>>
    Join,
    /// <<choice>>
    Choice,
    /// Composite (has sub-states)
    Composite,
    /// note
    Note,
}

#[derive(Debug, Clone)]
pub struct StateNode {
    pub id: String,
    /// Display label (for normal states, this is the state name; may have multi-line descriptions)
    pub label: String,
    pub state_type: StateType,
    /// If this is a composite state, these are the sub-statements
    pub doc: Vec<StateStmt>,
    /// Note text
    pub note_text: Option<String>,
    /// Note position (right/left)
    pub note_pos: Option<String>,
}

impl StateNode {
    pub fn new(id: &str, state_type: StateType) -> Self {
        StateNode {
            id: id.to_string(),
            label: id.to_string(),
            state_type,
            doc: Vec::new(),
            note_text: None,
            note_pos: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub enum StateStmt {
    State(StateNode),
    Transition(Transition),
    /// direction LR / TB / etc.
    Direction(String),
}

#[derive(Debug)]
pub struct StateDiagram {
    /// Top-level statements
    pub stmts: Vec<StateStmt>,
    /// Overall direction
    pub direction: String,
}

// ─── Parser ──────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<StateDiagram> {
    let mut diag = StateDiagram {
        stmts: Vec::new(),
        direction: "TB".to_string(),
    };

    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    // Skip header line (stateDiagram-v2 / stateDiagram)
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with("stateDiagram") {
            i += 1;
            break;
        }
        i += 1;
    }

    let (stmts, dir) = parse_block(&lines, &mut i, false);
    diag.stmts = stmts;
    // Look for direction in top-level stmts
    for stmt in &diag.stmts {
        if let StateStmt::Direction(d) = stmt {
            diag.direction = d.clone();
        }
    }
    let _ = dir;

    crate::error::ParseResult::ok(diag)
}

/// Parse a block of statements until we hit `end` or end-of-input.
/// Returns (stmts, direction).
/// `inside_state` is true when we are inside a `state X { ... }` block.
fn parse_block(lines: &[&str], i: &mut usize, inside_state: bool) -> (Vec<StateStmt>, String) {
    let mut stmts: Vec<StateStmt> = Vec::new();
    let mut direction = "TB".to_string();

    while *i < lines.len() {
        let raw = lines[*i];
        let line = strip_comment(raw).trim().to_string();
        *i += 1;

        if line.is_empty() {
            continue;
        }

        // End of composite block
        if line == "}" || line == "end" {
            if inside_state {
                break;
            }
            continue;
        }

        // direction statement
        if let Some(rest) = line.strip_prefix("direction ") {
            direction = rest.trim().to_string();
            stmts.push(StateStmt::Direction(direction.clone()));
            continue;
        }

        // Note: "note right of X" / "note left of X"
        if line.starts_with("note ") {
            if let Some(note_stmt) = parse_note(&line, lines, i) {
                stmts.push(note_stmt);
            }
            continue;
        }

        // `state X <<type>>` or `state X { ... }` or `state "label" as X`
        if let Some(rest) = line.strip_prefix("state ") {
            if let Some(stmt) = parse_state_decl(rest, lines, i) {
                stmts.push(stmt);
            }
            continue;
        }

        // Transition: X --> Y [: label]
        if line.contains("-->") {
            if let Some(t) = parse_transition(&line) {
                stmts.push(StateStmt::Transition(t));
            }
            continue;
        }

        // Could be a bare state name or `StateId: description` (implicit declaration)
        if !line.is_empty() && !line.starts_with("%%") {
            // Check for `StateId: description` (description form)
            if let Some(colon_pos) = line.find(':') {
                let id_part = line[..colon_pos].trim();
                let desc_part = line[colon_pos + 1..].trim();
                // Validate: id_part must look like a valid state ID (no spaces), desc non-empty
                if is_valid_state_id(id_part) && !id_part.contains(' ') && !desc_part.is_empty() {
                    let mut node = StateNode::new(&resolve_id(id_part), StateType::Normal);
                    node.label = desc_part.to_string();
                    stmts.push(StateStmt::State(node));
                    continue;
                }
            }
            // Bare state name
            let id = line.trim().to_string();
            if is_valid_state_id(&id) {
                let node = StateNode::new(&resolve_id(&id), StateType::Normal);
                stmts.push(StateStmt::State(node));
            }
        }
    }

    (stmts, direction)
}

/// Parse `state DECL` where DECL is one of:
///   - `"Quoted Label" as StateId`
///   - `StateId <<fork>>`
///   - `StateId <<join>>`
///   - `StateId <<choice>>`
///   - `StateId { ... }` (composite)
///   - `StateId` (bare)
fn parse_state_decl(rest: &str, lines: &[&str], i: &mut usize) -> Option<StateStmt> {
    let rest = rest.trim();

    // `"Quoted Label" as StateId`
    if let Some(rest_stripped) = rest.strip_prefix('"') {
        if let Some(close) = rest_stripped.find('"') {
            let label = &rest_stripped[..close];
            let after = rest_stripped[close + 1..].trim();
            let id = if let Some(as_rest) = after.strip_prefix("as ") {
                as_rest.trim().to_string()
            } else {
                // no `as` — use label as id
                label.to_string()
            };
            let mut node = StateNode::new(&id, StateType::Normal);
            node.label = label.to_string();
            return Some(StateStmt::State(node));
        }
    }

    // `StateId <<type>>`
    if let Some(bracket_start) = rest.find("<<") {
        if let Some(bracket_end) = rest.find(">>") {
            let id = rest[..bracket_start].trim().to_string();
            let type_str = &rest[bracket_start + 2..bracket_end];
            let state_type = match type_str.trim() {
                "fork" => StateType::Fork,
                "join" => StateType::Join,
                "choice" => StateType::Choice,
                _ => StateType::Normal,
            };
            let node = StateNode::new(&id, state_type);
            return Some(StateStmt::State(node));
        }
    }

    // `StateId { ... }` — composite state with optional `{` on same line or next
    // Check if rest ends with `{` or the next non-empty line is `{`
    let (id_part, open_brace_here) = if rest.ends_with('{') {
        (rest.trim_end_matches('{').trim(), true)
    } else {
        (rest, false)
    };

    let id = id_part.trim().to_string();

    // If `{` is on same line or peek at next line
    let has_brace = open_brace_here || peek_brace(lines, *i);

    if has_brace {
        if !open_brace_here {
            // consume the `{` line
            *i += 1;
        }
        let (sub_stmts, sub_dir) = parse_block(lines, i, true);
        let mut node = StateNode::new(&id, StateType::Composite);
        node.doc = sub_stmts;
        // Propagate direction if specified
        for stmt in &node.doc {
            if let StateStmt::Direction(d) = stmt {
                node.label = id.clone(); // keep label
                let _ = d; // direction is embedded in doc stmts
            }
        }
        let _ = sub_dir;
        return Some(StateStmt::State(node));
    }

    // Bare `state X`
    if !id.is_empty() {
        let node = StateNode::new(&id, StateType::Normal);
        return Some(StateStmt::State(node));
    }

    None
}

/// Parse a note statement.
/// Syntax:
///   note right of StateName
///     Text here
///   end note
fn parse_note(line: &str, lines: &[&str], i: &mut usize) -> Option<StateStmt> {
    // "note right of X" or "note left of X"
    let rest = &line["note ".len()..];
    let (pos, state_id) = if let Some(r) = rest.strip_prefix("right of ") {
        ("right", r.trim().to_string())
    } else if let Some(r) = rest.strip_prefix("left of ") {
        ("left", r.trim().to_string())
    } else {
        return None;
    };

    // Collect note text until "end note"
    let mut note_lines: Vec<String> = Vec::new();
    while *i < lines.len() {
        let raw = lines[*i];
        let l = strip_comment(raw).trim().to_string();
        *i += 1;
        if l == "end note" {
            break;
        }
        note_lines.push(l);
    }
    let note_text = note_lines.join("\n");

    let note_id = fresh_id("note_");
    let mut node = StateNode::new(&note_id, StateType::Note);
    node.note_text = Some(note_text);
    node.note_pos = Some(pos.to_string());
    // The note is "attached to" state_id — we encode as a note transition
    // We produce a fake note node + a transition to the target state
    // For simplicity: embed as a synthetic StateNode with label = state_id
    node.label = state_id;
    Some(StateStmt::State(node))
}

fn parse_transition(line: &str) -> Option<Transition> {
    // Split on -->
    let parts: Vec<&str> = line.splitn(2, "-->").collect();
    if parts.len() != 2 {
        return None;
    }
    let from_raw = parts[0].trim();
    let rest = parts[1].trim();

    // rest may have `: label`
    let (to_raw, label) = if let Some(col) = rest.find(':') {
        let to = rest[..col].trim();
        let lbl = rest[col + 1..].trim().to_string();
        (to, if lbl.is_empty() { None } else { Some(lbl) })
    } else {
        (rest, None)
    };

    let from = resolve_id(from_raw.trim());
    let to = resolve_id(to_raw.trim());

    if from.is_empty() || to.is_empty() {
        return None;
    }

    Some(Transition { from, to, label })
}

/// [*] maps to a special sentinel id that we resolve during rendering
fn resolve_id(raw: &str) -> String {
    match raw {
        "[*]" => "[*]".to_string(),
        s => s.to_string(),
    }
}

fn is_valid_state_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with("%%")
        && !id.starts_with("--")
        && id
            .chars()
            .next()
            .map(|c| c.is_alphanumeric() || c == '[' || c == '_')
            .unwrap_or(false)
}

fn strip_comment(line: &str) -> &str {
    if let Some(pos) = line.find("%%") {
        &line[..pos]
    } else {
        line
    }
}

/// Peek at lines starting at index to see if the next non-empty line is `{`
fn peek_brace(lines: &[&str], from: usize) -> bool {
    for &l in &lines[from..] {
        let t = l.trim();
        if t.is_empty() {
            continue;
        }
        return t == "{";
    }
    false
}

// ─── Flattening helpers ───────────────────────────────────────────────────────

/// Flatten a StateDiagram into a list of all (unique) state nodes and
/// transitions, properly numbering [*] references per context.
///
/// Each composite state "owns" its own start/end [*] instances.
/// Returns (states, transitions) where states are unique by id.
pub struct FlatGraph {
    /// All unique state nodes keyed by id
    pub states: indexmap::IndexMap<String, StateNode>,
    /// All transitions (from, to, label)
    pub transitions: Vec<Transition>,
    /// Subgraph membership: state_id -> parent_composite_id
    pub parent: std::collections::HashMap<String, String>,
}

pub fn flatten(diag: &StateDiagram) -> FlatGraph {
    let mut graph = FlatGraph {
        states: indexmap::IndexMap::new(),
        transitions: Vec::new(),
        parent: std::collections::HashMap::new(),
    };

    flatten_stmts(&diag.stmts, None, &mut graph);
    graph
}

fn flatten_stmts(stmts: &[StateStmt], parent_id: Option<&str>, graph: &mut FlatGraph) {
    // First pass: allocate unique ids for [*] in this context
    // Count how many [*] transitions appear to decide start vs end
    let star_count = stmts
        .iter()
        .filter(|s| {
            if let StateStmt::Transition(t) = s {
                t.from == "[*]" || t.to == "[*]"
            } else {
                false
            }
        })
        .count();
    let _ = star_count;

    // Per-context ids for [*] start/end
    let ctx_prefix = parent_id.unwrap_or("root");
    let start_id = format!("{}_start", ctx_prefix);
    let end_id = format!("{}_end", ctx_prefix);

    for stmt in stmts {
        match stmt {
            StateStmt::State(node) => {
                match node.state_type {
                    StateType::Composite => {
                        // Register the composite node itself
                        let mut n = node.clone();
                        n.doc = Vec::new(); // clear, children handled separately
                        graph.states.insert(node.id.clone(), n);
                        if let Some(pid) = parent_id {
                            graph.parent.insert(node.id.clone(), pid.to_string());
                        }
                        // Recurse into children
                        flatten_stmts(&node.doc, Some(&node.id), graph);
                    }
                    StateType::Note => {
                        // Note nodes: we register them but they're rendered specially
                        graph.states.insert(node.id.clone(), node.clone());
                        if let Some(pid) = parent_id {
                            graph.parent.insert(node.id.clone(), pid.to_string());
                        }
                    }
                    _ => {
                        graph
                            .states
                            .entry(node.id.clone())
                            .or_insert_with(|| node.clone());
                        if let Some(pid) = parent_id {
                            graph
                                .parent
                                .entry(node.id.clone())
                                .or_insert_with(|| pid.to_string());
                        }
                    }
                }
            }
            StateStmt::Transition(t) => {
                let from = if t.from == "[*]" {
                    // Register start node if not yet
                    if !graph.states.contains_key(&start_id) {
                        let mut n = StateNode::new(&start_id, StateType::Start);
                        n.label = start_id.clone();
                        graph.states.insert(start_id.clone(), n);
                        if let Some(pid) = parent_id {
                            graph.parent.insert(start_id.clone(), pid.to_string());
                        }
                    }
                    start_id.clone()
                } else {
                    // Ensure state exists
                    graph
                        .states
                        .entry(t.from.clone())
                        .or_insert_with(|| StateNode::new(&t.from, StateType::Normal));
                    if let Some(pid) = parent_id {
                        graph
                            .parent
                            .entry(t.from.clone())
                            .or_insert_with(|| pid.to_string());
                    }
                    t.from.clone()
                };

                let to = if t.to == "[*]" {
                    if !graph.states.contains_key(&end_id) {
                        let mut n = StateNode::new(&end_id, StateType::End);
                        n.label = end_id.clone();
                        graph.states.insert(end_id.clone(), n);
                        if let Some(pid) = parent_id {
                            graph.parent.insert(end_id.clone(), pid.to_string());
                        }
                    }
                    end_id.clone()
                } else {
                    graph
                        .states
                        .entry(t.to.clone())
                        .or_insert_with(|| StateNode::new(&t.to, StateType::Normal));
                    if let Some(pid) = parent_id {
                        graph
                            .parent
                            .entry(t.to.clone())
                            .or_insert_with(|| pid.to_string());
                    }
                    t.to.clone()
                };

                graph.transitions.push(Transition {
                    from,
                    to,
                    label: t.label.clone(),
                });
            }
            StateStmt::Direction(_) => {}
        }
    }
}
