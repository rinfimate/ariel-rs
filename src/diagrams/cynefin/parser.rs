// Faithful Rust port of mermaid/src/diagrams/cynefin/ parser + DB.
//
// Grammar (cynefin):
//   cynefin
//   [title <text>]
//   [accTitle: <text>]
//   [accDescr: <text>]
//
//   domain complex
//     item "label"
//   domain complicated
//     item "label"
//   domain clear
//     item "label"
//   domain chaotic
//     item "label"
//   domain confusion
//     item "label"
//
//   transition complex --> chaotic [: "label"]

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DomainName {
    Complex,
    Complicated,
    Clear,
    Chaotic,
    Confusion,
}

impl DomainName {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().trim() {
            "complex" => Some(DomainName::Complex),
            "complicated" => Some(DomainName::Complicated),
            "clear" | "obvious" | "simple" => Some(DomainName::Clear),
            "chaotic" => Some(DomainName::Chaotic),
            "confusion" | "disorder" => Some(DomainName::Confusion),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DomainName::Complex => "Complex",
            DomainName::Complicated => "Complicated",
            DomainName::Clear => "Clear",
            DomainName::Chaotic => "Chaotic",
            DomainName::Confusion => "Confusion",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CynefinItem {
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct CynefinDomain {
    pub name: DomainName,
    pub items: Vec<CynefinItem>,
}

#[derive(Debug, Clone)]
pub struct CynefinTransition {
    pub from: DomainName,
    pub to: DomainName,
    pub label: Option<String>,
}

#[derive(Debug)]
pub struct CynefinDiagram {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_description: Option<String>,
    pub domains: Vec<CynefinDomain>,
    pub transitions: Vec<CynefinTransition>,
}

// ─── Parser ───────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<CynefinDiagram> {
    let mut title: Option<String> = None;
    let mut acc_title: Option<String> = None;
    let mut acc_description: Option<String> = None;
    let mut domains: std::collections::HashMap<String, CynefinDomain> =
        std::collections::HashMap::new();
    let mut transitions: Vec<CynefinTransition> = Vec::new();
    // Order of domain declarations for deterministic output
    let mut domain_order: Vec<String> = Vec::new();

    let mut in_header = true;
    let mut current_domain: Option<String> = None;
    let mut in_acc_descr_block = false;

    for raw in input.lines() {
        let line = if let Some(p) = raw.find("%%") {
            &raw[..p]
        } else {
            raw
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if in_acc_descr_block {
                in_acc_descr_block = false;
            }
            continue;
        }

        if in_header {
            if trimmed.starts_with("cynefin") {
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
            current_domain = None;
            continue;
        }
        if trimmed == "title" {
            title = Some(String::new());
            continue;
        }

        // accTitle
        if let Some(rest) = trimmed
            .strip_prefix("accTitle:")
            .or_else(|| trimmed.strip_prefix("accTitle :"))
        {
            acc_title = Some(rest.trim().to_string());
            continue;
        }

        // accDescr (single line or multi-line block)
        if let Some(rest) = trimmed
            .strip_prefix("accDescr:")
            .or_else(|| trimmed.strip_prefix("accDescr :"))
        {
            let rest = rest.trim();
            if rest.starts_with('{') {
                in_acc_descr_block = true;
                continue;
            }
            acc_description = Some(rest.to_string());
            continue;
        }
        if in_acc_descr_block {
            if trimmed == "}" {
                in_acc_descr_block = false;
            } else {
                let prev = acc_description.get_or_insert_with(String::new);
                if !prev.is_empty() {
                    prev.push(' ');
                }
                prev.push_str(trimmed);
            }
            continue;
        }

        // domain <name>
        if let Some(rest) = trimmed
            .strip_prefix("domain ")
            .or_else(|| trimmed.strip_prefix("domain\t"))
        {
            let dn = rest.trim().to_lowercase();
            if let Some(domain_name) = DomainName::from_str(&dn) {
                let key = dn.clone();
                current_domain = Some(key.clone());
                if let std::collections::hash_map::Entry::Vacant(e) = domains.entry(key.clone()) {
                    domain_order.push(key.clone());
                    e.insert(CynefinDomain {
                        name: domain_name,
                        items: Vec::new(),
                    });
                }
            }
            continue;
        }

        // item "<label>" — within current domain
        if let Some(rest) = trimmed
            .strip_prefix("item ")
            .or_else(|| trimmed.strip_prefix("item\t"))
        {
            if let Some(key) = &current_domain {
                let label = strip_quotes(rest.trim()).to_string();
                if let Some(dom) = domains.get_mut(key) {
                    dom.items.push(CynefinItem { label });
                }
            }
            continue;
        }

        // transition <from> --> <to> [: "label"]
        if trimmed.contains("-->") {
            if let Some(trans) = parse_transition(trimmed) {
                transitions.push(trans);
            }
            continue;
        }

        // Also support: <from> -> <to>
        if trimmed.contains("->") && !trimmed.contains("-->") {
            if let Some(trans) = parse_transition_single(trimmed) {
                transitions.push(trans);
            }
            continue;
        }
    }

    // Ensure all five standard domains are present (with empty items if not declared)
    let all_domains_ordered = ["complex", "complicated", "chaotic", "clear", "confusion"];
    let mut final_domains: Vec<CynefinDomain> = Vec::new();

    // First add declared domains in order
    for key in &domain_order {
        if let Some(dom) = domains.remove(key) {
            final_domains.push(dom);
        }
    }

    // Then add any missing standard domains (empty)
    for &dn_str in &all_domains_ordered {
        let already = final_domains
            .iter()
            .any(|d| d.name == DomainName::from_str(dn_str).unwrap());
        if !already {
            if let Some(dn) = DomainName::from_str(dn_str) {
                final_domains.push(CynefinDomain {
                    name: dn,
                    items: Vec::new(),
                });
            }
        }
    }

    crate::error::ParseResult::ok(CynefinDiagram {
        title,
        acc_title,
        acc_description,
        domains: final_domains,
        transitions,
    })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn parse_transition(s: &str) -> Option<CynefinTransition> {
    let pos = s.find("-->")?;
    let from_str = s[..pos].trim();
    let after = s[pos + 3..].trim();

    // Optional label after ":"
    let (to_str, label) = if let Some(colon) = after.find(':') {
        let to = after[..colon].trim();
        let lbl = strip_quotes(after[colon + 1..].trim()).to_string();
        (to, if lbl.is_empty() { None } else { Some(lbl) })
    } else {
        (after, None)
    };

    let from = DomainName::from_str(from_str)?;
    let to = DomainName::from_str(to_str)?;
    if from == to {
        return None;
    } // filter self-loops
    Some(CynefinTransition { from, to, label })
}

fn parse_transition_single(s: &str) -> Option<CynefinTransition> {
    let pos = s.find("->")?;
    let from_str = s[..pos].trim();
    let after = s[pos + 2..].trim();
    let (to_str, label) = if let Some(colon) = after.find(':') {
        let to = after[..colon].trim();
        let lbl = strip_quotes(after[colon + 1..].trim()).to_string();
        (to, if lbl.is_empty() { None } else { Some(lbl) })
    } else {
        (after, None)
    };
    let from = DomainName::from_str(from_str)?;
    let to = DomainName::from_str(to_str)?;
    if from == to {
        return None;
    }
    Some(CynefinTransition { from, to, label })
}

fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        return &s[1..s.len() - 1];
    }
    s
}
