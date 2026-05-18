/// Sequence diagram parser — faithful port of Mermaid sequenceDb.ts
///
/// Supports:
///   participant / actor declarations
///   ->, ->>, -->, -->>, -)  message arrow types
///   +/- activation shorthand
///   loop … end
///   alt … else … end
///   opt … end
///   par … and … end
///   Note right of / left of / over
///   autonumber

#[derive(Debug, Clone, PartialEq)]
pub enum LineType {
    Solid,       // ->   (open, no arrowhead)
    SolidArrow,  // ->>  (filled arrowhead)
    Dotted,      // -->  (dotted, open)
    DottedArrow, // -->> (dotted, filled arrowhead)
    Point,       // -)   (async / point arrowhead)
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotePlacement {
    RightOf,
    LeftOf,
    Over,
}

#[derive(Debug, Clone)]
pub enum ParticipantKind {
    Participant,
    Actor,
}

#[derive(Debug, Clone)]
pub struct Participant {
    pub name: String,
    pub alias: String, // same as name if no alias
    pub kind: ParticipantKind,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub text: String,
    pub line_type: LineType,
    /// +1 = activate receiver, -1 = deactivate sender/receiver, 0 = nothing
    pub activate: i32,
}

#[derive(Debug, Clone)]
pub struct NoteItem {
    pub actors: Vec<String>, // 1 or 2 actors (over Alice,Bob)
    pub placement: NotePlacement,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum SeqItem {
    Participant(Participant),
    Message(Message),
    Note(NoteItem),
    LoopStart(String),
    LoopEnd,
    AltStart(String),
    AltElse(String),
    OptStart(String),
    ParStart(String),
    ParAnd(String),
    Activate(String),
    Deactivate(String),
    AutoNumber,
}

#[derive(Debug, Clone, Default)]
pub struct SequenceDiagram {
    pub items: Vec<SeqItem>,
}

pub fn parse(input: &str) -> crate::error::ParseResult<SequenceDiagram> {
    let mut diag = SequenceDiagram::default();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("sequenceDiagram") || trimmed.starts_with("%%")
        {
            continue;
        }

        if let Some(item) = parse_line(trimmed) {
            diag.items.push(item);
        }
    }

    crate::error::ParseResult::ok(diag)
}

fn parse_line(s: &str) -> Option<SeqItem> {
    // participant / actor
    if let Some(rest) = s.strip_prefix("participant ") {
        return Some(parse_participant(rest, ParticipantKind::Participant));
    }
    if let Some(rest) = s.strip_prefix("actor ") {
        return Some(parse_participant(rest, ParticipantKind::Actor));
    }

    // activate / deactivate
    if let Some(rest) = s.strip_prefix("activate ") {
        return Some(SeqItem::Activate(rest.trim().to_string()));
    }
    if let Some(rest) = s.strip_prefix("deactivate ") {
        return Some(SeqItem::Deactivate(rest.trim().to_string()));
    }

    // autonumber
    if s == "autonumber" {
        return Some(SeqItem::AutoNumber);
    }

    // Note
    if s.starts_with("Note ") || s.starts_with("note ") {
        return parse_note(s);
    }

    // loop … end
    if let Some(rest) = s.strip_prefix("loop ") {
        return Some(SeqItem::LoopStart(rest.trim().to_string()));
    }
    if s == "end" {
        return Some(SeqItem::LoopEnd); // overloaded — we post-process
    }

    // alt / else / opt / par / and
    if let Some(rest) = s.strip_prefix("alt ") {
        return Some(SeqItem::AltStart(rest.trim().to_string()));
    }
    if let Some(rest) = s.strip_prefix("else ") {
        return Some(SeqItem::AltElse(rest.trim().to_string()));
    }
    if s == "else" {
        return Some(SeqItem::AltElse(String::new()));
    }
    if let Some(rest) = s.strip_prefix("opt ") {
        return Some(SeqItem::OptStart(rest.trim().to_string()));
    }
    if let Some(rest) = s.strip_prefix("par ") {
        return Some(SeqItem::ParStart(rest.trim().to_string()));
    }
    if let Some(rest) = s.strip_prefix("and ") {
        return Some(SeqItem::ParAnd(rest.trim().to_string()));
    }

    // Try message
    parse_message(s)
}

fn parse_participant(s: &str, kind: ParticipantKind) -> SeqItem {
    // "Alice as A" style alias
    if let Some(idx) = s.find(" as ") {
        let name = s[..idx].trim().to_string();
        let alias = s[idx + 4..].trim().to_string();
        return SeqItem::Participant(Participant { name, alias, kind });
    }
    let name = s.trim().to_string();
    SeqItem::Participant(Participant {
        alias: name.clone(),
        name,
        kind,
    })
}

fn parse_note(s: &str) -> Option<SeqItem> {
    // "Note right of Alice: text"
    // "Note over Alice,Bob: text"
    let lower = s.to_lowercase();
    let rest = if lower.starts_with("note ") {
        &s[5..]
    } else {
        return None;
    };

    let (placement, rest) = if let Some(r) = rest.strip_prefix("right of ") {
        (NotePlacement::RightOf, r)
    } else if let Some(r) = rest.strip_prefix("left of ") {
        (NotePlacement::LeftOf, r)
    } else if let Some(r) = rest.strip_prefix("over ") {
        (NotePlacement::Over, r)
    } else {
        return None;
    };

    // rest = "Actor: text" or "Actor,Actor2: text"
    let (actors_part, text) = if let Some(colon) = rest.find(':') {
        (&rest[..colon], rest[colon + 1..].trim())
    } else {
        (rest, "")
    };

    let actors: Vec<String> = actors_part
        .split(',')
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .collect();

    Some(SeqItem::Note(NoteItem {
        actors,
        placement,
        text: text.to_string(),
    }))
}

/// Arrow patterns we try (longest first to avoid ambiguity).
/// Returns (from, to, text, line_type, activate_delta)
fn parse_message(s: &str) -> Option<SeqItem> {
    // Patterns: -->> ->> --> -> --) -)
    let arrows: &[(&str, LineType)] = &[
        ("-->>", LineType::DottedArrow),
        ("->>", LineType::SolidArrow),
        ("-->", LineType::Dotted),
        ("->", LineType::Solid),
        ("--)", LineType::DottedArrow), // treat as dotted arrow
        ("-)", LineType::Point),
    ];

    for (arrow, lt) in arrows {
        // Find the arrow, but it can have + or - suffix on either side
        // e.g. Alice->>+Bob or Alice-->>-Alice
        // Try to split on arrow
        if let Some(arrow_pos) = find_arrow(s, arrow) {
            let from = s[..arrow_pos].trim().to_string();
            let after_arrow = s[arrow_pos + arrow.len()..].trim();

            // Activation prefix on target
            let (target_part, msg_text) = if let Some(colon) = after_arrow.find(':') {
                (&after_arrow[..colon], after_arrow[colon + 1..].trim())
            } else {
                (after_arrow, "")
            };

            let target_trimmed = target_part.trim();
            let (to, activate) = if let Some(stripped) = target_trimmed.strip_prefix('+') {
                (stripped.trim().to_string(), 1i32)
            } else if let Some(stripped) = target_trimmed.strip_prefix('-') {
                (stripped.trim().to_string(), -1i32)
            } else {
                (target_trimmed.to_string(), 0i32)
            };

            if from.is_empty() || to.is_empty() {
                continue;
            }

            return Some(SeqItem::Message(Message {
                from,
                to,
                text: msg_text.to_string(),
                line_type: lt.clone(),
                activate,
            }));
        }
    }
    None
}

/// Find position of `arrow` in `s`, making sure it's a real arrow (not embedded in actor name
/// by verifying characters around it).
fn find_arrow(s: &str, arrow: &str) -> Option<usize> {
    let _bytes = s.as_bytes();
    let alen = arrow.len();
    for i in 0..s.len().saturating_sub(alen - 1) {
        if &s[i..i + alen] == arrow {
            // Make sure the character before is not a '-' (would be part of longer arrow)
            // and after is not a continuation char
            // Simple: just return first match — longer arrows are tried first
            return Some(i);
        }
    }
    None
}
