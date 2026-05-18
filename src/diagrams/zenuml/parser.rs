/// Parser for ZenUML sequence diagram syntax.
///
/// ZenUML is a sequence diagram DSL supported by Mermaid via @mermaid-js/mermaid-zenuml.
/// Since the renderer lives in an external Vue.js package rather than a single
/// translatable .ts file, this port implements the documented ZenUML syntax faithfully.
///
/// Reference: https://mermaid.js.org/syntax/zenuml.html
///
/// Grammar overview:
///   zenuml
///   [title <text>]
///   [participant declarations / aliases]
///   [messages and control structures]
///
/// Participant types: @Actor, @Database, @Boundary, @Control, @Entity, @Component
/// Message forms:
///   A->B: text          (async arrow)
///   A.method()          (sync call, implicit return)
///   a = A.method()      (sync call with return value)
///   return value
///   new ClassName
/// Control: if/else, while, for, forEach, loop, opt, par, try/catch/finally

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipantType {
    Default,
    Actor,
    Database,
    Boundary,
    Control,
    Entity,
    Component,
}

#[derive(Debug, Clone)]
pub struct Participant {
    pub id: String,
    pub label: String,
    pub ptype: ParticipantType,
}

#[derive(Debug, Clone)]
pub enum ZenUmlStatement {
    Message(Message),
    Block(Block),
    Return(String),
    Creation(String),
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub label: String,
    pub sync: bool, // true = sync (dotted return), false = async arrow
}

#[derive(Debug, Clone)]
pub struct Block {
    pub kind: BlockKind,
    pub condition: String,
    pub body: Vec<ZenUmlStatement>,
    pub else_body: Option<Vec<ZenUmlStatement>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    If,
    While,
    For,
    ForEach,
    Loop,
    Opt,
    Par,
    Try,
    Catch,
    Finally,
}

pub struct ZenUmlDiagram {
    pub title: Option<String>,
    pub participants: Vec<Participant>,
    pub statements: Vec<ZenUmlStatement>,
}

pub fn parse(input: &str) -> crate::error::ParseResult<ZenUmlDiagram> {
    let mut title: Option<String> = None;
    let mut participants: Vec<Participant> = Vec::new();
    let mut statements: Vec<ZenUmlStatement> = Vec::new();
    let mut header_seen = false;

    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let raw = lines[i];
        let trimmed = raw.trim();

        if trimmed.is_empty() || trimmed.starts_with("%%") {
            i += 1;
            continue;
        }

        if !header_seen {
            let lower = trimmed.to_lowercase();
            if lower == "zenuml" || lower.starts_with("zenuml ") {
                header_seen = true;
                i += 1;
                continue;
            }
            i += 1;
            continue;
        }

        // title
        if trimmed.to_lowercase().starts_with("title ") {
            title = Some(trimmed[6..].trim().to_string());
            i += 1;
            continue;
        }

        // comment
        if let Some(stripped) = trimmed.strip_prefix("//") {
            statements.push(ZenUmlStatement::Comment(stripped.trim().to_string()));
            i += 1;
            continue;
        }

        // Participant annotations: @Actor Bob, @Database db as "DB"
        if trimmed.starts_with('@') {
            if let Some(p) = parse_participant_decl(trimmed) {
                if !participants.iter().any(|x| x.id == p.id) {
                    participants.push(p);
                }
            }
            i += 1;
            continue;
        }

        // Alias: A as Alice
        if let Some(alias_pos) = find_alias(trimmed) {
            let id = trimmed[..alias_pos].trim().to_string();
            let label = trimmed[alias_pos + 3..]
                .trim()
                .trim_matches('"')
                .to_string();
            if !participants.iter().any(|x| x.id == id) {
                participants.push(Participant {
                    id,
                    label,
                    ptype: ParticipantType::Default,
                });
            }
            i += 1;
            continue;
        }

        // return
        if let Some(stripped) = trimmed.strip_prefix("return ") {
            statements.push(ZenUmlStatement::Return(stripped.trim().to_string()));
            i += 1;
            continue;
        }

        // new ClassName
        if let Some(stripped) = trimmed.strip_prefix("new ") {
            statements.push(ZenUmlStatement::Creation(stripped.trim().to_string()));
            i += 1;
            continue;
        }

        // Async message: A->B: text
        if let Some(msg) = parse_async_message(trimmed) {
            register_participant(msg.from.clone(), &mut participants);
            register_participant(msg.to.clone(), &mut participants);
            statements.push(ZenUmlStatement::Message(msg));
            i += 1;
            continue;
        }

        // Sync message: [a = ]A.method() [{...}]
        if let Some((msg, has_block)) = parse_sync_message(trimmed) {
            register_participant(msg.from.clone(), &mut participants);
            register_participant(msg.to.clone(), &mut participants);
            if has_block {
                // Collect nested block
                let (nested, consumed) = collect_block_body(&lines, i + 1);
                let outer_msg = msg.clone();
                statements.push(ZenUmlStatement::Message(outer_msg));
                let mut nested_stmts = parse_block_statements(&nested, &mut participants);
                statements.append(&mut nested_stmts);
                i += 1 + consumed;
            } else {
                statements.push(ZenUmlStatement::Message(msg));
                i += 1;
            }
            continue;
        }

        // Control structures: if / while / for / forEach / loop / opt / par / try
        if let Some(block_kind) = detect_block_kind(trimmed) {
            let condition = extract_condition(trimmed, block_kind);
            let (body_lines, consumed) = collect_block_body(&lines, i + 1);
            let body = parse_block_statements(&body_lines, &mut participants);
            statements.push(ZenUmlStatement::Block(Block {
                kind: block_kind,
                condition,
                body,
                else_body: None,
            }));
            i += 1 + consumed;
            continue;
        }

        // Bare participant declaration (just a name on its own line)
        if is_bare_participant(trimmed) {
            register_participant(trimmed.to_string(), &mut participants);
        }

        i += 1;
    }

    crate::error::ParseResult::ok(ZenUmlDiagram {
        title,
        participants,
        statements,
    })
}

fn parse_participant_decl(s: &str) -> Option<Participant> {
    let s = &s[1..]; // strip '@'
    let space = s.find(' ')?;
    let type_str = &s[..space];
    let rest = s[space..].trim();

    let ptype = match type_str.to_lowercase().as_str() {
        "actor" => ParticipantType::Actor,
        "database" => ParticipantType::Database,
        "boundary" => ParticipantType::Boundary,
        "control" => ParticipantType::Control,
        "entity" => ParticipantType::Entity,
        "component" => ParticipantType::Component,
        _ => return None,
    };

    // rest = "Name" or "Name as Label"
    let (id, label) = if let Some(as_pos) = find_alias(rest) {
        let id = rest[..as_pos].trim().to_string();
        let label = rest[as_pos + 3..].trim().trim_matches('"').to_string();
        (id, label)
    } else {
        (rest.to_string(), rest.trim_matches('"').to_string())
    };

    Some(Participant { id, label, ptype })
}

fn find_alias(s: &str) -> Option<usize> {
    // " as " surrounded by spaces
    s.find(" as ")
}

fn parse_async_message(s: &str) -> Option<Message> {
    let arrow = s.find("->")?;
    let from = s[..arrow].trim().to_string();
    let rest = &s[arrow + 2..];
    let colon = rest.find(':')?;
    let to = rest[..colon].trim().to_string();
    let label = rest[colon + 1..].trim().to_string();

    if from.is_empty() || to.is_empty() {
        return None;
    }
    Some(Message {
        from,
        to,
        label,
        sync: false,
    })
}

fn parse_sync_message(s: &str) -> Option<(Message, bool)> {
    // [return_var = ]Receiver.method(args) [{]
    let (ret_var, rest) = if let Some(eq_pos) = s.find('=') {
        let before = s[..eq_pos].trim();
        // Make sure no spaces in before (it's a variable name)
        if before.contains(' ') || before.contains('-') || before.contains('>') {
            (None, s)
        } else {
            (Some(before.to_string()), s[eq_pos + 1..].trim())
        }
    } else {
        (None, s)
    };

    // Now rest should be "Receiver.method(args)" optionally followed by " {" or "{"
    let dot = rest.find('.')?;
    let receiver = rest[..dot].trim().to_string();
    if receiver.is_empty() || receiver.contains(' ') || receiver.contains('>') {
        return None;
    }

    let after_dot = &rest[dot + 1..];
    let open_paren = after_dot.find('(')?;
    let method = after_dot[..open_paren].to_string();
    // Find matching close paren
    let close_paren = after_dot[open_paren..].find(')')? + open_paren;
    let _args = &after_dot[open_paren + 1..close_paren];

    let after_call = after_dot[close_paren + 1..].trim();
    let has_block = after_call.starts_with('{') || after_call == "{";

    let label = if _args.is_empty() {
        format!("{method}()")
    } else {
        format!("{method}({_args})")
    };

    // The receiver IS the "to" participant, caller is unknown without context.
    // In ZenUML, the implicit caller context is tracked.
    // For simplicity: if we see "A.method()" it means: current_caller -> A: method()
    // We'll use a placeholder "self" for from unless context says otherwise.
    let _ = ret_var; // assignment target not used by renderer
    Some((
        Message {
            from: "self".to_string(),
            to: receiver,
            label,
            sync: true,
        },
        has_block,
    ))
}

fn detect_block_kind(s: &str) -> Option<BlockKind> {
    let lower = s.to_lowercase();
    let w = lower.split('(').next().unwrap_or("").trim();
    match w {
        "if" => Some(BlockKind::If),
        "while" => Some(BlockKind::While),
        "for" => Some(BlockKind::For),
        "foreach" => Some(BlockKind::ForEach),
        "loop" => Some(BlockKind::Loop),
        "opt" => Some(BlockKind::Opt),
        "par" => Some(BlockKind::Par),
        "try" => Some(BlockKind::Try),
        "catch" => Some(BlockKind::Catch),
        "finally" => Some(BlockKind::Finally),
        _ => None,
    }
}

fn extract_condition(s: &str, _kind: BlockKind) -> String {
    if let Some(start) = s.find('(') {
        if let Some(end) = s.rfind(')') {
            return s[start + 1..end].trim().to_string();
        }
    }
    // For "loop", "opt", "par" — no condition
    s.to_string()
}

fn collect_block_body<'a>(lines: &[&'a str], start: usize) -> (Vec<&'a str>, usize) {
    // Find opening '{' and matching '}'
    let mut depth = 0i32;
    let mut body: Vec<&str> = Vec::new();
    let mut found_open = false;
    let mut consumed = 0;

    for line in lines.iter().skip(start) {
        let t = line.trim();
        consumed += 1;

        for ch in t.chars() {
            if ch == '{' {
                depth += 1;
                found_open = true;
            } else if ch == '}' {
                depth -= 1;
            }
        }

        if !found_open {
            // Lines before the opening brace (shouldn't happen but handle gracefully)
            continue;
        }

        if depth <= 0 {
            // This line closes the block
            // Add inner content (stripping outer braces)
            let inner = t.trim_matches(|c: char| c == '{' || c == '}').trim();
            if !inner.is_empty() {
                body.push(inner);
            }
            break;
        } else {
            // Inner content
            let stripped = if let Some(s) = t.strip_prefix('{') {
                s
            } else {
                t
            };
            body.push(stripped.trim());
        }
    }

    (body, consumed)
}

fn parse_block_statements(
    lines: &[&str],
    participants: &mut Vec<Participant>,
) -> Vec<ZenUmlStatement> {
    let mut stmts = Vec::new();
    for &line in lines {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(stripped) = t.strip_prefix("//") {
            stmts.push(ZenUmlStatement::Comment(stripped.trim().to_string()));
            continue;
        }
        if let Some(stripped) = t.strip_prefix("return ") {
            stmts.push(ZenUmlStatement::Return(stripped.trim().to_string()));
            continue;
        }
        if let Some(msg) = parse_async_message(t) {
            register_participant(msg.from.clone(), participants);
            register_participant(msg.to.clone(), participants);
            stmts.push(ZenUmlStatement::Message(msg));
            continue;
        }
        if let Some((msg, _)) = parse_sync_message(t) {
            register_participant(msg.to.clone(), participants);
            stmts.push(ZenUmlStatement::Message(msg));
        }
    }
    stmts
}

fn register_participant(id: String, participants: &mut Vec<Participant>) {
    if id == "self" {
        return;
    }
    if !participants.iter().any(|p| p.id == id) {
        participants.push(Participant {
            label: id.clone(),
            id,
            ptype: ParticipantType::Default,
        });
    }
}

fn is_bare_participant(s: &str) -> bool {
    // A bare participant is a single identifier (no spaces, no special chars)
    !s.contains(' ')
        && !s.contains('.')
        && !s.contains('-')
        && !s.contains('@')
        && !s.contains('(')
        && s.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parse() {
        let input = "zenuml\n    title Hello\n    Alice->Bob: Hello\n    Bob->Alice: Hi\n";
        let d = parse(input).diagram;
        assert_eq!(d.title.as_deref(), Some("Hello"));
        assert_eq!(d.participants.len(), 2);
        assert_eq!(d.statements.len(), 2);
    }

    #[test]
    fn actor_annotation() {
        let input = "zenuml\n    @Actor Alice\n    Alice->Bob: Hello\n";
        let d = parse(input).diagram;
        assert_eq!(d.participants[0].ptype, ParticipantType::Actor);
    }

    // ── new tests ─────────────────────────────────────────────────────────────

    #[test]
    fn database_annotation() {
        let input = "zenuml\n  @Database db\n  Alice->db: query\n";
        let d = parse(input).diagram;
        let db = d
            .participants
            .iter()
            .find(|p| p.id == "db")
            .expect("db participant");
        assert_eq!(db.ptype, ParticipantType::Database);
    }

    #[test]
    fn boundary_annotation() {
        let input = "zenuml\n  @Boundary web\n";
        let d = parse(input).diagram;
        assert_eq!(d.participants[0].ptype, ParticipantType::Boundary);
    }

    #[test]
    fn control_annotation() {
        let input = "zenuml\n  @Control ctrl\n";
        let d = parse(input).diagram;
        assert_eq!(d.participants[0].ptype, ParticipantType::Control);
    }

    #[test]
    fn entity_annotation() {
        let input = "zenuml\n  @Entity ent\n";
        let d = parse(input).diagram;
        assert_eq!(d.participants[0].ptype, ParticipantType::Entity);
    }

    #[test]
    fn component_annotation() {
        let input = "zenuml\n  @Component comp\n";
        let d = parse(input).diagram;
        assert_eq!(d.participants[0].ptype, ParticipantType::Component);
    }

    #[test]
    fn participant_alias() {
        let input = "zenuml\n  A as Alice\n  A->B: hello\n";
        let d = parse(input).diagram;
        let aliased = d
            .participants
            .iter()
            .find(|p| p.id == "A")
            .expect("A participant");
        assert_eq!(aliased.label, "Alice");
    }

    #[test]
    fn sync_method_call() {
        let input = "zenuml\n  A->B: hi\n  B.process()\n";
        let d = parse(input).diagram;
        // Both async message and sync method call should produce statements
        assert_eq!(d.statements.len(), 2);
        if let ZenUmlStatement::Message(msg) = &d.statements[1] {
            assert!(msg.sync);
            assert_eq!(msg.to, "B");
            assert_eq!(msg.label, "process()");
        } else {
            panic!("expected Message statement");
        }
    }

    #[test]
    fn sync_method_with_args() {
        let input = "zenuml\n  Service.call(arg1, arg2)\n";
        let d = parse(input).diagram;
        if let ZenUmlStatement::Message(msg) = &d.statements[0] {
            assert_eq!(msg.label, "call(arg1, arg2)");
        } else {
            panic!("expected Message statement");
        }
    }

    #[test]
    fn return_statement() {
        let input = "zenuml\n  Alice->Bob: request\n  return result\n";
        let d = parse(input).diagram;
        assert_eq!(d.statements.len(), 2);
        if let ZenUmlStatement::Return(val) = &d.statements[1] {
            assert_eq!(val, "result");
        } else {
            panic!("expected Return statement");
        }
    }

    #[test]
    fn creation_statement() {
        let input = "zenuml\n  new MyObject\n";
        let d = parse(input).diagram;
        if let ZenUmlStatement::Creation(name) = &d.statements[0] {
            assert_eq!(name, "MyObject");
        } else {
            panic!("expected Creation statement");
        }
    }

    #[test]
    fn comment_statement() {
        let input = "zenuml\n  // This is a comment\n  Alice->Bob: hello\n";
        let d = parse(input).diagram;
        if let ZenUmlStatement::Comment(text) = &d.statements[0] {
            assert_eq!(text, "This is a comment");
        } else {
            panic!("expected Comment statement");
        }
    }

    #[test]
    fn if_block() {
        let input =
            "zenuml\n  Alice->Bob: request\n  if(condition) {\n    Bob->Alice: response\n  }\n";
        let d = parse(input).diagram;
        let block_stmt = d
            .statements
            .iter()
            .find(|s| matches!(s, ZenUmlStatement::Block(_)));
        assert!(block_stmt.is_some(), "should have a Block statement");
        if let Some(ZenUmlStatement::Block(b)) = block_stmt {
            assert_eq!(b.kind, BlockKind::If);
            assert_eq!(b.condition, "condition");
        }
    }

    #[test]
    fn while_block() {
        let input = "zenuml\n  while(retry) {\n    A->B: try\n  }\n";
        let d = parse(input).diagram;
        let block_stmt = d
            .statements
            .iter()
            .find(|s| matches!(s, ZenUmlStatement::Block(_)));
        if let Some(ZenUmlStatement::Block(b)) = block_stmt {
            assert_eq!(b.kind, BlockKind::While);
        } else {
            panic!("expected Block statement");
        }
    }

    #[test]
    fn loop_block() {
        let input = "zenuml\n  loop() {\n    A->B: ping\n  }\n";
        let d = parse(input).diagram;
        let block_stmt = d
            .statements
            .iter()
            .find(|s| matches!(s, ZenUmlStatement::Block(_)));
        if let Some(ZenUmlStatement::Block(b)) = block_stmt {
            assert_eq!(b.kind, BlockKind::Loop);
        } else {
            panic!("expected Block statement");
        }
    }

    #[test]
    fn opt_block() {
        let input = "zenuml\n  opt() {\n    A->B: optional\n  }\n";
        let d = parse(input).diagram;
        let block_stmt = d
            .statements
            .iter()
            .find(|s| matches!(s, ZenUmlStatement::Block(_)));
        if let Some(ZenUmlStatement::Block(b)) = block_stmt {
            assert_eq!(b.kind, BlockKind::Opt);
        } else {
            panic!("expected Block statement");
        }
    }

    #[test]
    fn par_block() {
        let input = "zenuml\n  par() {\n    A->B: parallel\n  }\n";
        let d = parse(input).diagram;
        let block_stmt = d
            .statements
            .iter()
            .find(|s| matches!(s, ZenUmlStatement::Block(_)));
        if let Some(ZenUmlStatement::Block(b)) = block_stmt {
            assert_eq!(b.kind, BlockKind::Par);
        } else {
            panic!("expected Block statement");
        }
    }

    #[test]
    fn try_block() {
        let input = "zenuml\n  try() {\n    A->B: risky\n  }\n";
        let d = parse(input).diagram;
        let block_stmt = d
            .statements
            .iter()
            .find(|s| matches!(s, ZenUmlStatement::Block(_)));
        if let Some(ZenUmlStatement::Block(b)) = block_stmt {
            assert_eq!(b.kind, BlockKind::Try);
        } else {
            panic!("expected Block statement");
        }
    }

    #[test]
    fn for_block() {
        let input = "zenuml\n  for(i in items) {\n    A->B: item\n  }\n";
        let d = parse(input).diagram;
        let block_stmt = d
            .statements
            .iter()
            .find(|s| matches!(s, ZenUmlStatement::Block(_)));
        if let Some(ZenUmlStatement::Block(b)) = block_stmt {
            assert_eq!(b.kind, BlockKind::For);
        } else {
            panic!("expected Block statement");
        }
    }

    #[test]
    fn foreach_block() {
        let input = "zenuml\n  forEach(item) {\n    A->B: process\n  }\n";
        let d = parse(input).diagram;
        let block_stmt = d
            .statements
            .iter()
            .find(|s| matches!(s, ZenUmlStatement::Block(_)));
        if let Some(ZenUmlStatement::Block(b)) = block_stmt {
            assert_eq!(b.kind, BlockKind::ForEach);
        } else {
            panic!("expected Block statement");
        }
    }

    #[test]
    fn empty_lines_and_comments_skipped() {
        let input = "zenuml\n\n  %% mermaid comment\n  Alice->Bob: hi\n";
        let d = parse(input).diagram;
        assert_eq!(d.statements.len(), 1);
    }

    #[test]
    fn bare_participant_registered() {
        let input = "zenuml\n  MyService\n";
        let d = parse(input).diagram;
        assert!(d.participants.iter().any(|p| p.id == "MyService"));
    }

    #[test]
    fn duplicate_participants_not_doubled() {
        let input = "zenuml\n  Alice->Bob: hi\n  Alice->Bob: bye\n";
        let d = parse(input).diagram;
        let alice_count = d.participants.iter().filter(|p| p.id == "Alice").count();
        assert_eq!(alice_count, 1);
    }

    #[test]
    fn actor_with_alias() {
        let input = "zenuml\n  @Actor Bob as \"Robert\"\n";
        let d = parse(input).diagram;
        let p = d.participants.iter().find(|p| p.id == "Bob").expect("Bob");
        assert_eq!(p.ptype, ParticipantType::Actor);
        assert_eq!(p.label, "Robert");
    }

    #[test]
    fn assigned_sync_call() {
        let input = "zenuml\n  result = Service.fetch()\n";
        let d = parse(input).diagram;
        assert_eq!(d.statements.len(), 1);
        if let ZenUmlStatement::Message(msg) = &d.statements[0] {
            assert!(msg.sync);
            assert_eq!(msg.to, "Service");
        } else {
            panic!("expected Message statement");
        }
    }

    #[test]
    fn no_title_is_none() {
        let input = "zenuml\n  Alice->Bob: hello\n";
        let d = parse(input).diagram;
        assert!(d.title.is_none());
    }

    #[test]
    fn parse_result_is_ok() {
        let input = "zenuml\n  Alice->Bob: hello\n";
        let result = parse(input);
        assert!(result.is_ok());
    }
}
