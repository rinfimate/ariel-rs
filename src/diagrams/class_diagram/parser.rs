// Mermaid class diagram parser — faithful port of classDb.ts
// Supports the full class diagram syntax used in Mermaid v11.

use std::collections::HashMap;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,    // +
    Private,   // -
    Protected, // #
    Package,   // ~
    None,
}

impl Visibility {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Public),
            '-' => Some(Self::Private),
            '#' => Some(Self::Protected),
            '~' => Some(Self::Package),
            _ => None,
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Public => "+",
            Self::Private => "-",
            Self::Protected => "#",
            Self::Package => "~",
            Self::None => "",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Classifier {
    Abstract, // *
    Static,   // $
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemberType {
    Method,
    Attribute,
}

/// A class member (attribute or method).
#[derive(Debug, Clone)]
pub struct ClassMember {
    pub visibility: Visibility,
    pub id: String,
    pub parameters: String,  // only for methods
    pub return_type: String, // only for methods
    pub member_type: MemberType,
}

impl ClassMember {
    /// The display text shown in the class box.
    pub fn display_text(&self) -> String {
        let vis = self.visibility.to_str();
        let name = &self.id;
        match self.member_type {
            MemberType::Attribute => {
                format!("{}{}", vis, name)
            }
            MemberType::Method => {
                let params = &self.parameters;
                let ret = if self.return_type.is_empty() {
                    String::new()
                } else {
                    format!(" : {}", self.return_type)
                };
                format!("{}{}({}){}", vis, name, params, ret)
            }
        }
    }

    /// Parse a member string into a ClassMember (mirrors classTypes.ts ClassMember.parseMember).
    pub fn parse(input: &str, member_type: MemberType) -> Self {
        let mut visibility = Visibility::None;
        let mut classifier = Classifier::None;
        let mut id = String::new();
        let mut parameters = String::new();
        let mut return_type = String::new();

        let input = input.trim();

        match member_type {
            MemberType::Method => {
                // regex: ([#+~-])?(.+)\((.*)\)([\s$*])?(.*)([$*])?
                let method_re = parse_method(input);
                if let Some((vis_char, name, params, potential_cls, ret)) = method_re {
                    if let Some(v) = vis_char {
                        visibility = Visibility::from_char(v).unwrap_or(Visibility::None);
                    }
                    id = name;
                    parameters = params;
                    classifier = match potential_cls {
                        '$' => Classifier::Static,
                        '*' => Classifier::Abstract,
                        _ => {
                            // Check last char of return_type
                            let last = ret.chars().last().unwrap_or(' ');
                            if last == '$' || last == '*' {
                                let cls = if last == '$' {
                                    Classifier::Static
                                } else {
                                    Classifier::Abstract
                                };
                                return_type = ret[..ret.len() - 1].to_string();
                                cls
                            } else {
                                return_type = ret.clone();
                                Classifier::None
                            }
                        }
                    };
                    if classifier == Classifier::None && !return_type.is_empty() {
                        // Already set above
                    } else if classifier != Classifier::None && return_type.is_empty() {
                        return_type = ret;
                    }
                }
            }
            MemberType::Attribute => {
                let chars: Vec<char> = input.chars().collect();
                let len = chars.len();
                if len == 0 {
                    return ClassMember {
                        visibility,
                        id,
                        parameters,
                        return_type,
                        member_type,
                    };
                }
                let first = chars[0];
                if Visibility::from_char(first).is_some() {
                    visibility = Visibility::from_char(first).unwrap();
                }
                let last = chars[len - 1];
                if last == '$' {
                    classifier = Classifier::Static;
                } else if last == '*' {
                    classifier = Classifier::Abstract;
                }
                let start = if visibility != Visibility::None { 1 } else { 0 };
                let end = if classifier != Classifier::None {
                    len - 1
                } else {
                    len
                };
                id = chars[start..end]
                    .iter()
                    .collect::<String>()
                    .trim()
                    .to_string();
            }
        }

        ClassMember {
            visibility,
            id,
            parameters,
            return_type,
            member_type,
        }
    }
}

/// Parse method signature: returns (visibility_char, name, params, classifier_char, return_type)
fn parse_method(input: &str) -> Option<(Option<char>, String, String, char, String)> {
    let paren_open = input.find('(')?;
    let paren_close = input.rfind(')')?;
    if paren_close <= paren_open {
        return None;
    }

    let before_paren = &input[..paren_open];
    let params = input[paren_open + 1..paren_close].trim().to_string();
    let after_paren = input[paren_close + 1..].trim();

    let before_chars: Vec<char> = before_paren.chars().collect();
    let vis_char = if !before_chars.is_empty() {
        let c = before_chars[0];
        if matches!(c, '+' | '-' | '#' | '~') {
            Some(c)
        } else {
            None
        }
    } else {
        None
    };
    let name_start = if vis_char.is_some() { 1 } else { 0 };
    let name: String = before_chars[name_start..]
        .iter()
        .collect::<String>()
        .trim()
        .to_string();

    // after_paren: optional classifier [$*] + optional return type
    let (potential_cls, ret) = if let Some(first_char) = after_paren.chars().next() {
        if first_char == '$' || first_char == '*' {
            (first_char, after_paren[1..].trim().to_string())
        } else {
            (' ', after_paren.to_string())
        }
    } else {
        (' ', String::new())
    };

    Some((vis_char, name, params, potential_cls, ret))
}

// ─── Relation types ───────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum RelationType {
    Extension,   // <|-- or --|>
    Composition, // *-- or --*
    Aggregation, // o-- or --o
    Association, // --> or <--
    Link,        // --
    Dependency,  // ..> or <..  (dashed)
    Realization, // ..|> or <|.. (dashed + extension)
    DashedLink,  // ..
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineStyle {
    Solid,
    Dashed,
}

/// Which end of the line has which marker.
#[derive(Debug, Clone, PartialEq)]
pub enum EndType {
    None,
    Arrow,       // > or <
    Extension,   // |> or <|
    Composition, // *
    Aggregation, // o
}

#[derive(Debug, Clone)]
pub struct ClassRelation {
    pub id1: String,
    pub id2: String,
    pub title: String,  // edge label
    pub title1: String, // cardinality on id1 side
    pub title2: String, // cardinality on id2 side
    pub line_style: LineStyle,
    pub start: EndType, // marker at id1 end
    pub end: EndType,   // marker at id2 end
}

// ─── Class node ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ClassNode {
    pub label: String,
    pub annotations: Vec<String>, // e.g. "interface", "enumeration", "abstract"
    pub members: Vec<ClassMember>, // attributes
    pub methods: Vec<ClassMember>, // methods
}

// ─── Diagram ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ClassDiagram {
    pub direction: String,
    pub classes: HashMap<String, ClassNode>,
    pub relations: Vec<ClassRelation>,
    /// Insertion order for rendering
    pub class_order: Vec<String>,
}

// ─── Parser ───────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<ClassDiagram> {
    let mut diag = ClassDiagram {
        direction: "TB".to_string(),
        classes: HashMap::new(),
        relations: Vec::new(),
        class_order: Vec::new(),
    };

    let mut lines = input.lines().peekable();

    while let Some(raw_line) = lines.next() {
        let line = strip_comment(raw_line).trim().to_string();
        if line.is_empty() {
            continue;
        }

        // Header
        if line.starts_with("classDiagram") {
            if let Some(rest) = line.strip_prefix("classDiagram-v2") {
                let dir = rest.trim();
                if !dir.is_empty() {
                    diag.direction = dir.to_uppercase();
                }
            } else if let Some(rest) = line.strip_prefix("classDiagram") {
                let dir = rest.trim();
                if !dir.is_empty() {
                    diag.direction = dir.to_uppercase();
                }
            }
            continue;
        }

        // title / accTitle / accDescr / link / click — skip
        if line.starts_with("title")
            || line.starts_with("accTitle")
            || line.starts_with("accDescr")
            || line.starts_with("link ")
            || line.starts_with("click ")
        {
            continue;
        }

        // direction keyword
        if let Some(stripped) = line.strip_prefix("direction ") {
            let dir = stripped.trim().to_uppercase();
            diag.direction = dir;
            continue;
        }

        // class block: "class Name {" or "class Name"
        if let Some(stripped) = line.strip_prefix("class ") {
            let rest = stripped.trim();
            parse_class_block(rest, &mut lines, &mut diag);
            continue;
        }

        // Relation lines: classA --> classB or classA <|-- classB etc.
        if let Some(rel) = try_parse_relation(&line) {
            // Ensure both classes exist
            ensure_class(&mut diag, &rel.id1);
            ensure_class(&mut diag, &rel.id2);
            diag.relations.push(rel);
            continue;
        }

        // Annotation standalone: <<interface>> ClassName
        if line.starts_with("<<") {
            if let Some(close) = line.find(">>") {
                let annotation = line[2..close].trim().to_string();
                let class_name = line[close + 2..].trim();
                if !class_name.is_empty() {
                    ensure_class(&mut diag, class_name);
                    if let Some(cls) = diag.classes.get_mut(class_name) {
                        cls.annotations.push(annotation);
                    }
                }
            }
            continue;
        }

        // ClassName : memberDefinition  (with or without space before colon)
        // Handles both "Animal : +int age" and "Animal: +isMammal()"
        let colon_match = line
            .find(" : ")
            .map(|i| (i, i + 3))
            .or_else(|| line.find(": ").map(|i| (i, i + 2)));
        if let Some((colon, after)) = colon_match {
            let class_name = line[..colon].trim();
            let member_def = line[after..].trim();
            if !class_name.contains(' ') && !class_name.is_empty() {
                ensure_class(&mut diag, class_name);
                add_member_to_class(&mut diag, class_name, member_def);
                continue;
            }
        }
    }

    crate::error::ParseResult::ok(diag)
}

fn strip_comment(line: &str) -> &str {
    if let Some(pos) = line.find("%%") {
        &line[..pos]
    } else {
        line
    }
}

fn ensure_class(diag: &mut ClassDiagram, id: &str) {
    if !diag.classes.contains_key(id) {
        diag.class_order.push(id.to_string());
        diag.classes.insert(
            id.to_string(),
            ClassNode {
                label: id.to_string(),
                annotations: Vec::new(),
                members: Vec::new(),
                methods: Vec::new(),
            },
        );
    }
}

/// Parse "ClassName {" or "ClassName" and consume block lines.
fn parse_class_block<'a>(
    rest: &str,
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    diag: &mut ClassDiagram,
) {
    // rest might be: "Animal", "Animal {", "Animal {}", "Animal{interface}"
    let (class_name, has_block) = if let Some(brace) = rest.find('{') {
        let name = rest[..brace].trim().to_string();
        let inline = rest[brace + 1..]
            .trim()
            .trim_end_matches('}')
            .trim()
            .to_string();
        (name, Some(inline))
    } else {
        (rest.trim().to_string(), None)
    };

    if class_name.is_empty() {
        return;
    }
    ensure_class(diag, &class_name);

    match has_block {
        Some(inline_body) if !inline_body.is_empty() => {
            // Single-line block like "class Foo { +field }"
            parse_class_body_line(&class_name, &inline_body, diag);
        }
        Some(_) => {
            // Empty inline block or opening brace — read until closing brace
            consume_class_block(&class_name, lines, diag);
        }
        None => {
            // No block — just a class declaration
        }
    }
}

fn consume_class_block<'a>(
    class_name: &str,
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    diag: &mut ClassDiagram,
) {
    loop {
        match lines.next() {
            None => break,
            Some(raw) => {
                let line = strip_comment(raw).trim().to_string();
                if line == "}" || line == "}}" {
                    break;
                }
                if line.is_empty() {
                    continue;
                }
                parse_class_body_line(class_name, &line, diag);
            }
        }
    }
}

fn parse_class_body_line(class_name: &str, line: &str, diag: &mut ClassDiagram) {
    let line = line.trim();

    // Annotation: <<interface>> or <<enumeration>>
    if line.starts_with("<<") {
        if let Some(close) = line.find(">>") {
            let annotation = line[2..close].trim().to_string();
            if let Some(cls) = diag.classes.get_mut(class_name) {
                cls.annotations.push(annotation);
            }
            return;
        }
    }

    // Member definition
    add_member_to_class(diag, class_name, line);
}

fn add_member_to_class(diag: &mut ClassDiagram, class_name: &str, def: &str) {
    let def = def.trim();
    if def.is_empty() {
        return;
    }

    let member = if def.contains('(') {
        ClassMember::parse(def, MemberType::Method)
    } else {
        ClassMember::parse(def, MemberType::Attribute)
    };

    if let Some(cls) = diag.classes.get_mut(class_name) {
        match member.member_type {
            MemberType::Method => cls.methods.push(member),
            MemberType::Attribute => cls.members.push(member),
        }
    }
}

// ─── Relation parser ──────────────────────────────────────────────────────────

/// Try to parse a relation from a line like:
///   Animal <|-- Dog
///   classA --|> classB : Inheritance
///   Customer "1" --> "*" Ticket
///   classC --* classD : Composition
fn try_parse_relation(line: &str) -> Option<ClassRelation> {
    // Split off optional label at the end: " : Label"
    let (main, title) = if let Some(colon_pos) = find_label_colon(line) {
        let title = line[colon_pos + 1..].trim().to_string();
        (&line[..colon_pos], title)
    } else {
        (line, String::new())
    };

    let main = main.trim();

    // Split by whitespace tokens
    let tokens: Vec<&str> = main.split_whitespace().collect();
    if tokens.len() < 3 {
        return None;
    }

    // Find the relation operator token(s)
    // Tokens: [id1, optional_cardinality, operator, optional_cardinality, id2]
    // or: [id1, operator, id2]
    // or: [id1, "card", operator, "card", id2]

    // Try to find relation operator by scanning tokens
    let mut rel_idx: Option<usize> = None;
    for (i, tok) in tokens.iter().enumerate() {
        if i == 0 {
            continue;
        } // skip first (class name)
        let t = *tok;
        // Strip cardinality quotes
        let t_inner = t.trim_matches('"').trim_matches('\'');
        if is_relation_op(t_inner) || is_relation_op(t) {
            rel_idx = Some(i);
            break;
        }
    }

    let rel_idx = rel_idx?;

    // id1 may include a quoted cardinality before the operator
    // id2 may include a quoted cardinality after the operator
    let (id1, card1) = extract_id_and_cardinality(&tokens[..rel_idx]);
    let (id2, card2) = extract_id_and_cardinality(&tokens[rel_idx + 1..]);

    if id1.is_empty() || id2.is_empty() {
        return None;
    }

    let op = tokens[rel_idx];
    let (start, end, line_style) = parse_relation_op(op);

    Some(ClassRelation {
        id1,
        id2,
        title,
        title1: card1,
        title2: card2,
        line_style,
        start,
        end,
    })
}

/// Find the last " : " that is outside of quotes (for label).
fn find_label_colon(line: &str) -> Option<usize> {
    // Look for " : " pattern from the right
    let bytes = line.as_bytes();
    let mut i = line.len().saturating_sub(1);
    loop {
        if i + 2 >= line.len() {
            if i == 0 {
                break;
            }
            i = i.saturating_sub(1);
            continue;
        }
        // Check from i backwards for " : "
        if i == 0 {
            break;
        }
        i -= 1;
    }
    // Simpler: rfind " : "
    let pattern = " : ";
    let pat_bytes = pattern.as_bytes();
    if bytes.len() < pat_bytes.len() {
        return None;
    }
    let mut last = None;
    for start in 0..=(bytes.len() - pat_bytes.len()) {
        if &bytes[start..start + pat_bytes.len()] == pat_bytes {
            last = Some(start + 1); // position of ':'
        }
    }
    last
}

fn extract_id_and_cardinality(tokens: &[&str]) -> (String, String) {
    // tokens might be: ["Animal"]
    // or: ["Animal", "\"1\""] — no, cardinality is BETWEEN id and operator
    // Actually: ["\"1\"", "Ticket"] or ["Animal"]
    // or just a single name
    match tokens.len() {
        0 => (String::new(), String::new()),
        1 => {
            let t = tokens[0];
            let name = t.trim_matches('"').trim_matches('\'');
            // If it's a cardinality-only token, it shouldn't have a class name
            // Actually for "Animal" or similar, this is just the class name
            (name.to_string(), String::new())
        }
        2 => {
            // ["ClassName", "\"cardinality\""] or ["\"cardinality\"", "ClassName"]
            let first = tokens[0].trim_matches('"').trim_matches('\'');
            let second = tokens[1].trim_matches('"').trim_matches('\'');
            // The class name is the identifier, cardinality is usually quoted
            // The unquoted one is the class name
            if tokens[0].starts_with('"') || tokens[0].starts_with('\'') {
                (second.to_string(), first.to_string())
            } else {
                (first.to_string(), second.to_string())
            }
        }
        _ => {
            // Take first non-quoted as class name, last quoted as cardinality
            let name = tokens[0].trim_matches('"').trim_matches('\'').to_string();
            let card = tokens
                .last()
                .map(|t| t.trim_matches('"').trim_matches('\'').to_string())
                .unwrap_or_default();
            (name, card)
        }
    }
}

fn is_relation_op(s: &str) -> bool {
    // All known relation operators
    matches!(
        s,
        "<|--"
            | "--|>"
            | "<|.."
            | "..|>"
            | "--*"
            | "*--"
            | "--o"
            | "o--"
            | "-->"
            | "<--"
            | "<-->"
            | "..->"
            | "<-.."
            | "..<"
            | "--"
            | ".."
            | "<.."
            | "..>"
    ) || is_complex_relation_op(s)
}

fn is_complex_relation_op(s: &str) -> bool {
    // Check known patterns:
    // Starts with arrow chars and has enough length
    let solidrel = s.contains("--");
    let dashrel = s.contains("..");
    (solidrel || dashrel)
        && s.len() >= 2
        && (s.contains('|')
            || s.contains('*')
            || s.contains('o')
            || s.contains('>')
            || s.contains('<')
            || s == "--"
            || s == "..")
}

/// Parse relation operator into (start_end_type, end_end_type, line_style).
/// Direction: id1 [start_marker] ---- [end_marker] id2
fn parse_relation_op(op: &str) -> (EndType, EndType, LineStyle) {
    let line_style = if op.contains("..") {
        LineStyle::Dashed
    } else {
        LineStyle::Solid
    };

    // Normalize: replace ".." and "--" with a separator
    let norm = op.replace("..", "~").replace("--", "~");

    // Extract left marker (before ~) and right marker (after ~)
    let parts: Vec<&str> = norm.split('~').collect();
    let left = parts.first().copied().unwrap_or("");
    let right = parts.last().copied().unwrap_or("");

    let start = parse_end_marker(left, true);
    let end = parse_end_marker(right, false);

    (start, end, line_style)
}

fn parse_end_marker(s: &str, is_left: bool) -> EndType {
    if s.is_empty() {
        return EndType::None;
    }
    // For left side: markers pointing INTO the line (like <|, <, o, *)
    // For right side: markers pointing OUT of the line (like |>, >, o, *)
    if s.contains("|>") || s.contains("<|") {
        EndType::Extension
    } else if s.contains('*') {
        EndType::Composition
    } else if s.contains('o') {
        EndType::Aggregation
    } else if (is_left && s.contains('<')) || (!is_left && s.contains('>')) {
        EndType::Arrow
    } else {
        EndType::None
    }
}
