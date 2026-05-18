// Faithful Rust port of mermaid/src/diagrams/architecture/ parser + DB.
//
// Grammar (architecture-beta):
//   architecture-beta
//   group <id>(<icon>)[<title>] [in <parent>]
//   service <id>(<icon>)[<title>] [in <group>]
//   junction <id> [in <group>]
//   <lhsId>:<lhsDir> -- <rhsDir>:<rhsId>         (bidirectional)
//   <lhsId>:<lhsDir> --> <rhsDir>:<rhsId>         (arrow right)
//   <lhsId>:<lhsDir> <-- <rhsDir>:<rhsId>         (arrow left)

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    L,
    R,
    T,
    B,
}

impl Direction {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "L" => Some(Direction::L),
            "R" => Some(Direction::R),
            "T" => Some(Direction::T),
            "B" => Some(Direction::B),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArchService {
    pub id: String,
    pub icon: Option<String>,
    pub title: Option<String>,
    pub in_group: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArchGroup {
    pub id: String,
    pub icon: Option<String>,
    pub title: Option<String>,
    pub in_group: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArchJunction {
    pub id: String,
    pub in_group: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArchEdge {
    pub lhs_id: String,
    pub lhs_dir: Direction,
    pub lhs_into: bool, // arrow pointing into lhs
    pub rhs_id: String,
    pub rhs_dir: Direction,
    pub rhs_into: bool, // arrow pointing into rhs
    pub title: Option<String>,
}

#[derive(Debug)]
pub struct ArchDiagram {
    pub groups: Vec<ArchGroup>,
    pub services: Vec<ArchService>,
    pub junctions: Vec<ArchJunction>,
    pub edges: Vec<ArchEdge>,
}

// ─── Parser ───────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<ArchDiagram> {
    let mut groups: Vec<ArchGroup> = Vec::new();
    let mut services: Vec<ArchService> = Vec::new();
    let mut junctions: Vec<ArchJunction> = Vec::new();
    let mut edges: Vec<ArchEdge> = Vec::new();

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

        if in_header {
            if trimmed.starts_with("architecture-beta") || trimmed.starts_with("architecture") {
                in_header = false;
            }
            continue;
        }

        // title / accTitle / accDescr – skip
        if trimmed.starts_with("title")
            || trimmed.starts_with("accTitle")
            || trimmed.starts_with("accDescr")
        {
            continue;
        }

        // group <id>(<icon>)[<title>] [in <parent>]
        if let Some(rest) = trimmed.strip_prefix("group ") {
            if let Some(grp) = parse_node_decl(rest.trim()) {
                let in_group = parse_in_clause(rest.trim());
                groups.push(ArchGroup {
                    id: grp.0,
                    icon: grp.1,
                    title: grp.2,
                    in_group,
                });
            }
            continue;
        }

        // service <id>(<icon>)[<title>] [in <group>]
        if let Some(rest) = trimmed.strip_prefix("service ") {
            if let Some(svc) = parse_node_decl(rest.trim()) {
                let in_group = parse_in_clause(rest.trim());
                services.push(ArchService {
                    id: svc.0,
                    icon: svc.1,
                    title: svc.2,
                    in_group,
                });
            }
            continue;
        }

        // junction <id> [in <group>]
        if let Some(rest) = trimmed.strip_prefix("junction ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            let id = parts.first().map(|s| s.to_string()).unwrap_or_default();
            let in_group = parse_in_clause(rest.trim());
            if !id.is_empty() {
                junctions.push(ArchJunction { id, in_group });
            }
            continue;
        }

        // Edge line: contains ":L", ":R", ":T", or ":B"
        if trimmed.contains(':')
            && (trimmed.contains(" -- ") || trimmed.contains(" --> ") || trimmed.contains(" <-- "))
        {
            if let Some(edge) = parse_edge_line(trimmed) {
                edges.push(edge);
            }
            continue;
        }
    }

    crate::error::ParseResult::ok(ArchDiagram {
        groups,
        services,
        junctions,
        edges,
    })
}

// ─── Node declaration parser ──────────────────────────────────────────────────

/// Parse "id(icon)[title] [in group]" returning (id, icon, title).
fn parse_node_decl(s: &str) -> Option<(String, Option<String>, Option<String>)> {
    // id ends at '(' or '[' or whitespace
    let id_end = s
        .find(|c: char| c == '(' || c == '[' || c.is_whitespace())
        .unwrap_or(s.len());
    let id = s[..id_end].trim().to_string();
    if id.is_empty() {
        return None;
    }

    let rest = &s[id_end..];

    let icon = extract_paren(rest);
    let title = extract_bracket(rest);

    Some((id, icon, title))
}

fn extract_paren(s: &str) -> Option<String> {
    if let Some(start) = s.find('(') {
        if let Some(end) = s[start..].find(')') {
            let inner = s[start + 1..start + end].trim().to_string();
            return if inner.is_empty() { None } else { Some(inner) };
        }
    }
    None
}

fn extract_bracket(s: &str) -> Option<String> {
    if let Some(start) = s.find('[') {
        if let Some(end) = s[start..].find(']') {
            let inner = s[start + 1..start + end]
                .trim()
                .trim_matches('"')
                .to_string();
            return if inner.is_empty() { None } else { Some(inner) };
        }
    }
    None
}

/// Extract "in <id>" clause from a line.
fn parse_in_clause(s: &str) -> Option<String> {
    // Look for " in " after the main declaration
    // We need to find "in" that appears after the closing bracket/paren
    let mut depth_paren = 0i32;
    let mut depth_bracket = 0i32;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] as char {
            '(' => depth_paren += 1,
            ')' => depth_paren -= 1,
            '[' => depth_bracket += 1,
            ']' => depth_bracket -= 1,
            _ => {}
        }
        i += 1;
        // Once both brackets are closed, scan for " in "
        if depth_paren == 0 && depth_bracket == 0 && i < bytes.len() {
            let remainder = &s[i..];
            if let Some(in_pos) = remainder.find(" in ") {
                let after_in = remainder[in_pos + 4..].trim();
                // The group id is the next whitespace-delimited token
                let group_id = after_in.split_whitespace().next()?.to_string();
                return if group_id.is_empty() {
                    None
                } else {
                    Some(group_id)
                };
            }
            break;
        }
    }
    // Simpler fallback: just look for " in " anywhere after first space
    if let Some(in_pos) = s.find(" in ") {
        let after_in = s[in_pos + 4..].trim();
        let group_id = after_in.split_whitespace().next().map(|s| s.to_string());
        return group_id.filter(|s| !s.is_empty());
    }
    None
}

// ─── Edge parser ──────────────────────────────────────────────────────────────

/// Parse lines like:
///   db:L -- R:server
///   db:L --> R:server
///   db:L <-- R:server
///   db:L --[label] R:server   (with optional label between --)
fn parse_edge_line(s: &str) -> Option<ArchEdge> {
    // Detect arrow style
    let (lhs_into, rhs_into, sep) = if s.contains(" <--> ") {
        (true, true, " <--> ")
    } else if s.contains(" --> ") {
        (false, true, " --> ")
    } else if s.contains(" <-- ") {
        (true, false, " <-- ")
    } else if s.contains(" -- ") {
        (false, false, " -- ")
    } else {
        return None;
    };

    // Also handle labels: db:L --[My Label]--> R:server
    // We need to find the separator robustly
    let (lhs_part, rhs_part, label) = split_edge_parts(s, sep)?;

    // lhs_part is "id:DIR" or "id:DIR" with optional group suffix
    let (lhs_id, lhs_dir) = parse_side(lhs_part.trim())?;
    let (rhs_id, rhs_dir) = parse_side(rhs_part.trim())?;

    Some(ArchEdge {
        lhs_id,
        lhs_dir,
        lhs_into,
        rhs_id,
        rhs_dir,
        rhs_into,
        title: label,
    })
}

fn split_edge_parts<'a>(s: &'a str, sep: &str) -> Option<(&'a str, &'a str, Option<String>)> {
    // Try to find sep in s
    if let Some(pos) = s.find(sep) {
        let lhs = &s[..pos];
        let rhs = &s[pos + sep.len()..];
        // Check for label in rhs (old style: label between dashes)
        return Some((lhs, rhs, None));
    }
    // Fallback: try " -- " with possible labels  "A:L --[label]-- B:R"
    // Find first occurrence of "--"
    let dd = s.find("--")?;
    let lhs = &s[..dd];
    let after_dd = &s[dd + 2..];
    // Find label if any
    let (label, rhs) = if after_dd.starts_with('[') {
        if let Some(end) = after_dd.find(']') {
            let lbl = after_dd[1..end].trim().to_string();
            let rest = &after_dd[end + 1..];
            // Skip any more dashes
            let rest = rest.trim_start_matches('-').trim_start();
            (Some(lbl), rest)
        } else {
            (None, after_dd.trim())
        }
    } else {
        (None, after_dd.trim_start_matches('-').trim_start())
    };
    Some((lhs, rhs, label))
}

/// Parse "id:DIR" or "DIR:id"
fn parse_side(s: &str) -> Option<(String, Direction)> {
    if let Some(pos) = s.rfind(':') {
        let left = s[..pos].trim();
        let right = s[pos + 1..].trim();
        // Right might be a direction
        if let Some(dir) = Direction::from_str(right) {
            return Some((left.to_string(), dir));
        }
        // Left might be a direction
        if let Some(dir) = Direction::from_str(left) {
            return Some((right.to_string(), dir));
        }
    }
    // Try first token as direction
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() == 2 {
        if let Some(dir) = Direction::from_str(parts[0]) {
            return Some((parts[1].trim().to_string(), dir));
        }
        if let Some(dir) = Direction::from_str(parts[1]) {
            return Some((parts[0].trim().to_string(), dir));
        }
    }
    None
}
