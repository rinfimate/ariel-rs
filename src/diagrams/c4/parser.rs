/// Parser for Mermaid C4 diagram syntax.
///
/// Faithful port of c4Db.ts.
///
/// Supported diagram types (first token on first non-blank line):
///   C4Context, C4Container, C4Component, C4Dynamic, C4Deployment
///
/// Supported statements:
///   title <text>
///   Person(id, label, descr?)
///   Person_Ext(id, label, descr?)
///   System(id, label, descr?)
///   System_Ext(id, label, descr?)
///   SystemDb(id, label, descr?)
///   SystemDb_Ext(id, label, descr?)
///   Container(id, label, techn?, descr?)
///   Container_Ext(id, label, techn?, descr?)
///   ContainerDb(id, label, techn?, descr?)
///   ContainerDb_Ext(id, label, techn?, descr?)
///   Component(id, label, techn?, descr?)
///   Component_Ext(id, label, techn?, descr?)
///   ComponentDb(id, label, techn?, descr?)
///   Rel(from, to, label, techn?)
///   BiRel(from, to, label, techn?)
///   Boundary(id, label) { ... }
///   System_Boundary(id, label) { ... }
///   Container_Boundary(id, label) { ... }
///   UpdateElementStyle(id, ...)
///   UpdateRelStyle(from, to, ...)
///   UpdateLayoutConfig(...)

#[derive(Debug, Clone, PartialEq)]
pub enum C4DiagramType {
    Context,
    Container,
    Component,
    Dynamic,
    Deployment,
}

#[derive(Debug, Clone, PartialEq)]
pub enum C4ElementType {
    Person,
    PersonExt,
    System,
    SystemExt,
    SystemDb,
    SystemDbExt,
    Container,
    ContainerExt,
    ContainerDb,
    ContainerDbExt,
    Component,
    ComponentExt,
    ComponentDb,
    ComponentDbExt,
    // Deployment
    Node,
    NodeExt,
}

#[derive(Debug, Clone)]
pub struct C4Element {
    pub id: String,
    pub label: String,
    pub descr: String,
    pub el_type: C4ElementType,
    pub boundary_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum C4RelType {
    Rel,
    BiRel,
    RelBack,
    RelNeighbor,
    RelBackNeighbor,
}

#[derive(Debug, Clone)]
pub struct C4Rel {
    pub from: String,
    pub to: String,
    pub label: String,
    pub techn: String,
    pub rel_type: C4RelType,
}

#[derive(Debug, Clone)]
pub struct C4Boundary {
    pub id: String,
    pub label: String,
    pub boundary_type: String,
    /// The id of the enclosing boundary, or None for top-level boundaries.
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct C4Diagram {
    pub diagram_type: Option<C4DiagramType>,
    pub title: Option<String>,
    pub elements: Vec<C4Element>,
    pub rels: Vec<C4Rel>,
    pub boundaries: Vec<C4Boundary>,
}

pub fn parse(input: &str) -> crate::error::ParseResult<C4Diagram> {
    let mut diag = C4Diagram::default();
    // Stack of (boundary_id, boundary_label) for nested boundaries
    let mut boundary_stack: Vec<String> = Vec::new();

    for raw_line in input.lines() {
        let line = strip_comment(raw_line).trim().to_string();
        if line.is_empty() {
            continue;
        }

        // Diagram type declaration
        if diag.diagram_type.is_none() {
            match line.as_str() {
                "C4Context" => {
                    diag.diagram_type = Some(C4DiagramType::Context);
                    continue;
                }
                "C4Container" => {
                    diag.diagram_type = Some(C4DiagramType::Container);
                    continue;
                }
                "C4Component" => {
                    diag.diagram_type = Some(C4DiagramType::Component);
                    continue;
                }
                "C4Dynamic" => {
                    diag.diagram_type = Some(C4DiagramType::Dynamic);
                    continue;
                }
                "C4Deployment" => {
                    diag.diagram_type = Some(C4DiagramType::Deployment);
                    continue;
                }
                _ => {}
            }
            // Check for prefixed
            if line.starts_with("C4Context")
                || line.starts_with("C4Container")
                || line.starts_with("C4Component")
                || line.starts_with("C4Dynamic")
                || line.starts_with("C4Deployment")
            {
                let first = line.split_whitespace().next().unwrap_or("");
                match first {
                    "C4Context" => diag.diagram_type = Some(C4DiagramType::Context),
                    "C4Container" => diag.diagram_type = Some(C4DiagramType::Container),
                    "C4Component" => diag.diagram_type = Some(C4DiagramType::Component),
                    "C4Dynamic" => diag.diagram_type = Some(C4DiagramType::Dynamic),
                    "C4Deployment" => diag.diagram_type = Some(C4DiagramType::Deployment),
                    _ => {}
                }
                continue;
            }
        }

        // accTitle / accDescr — skip
        if line.starts_with("accTitle") || line.starts_with("accDescr") {
            continue;
        }

        // title
        if let Some(rest) = line
            .strip_prefix("title ")
            .or_else(|| line.strip_prefix("title\t"))
        {
            diag.title = Some(rest.trim().to_string());
            continue;
        }

        // End of boundary block
        if line == "}" {
            boundary_stack.pop();
            continue;
        }

        // Boundary declarations (may open a block with `{` on same line or next)
        if let Some(bd) = try_parse_boundary(&line) {
            let parent_id = boundary_stack.last().cloned();
            diag.boundaries.push(C4Boundary {
                id: bd.0.clone(),
                label: bd.1,
                boundary_type: bd.2,
                parent_id,
            });
            // If line ends with `{`, push to stack
            if line.trim_end().ends_with('{') {
                boundary_stack.push(bd.0);
            }
            continue;
        }

        // UpdateElementStyle, UpdateRelStyle, UpdateLayoutConfig — skip
        if line.starts_with("UpdateElementStyle")
            || line.starts_with("UpdateRelStyle")
            || line.starts_with("UpdateLayoutConfig")
        {
            continue;
        }

        // Relationship
        if let Some(rel) = try_parse_rel(&line) {
            diag.rels.push(rel);
            continue;
        }

        // Element
        if let Some(el) = try_parse_element(&line, boundary_stack.last().map(|s| s.as_str())) {
            diag.elements.push(el);
            continue;
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

/// Try to parse a boundary line. Returns (id, label, type) or None.
fn try_parse_boundary(line: &str) -> Option<(String, String, String)> {
    let prefixes = [
        ("Enterprise_Boundary", "enterprise"),
        ("System_Boundary", "system"),
        ("Container_Boundary", "container"),
        ("Boundary", "boundary"),
    ];
    for (prefix, btype) in &prefixes {
        if let Some(stripped) = line.strip_prefix(prefix) {
            let rest = stripped.trim();
            let args_part = rest.trim_start_matches('(').trim_end_matches('{').trim();
            let args_part = if let Some(p) = args_part.find(')') {
                &args_part[..p]
            } else {
                args_part
            };
            let args = parse_args(args_part);
            if !args.is_empty() {
                let id = args[0].clone();
                let label = args.get(1).cloned().unwrap_or_default();
                return Some((id, label, btype.to_string()));
            }
        }
    }
    None
}

/// Try to parse a relationship line.
fn try_parse_rel(line: &str) -> Option<C4Rel> {
    let rel_prefixes: &[(&str, C4RelType)] = &[
        ("BiRel_Back", C4RelType::RelBack),
        ("BiRel_Neighbor", C4RelType::BiRel),
        ("BiRel_Back_Neighbor", C4RelType::BiRel),
        ("BiRel", C4RelType::BiRel),
        ("Rel_Back_Neighbor", C4RelType::RelBackNeighbor),
        ("Rel_Neighbor", C4RelType::RelNeighbor),
        ("Rel_Back", C4RelType::RelBack),
        ("Rel_D", C4RelType::Rel),
        ("Rel_U", C4RelType::Rel),
        ("Rel_L", C4RelType::Rel),
        ("Rel_R", C4RelType::Rel),
        ("Rel", C4RelType::Rel),
    ];
    for (prefix, rel_type) in rel_prefixes {
        if let Some(stripped) = line.strip_prefix(prefix) {
            let rest = stripped.trim();
            if !rest.starts_with('(') {
                continue;
            }
            let inner = extract_parens(rest)?;
            let args = parse_args(inner);
            if args.len() < 3 {
                return None;
            }
            return Some(C4Rel {
                from: args[0].clone(),
                to: args[1].clone(),
                label: args[2].clone(),
                techn: args.get(3).cloned().unwrap_or_default(),
                rel_type: rel_type.clone(),
            });
        }
    }
    None
}

/// Try to parse an element line.
fn try_parse_element(line: &str, boundary_id: Option<&str>) -> Option<C4Element> {
    let el_prefixes: &[(&str, C4ElementType)] = &[
        ("Person_Ext", C4ElementType::PersonExt),
        ("Person", C4ElementType::Person),
        ("SystemDb_Ext", C4ElementType::SystemDbExt),
        ("SystemDb", C4ElementType::SystemDb),
        ("System_Ext", C4ElementType::SystemExt),
        ("System", C4ElementType::System),
        ("ContainerDb_Ext", C4ElementType::ContainerDbExt),
        ("ContainerDb", C4ElementType::ContainerDb),
        ("Container_Ext", C4ElementType::ContainerExt),
        ("Container", C4ElementType::Container),
        ("ComponentDb_Ext", C4ElementType::ComponentDbExt),
        ("ComponentDb", C4ElementType::ComponentDb),
        ("Component_Ext", C4ElementType::ComponentExt),
        ("Component", C4ElementType::Component),
        ("Node_Ext", C4ElementType::NodeExt),
        ("Node", C4ElementType::Node),
    ];
    for (prefix, el_type) in el_prefixes {
        if let Some(stripped) = line.strip_prefix(prefix) {
            let rest = stripped.trim();
            if !rest.starts_with('(') {
                continue;
            }
            let inner = extract_parens(rest)?;
            let args = parse_args(inner);
            if args.is_empty() {
                return None;
            }
            return Some(C4Element {
                id: args[0].clone(),
                label: args.get(1).cloned().unwrap_or_default(),
                descr: args.get(2).cloned().unwrap_or_default(),
                el_type: el_type.clone(),
                boundary_id: boundary_id.map(|s| s.to_string()),
            });
        }
    }
    None
}

/// Extract the text inside the first parenthesis pair, handling nesting.
fn extract_parens(s: &str) -> Option<&str> {
    let start = s.find('(')?;
    let mut depth = 0i32;
    for (i, c) in s[start..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[start + 1..start + i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse comma-separated arguments, stripping surrounding quotes.
pub fn parse_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = '"';
    let mut depth = 0i32;

    for c in s.chars() {
        match c {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = c;
            }
            c if in_quote && c == quote_char => {
                in_quote = false;
            }
            '(' if !in_quote => {
                depth += 1;
                current.push(c);
            }
            ')' if !in_quote => {
                depth -= 1;
                current.push(c);
            }
            ',' if !in_quote && depth == 0 => {
                args.push(current.trim().to_string());
                current = String::new();
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.trim().is_empty() {
        args.push(current.trim().to_string());
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_context() {
        let input = "C4Context\n    title System Context diagram\n    Person(customerA, \"Banking Customer A\", \"A customer of the bank\")\n    System(SystemAA, \"Internet Banking System\", \"Allows customers to view information\")\n    Rel(customerA, SystemAA, \"Uses\")";
        let diag = parse(input).diagram;
        assert_eq!(diag.diagram_type, Some(C4DiagramType::Context));
        assert_eq!(diag.title.as_deref(), Some("System Context diagram"));
        assert_eq!(diag.elements.len(), 2);
        assert_eq!(diag.rels.len(), 1);
        assert_eq!(diag.elements[0].id, "customerA");
        assert_eq!(diag.elements[0].label, "Banking Customer A");
        assert_eq!(diag.rels[0].from, "customerA");
        assert_eq!(diag.rels[0].to, "SystemAA");
    }

    #[test]
    fn parse_args_quoted() {
        let args = parse_args(r#"id, "Label with spaces", "Desc""#);
        assert_eq!(args, vec!["id", "Label with spaces", "Desc"]);
    }
}
