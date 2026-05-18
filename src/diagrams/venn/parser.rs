// Faithful Rust port of mermaid/src/diagrams/venn/parser/venn.jison + vennDB.ts
//
// Grammar (venn / vennDiagram):
//   venn-beta | vennDiagram
//   [title <text>]
//   set  <id> ["label"] [: <size>]
//   union <id1>, <id2>[, ...] ["label"] [: <size>]
//   text  <sets...> <id> ["label"]
//   style <sets...> property: value[, property: value]*

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct VennSet {
    /// Single identifier — e.g. "A"
    pub id: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VennIntersection {
    /// Two-or-more identifiers that form the intersection key, sorted.
    pub sets: Vec<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VennTextNode {
    pub sets: Vec<String>, // sorted set identifiers this text belongs to
    pub id: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VennStyleEntry {
    pub targets: Vec<String>, // sorted set identifiers
    pub styles: HashMap<String, String>,
}

#[derive(Debug)]
pub struct VennDiagram {
    pub title: Option<String>,
    pub sets: Vec<VennSet>,
    pub intersections: Vec<VennIntersection>,
    pub text_nodes: Vec<VennTextNode>,
    pub style_entries: Vec<VennStyleEntry>,
}

// ─── Parser ───────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<VennDiagram> {
    let mut title: Option<String> = None;
    let mut sets: Vec<VennSet> = Vec::new();
    let mut intersections: Vec<VennIntersection> = Vec::new();
    let mut text_nodes: Vec<VennTextNode> = Vec::new();
    let mut style_entries: Vec<VennStyleEntry> = Vec::new();
    let mut known_sets: HashSet<String> = HashSet::new();

    let mut in_header = true;

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

        // Header
        if in_header {
            if trimmed.starts_with("venn") {
                in_header = false;
            }
            continue;
        }

        // title
        if let Some(rest) = trimmed
            .strip_prefix("title ")
            .or_else(|| trimmed.strip_prefix("title\t"))
        {
            title = Some(normalize_text(rest.trim()));
            continue;
        }
        if trimmed == "title" {
            title = Some(String::new());
            continue;
        }

        // accTitle / accDescr – skip
        if trimmed.starts_with("accTitle") || trimmed.starts_with("accDescr") {
            continue;
        }

        // set id ["label"] [: size]
        if let Some(rest) = trimmed
            .strip_prefix("set ")
            .or_else(|| trimmed.strip_prefix("set\t"))
        {
            let (id, label, _size) = parse_set_line(rest.trim());
            let norm_id = normalize_text(&id);
            known_sets.insert(norm_id.clone());
            sets.push(VennSet {
                id: norm_id,
                label: label.map(|l| normalize_text(&l)),
            });
            continue;
        }

        // union id1, id2[, ...] ["label"] [: size]
        if let Some(rest) = trimmed
            .strip_prefix("union ")
            .or_else(|| trimmed.strip_prefix("union\t"))
        {
            let (ids, label, _size) = parse_union_line(rest.trim());
            let mut norm_ids: Vec<String> = ids.iter().map(|id| normalize_text(id)).collect();
            norm_ids.sort();
            intersections.push(VennIntersection {
                sets: norm_ids,
                label: label.map(|l| normalize_text(&l)),
            });
            continue;
        }

        // text sets... id ["label"]
        if let Some(rest) = trimmed
            .strip_prefix("text ")
            .or_else(|| trimmed.strip_prefix("text\t"))
        {
            if let Some(tn) = parse_text_line(rest.trim()) {
                text_nodes.push(tn);
            }
            continue;
        }

        // style sets... property: value[, ...]
        if let Some(rest) = trimmed
            .strip_prefix("style ")
            .or_else(|| trimmed.strip_prefix("style\t"))
        {
            if let Some(se) = parse_style_line(rest.trim()) {
                style_entries.push(se);
            }
            continue;
        }
    }

    crate::error::ParseResult::ok(VennDiagram {
        title,
        sets,
        intersections,
        text_nodes,
        style_entries,
    })
}

// ─── Line parsers ─────────────────────────────────────────────────────────────

/// Parse "id ["label"] [: size]"
fn parse_set_line(s: &str) -> (String, Option<String>, Option<f64>) {
    parse_id_label_size(s)
}

/// Parse "id1, id2[, ...] ["label"] [: size]"
/// Returns (ids, label, size)
fn parse_union_line(s: &str) -> (Vec<String>, Option<String>, Option<f64>) {
    // Identifiers are comma-separated, terminated by an optional bracket label or colon+number
    // Strategy: split on commas, last item may contain "[label]" or ": size"

    // Find where the bracket label or colon-size starts after the identifiers
    // We look for the first '[' or ':' that is NOT part of an identifier
    let (ids_part, rest) = split_ids_from_rest(s);

    let ids: Vec<String> = ids_part
        .split(',')
        .map(|p| p.trim().trim_matches('"').to_string())
        .filter(|p| !p.is_empty())
        .collect();

    let (label, size) = parse_label_size(rest.trim());
    (ids, label, size)
}

/// Split "id1, id2 [label] : size" into ("id1, id2", "[label] : size")
/// Identifiers are alphanumeric + underscores. Stop at '[' or at ':' not inside an identifier.
fn split_ids_from_rest(s: &str) -> (&str, &str) {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '[' || c == '"' {
            // Start of label
            return (&s[..i], &s[i..]);
        }
        if c == ':' {
            // Check next char — if it's a digit/space it's a size separator
            // But it might also be part of the identifier list (not typical)
            return (&s[..i], &s[i..]);
        }
        i += 1;
    }
    (s, "")
}

/// Parse "id ["label"] [: size]" returning (id, label, size)
fn parse_id_label_size(s: &str) -> (String, Option<String>, Option<f64>) {
    // id is first token (space or bracket terminates it)
    let id_end = s
        .find(|c: char| c.is_whitespace() || c == '[' || c == ':')
        .unwrap_or(s.len());
    let id = s[..id_end].trim().trim_matches('"').to_string();
    let rest = s[id_end..].trim();
    let (label, size) = parse_label_size(rest);
    (id, label, size)
}

/// Parse `["label"] [: size]` from a string that already has the id consumed.
fn parse_label_size(s: &str) -> (Option<String>, Option<f64>) {
    let mut label: Option<String> = None;
    let mut size: Option<f64> = None;
    let mut rest = s;

    // Optional bracket label
    if rest.starts_with('[') || rest.starts_with('"') {
        if let Some(label_str) = extract_bracket_label(&mut rest) {
            label = Some(normalize_text(&label_str));
        }
        rest = rest.trim();
    }

    // Optional ": size"
    if let Some(after_colon) = rest.strip_prefix(':') {
        size = after_colon.trim().parse::<f64>().ok();
    }

    (label, size)
}

/// Extract a `["label"]` or `[label]` or `"label"` from the start of *s,
/// advancing s past it.
fn extract_bracket_label(s: &mut &str) -> Option<String> {
    let t = s.trim_start();
    if t.starts_with('[') {
        if let Some(end) = t.find(']') {
            let inner = t[1..end].trim().trim_matches('"').to_string();
            *s = &t[end + 1..];
            return if inner.is_empty() { None } else { Some(inner) };
        }
    } else if let Some(inner_start) = t.strip_prefix('"') {
        if let Some(end) = inner_start.find('"') {
            let inner = inner_start[..end].to_string();
            *s = &inner_start[end + 1..];
            return if inner.is_empty() { None } else { Some(inner) };
        }
    }
    None
}

/// Parse "id1, id2, ... text_id ["label"]"
fn parse_text_line(s: &str) -> Option<VennTextNode> {
    // The text line format from the jison grammar:
    //   TEXT identifierList id [bracketLabel]
    // where identifierList is comma-separated set IDs.
    // We split on commas for the set list; the last token before any '[' is the text node id.

    // Find the bracket label first
    let (body, bracket_label) = if let Some(bi) = s.find('[') {
        let lbl = extract_bracket_label_from_str(&s[bi..]);
        (&s[..bi], lbl)
    } else {
        (s, None)
    };

    let tokens: Vec<&str> = body
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect();
    if tokens.is_empty() {
        return None;
    }

    // Last token is the text node id; rest are set identifiers
    let (sets_tokens, id_token) = if tokens.len() == 1 {
        (tokens.as_slice(), tokens[0])
    } else {
        (&tokens[..tokens.len() - 1], tokens[tokens.len() - 1])
    };

    // The id_token itself might be space-separated: "setA setB textId"
    // Try to split it further
    let id_parts: Vec<&str> = id_token.split_whitespace().collect();
    let (extra_sets, actual_id) = if id_parts.len() > 1 {
        (
            &id_parts[..id_parts.len() - 1],
            id_parts[id_parts.len() - 1],
        )
    } else {
        (&id_parts[..0], id_parts[0])
    };

    let mut sets: Vec<String> = sets_tokens
        .iter()
        .flat_map(|t| t.split_whitespace())
        .map(normalize_text)
        .collect();
    for es in extra_sets {
        sets.push(normalize_text(es));
    }
    sets.sort();

    let id = normalize_text(actual_id);
    let label = bracket_label.map(|l| normalize_text(&l));

    Some(VennTextNode { sets, id, label })
}

fn extract_bracket_label_from_str(s: &str) -> Option<String> {
    let t = s.trim();
    if t.starts_with('[') {
        if let Some(end) = t.find(']') {
            let inner = t[1..end].trim().trim_matches('"').to_string();
            return if inner.is_empty() { None } else { Some(inner) };
        }
    }
    None
}

/// Parse "id1, id2, ... prop: value[, prop: value]*"
fn parse_style_line(s: &str) -> Option<VennStyleEntry> {
    // Strategy: the style properties begin at the first "word: value" pair.
    // Identifiers come before them separated by commas.
    // A CSS property key does not contain spaces; it does contain alphanumerics and hyphens.
    // We detect the boundary where a token followed by ':' is a style property key.

    // Split on commas first, then identify which tokens are CSS properties
    let parts: Vec<&str> = s.split(',').map(str::trim).collect();
    let mut set_ids: Vec<String> = Vec::new();
    let mut styles: HashMap<String, String> = HashMap::new();
    let mut in_styles = false;

    let mut i = 0;
    while i < parts.len() {
        let part = parts[i];
        if !in_styles {
            // Check if this part contains a CSS property (key: value)
            if let Some(cp) = part.find(':') {
                let key = part[..cp].trim();
                // CSS property key: no spaces, only alnum and hyphens
                let is_css_key = key.chars().all(|c| c.is_alphanumeric() || c == '-')
                    && !key.is_empty()
                    && !key.contains(' ');
                if is_css_key && !set_ids.is_empty() {
                    in_styles = true;
                    let val = normalize_style_val(part[cp + 1..].trim());
                    styles.insert(key.to_string(), val);
                    i += 1;
                    continue;
                } else {
                    // It's a set id that happens to look like "A: size" — treat as set ID only
                    set_ids.push(normalize_text(
                        part.split(':').next().unwrap_or(part).trim(),
                    ));
                }
            } else {
                set_ids.push(normalize_text(part));
            }
        } else {
            // We're in the styles section
            if let Some(cp) = part.find(':') {
                let key = part[..cp].trim().to_string();
                let val = normalize_style_val(part[cp + 1..].trim());
                styles.insert(key, val);
            }
        }
        i += 1;
    }

    set_ids.sort();
    let targets: Vec<String> = set_ids.into_iter().filter(|s| !s.is_empty()).collect();
    if targets.is_empty() {
        return None;
    }

    Some(VennStyleEntry { targets, styles })
}

fn normalize_text(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalize_style_val(s: &str) -> String {
    normalize_text(s)
}
