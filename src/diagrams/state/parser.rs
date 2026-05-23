/// Translation of Mermaid stateDb.ts + dataFetcher.ts

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    State,     // "rect"
    Start,     // "stateStart"
    End,       // "stateEnd"
    Fork,      // "fork"
    Join,      // "join"
    Choice,    // "choice"
    Divider,   // "divider"
    Group,     // "roundedWithTitle"
    NoteGroup, // "noteGroup"
    Note,      // "note"
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub shape: Shape,
    pub label: String,
    pub is_group: bool,
    pub parent_id: Option<String>,
    pub dir: String, // explicit direction (e.g. "LR"), empty if none
    #[allow(dead_code)]
    pub padding: f64,
    pub explicit_dir: bool,
}

#[derive(Debug, Clone)]
pub struct Edge {
    #[allow(dead_code)]
    pub id: String,
    pub start: String,
    pub end: String,
    pub label: String,
    pub classes: String,
}

#[derive(Debug)]
pub struct StateDiagram {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub direction: String,
}

// ─── Internal parse tree ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParsedState {
    pub id: String,
    pub state_type: String,
    pub description: Option<String>,
    pub doc: Vec<ParsedItem>,
    pub note: Option<ParsedNote>,
    pub start: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ParsedNote {
    pub position: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum ParsedItem {
    State(ParsedState),
    Relation {
        state1: ParsedState,
        state2: ParsedState,
        description: Option<String>,
    },
    Direction(String),
}

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> StateDiagram {
    let (raw_items, direction) = parse_raw(input);
    let mut root_doc = raw_items;
    let mut div_cnt = 0usize;
    doc_translator(&mut root_doc, "root", &mut div_cnt);
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut counter = 0usize;
    setup_doc(None, &root_doc, &mut nodes, &mut edges, true, &mut counter);
    StateDiagram {
        nodes,
        edges,
        direction,
    }
}

// ─── docTranslator ───────────────────────────────────────────────────────────

fn doc_translator(doc: &mut Vec<ParsedItem>, parent_id: &str, div_cnt: &mut usize) {
    let has_divider = doc
        .iter()
        .any(|i| matches!(i, ParsedItem::State(s) if s.state_type == "divider"));
    if has_divider {
        let original = std::mem::take(doc);
        let mut regions: Vec<Vec<ParsedItem>> = Vec::new();
        let mut cur: Vec<ParsedItem> = Vec::new();
        for item in original {
            if matches!(&item, ParsedItem::State(s) if s.state_type == "divider") {
                regions.push(std::mem::take(&mut cur));
            } else {
                cur.push(item);
            }
        }
        regions.push(cur);
        for mut region in regions {
            if region.is_empty() {
                continue;
            }
            *div_cnt += 1;
            let div_id = format!("divider-id-{}", div_cnt);
            doc_translator(&mut region, &div_id, div_cnt);
            doc.push(ParsedItem::State(ParsedState {
                id: div_id,
                state_type: "divider".into(),
                description: None,
                doc: region,
                note: None,
                start: None,
            }));
        }
    } else {
        for item in doc.iter_mut() {
            match item {
                ParsedItem::Relation { state1, state2, .. } => {
                    if state1.id == "[*]" {
                        state1.id = format!("{}_start", parent_id);
                        state1.start = Some(true);
                    }
                    if state2.id == "[*]" {
                        state2.id = format!("{}_end", parent_id);
                        state2.start = Some(false);
                    }
                }
                ParsedItem::State(s) if !s.doc.is_empty() => {
                    doc_translator(&mut s.doc, &s.id, div_cnt);
                }
                _ => {}
            }
        }
    }
}

// ─── setupDoc ────────────────────────────────────────────────────────────────

fn setup_doc(
    parent: Option<&ParsedState>,
    doc: &[ParsedItem],
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
    alt_flag: bool,
    counter: &mut usize,
) {
    for item in doc {
        match item {
            ParsedItem::State(s) => data_fetcher(parent, s, nodes, edges, alt_flag, counter),
            ParsedItem::Relation {
                state1,
                state2,
                description,
            } => {
                data_fetcher(parent, state1, nodes, edges, alt_flag, counter);
                data_fetcher(parent, state2, nodes, edges, alt_flag, counter);
                *counter += 1;
                edges.push(Edge {
                    id: format!("edge{}", counter),
                    start: state1.id.clone(),
                    end: state2.id.clone(),
                    label: description.clone().unwrap_or_default(),
                    classes: "transition".into(),
                });
            }
            ParsedItem::Direction(_) => {}
        }
    }
}

// ─── dataFetcher ─────────────────────────────────────────────────────────────

fn data_fetcher(
    parent: Option<&ParsedState>,
    parsed_item: &ParsedState,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
    alt_flag: bool,
    counter: &mut usize,
) {
    let item_id = &parsed_item.id;
    if item_id == "root" {
        return;
    }

    let mut shape = if parsed_item.start == Some(true) {
        Shape::Start
    } else if parsed_item.start == Some(false) {
        Shape::End
    } else {
        match parsed_item.state_type.as_str() {
            "fork" => Shape::Fork,
            "join" => Shape::Join,
            "choice" => Shape::Choice,
            "divider" => Shape::Divider,
            _ => Shape::State,
        }
    };

    let is_group = !parsed_item.doc.is_empty();
    if is_group && parsed_item.state_type == "default" {
        shape = Shape::Group;
    }

    let label = if matches!(
        shape,
        Shape::Start | Shape::End | Shape::Fork | Shape::Join | Shape::Divider
    ) {
        String::new()
    } else {
        parsed_item
            .description
            .clone()
            .map(|d| d.trim_start_matches(':').trim().to_string())
            .unwrap_or_else(|| item_id.clone())
    };

    let dir = find_dir(&parsed_item.doc);
    let explicit_dir = parsed_item
        .doc
        .iter()
        .any(|i| matches!(i, ParsedItem::Direction(_)));
    let parent_id: Option<String> = parent.filter(|p| p.id != "root").map(|p| p.id.clone());

    if let Some(ref note) = parsed_item.note {
        let note_id = format!("{}----note-{}", item_id, counter);
        let group_id = format!("{}----parent", item_id);
        *counter += 1;
        insert_or_update(
            nodes,
            Node {
                id: group_id.clone(),
                shape: Shape::NoteGroup,
                label: note.text.clone(),
                is_group: true,
                parent_id: parent_id.clone(),
                dir: String::new(),
                padding: 16.0,
                explicit_dir: false,
            },
        );
        insert_or_update(
            nodes,
            Node {
                id: note_id.clone(),
                shape: Shape::Note,
                label: note.text.clone(),
                is_group: false,
                parent_id: Some(group_id.clone()),
                dir: String::new(),
                padding: 8.0,
                explicit_dir: false,
            },
        );
        insert_or_update(
            nodes,
            Node {
                id: item_id.clone(),
                shape: shape.clone(),
                label: label.clone(),
                is_group,
                parent_id: parent_id.clone(),
                dir: dir.clone(),
                padding: if is_group { 16.0 } else { 8.0 },
                explicit_dir,
            },
        );
        let (from, to) = if note.position == "left of" {
            (note_id.clone(), item_id.clone())
        } else {
            (item_id.clone(), note_id.clone())
        };
        *counter += 1;
        edges.push(Edge {
            id: format!("{}-{}", from, to),
            start: from,
            end: to,
            label: String::new(),
            classes: "note-edge".into(),
        });
    } else {
        insert_or_update(
            nodes,
            Node {
                id: item_id.clone(),
                shape: shape.clone(),
                label,
                is_group,
                parent_id,
                dir: dir.clone(),
                padding: if is_group { 16.0 } else { 8.0 },
                explicit_dir,
            },
        );
    }

    if !parsed_item.doc.is_empty() {
        setup_doc(
            Some(parsed_item),
            &parsed_item.doc,
            nodes,
            edges,
            !alt_flag,
            counter,
        );
    }
}

fn insert_or_update(nodes: &mut Vec<Node>, new: Node) {
    if let Some(ex) = nodes.iter_mut().find(|n| n.id == new.id) {
        if new.is_group && !ex.is_group {
            ex.is_group = true;
            ex.shape = new.shape;
        }
        if !new.label.is_empty() && ex.label == ex.id {
            ex.label = new.label;
        }
        if !new.dir.is_empty() && ex.dir.is_empty() {
            ex.dir = new.dir;
        }
        if new.explicit_dir {
            ex.explicit_dir = true;
        }
        return;
    }
    nodes.push(new);
}

fn find_dir(doc: &[ParsedItem]) -> String {
    for item in doc {
        if let ParsedItem::Direction(d) = item {
            return d.clone();
        }
    }
    String::new()
}

// ─── Raw text parser ──────────────────────────────────────────────────────────

fn parse_raw(input: &str) -> (Vec<ParsedItem>, String) {
    let mut items: Vec<ParsedItem> = Vec::new();
    let mut stack: Vec<(String, Vec<ParsedItem>)> = Vec::new();
    let mut direction = "TB".to_string();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = strip_comment(lines[i]).trim().to_string();
        i += 1;
        if line.is_empty() {
            continue;
        }
        if line.starts_with("stateDiagram")
            || line.starts_with("accTitle")
            || line.starts_with("accDescr")
        {
            continue;
        }
        if line.starts_with("hide empty") {
            continue;
        }
        if let Some(d) = line.strip_prefix("direction ") {
            let d = d.trim().to_uppercase();
            if stack.is_empty() {
                direction = d.clone();
            }
            push(&mut items, &mut stack, ParsedItem::Direction(d));
            continue;
        }
        if line == "}" {
            if let Some((id, children)) = stack.pop() {
                push(
                    &mut items,
                    &mut stack,
                    ParsedItem::State(ParsedState {
                        id,
                        state_type: "default".into(),
                        description: None,
                        doc: children,
                        note: None,
                        start: None,
                    }),
                );
            }
            continue;
        }
        if line == "--" {
            push(
                &mut items,
                &mut stack,
                ParsedItem::State(ParsedState {
                    id: "__divider__".into(),
                    state_type: "divider".into(),
                    description: None,
                    doc: vec![],
                    note: None,
                    start: None,
                }),
            );
            continue;
        }
        if line.starts_with("note ") && !line.contains("-->") && !line.contains(':') {
            let rest = &line[5..];
            if let Some((pos, target)) = parse_note_pos(rest) {
                let mut txt = Vec::new();
                while i < lines.len() {
                    let nl = strip_comment(lines[i]).trim().to_string();
                    i += 1;
                    if nl == "end note" {
                        break;
                    }
                    txt.push(nl);
                }
                push(
                    &mut items,
                    &mut stack,
                    ParsedItem::State(ParsedState {
                        id: target,
                        state_type: "default".into(),
                        description: None,
                        doc: vec![],
                        note: Some(ParsedNote {
                            position: pos,
                            text: txt.join("\n").trim().to_string(),
                        }),
                        start: None,
                    }),
                );
            }
            continue;
        }
        if line.starts_with("note ") && line.contains(':') && !line.contains("-->") {
            let rest = &line[5..];
            if let Some(c) = rest.find(':') {
                if let Some((pos, target)) = parse_note_pos(&rest[..c]) {
                    let text = rest[c + 1..].trim().to_string();
                    push(
                        &mut items,
                        &mut stack,
                        ParsedItem::State(ParsedState {
                            id: target,
                            state_type: "default".into(),
                            description: None,
                            doc: vec![],
                            note: Some(ParsedNote {
                                position: pos,
                                text,
                            }),
                            start: None,
                        }),
                    );
                }
            }
            continue;
        }
        if line.ends_with('{') && !line.contains("-->") {
            let inner = line.trim_end_matches('{').trim();
            let id = if let Some(r) = inner.strip_prefix("state ") {
                parse_state_id(r.trim())
            } else {
                inner.to_string()
            };
            stack.push((id, Vec::new()));
            continue;
        }
        if line.starts_with("state ") && !line.contains("-->") {
            if let Some(item) = parse_state_decl(&line) {
                push(&mut items, &mut stack, item);
                continue;
            }
        }
        if line.contains("-->") {
            push(&mut items, &mut stack, parse_relation(&line));
            continue;
        }
        if !line.starts_with("%%") {
            let (id, desc) = if let Some(c) = line.find(':') {
                (clean_id(&line[..c]), Some(line[c + 1..].trim().to_string()))
            } else {
                (clean_id(&line), None)
            };
            if !id.is_empty() {
                push(
                    &mut items,
                    &mut stack,
                    ParsedItem::State(ParsedState {
                        id,
                        state_type: "default".into(),
                        description: desc,
                        doc: vec![],
                        note: None,
                        start: None,
                    }),
                );
            }
        }
    }
    while let Some((id, ch)) = stack.pop() {
        items.push(ParsedItem::State(ParsedState {
            id,
            state_type: "default".into(),
            description: None,
            doc: ch,
            note: None,
            start: None,
        }));
    }
    (items, direction)
}

fn push(items: &mut Vec<ParsedItem>, stack: &mut [(String, Vec<ParsedItem>)], item: ParsedItem) {
    if let Some((_, ch)) = stack.last_mut() {
        ch.push(item);
    } else {
        items.push(item);
    }
}
fn strip_comment(line: &str) -> &str {
    if let Some(p) = line.find("%%") {
        &line[..p]
    } else {
        line
    }
}
fn clean_id(s: &str) -> String {
    let s = if let Some(p) = s.find(":::") {
        &s[..p]
    } else {
        s
    };
    s.trim().to_string()
}
fn parse_note_pos(s: &str) -> Option<(String, String)> {
    let s = s.trim();
    if let Some(r) = s.strip_prefix("right of ") {
        Some(("right of".into(), r.trim().to_string()))
    } else {
        s.strip_prefix("left of ")
            .map(|r| ("left of".into(), r.trim().to_string()))
    }
}
fn parse_state_id(s: &str) -> String {
    if let Some(inner) = s.strip_prefix('"') {
        if let Some(eq) = inner.find('"') {
            let after = inner[eq + 1..].trim();
            if let Some(id) = after.strip_prefix("as ") {
                return id.trim().to_string();
            }
        }
    }
    if let Some(p) = s.find("<<") {
        return s[..p].trim().to_string();
    }
    s.trim().to_string()
}
fn parse_state_decl(line: &str) -> Option<ParsedItem> {
    let rest = line.strip_prefix("state ")?.trim();
    let (t, id) = if rest.contains("<<fork>>") || rest.contains("[[fork]]") {
        (
            "fork",
            rest.replace("<<fork>>", "")
                .replace("[[fork]]", "")
                .trim()
                .to_string(),
        )
    } else if rest.contains("<<join>>") || rest.contains("[[join]]") {
        (
            "join",
            rest.replace("<<join>>", "")
                .replace("[[join]]", "")
                .trim()
                .to_string(),
        )
    } else if rest.contains("<<choice>>") || rest.contains("[[choice]]") {
        (
            "choice",
            rest.replace("<<choice>>", "")
                .replace("[[choice]]", "")
                .trim()
                .to_string(),
        )
    } else {
        return None;
    };
    Some(ParsedItem::State(ParsedState {
        id,
        state_type: t.into(),
        description: None,
        doc: vec![],
        note: None,
        start: None,
    }))
}
fn parse_relation(line: &str) -> ParsedItem {
    let parts: Vec<&str> = line.splitn(2, "-->").collect();
    let raw1 = parts[0].trim();
    let rest = if parts.len() > 1 { parts[1].trim() } else { "" };
    let (raw2, desc) = if let Some(c) = rest.find(':') {
        (
            rest[..c].trim().to_string(),
            Some(rest[c + 1..].trim().to_string()),
        )
    } else {
        (rest.to_string(), None)
    };
    let s1 = ParsedState {
        id: clean_id(raw1),
        state_type: "default".into(),
        description: None,
        doc: vec![],
        note: None,
        start: None,
    };
    let s2 = ParsedState {
        id: clean_id(&raw2),
        state_type: "default".into(),
        description: None,
        doc: vec![],
        note: None,
        start: None,
    };
    ParsedItem::Relation {
        state1: s1,
        state2: s2,
        description: desc,
    }
}
