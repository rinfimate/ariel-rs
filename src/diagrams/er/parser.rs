// ER diagram parser — hand-written recursive-descent parser for Mermaid erDiagram syntax.
//
// Supports:
//   - Entity declarations with optional attribute blocks
//   - Relationship lines with crow's foot notation
//   - Relationship labels (quoted or unquoted)
//   - All standard cardinality markers: ||, |{, }|, o{, }o, |o, o|, ..
//   - Dashed relationship lines (..)

/// Attribute key types (PK, FK, UK, or none)
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeKey {
    PK,
    FK,
    UK,
    None,
}

/// One attribute row inside an entity block
#[derive(Debug, Clone)]
pub struct Attribute {
    pub attr_type: String,
    pub name: String,
    pub key: AttributeKey,
    pub comment: String,
}

/// An entity (table)
#[derive(Debug, Clone)]
pub struct Entity {
    pub name: String,
    pub attributes: Vec<Attribute>,
}

/// Cardinality value — one side of a relationship
#[derive(Debug, Clone, PartialEq)]
pub enum Cardinality {
    ZeroOrOne,  // o| or |o
    ExactlyOne, // ||
    ZeroOrMore, // o{ or }o
    OneOrMore,  // |{ or }|
}

/// Whether the relationship line is dashed (non-identifying) or solid (identifying)
#[derive(Debug, Clone, PartialEq)]
pub enum RelType {
    Identifying,    // solid --
    NonIdentifying, // dashed ..
}

/// A relationship between two entities
#[derive(Debug, Clone)]
pub struct Relationship {
    pub entity_a: String,
    pub card_a: Cardinality, // cardinality on the entity_a side
    pub rel_type: RelType,
    pub card_b: Cardinality, // cardinality on the entity_b side
    pub entity_b: String,
    pub label: String,
}

/// The full ER diagram (output of parsing)
#[derive(Debug, Default)]
pub struct ErDiagram {
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
}

// ── Parser ───────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<ErDiagram> {
    let mut diag = ErDiagram::default();
    let mut lines = input.lines().peekable();

    // Skip "erDiagram" header line
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed == "erDiagram" || trimmed.starts_with("erDiagram ") {
            break;
        }
    }

    let mut entity_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // Skip blank lines and comments
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        // Try to detect a relationship line — contains cardinality markers
        // Relationship: ENTITY_A CARDINALITY_A REL CARDINALITY_B ENTITY_B : label
        if let Some(rel) = try_parse_relationship(trimmed) {
            // Register entities mentioned in relationships
            if !entity_names.contains(&rel.entity_a) {
                entity_names.insert(rel.entity_a.clone());
                diag.entities.push(Entity {
                    name: rel.entity_a.clone(),
                    attributes: vec![],
                });
            }
            if !entity_names.contains(&rel.entity_b) {
                entity_names.insert(rel.entity_b.clone());
                diag.entities.push(Entity {
                    name: rel.entity_b.clone(),
                    attributes: vec![],
                });
            }
            diag.relationships.push(rel);
            continue;
        }

        // Entity block: ENTITY_NAME { ... }
        // Could be "ENTITY {" or "ENTITY" alone if entity has no attributes
        // Detect: line ends with '{' or is just a name followed by '{'
        let (entity_name, has_open_brace) = if trimmed.ends_with('{') {
            let name = trimmed.trim_end_matches('{').trim().to_string();
            (name, true)
        } else {
            // Bare entity name (no attributes, no relationship) — rare but valid
            (trimmed.to_string(), false)
        };

        if entity_name.is_empty() || entity_name.contains(' ') || entity_name.contains(':') {
            continue;
        }

        // Validate entity name: alphanumeric, underscore, dash
        if !is_valid_entity_name(&entity_name) {
            continue;
        }

        let attrs = if has_open_brace {
            parse_attribute_block(&mut lines)
        } else {
            vec![]
        };

        // Upsert entity (relationship scanning may have already added it without attrs)
        if let Some(existing) = diag.entities.iter_mut().find(|e| e.name == entity_name) {
            if !attrs.is_empty() {
                existing.attributes = attrs;
            }
        } else {
            entity_names.insert(entity_name.clone());
            diag.entities.push(Entity {
                name: entity_name,
                attributes: attrs,
            });
        }
    }

    crate::error::ParseResult::ok(diag)
}

fn is_valid_entity_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

fn parse_attribute_block<'a, I: Iterator<Item = &'a str>>(
    lines: &mut std::iter::Peekable<I>,
) -> Vec<Attribute> {
    let mut attrs = vec![];
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed == "}" {
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }
        if let Some(attr) = parse_attribute_line(trimmed) {
            attrs.push(attr);
        }
    }
    attrs
}

fn parse_attribute_line(s: &str) -> Option<Attribute> {
    // Format: type name [PK|FK|UK] ["comment"]
    // Examples:
    //   string name PK "the customer name"
    //   int id
    //   date created_at FK
    let parts: Vec<&str> = s
        .split('"')
        .next()
        .unwrap_or(s)
        .split_whitespace()
        .collect();
    let comment = if let Some(idx) = s.find('"') {
        let rest = &s[idx + 1..];
        rest.trim_end_matches('"').to_string()
    } else {
        String::new()
    };

    if parts.len() < 2 {
        return None;
    }

    let attr_type = parts[0].to_string();
    let name = parts[1].to_string();

    // Check if there's a key modifier (PK, FK, UK) — may appear as 3rd token
    // It could also be repeated: "PK, FK"
    let key = if parts.len() >= 3 {
        let key_str = parts[2].trim_end_matches(',').to_ascii_uppercase();
        match key_str.as_str() {
            "PK" => AttributeKey::PK,
            "FK" => AttributeKey::FK,
            "UK" => AttributeKey::UK,
            _ => AttributeKey::None,
        }
    } else {
        AttributeKey::None
    };

    Some(Attribute {
        attr_type,
        name,
        key,
        comment,
    })
}

/// Try to parse a relationship line.
/// Syntax: ENTITY_A CARD_A REL CARD_B ENTITY_B : "label"
/// where CARD_A REL CARD_B is a token like  ||--o{  or  }|..|{
fn try_parse_relationship(s: &str) -> Option<Relationship> {
    // Split on whitespace into tokens
    let tokens: Vec<&str> = s.split_whitespace().collect();
    // Minimum: ENTITY_A REL_TOKEN ENTITY_B [: label]
    // REL_TOKEN contains both cardinalities and the line type
    if tokens.len() < 3 {
        return None;
    }

    let entity_a = tokens[0].to_string();
    if !is_valid_entity_name(&entity_a) {
        return None;
    }

    let rel_token = tokens[1];
    let entity_b = tokens[2].to_string();
    if !is_valid_entity_name(&entity_b) {
        return None;
    }

    // Parse the label: either `: label` or `: "label"` later in tokens
    // tokens may be: ["ENTITY_A", "||--o{", "ENTITY_B", ":", "label"]
    // or just: ["ENTITY_A", "||--o{", "ENTITY_B"]
    let label = parse_label_tokens(&tokens[3..]);

    // Parse rel_token — format:  <card_a><line><card_b>
    // card notation: |, {, }, o
    // line notation: -- or ..
    let (card_a, rel_type, card_b) = parse_rel_token(rel_token)?;

    Some(Relationship {
        entity_a,
        card_a,
        rel_type,
        card_b,
        entity_b,
        label,
    })
}

fn parse_label_tokens(tokens: &[&str]) -> String {
    if tokens.is_empty() {
        return String::new();
    }
    // Skip leading ":"
    let start = if tokens[0] == ":" { 1 } else { 0 };
    let joined = tokens[start..].join(" ");
    // Strip surrounding quotes
    let trimmed = joined.trim().trim_matches('"').to_string();
    trimmed
}

/// Parse a relationship token like: ||--o{  |{..}|  }o..o{  etc.
/// Returns (card_a, rel_type, card_b)
///
/// The token has the structure:
///   LEFT_MARKER LINE_MARKER RIGHT_MARKER
///
/// Left markers (entity_a side, written on left, describe the relationship FROM entity_a's perspective):
///   |   = exactly one (left side barb)
///   o   = zero (circle, left side)
///   {   = many (crow's foot pointing left — actually in ERD {| means "one or more at B")
///   }   = many (pointing right)
///
/// In Mermaid's notation, the LEFT cardinality token describes entity_a's cardinality.
/// The full notation is: ENTITY_A CARD_A LINE CARD_B ENTITY_B
///
/// The LEFT token characters (before --/..) describe entity_a cardinality:
///   ||  -> ExactlyOne  (two bars)
///   |o or o| -> ZeroOrOne
///   |{ -> OneOrMore (from A's right towards B, one-or-more at B, exact at A)
///     Actually mermaid: entityA ||--o{ entityB means:
///       entityA: exactly one, entityB: zero or more
///
/// Let's parse it character by character.
/// The token splits into: left_card_str ++ line_str ++ right_card_str
/// where line_str is "--" or ".."
fn parse_rel_token(token: &str) -> Option<(Cardinality, RelType, Cardinality)> {
    // Find the line type (-- or ..)
    let (left_str, line_str, right_str) = split_rel_token(token)?;

    let rel_type = match line_str {
        "--" => RelType::Identifying,
        ".." => RelType::NonIdentifying,
        _ => return None,
    };

    let card_a = parse_left_cardinality(left_str)?;
    let card_b = parse_right_cardinality(right_str)?;

    Some((card_a, rel_type, card_b))
}

/// Split token into (left_chars, line_type, right_chars)
/// The line type is "--" or ".." in the middle of the token.
fn split_rel_token(token: &str) -> Option<(&str, &str, &str)> {
    if let Some(idx) = token.find("--") {
        let left = &token[..idx];
        let right = &token[idx + 2..];
        Some((left, "--", right))
    } else if let Some(idx) = token.find("..") {
        let left = &token[..idx];
        let right = &token[idx + 2..];
        Some((left, "..", right))
    } else {
        None
    }
}

/// Left side cardinality (entity_a side).
/// Mermaid notation for left side:
///   "||"  = exactly one (two bars on entity_a side)
///   "|o"  = zero or one (bar + circle, zero or one at entity_a)
///   "o|"  = zero or one (circle + bar)
///   "|{"  = one or more (bar + crow left, but pointing towards entity_a side)
///   "}|"  = one or more
///   "o{"  = zero or more
///   "}o"  = zero or more
fn parse_left_cardinality(s: &str) -> Option<Cardinality> {
    match s {
        "||" => Some(Cardinality::ExactlyOne),
        "|o" | "o|" => Some(Cardinality::ZeroOrOne),
        "|{" | "}|" => Some(Cardinality::OneOrMore),
        "o{" | "}o" => Some(Cardinality::ZeroOrMore),
        _ => {
            // Fallback: parse character by character
            parse_cardinality_chars(s)
        }
    }
}

/// Right side cardinality (entity_b side). Same mapping, just mirrored notation.
fn parse_right_cardinality(s: &str) -> Option<Cardinality> {
    match s {
        "||" => Some(Cardinality::ExactlyOne),
        "o|" | "|o" => Some(Cardinality::ZeroOrOne),
        "{|" | "|}" => Some(Cardinality::OneOrMore),
        "{o" | "o}" => Some(Cardinality::ZeroOrMore),
        // Also accept left-side-style tokens for robustness
        "}|" | "|{" => Some(Cardinality::OneOrMore),
        "}o" | "o{" => Some(Cardinality::ZeroOrMore),
        _ => parse_cardinality_chars(s),
    }
}

fn parse_cardinality_chars(s: &str) -> Option<Cardinality> {
    let has_pipe = s.contains('|');
    let has_brace = s.contains('{') || s.contains('}');
    let has_circle = s.contains('o');
    let pipe_count = s.chars().filter(|&c| c == '|').count();

    if pipe_count >= 2 && !has_brace && !has_circle {
        Some(Cardinality::ExactlyOne)
    } else if has_brace && has_circle {
        Some(Cardinality::ZeroOrMore)
    } else if has_brace && has_pipe {
        Some(Cardinality::OneOrMore)
    } else if has_pipe && has_circle {
        Some(Cardinality::ZeroOrOne)
    } else if has_brace {
        Some(Cardinality::OneOrMore)
    } else if has_circle {
        Some(Cardinality::ZeroOrOne)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        let input = r#"erDiagram
    CUSTOMER ||--o{ ORDER : places
    ORDER ||--|{ LINE-ITEM : contains
"#;
        let diag = parse(input).diagram;
        assert_eq!(diag.entities.len(), 3);
        assert_eq!(diag.relationships.len(), 2);

        let rel0 = &diag.relationships[0];
        assert_eq!(rel0.entity_a, "CUSTOMER");
        assert_eq!(rel0.entity_b, "ORDER");
        assert_eq!(rel0.card_a, Cardinality::ExactlyOne);
        assert_eq!(rel0.card_b, Cardinality::ZeroOrMore);
        assert_eq!(rel0.label, "places");

        let rel1 = &diag.relationships[1];
        assert_eq!(rel1.card_b, Cardinality::OneOrMore);
    }

    #[test]
    fn parse_attributes() {
        let input = r#"erDiagram
    CUSTOMER {
        string name PK
        string email
        int age
    }
    ORDER {
        int id PK
        date created
    }
    CUSTOMER ||--o{ ORDER : places
"#;
        let diag = parse(input).diagram;
        let customer = diag.entities.iter().find(|e| e.name == "CUSTOMER").unwrap();
        assert_eq!(customer.attributes.len(), 3);
        assert_eq!(customer.attributes[0].attr_type, "string");
        assert_eq!(customer.attributes[0].name, "name");
        assert_eq!(customer.attributes[0].key, AttributeKey::PK);
    }

    #[test]
    fn parse_dashed_relationship() {
        let input = r#"erDiagram
    PERSON }|..|{ PERSON : "is married to"
"#;
        let diag = parse(input).diagram;
        let rel = &diag.relationships[0];
        assert_eq!(rel.rel_type, RelType::NonIdentifying);
        assert_eq!(rel.label, "is married to");
    }
}
