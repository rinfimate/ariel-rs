// ER diagram parser — faithful port of Mermaid erDb.ts + erDiagram grammar
//
// erDb Cardinality values: ZERO_OR_ONE, ZERO_OR_MORE, ONE_OR_MORE, ONLY_ONE, MD_PARENT
// erDb Identification: NON_IDENTIFYING (dashed ..), IDENTIFYING (solid --)
//
// Grammar: rel.entityA relSpec.cardB relSpec.relType relSpec.cardA rel.entityB : rel.roleA
// Note: cardB is at the START of the line (entity_b marker), cardA is at the END (entity_a marker)

/// Cardinality — erDb.Cardinality values
#[derive(Debug, Clone, PartialEq)]
pub enum Cardinality {
    ZeroOrOne,
    ZeroOrMore,
    OneOrMore,
    OnlyOne,
    #[allow(dead_code)]
    MdParent,
}

/// Whether the relationship line is solid (IDENTIFYING) or dashed (NON_IDENTIFYING)
#[derive(Debug, Clone, PartialEq)]
pub enum Identification {
    Identifying,
    NonIdentifying,
}

/// RelSpec: the middle part of a relationship line
#[derive(Debug, Clone)]
pub struct RelSpec {
    pub card_a: Cardinality,      // marker at entityA end (end marker)
    pub rel_type: Identification, // solid or dashed
    pub card_b: Cardinality,      // marker at entityB end (start marker)
}

/// Attribute key types
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeKeyType {
    PK,
    FK,
    UK,
}

/// One attribute row
#[derive(Debug, Clone)]
pub struct Attribute {
    pub attribute_type: String,
    pub attribute_name: String,
    pub attribute_key_type_list: Vec<AttributeKeyType>,
    pub attribute_comment: String,
}

/// An entity — erDb entity with optional attributes
#[derive(Debug, Clone)]
pub struct EntityNode {
    pub id: String, // same as name (entityNameIds maps name→id in JS; we keep them equal)
    pub alias: String,
    pub attributes: Vec<Attribute>,
}

/// A relationship — erDb relationship object
#[derive(Debug, Clone)]
pub struct ErRelationship {
    pub entity_a: String, // entity id (= name)
    pub role_a: String,   // label
    pub entity_b: String, // entity id
    pub rel_spec: RelSpec,
}

/// Full ER diagram
#[derive(Debug, Default)]
pub struct ErDiagram {
    pub entities: Vec<EntityNode>,
    pub relationships: Vec<ErRelationship>,
}

// ── Public parse entry point ─────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<ErDiagram> {
    let mut diag = ErDiagram::default();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    let mut in_diagram = false;

    while i < lines.len() {
        let raw = lines[i];
        let trimmed = strip_comment(raw).trim();
        i += 1;

        if trimmed.is_empty() {
            continue;
        }

        // Detect diagram header
        if !in_diagram {
            if trimmed == "erDiagram" || trimmed.starts_with("erDiagram ") {
                in_diagram = true;
            }
            continue;
        }

        // title / accTitle / accDescr — skip
        if trimmed.starts_with("title")
            || trimmed.starts_with("accTitle")
            || trimmed.starts_with("accDescr")
        {
            continue;
        }

        // Entity block: "EntityName {" or "EntityName alias {"
        if trimmed.ends_with('{') {
            let before = trimmed.trim_end_matches('{').trim();
            let (name, alias) = parse_entity_name(before);
            ensure_entity(&mut diag, &name, &alias);
            // Read attributes until "}"
            while i < lines.len() {
                let araw = lines[i];
                let at = strip_comment(araw).trim();
                i += 1;
                if at == "}" {
                    break;
                }
                if at.is_empty() {
                    continue;
                }
                if let Some(attr) = parse_attribute(at) {
                    if let Some(e) = diag.entities.iter_mut().find(|e| e.id == name) {
                        e.attributes.push(attr);
                    }
                }
            }
            continue;
        }

        // Standalone entity: "EntityName" (no block)
        // Try to parse as relationship first; if not, treat as entity declaration
        if let Some(rel) = parse_relationship(trimmed) {
            ensure_entity(&mut diag, &rel.entity_a, &rel.entity_a);
            ensure_entity(&mut diag, &rel.entity_b, &rel.entity_b);
            diag.relationships.push(rel);
        } else if is_entity_name(trimmed) {
            let (name, alias) = parse_entity_name(trimmed);
            ensure_entity(&mut diag, &name, &alias);
        }
    }

    crate::error::ParseResult::ok(diag)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn strip_comment(s: &str) -> &str {
    if let Some(p) = s.find("%%") {
        &s[..p]
    } else {
        s
    }
}

fn parse_entity_name(s: &str) -> (String, String) {
    // "Name" or "Name[alias]" — alias is the display name
    let s = s.trim();
    if let Some(bi) = s.find('[') {
        let name = s[..bi].trim().to_string();
        let alias = s[bi + 1..].trim_end_matches(']').trim().to_string();
        (name, alias)
    } else {
        (s.to_string(), s.to_string())
    }
}

fn ensure_entity(diag: &mut ErDiagram, name: &str, alias: &str) {
    if !diag.entities.iter().any(|e| e.id == name) {
        diag.entities.push(EntityNode {
            id: name.to_string(),
            alias: alias.to_string(),
            attributes: Vec::new(),
        });
    }
}

fn is_entity_name(s: &str) -> bool {
    // Simple heuristic: no spaces, no relationship markers
    !s.contains("--") && !s.contains("..") && !s.contains(':') && !s.contains('{')
}

/// Parse an attribute line: "type name [key] [\"comment\"]"
fn parse_attribute(s: &str) -> Option<Attribute> {
    let mut parts = s.split_whitespace();
    let attr_type = parts.next()?.to_string();
    let attr_name = parts.next()?.to_string();
    let mut key_types = Vec::new();
    let mut comment = String::new();

    // Remaining tokens: key types (PK, FK, UK) and/or comment (in quotes)
    let mut in_comment = false;
    let mut comment_parts = Vec::new();
    for token in parts {
        if in_comment {
            let t = token.trim_end_matches('"');
            comment_parts.push(t);
            if token.ends_with('"') {
                break;
            }
        } else if token.starts_with('"') {
            in_comment = true;
            let t = token.trim_start_matches('"');
            let t = t.trim_end_matches('"');
            comment_parts.push(t);
            if token.ends_with('"') && token.len() > 1 {
                break;
            }
        } else {
            match token.to_uppercase().as_str() {
                "PK" => key_types.push(AttributeKeyType::PK),
                "FK" => key_types.push(AttributeKeyType::FK),
                "UK" => key_types.push(AttributeKeyType::UK),
                _ => {}
            }
        }
    }
    if !comment_parts.is_empty() {
        comment = comment_parts.join(" ");
    }

    Some(Attribute {
        attribute_type: attr_type,
        attribute_name: attr_name,
        attribute_key_type_list: key_types,
        attribute_comment: comment,
    })
}

/// Parse a relationship line: "EntityA cardB relType cardA EntityB : label"
/// Grammar (from Mermaid): entityA  relSpec.cardB-relSpec.relType-relSpec.cardA  entityB  ":"  roleA
fn parse_relationship(s: &str) -> Option<ErRelationship> {
    // Split on ":" to get label
    let (lhs, role_a) = if let Some(ci) = s.find(':') {
        let label = s[ci + 1..].trim().trim_matches('"').to_string();
        (&s[..ci], label)
    } else {
        (s, String::new())
    };

    let tokens: Vec<&str> = lhs.split_whitespace().collect();
    if tokens.len() < 3 {
        return None;
    }

    let entity_a = tokens[0].to_string();
    let rel_str = tokens[1];
    let entity_b = tokens[tokens.len() - 1].to_string();

    // Parse rel_str: e.g. "||--o{" or "}o..||"
    // Format: <cardB_marker><rel_type><cardA_marker>
    let rel_spec = parse_rel_spec(rel_str)?;

    Some(ErRelationship {
        entity_a,
        role_a,
        entity_b,
        rel_spec,
    })
}

/// Parse relationship specification string like "||--o{" or "}o..||" or "o|..|{"
/// Returns RelSpec with card_b (start/entityB marker) and card_a (end/entityA marker)
fn parse_rel_spec(s: &str) -> Option<RelSpec> {
    // Find the rel_type separator: "--" (identifying) or ".." (non-identifying)
    let (card_b_str, rel_type, card_a_str) = if let Some(p) = s.find("--") {
        (&s[..p], Identification::Identifying, &s[p + 2..])
    } else if let Some(p) = s.find("..") {
        (&s[..p], Identification::NonIdentifying, &s[p + 2..])
    } else {
        return None;
    };

    let card_b = parse_cardinality_start(card_b_str)?;
    let card_a = parse_cardinality_end(card_a_str)?;

    Some(RelSpec {
        card_a,
        rel_type,
        card_b,
    })
}

/// Parse the START (entity_b side) cardinality marker
/// Markers at the start of the rel spec: }|  }o  ||  o|  |{  o{
fn parse_cardinality_start(s: &str) -> Option<Cardinality> {
    match s {
        "|o" | "o|" => Some(Cardinality::ZeroOrOne),
        "||" => Some(Cardinality::OnlyOne),
        "}o" | "o{" => Some(Cardinality::ZeroOrMore),
        "}|" | "|{" => Some(Cardinality::OneOrMore),
        "}" => Some(Cardinality::OneOrMore),
        "|" => Some(Cardinality::OnlyOne),
        _ => None,
    }
}

/// Parse the END (entity_a side) cardinality marker
/// Markers at the end of the rel spec: |{  o{  ||  o|  |}  |o
fn parse_cardinality_end(s: &str) -> Option<Cardinality> {
    match s {
        "o|" | "|o" => Some(Cardinality::ZeroOrOne),
        "||" => Some(Cardinality::OnlyOne),
        "o{" | "}o" => Some(Cardinality::ZeroOrMore),
        "|{" | "}|" => Some(Cardinality::OneOrMore),
        "{" => Some(Cardinality::OneOrMore),
        "|" => Some(Cardinality::OnlyOne),
        _ => None,
    }
}
