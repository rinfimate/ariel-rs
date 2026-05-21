// Faithful Rust port of the Mermaid EventModeling Langium grammar.
//
// Grammar (distilled from the EventModelingGrammar JSON embedded in the bundle):
//
//   EventModel:
//     "eventmodeling"
//     ( accDescr | accTitle | title | frame | dataEntity | ... )*
//
//   EmTimeFrame ("tf" | "timeframe"):
//     ("tf" | "timeframe") name=EM_FID modelEntityType entityIdentifier
//     ("->" sourceFrames+=EM_FID)*
//     ("[[" dataRef "]]")?
//     inlineData?
//
//   EmResetFrame ("rf" | "resetframe"):
//     ("rf" | "resetframe") name=EM_FID modelEntityType entityIdentifier
//     ("->>" sourceFrames+=EM_FID)*
//     ("[[" dataRef "]]")?
//     inlineData?
//
//   EmModelEntityType: "rmo"|"readmodel"|"ui"|"cmd"|"command"|"evt"|"event"|"pcr"|"processor"
//   EM_FID: /\d{1,3}/         — 1-3 digit frame identifier
//   QualifiedName: EM_ID ("." EM_ID)*
//   EM_ID: /[_a-zA-Z][\w_]*/
//
//   EmDataEntity: "data" name=EM_ID dataBlock
//   inlineData: ("`" dataType "`")? EM_DATA_INLINE
//   EM_DATA_INLINE: /\{(.*)\}|"(.*)"|'(.*)'/
//
// Source frame relation operator: "->" or "->>" (both accepted; grammar uses "->>").

// ─── Types ────────────────────────────────────────────────────────────────────

/// Entity type of an event modeling frame (`modelEntityType`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityType {
    /// "ui" — UI screen / automation step.
    Ui,
    /// "pcr" | "processor" — processor / policy.
    Processor,
    /// "rmo" | "readmodel" — read model / view.
    ReadModel,
    /// "cmd" | "command" — command.
    Command,
    /// "evt" | "event" — domain event.
    Event,
}

impl EntityType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "ui" => Some(EntityType::Ui),
            "pcr" | "processor" => Some(EntityType::Processor),
            "rmo" | "readmodel" => Some(EntityType::ReadModel),
            "cmd" | "command" => Some(EntityType::Command),
            "evt" | "event" => Some(EntityType::Event),
            _ => None,
        }
    }
}

/// A single event modeling frame (`EmTimeFrame` or `EmResetFrame`).
#[derive(Debug, Clone)]
pub struct EmFrame {
    /// EM_FID — 1-3 digit string used as the cross-reference key.
    pub name: String,
    /// Type of this frame.
    pub entity_type: EntityType,
    /// Qualified identifier: `Name` or `Namespace.Name`.
    pub entity_id: String,
    /// Source frame names this frame receives from (`->> frameName`).
    pub source_refs: Vec<String>,
    /// True if this is a reset frame (`rf` / `resetframe`).
    pub is_reset: bool,
    /// Optional inline data value `{ ... }`.
    #[allow(dead_code)]
    pub inline_data: Option<String>,
    /// Optional reference to an `EmDataEntity` name.
    #[allow(dead_code)]
    pub data_ref: Option<String>,
}

/// A named data block.
#[derive(Debug, Clone)]
pub struct EmDataEntity {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub data: String,
}

/// Parsed event modeling diagram.
#[derive(Debug)]
pub struct EventModelDiagram {
    pub title: Option<String>,
    pub frames: Vec<EmFrame>,
    #[allow(dead_code)]
    pub data_entities: Vec<EmDataEntity>,
}

// ─── Parser ───────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<EventModelDiagram> {
    let mut title: Option<String> = None;
    let mut frames: Vec<EmFrame> = Vec::new();
    let mut data_entities: Vec<EmDataEntity> = Vec::new();

    // Multi-line data block accumulation.
    let mut data_block_name: Option<String> = None;
    let mut data_block_lines: Vec<String> = Vec::new();
    let mut in_data_block = false;

    let mut in_header = true;

    for raw in input.lines() {
        // Strip single-line comments (%%)
        let line = if let Some(p) = raw.find("%%") {
            &raw[..p]
        } else {
            raw
        };
        let trimmed = line.trim();

        // ── Header ────────────────────────────────────────────────────────────
        if in_header {
            if trimmed.starts_with("eventmodeling") {
                in_header = false;
            }
            continue;
        }

        // ── Data block accumulation ───────────────────────────────────────────
        if in_data_block {
            if trimmed == "}" {
                if let Some(name) = data_block_name.take() {
                    data_entities.push(EmDataEntity {
                        name,
                        data: data_block_lines.join("\n"),
                    });
                }
                data_block_lines.clear();
                in_data_block = false;
            } else {
                data_block_lines.push(trimmed.to_string());
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        // ── title ─────────────────────────────────────────────────────────────
        // EM_TITLE terminal: /[\t ]*title(?:[\t ][^\n\r]*?(?=%%)|[\t ][^\n\r]*|)/
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

        // ── accTitle / accDescr — skip ─────────────────────────────────────────
        if trimmed.starts_with("accTitle") || trimmed.starts_with("accDescr") {
            continue;
        }

        // ── data <name> { ... } ───────────────────────────────────────────────
        if let Some(rest) = trimmed
            .strip_prefix("data ")
            .or_else(|| trimmed.strip_prefix("data\t"))
        {
            let name_tok = rest
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            // Inline single-line block: data Foo { ... }
            if let Some(brace_pos) = rest.find('{') {
                let after = rest[brace_pos + 1..].trim();
                let content = if let Some(end) = after.rfind('}') {
                    after[..end].trim().to_string()
                } else {
                    after.to_string()
                };
                if !name_tok.is_empty() {
                    data_entities.push(EmDataEntity {
                        name: name_tok,
                        data: content,
                    });
                }
            } else if !name_tok.is_empty() {
                // Multi-line block — next lines until "}" are the content
                data_block_name = Some(name_tok);
                data_block_lines.clear();
                in_data_block = true;
            }
            continue;
        }

        // ── tf / timeframe / rf / resetframe ──────────────────────────────────
        let (is_reset, rest_after_kw) = if let Some(r) = trimmed
            .strip_prefix("rf ")
            .or_else(|| trimmed.strip_prefix("rf\t"))
            .or_else(|| trimmed.strip_prefix("resetframe "))
            .or_else(|| trimmed.strip_prefix("resetframe\t"))
        {
            (true, r.trim())
        } else if let Some(r) = trimmed
            .strip_prefix("tf ")
            .or_else(|| trimmed.strip_prefix("tf\t"))
            .or_else(|| trimmed.strip_prefix("timeframe "))
            .or_else(|| trimmed.strip_prefix("timeframe\t"))
        {
            (false, r.trim())
        } else {
            continue;
        };

        if let Some(frame) = parse_frame_line(rest_after_kw, is_reset) {
            frames.push(frame);
        }
    }

    crate::error::ParseResult::ok(EventModelDiagram {
        title,
        frames,
        data_entities,
    })
}

// ─── Frame line parser ────────────────────────────────────────────────────────

/// Parse the part of a frame line after `tf`/`rf`:
///   `name modelEntityType entityIdentifier (->> sourceRef)* ([[dataRef]])? inlineData?`
fn parse_frame_line(s: &str, is_reset: bool) -> Option<EmFrame> {
    let tokens = tokenize_frame(s);
    let mut pos = 0;

    // name = EM_FID (/\d{1,3}/)
    let name = tokens.get(pos)?.clone();
    if !name.chars().all(|c| c.is_ascii_digit()) || name.is_empty() || name.len() > 3 {
        return None;
    }
    pos += 1;

    // modelEntityType
    let type_tok = tokens.get(pos)?;
    let entity_type = EntityType::from_str(type_tok)?;
    pos += 1;

    // entityIdentifier = QualifiedName (EM_ID ("." EM_ID)*)
    let entity_id = tokens.get(pos)?.clone();
    if entity_id.is_empty() {
        return None;
    }
    pos += 1;

    // source refs: (->> | ->) frameId ...
    let mut source_refs: Vec<String> = Vec::new();
    while pos < tokens.len() {
        let tok = &tokens[pos];
        if tok == "->>" || tok == "->" {
            pos += 1;
            if let Some(ref_id) = tokens.get(pos) {
                if ref_id.chars().all(|c| c.is_ascii_digit()) && !ref_id.is_empty() {
                    source_refs.push(ref_id.clone());
                    pos += 1;
                }
            }
        } else {
            break;
        }
    }

    // optional [[dataRef]]
    let mut data_ref: Option<String> = None;
    if pos < tokens.len() && tokens[pos] == "[[" {
        pos += 1;
        if let Some(dr) = tokens.get(pos) {
            data_ref = Some(dr.clone());
            pos += 1;
        }
        // consume "]]"
        if tokens.get(pos).map(|t| t == "]]").unwrap_or(false) {
            pos += 1;
        }
    }

    // optional inline data: { ... } or "`type`" { ... }
    let inline_data = if pos < tokens.len() {
        // Reconstruct remaining as inline data string
        let rest: String = tokens[pos..].join(" ");
        extract_inline_data(&rest)
    } else {
        None
    };

    // Suppress unused warning for tokens variable (fully consumed above)
    let _ = tokens.len();

    Some(EmFrame {
        name,
        entity_type,
        entity_id,
        source_refs,
        is_reset,
        inline_data,
        data_ref,
    })
}

/// Simple tokenizer that splits on whitespace but keeps `->>`  / `->` / `[[` / `]]` as single tokens.
fn tokenize_frame(s: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut chars = s.chars().peekable();
    loop {
        // skip whitespace
        while chars.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
            chars.next();
        }
        let Some(&ch) = chars.peek() else { break };

        if ch == '-' {
            chars.next();
            if chars.peek() == Some(&'>') {
                chars.next();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push("->>".to_string());
                } else {
                    tokens.push("->".to_string());
                }
            } else {
                tokens.push("-".to_string());
            }
        } else if ch == '[' {
            chars.next();
            if chars.peek() == Some(&'[') {
                chars.next();
                tokens.push("[[".to_string());
            } else {
                tokens.push("[".to_string());
            }
        } else if ch == ']' {
            chars.next();
            if chars.peek() == Some(&']') {
                chars.next();
                tokens.push("]]".to_string());
            } else {
                tokens.push("]".to_string());
            }
        } else if ch == '{' || ch == '`' {
            // Start of inline data — collect the rest of the string as one token
            let rest: String = chars.collect();
            tokens.push(rest);
            break;
        } else {
            // Regular token: read until whitespace
            let mut tok = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '[' || c == ']' || c == '-' {
                    break;
                }
                tok.push(c);
                chars.next();
            }
            if !tok.is_empty() {
                tokens.push(tok);
            }
        }
    }
    tokens
}

/// Extract the content from `{ ... }` or `"..."` or `'...'` inline data literals.
/// Mirrors the `EM_DATA_INLINE` terminal: `/\{(.*)\}|"(.*)"|'(.*)'/`
fn extract_inline_data(s: &str) -> Option<String> {
    let s = s.trim();
    if s.starts_with('{') {
        if let Some(end) = s.rfind('}') {
            return Some(s[1..end].trim().to_string());
        }
        return Some(s[1..].trim().to_string());
    }
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        return Some(s[1..s.len() - 1].to_string());
    }
    // backtick-type prefix: "`json`{ ... }"
    if s.starts_with('`') {
        if let Some(end_bt) = s[1..].find('`') {
            let after = s[end_bt + 2..].trim();
            return extract_inline_data(after);
        }
    }
    None
}
