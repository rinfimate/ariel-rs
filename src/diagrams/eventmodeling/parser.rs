// Faithful Rust port of mermaid/src/diagrams/eventmodeling/parser.ts + db.ts
//
// Event Modeling diagram:
//   eventmodeling
//   [title <text>]
//
//   Swimlane definitions establish rows (UI Automation, Command/ReadModel, Events, ...)
//   Boxes are placed within swimlanes with visual types.
//   Relations connect boxes with arrows.
//
// The original uses an @mermaid-js/parser AST (EmFrame / EventModel).
// We implement a line-based parser that captures the essential structure:
//
//   swimlane <label>
//   box <type> "<text>" [in <swimlane>]
//   relation <source_box> --> <target_box>
//
// Visual types from EventModeling spec:
//   command  → blue
//   event    → orange
//   readmodel / view → green
//   ui / screen → grey

#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    Command,
    Event,
    ReadModel,
    UiAutomation,
    Unknown,
}

impl BoxType {
    pub fn fill(&self) -> &'static str {
        match self {
            BoxType::Command => "#1E90FF",
            BoxType::Event => "#FFA500",
            BoxType::ReadModel => "#32CD32",
            BoxType::UiAutomation => "#808080",
            BoxType::Unknown => "#AAAAAA",
        }
    }

    pub fn stroke(&self) -> &'static str {
        match self {
            BoxType::Command => "#0060CC",
            BoxType::Event => "#CC7700",
            BoxType::ReadModel => "#009900",
            BoxType::UiAutomation => "#555555",
            BoxType::Unknown => "#777777",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "command" => BoxType::Command,
            "event" => BoxType::Event,
            "readmodel" | "read-model" | "view" | "read_model" => BoxType::ReadModel,
            "ui" | "screen" | "uiautomation" | "ui-automation" => BoxType::UiAutomation,
            _ => BoxType::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmBox {
    pub id: String,
    pub box_type: BoxType,
    pub text: String,
    pub swimlane: String,
}

#[derive(Debug, Clone)]
pub struct EmSwimlane {
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct EmRelation {
    pub source: String,
    pub target: String,
}

#[derive(Debug)]
pub struct EventModelDiagram {
    pub title: Option<String>,
    pub swimlanes: Vec<EmSwimlane>,
    pub boxes: Vec<EmBox>,
    pub relations: Vec<EmRelation>,
}

// ─── Parser ───────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<EventModelDiagram> {
    let mut title: Option<String> = None;
    let mut swimlanes: Vec<EmSwimlane> = Vec::new();
    let mut boxes: Vec<EmBox> = Vec::new();
    let mut relations: Vec<EmRelation> = Vec::new();
    let mut box_counter = 0usize;

    let mut in_header = true;
    let mut current_swimlane: Option<String> = None;

    for raw in input.lines() {
        let line = if let Some(p) = raw.find("%%") {
            &raw[..p]
        } else {
            raw
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if in_header {
            if trimmed.starts_with("eventmodeling") || trimmed.starts_with("event-modeling") {
                in_header = false;
            }
            continue;
        }

        // title
        if let Some(rest) = trimmed
            .strip_prefix("title ")
            .or_else(|| trimmed.strip_prefix("title\t"))
        {
            title = Some(rest.trim().to_string());
            continue;
        }

        // accTitle / accDescr – skip
        if trimmed.starts_with("accTitle") || trimmed.starts_with("accDescr") {
            continue;
        }

        // swimlane <label>
        if let Some(rest) = trimmed
            .strip_prefix("swimlane ")
            .or_else(|| trimmed.strip_prefix("swimlane\t"))
        {
            let label = strip_quotes(rest.trim()).to_string();
            current_swimlane = Some(label.clone());
            if !swimlanes.iter().any(|s| s.label == label) {
                swimlanes.push(EmSwimlane { label });
            }
            continue;
        }

        // box <type> "<text>" [in <swimlane>]
        // Also: <type> "<text>" [in <swimlane>]
        if let Some((bx, new_swimlane)) =
            parse_box_line(trimmed, &mut box_counter, &current_swimlane)
        {
            if let Some(sl) = &new_swimlane {
                if !swimlanes.iter().any(|s| &s.label == sl) {
                    swimlanes.push(EmSwimlane { label: sl.clone() });
                }
            }
            boxes.push(bx);
            continue;
        }

        // relation: <source> --> <target>  or  <source> -> <target>
        if trimmed.contains("-->") || (trimmed.contains("->") && !trimmed.contains("-->")) {
            let sep = if trimmed.contains("-->") { "-->" } else { "->" };
            if let Some(pos) = trimmed.find(sep) {
                let src = trimmed[..pos].trim().to_string();
                let tgt = trimmed[pos + sep.len()..].trim().to_string();
                if !src.is_empty() && !tgt.is_empty() {
                    relations.push(EmRelation {
                        source: src,
                        target: tgt,
                    });
                }
            }
            continue;
        }
    }

    // Ensure default swimlanes exist if none were declared
    if swimlanes.is_empty() && !boxes.is_empty() {
        let sl = EmSwimlane {
            label: "Default".to_string(),
        };
        swimlanes.push(sl);
        for bx in &mut boxes {
            if bx.swimlane.is_empty() {
                bx.swimlane = "Default".to_string();
            }
        }
    }

    crate::error::ParseResult::ok(EventModelDiagram {
        title,
        swimlanes,
        boxes,
        relations,
    })
}

// ─── Line parsers ─────────────────────────────────────────────────────────────

fn parse_box_line(
    s: &str,
    counter: &mut usize,
    current_swimlane: &Option<String>,
) -> Option<(EmBox, Option<String>)> {
    // Try "box <type> <text> [in <swimlane>]"
    let rest = if let Some(r) = s.strip_prefix("box ").or_else(|| s.strip_prefix("box\t")) {
        r.trim()
    } else {
        // Also accept lines that start with a known type keyword
        s
    };

    // Determine type from first token
    let first_space = rest.find(|c: char| c.is_whitespace());
    let (type_token, after_type) = if let Some(p) = first_space {
        (&rest[..p], rest[p..].trim())
    } else {
        (rest, "")
    };

    let box_type = BoxType::from_str(type_token);
    if box_type == BoxType::Unknown && !s.starts_with("box") {
        return None;
    }

    // Parse text (quoted or unquoted up to "in")
    let (text, after_text) = parse_text_token(after_type);
    if text.is_empty() {
        return None;
    }

    // Parse optional "in <swimlane>"
    let (swimlane, new_swimlane) = if let Some(in_rest) = after_text
        .trim()
        .strip_prefix("in ")
        .or_else(|| after_text.trim().strip_prefix("in\t"))
    {
        let sl = strip_quotes(in_rest.trim()).to_string();
        let new_sl = if current_swimlane.as_deref() != Some(&sl) {
            Some(sl.clone())
        } else {
            None
        };
        (sl, new_sl)
    } else {
        (current_swimlane.clone().unwrap_or_default(), None)
    };

    let id = format!("box{}", *counter);
    *counter += 1;

    Some((
        EmBox {
            id,
            box_type,
            text,
            swimlane,
        },
        new_swimlane,
    ))
}

fn parse_text_token(s: &str) -> (String, &str) {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix('"') {
        // Quoted string
        if let Some(end) = rest.find('"') {
            return (rest[..end].to_string(), &rest[end + 1..]);
        }
        return (rest.to_string(), "");
    }
    // Unquoted: up to " in " or end
    if let Some(pos) = s.find(" in ") {
        (s[..pos].trim().to_string(), &s[pos..])
    } else {
        (s.trim().to_string(), "")
    }
}

fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}
