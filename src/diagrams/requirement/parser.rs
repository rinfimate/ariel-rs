/// Parser for Mermaid Requirement Diagram syntax.
///
/// Faithful port of requirementDb.ts.
///
/// Grammar:
///   requirementDiagram
///
///   requirement|functionalRequirement|... <name> {
///       id: <value>
///       text: <value>
///       risk: low|medium|high
///       verifymethod: analysis|demonstration|inspection|test
///   }
///
///   element <name> {
///       type: <value>
///       docref: <value>
///   }
///
///   <src> - <type> -> <dst>
///   Relationship types: contains, copies, derives, satisfies,
///                       verifies, refines, traces
///
/// Example:
///   requirementDiagram
///       requirement test_req {
///           id: 1
///           text: the test text.
///           risk: high
///           verifymethod: test
///       }
///       element test_entity {
///           type: simulation
///       }
///       test_entity - satisfies -> test_req

#[derive(Debug, Clone, PartialEq)]
pub enum RequirementType {
    Requirement,
    FunctionalRequirement,
    InterfaceRequirement,
    PerformanceRequirement,
    PhysicalRequirement,
    DesignConstraint,
}

impl RequirementType {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Requirement => "Requirement",
            Self::FunctionalRequirement => "Functional Requirement",
            Self::InterfaceRequirement => "Interface Requirement",
            Self::PerformanceRequirement => "Performance Requirement",
            Self::PhysicalRequirement => "Physical Requirement",
            Self::DesignConstraint => "Design Constraint",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Unknown,
}

impl RiskLevel {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Unknown => "",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerifyMethod {
    Analysis,
    Demonstration,
    Inspection,
    Test,
    Unknown,
}

impl VerifyMethod {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Analysis => "Analysis",
            Self::Demonstration => "Demonstration",
            Self::Inspection => "Inspection",
            Self::Test => "Test",
            Self::Unknown => "",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Requirement {
    pub name: String,
    pub req_type: RequirementType,
    pub id: String,
    pub text: String,
    pub risk: RiskLevel,
    pub verify_method: VerifyMethod,
}

#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub elem_type: String,
    pub doc_ref: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipType {
    Contains,
    Copies,
    Derives,
    Satisfies,
    Verifies,
    Refines,
    Traces,
}

impl RelationshipType {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::Copies => "copies",
            Self::Derives => "derives",
            Self::Satisfies => "satisfies",
            Self::Verifies => "verifies",
            Self::Refines => "refines",
            Self::Traces => "traces",
        }
    }

    /// Whether this relationship uses a solid line (contains) or dashed (others).
    pub fn is_contains(&self) -> bool {
        *self == RelationshipType::Contains
    }
}

#[derive(Debug, Clone)]
pub struct Relation {
    pub src: String,
    pub dst: String,
    pub rel_type: RelationshipType,
}

#[derive(Debug, Default)]
pub struct RequirementDiagram {
    pub title: Option<String>,
    pub requirements: Vec<Requirement>,
    pub elements: Vec<Element>,
    pub relations: Vec<Relation>,
}

// ── Parser ────────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<RequirementDiagram> {
    let mut diag = RequirementDiagram::default();
    let mut header_seen = false;

    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0usize;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        i += 1;

        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        if !header_seen {
            if trimmed == "requirementDiagram" || trimmed.starts_with("requirementDiagram ") {
                header_seen = true;
            }
            continue;
        }

        // Title directive
        if let Some(rest) = trimmed.strip_prefix("title") {
            if let Some(ch) = rest.chars().next() {
                if ch == ' ' || ch == '\t' {
                    diag.title = Some(rest.trim().to_string());
                }
            }
            continue;
        }

        // Requirement block
        if let Some((req_type, name)) = try_parse_requirement_header(trimmed) {
            // Parse the block body { ... }
            let body = if trimmed.ends_with('{') {
                collect_block(&lines, &mut i)
            } else {
                vec![]
            };
            let mut req = Requirement {
                name,
                req_type,
                id: String::new(),
                text: String::new(),
                risk: RiskLevel::Unknown,
                verify_method: VerifyMethod::Unknown,
            };
            for bline in body {
                parse_req_body_line(bline, &mut req);
            }
            // Upsert: if name already present, update
            if let Some(existing) = diag.requirements.iter_mut().find(|r| r.name == req.name) {
                *existing = req;
            } else {
                diag.requirements.push(req);
            }
            continue;
        }

        // Element block
        if let Some(name) = try_parse_element_header(trimmed) {
            let body = if trimmed.ends_with('{') {
                collect_block(&lines, &mut i)
            } else {
                vec![]
            };
            let mut elem = Element {
                name,
                elem_type: String::new(),
                doc_ref: String::new(),
            };
            for bline in body {
                parse_elem_body_line(bline, &mut elem);
            }
            if let Some(existing) = diag.elements.iter_mut().find(|e| e.name == elem.name) {
                *existing = elem;
            } else {
                diag.elements.push(elem);
            }
            continue;
        }

        // Relationship line: <src> - <reltype> -> <dst>
        // Also accept: <src> - <reltype> - <dst>  (bidirectional, map to same type)
        if let Some(rel) = try_parse_relation(trimmed) {
            diag.relations.push(rel);
            continue;
        }
    }

    crate::error::ParseResult::ok(diag)
}

/// Parse a requirement type keyword + name from a header line like:
///   "requirement test_req {"  or "functionalRequirement foo {"
fn try_parse_requirement_header(line: &str) -> Option<(RequirementType, String)> {
    // Check all requirement-type keywords (case-insensitive prefix matching)
    let keywords: &[(&str, RequirementType)] = &[
        (
            "functionalrequirement",
            RequirementType::FunctionalRequirement,
        ),
        (
            "interfacerequirement",
            RequirementType::InterfaceRequirement,
        ),
        (
            "performancerequirement",
            RequirementType::PerformanceRequirement,
        ),
        ("physicalrequirement", RequirementType::PhysicalRequirement),
        ("designconstraint", RequirementType::DesignConstraint),
        ("requirement", RequirementType::Requirement),
    ];

    let lower = line.to_ascii_lowercase();
    for (kw, rt) in keywords {
        if lower.starts_with(kw) {
            let after = &line[kw.len()..];
            if after.starts_with([' ', '\t']) {
                let name_part = after.trim().trim_end_matches('{').trim().to_string();
                if !name_part.is_empty() {
                    return Some((rt.clone(), name_part));
                }
            }
        }
    }
    None
}

/// Parse an element header: "element <name> {" or "element <name>"
fn try_parse_element_header(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    if lower.starts_with("element") {
        let after = &line["element".len()..];
        if after.starts_with([' ', '\t']) {
            let name = after.trim().trim_end_matches('{').trim().to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

/// Collect lines until the closing `}`, return the body lines (trimmed).
fn collect_block<'a>(lines: &[&'a str], i: &mut usize) -> Vec<&'a str> {
    let mut body = Vec::new();
    while *i < lines.len() {
        let t = lines[*i].trim();
        *i += 1;
        if t == "}" {
            break;
        }
        if !t.is_empty() && !t.starts_with("%%") {
            body.push(t);
        }
    }
    body
}

/// Parse one line inside a requirement block.
fn parse_req_body_line(line: &str, req: &mut Requirement) {
    if let Some(pos) = line.find(':') {
        let key = line[..pos].trim().to_ascii_lowercase();
        let val = line[pos + 1..].trim().to_string();
        match key.as_str() {
            "id" => req.id = val,
            "text" => req.text = val,
            "risk" => {
                req.risk = match val.to_ascii_lowercase().as_str() {
                    "low" | "low_risk" => RiskLevel::Low,
                    "med" | "medium" | "med_risk" => RiskLevel::Medium,
                    "high" | "high_risk" => RiskLevel::High,
                    _ => RiskLevel::Unknown,
                }
            }
            "verifymethod" | "verify_method" | "verifyMethod" => {
                req.verify_method = match val.to_ascii_lowercase().as_str() {
                    "analysis" | "verify_analysis" => VerifyMethod::Analysis,
                    "demonstration" | "verify_demonstration" => VerifyMethod::Demonstration,
                    "inspection" | "verify_inspection" => VerifyMethod::Inspection,
                    "test" | "verify_test" => VerifyMethod::Test,
                    _ => VerifyMethod::Unknown,
                }
            }
            _ => {}
        }
    }
}

/// Parse one line inside an element block.
fn parse_elem_body_line(line: &str, elem: &mut Element) {
    if let Some(pos) = line.find(':') {
        let key = line[..pos].trim().to_ascii_lowercase();
        let val = line[pos + 1..].trim().to_string();
        match key.as_str() {
            "type" => elem.elem_type = val,
            "docref" | "doc_ref" | "docRef" => elem.doc_ref = val,
            _ => {}
        }
    }
}

/// Try to parse a relationship line.
/// Formats:
///   src - reltype -> dst
///   src - reltype - dst
fn try_parse_relation(line: &str) -> Option<Relation> {
    // Split on " - "
    // We look for the pattern: TOKEN ' - ' RELTYPE ' -> ' TOKEN
    // or: TOKEN ' - ' RELTYPE ' - ' TOKEN
    let parts: Vec<&str> = line.split(" - ").collect();
    if parts.len() < 2 {
        return None;
    }

    let src = parts[0].trim().to_string();
    if src.is_empty() {
        return None;
    }

    // The middle+right part: "reltype -> dst" or "reltype - dst" or just "reltype -> dst"
    let rest = parts[1..].join(" - ");

    // Try " -> " separator first, then " - " again
    let (rel_str, dst) = if let Some(arrow_pos) = rest.find(" -> ") {
        let r = rest[..arrow_pos].trim();
        let d = rest[arrow_pos + 4..].trim();
        (r, d)
    } else if rest.contains(" - ") {
        // Bidirectional notation
        let mut sp = rest.splitn(2, " - ");
        let r = sp.next()?.trim();
        let d = sp.next()?.trim();
        (r, d)
    } else {
        return None;
    };

    if dst.is_empty() || rel_str.is_empty() {
        return None;
    }

    let rel_type = parse_relation_type(rel_str)?;

    Some(Relation {
        src,
        dst: dst.to_string(),
        rel_type,
    })
}

fn parse_relation_type(s: &str) -> Option<RelationshipType> {
    match s.to_ascii_lowercase().as_str() {
        "contains" => Some(RelationshipType::Contains),
        "copies" => Some(RelationshipType::Copies),
        "derives" => Some(RelationshipType::Derives),
        "satisfies" => Some(RelationshipType::Satisfies),
        "verifies" => Some(RelationshipType::Verifies),
        "refines" => Some(RelationshipType::Refines),
        "traces" => Some(RelationshipType::Traces),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "requirementDiagram\n    requirement test_req {\n        id: 1\n        text: the test text.\n        risk: high\n        verifymethod: test\n    }\n    element test_entity {\n        type: simulation\n    }\n    test_entity - satisfies -> test_req";

    #[test]
    fn parse_basic() {
        let d = parse(EXAMPLE).diagram;
        assert_eq!(d.requirements.len(), 1);
        assert_eq!(d.elements.len(), 1);
        assert_eq!(d.relations.len(), 1);

        let req = &d.requirements[0];
        assert_eq!(req.name, "test_req");
        assert_eq!(req.id, "1");
        assert_eq!(req.text, "the test text.");
        assert_eq!(req.risk, RiskLevel::High);
        assert_eq!(req.verify_method, VerifyMethod::Test);
        assert_eq!(req.req_type, RequirementType::Requirement);

        let elem = &d.elements[0];
        assert_eq!(elem.name, "test_entity");
        assert_eq!(elem.elem_type, "simulation");

        let rel = &d.relations[0];
        assert_eq!(rel.src, "test_entity");
        assert_eq!(rel.dst, "test_req");
        assert_eq!(rel.rel_type, RelationshipType::Satisfies);
    }

    #[test]
    fn parse_requirement_types() {
        let input = "requirementDiagram\n    functionalRequirement fr1 {\n        id: 2\n        text: func req\n        risk: low\n        verifymethod: analysis\n    }";
        let d = parse(input).diagram;
        assert_eq!(
            d.requirements[0].req_type,
            RequirementType::FunctionalRequirement
        );
        assert_eq!(d.requirements[0].risk, RiskLevel::Low);
        assert_eq!(d.requirements[0].verify_method, VerifyMethod::Analysis);
    }

    #[test]
    fn parse_relations() {
        let input = "requirementDiagram\n    requirement A {}\n    requirement B {}\n    A - contains -> B\n    B - verifies -> A";
        let d = parse(input).diagram;
        assert_eq!(d.relations.len(), 2);
        assert_eq!(d.relations[0].rel_type, RelationshipType::Contains);
        assert_eq!(d.relations[1].rel_type, RelationshipType::Verifies);
    }
}
