/// Parser for Mermaid Wardley map diagram syntax.
///
/// Faithful port of wardleyParser.ts / wardleyDb.ts populateDb().
///
/// Grammar (subset — full grammar uses PEG):
///   wardley
///   title <text>
///   component <name> [<visibility>, <evolution>]
///   anchor <name> [<visibility>, <evolution>]
///   note <name> [<visibility>, <evolution>]
///   <name>-><name> [<label>]
///   <name>--><name> [<label>]  (dashed)
///   pipeline <name>
///   evolution <genesis>+<custom>+<product>+<commodity>
///
/// Coordinates: visibility is Y (0=invisible, 1=visible), evolution is X (0=genesis, 1=commodity)
/// Both can be 0-1 decimal or 0-100 percentage.
/// Node/component types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WardleyNodeKind {
    Component,
    Anchor,
    Note,
}

/// Sourcing strategy overlay
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sourcing {
    Build,
    Buy,
    Outsource,
    Market,
    None,
}

#[derive(Debug, Clone)]
pub struct WardleyNode {
    pub id: String,
    pub label: String,
    pub visibility: f64, // Y axis, 0–1
    pub evolution: f64,  // X axis, 0–1
    pub kind: WardleyNodeKind,
    pub sourcing: Sourcing,
    pub inertia: bool,
}

#[derive(Debug, Clone)]
pub struct WardleyLink {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub dashed: bool,
}

#[derive(Debug, Clone)]
pub struct WardleyAnnotation {
    pub number: u32,
    pub visibility: f64,
    pub evolution: f64,
}

#[derive(Debug, Clone)]
pub struct EvolutionStages {
    pub stages: Vec<(String, f64)>, // (label, boundary 0-1)
}

impl Default for EvolutionStages {
    fn default() -> Self {
        // Mermaid defaults: Genesis | Custom | Product | Commodity
        EvolutionStages {
            stages: vec![
                ("Genesis".into(), 0.0),
                ("Custom Built".into(), 0.25),
                ("Product".into(), 0.5),
                ("Commodity".into(), 0.75),
            ],
        }
    }
}

pub struct WardleyDiagram {
    pub title: Option<String>,
    pub nodes: Vec<WardleyNode>,
    pub links: Vec<WardleyLink>,
    pub annotations: Vec<WardleyAnnotation>,
    pub evolution: EvolutionStages,
    /// Canvas size hint (width_pct, height_pct) — not always present
    pub width: f64,
    pub height: f64,
}

impl Default for WardleyDiagram {
    fn default() -> Self {
        WardleyDiagram {
            title: None,
            nodes: Vec::new(),
            links: Vec::new(),
            annotations: Vec::new(),
            evolution: EvolutionStages::default(),
            width: 100.0,
            height: 100.0,
        }
    }
}

/// Convert a raw coordinate (0-1 or 0-100) to 0-1 range.
fn to_pct(v: f64) -> f64 {
    if v > 1.0 {
        v / 100.0
    } else {
        v
    }
}

pub fn parse(input: &str) -> crate::error::ParseResult<WardleyDiagram> {
    let mut diag = WardleyDiagram::default();
    let mut header_seen = false;
    let mut annotation_counter: u32 = 0;

    for raw_line in input.lines() {
        let trimmed = raw_line.trim();

        if trimmed.is_empty() || trimmed.starts_with("%%") || trimmed.starts_with("//") {
            continue;
        }

        if !header_seen {
            let lower = trimmed.to_lowercase();
            if lower == "wardley" || lower.starts_with("wardley ") {
                header_seen = true;
            }
            continue;
        }

        // title
        if let Some(rest) = trimmed.strip_prefix("title ").or_else(|| {
            if trimmed.to_lowercase().starts_with("title ") {
                Some(&trimmed[6..])
            } else {
                None
            }
        }) {
            diag.title = Some(rest.trim().to_string());
            continue;
        }

        // evolution <label>+<label>+...
        if trimmed.to_lowercase().starts_with("evolution ") {
            let rest = &trimmed[10..].trim();
            let parts: Vec<&str> = rest.split('+').collect();
            if !parts.is_empty() {
                diag.evolution.stages.clear();
                let step = 1.0 / parts.len() as f64;
                for (i, part) in parts.iter().enumerate() {
                    diag.evolution
                        .stages
                        .push((part.trim().to_string(), i as f64 * step));
                }
            }
            continue;
        }

        // size <width> <height>
        if trimmed.to_lowercase().starts_with("size ") {
            let rest = &trimmed[5..].trim();
            let parts: Vec<f64> = rest
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if parts.len() >= 2 {
                diag.width = parts[0];
                diag.height = parts[1];
            }
            continue;
        }

        // pipeline <name> — skip (not rendered)
        if trimmed.to_lowercase().starts_with("pipeline ") {
            continue;
        }

        // annotation <number> [vis, evo] <text>
        if trimmed.to_lowercase().starts_with("annotation ") {
            let rest = &trimmed[11..].trim();
            if let Some(ann) = parse_annotation(rest, &mut annotation_counter) {
                diag.annotations.push(ann);
            }
            continue;
        }

        // annotations (box position line — usually "annotations [vis, evo]")
        if trimmed.to_lowercase().starts_with("annotations ") {
            continue; // skip box positioning for now
        }

        // trend / accelerator / deaccelerator (skip)
        if trimmed.to_lowercase().starts_with("trend ")
            || trimmed.to_lowercase().starts_with("accelerator ")
            || trimmed.to_lowercase().starts_with("deaccelerator ")
        {
            continue;
        }

        // Links: name->name or name-->name
        if let Some(link) = parse_link(trimmed) {
            diag.links.push(link);
            continue;
        }

        // component, anchor, note
        let lower = trimmed.to_lowercase();
        let (kind, rest) = if lower.starts_with("component ") {
            (WardleyNodeKind::Component, &trimmed[10..])
        } else if lower.starts_with("anchor ") {
            (WardleyNodeKind::Anchor, &trimmed[7..])
        } else if lower.starts_with("note ") {
            (WardleyNodeKind::Note, &trimmed[5..])
        } else {
            continue;
        };

        if let Some(node) = parse_node(rest.trim(), kind) {
            diag.nodes.push(node);
        }
    }

    crate::error::ParseResult::ok(diag)
}

/// Parse "Name [visibility, evolution]" with optional sourcing/inertia suffixes.
fn parse_node(rest: &str, kind: WardleyNodeKind) -> Option<WardleyNode> {
    // Extract label (possibly quoted)
    let (label, remainder) = if let Some(stripped) = rest.strip_prefix('"') {
        let end = stripped.find('"')?;
        (stripped[..end].to_string(), stripped[end + 1..].trim())
    } else {
        let bracket = rest.find('[').unwrap_or(rest.len());
        (rest[..bracket].trim().to_string(), rest[bracket..].trim())
    };

    // Parse coordinates [vis, evo]
    let (vis, evo, sourcing, inertia) = if let Some(bracket_content) = parse_brackets(remainder) {
        let mut parts = bracket_content.splitn(10, ',');
        let vis_raw: f64 = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0.5);
        let evo_raw: f64 = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0.5);
        let vis = to_pct(vis_raw);
        let evo = to_pct(evo_raw);

        // Additional parts: sourcing, inertia
        let mut sourcing = Sourcing::None;
        let mut inertia = false;
        for part in parts {
            let p = part.trim().to_lowercase();
            match p.as_str() {
                "build" => sourcing = Sourcing::Build,
                "buy" => sourcing = Sourcing::Buy,
                "outsource" | "outsourced" => sourcing = Sourcing::Outsource,
                "market" => sourcing = Sourcing::Market,
                "inertia" => inertia = true,
                _ => {}
            }
        }
        (vis, evo, sourcing, inertia)
    } else {
        (0.5, 0.5, Sourcing::None, false)
    };

    let id = label.clone();
    Some(WardleyNode {
        id,
        label,
        visibility: vis,
        evolution: evo,
        kind,
        sourcing,
        inertia,
    })
}

fn parse_brackets(s: &str) -> Option<String> {
    let start = s.find('[')?;
    let end = s[start..].find(']')?;
    Some(s[start + 1..start + end].to_string())
}

fn parse_link(s: &str) -> Option<WardleyLink> {
    // Detect "-->" (dashed) or "->" (solid)
    let (dashed, sep) = if s.contains("-->") {
        (true, "-->")
    } else if s.contains("->") {
        (false, "->")
    } else {
        return None;
    };

    let parts: Vec<&str> = s.splitn(2, sep).collect();
    if parts.len() != 2 {
        return None;
    }

    let from = parts[0].trim().to_string();
    let rest = parts[1].trim();

    // The rest may be: "ToName" or "ToName: label" or "ToName Label"
    let (to, label) = if let Some(colon) = rest.find(':') {
        (
            rest[..colon].trim().to_string(),
            Some(rest[colon + 1..].trim().to_string()),
        )
    } else if let Some(bracket) = rest.find('[') {
        (
            rest[..bracket].trim().to_string(),
            parse_brackets(&rest[bracket..]).map(|s| s.trim().to_string()),
        )
    } else {
        (rest.to_string(), None)
    };

    if from.is_empty() || to.is_empty() {
        return None;
    }

    Some(WardleyLink {
        from,
        to,
        label,
        dashed,
    })
}

fn parse_annotation(rest: &str, counter: &mut u32) -> Option<WardleyAnnotation> {
    *counter += 1;
    let number = *counter;

    // Format: "N [vis, evo] text" or "[vis, evo] text"
    let rest = rest.trim();

    // Try to extract number at start
    let rest = if let Some(space) = rest.find(' ') {
        let maybe_num = &rest[..space];
        if maybe_num.parse::<u32>().is_ok() {
            rest[space..].trim()
        } else {
            rest
        }
    } else {
        rest
    };

    // Extract [vis, evo]
    let (vis, evo, text_part) = if let Some(bstart) = rest.find('[') {
        if let Some(bend) = rest[bstart..].find(']') {
            let coords = &rest[bstart + 1..bstart + bend];
            let mut parts = coords.splitn(2, ',');
            let vis: f64 = parts
                .next()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0.5);
            let evo: f64 = parts
                .next()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0.5);
            let text_part = rest[bstart + bend + 1..].trim().to_string();
            (to_pct(vis), to_pct(evo), text_part)
        } else {
            (0.5, 0.5, rest.to_string())
        }
    } else {
        (0.5, 0.5, rest.to_string())
    };

    let _ = text_part; // not used by renderer
    Some(WardleyAnnotation {
        number,
        visibility: vis,
        evolution: evo,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_wardley() {
        let input = "wardley\n    title My Map\n    component UserNeed [0.9, 0.1]\n    component Backend [0.5, 0.7]\n    UserNeed->Backend\n";
        let d = parse(input).diagram;
        assert_eq!(d.title.as_deref(), Some("My Map"));
        assert_eq!(d.nodes.len(), 2);
        assert!((d.nodes[0].visibility - 0.9).abs() < 0.01);
        assert!((d.nodes[0].evolution - 0.1).abs() < 0.01);
        assert_eq!(d.links.len(), 1);
        assert_eq!(d.links[0].from, "UserNeed");
        assert_eq!(d.links[0].to, "Backend");
    }

    #[test]
    fn dashed_link() {
        let input = "wardley\n    component A [0.5, 0.5]\n    component B [0.3, 0.8]\n    A-->B\n";
        let d = parse(input).diagram;
        assert!(d.links[0].dashed);
    }
}
