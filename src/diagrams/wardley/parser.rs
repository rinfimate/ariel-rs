//! Parser for Mermaid Wardley map diagram syntax.
//!
//! Faithful port of wardleyParser.ts / wardleyDb.ts populateDb().
//!
//! Grammar (subset — full grammar uses PEG):
//!   wardley[-beta]
//!   title <text>
//!   component <name> [<visibility>, <evolution>] [label [dx, dy]]
//!   anchor <name> [<visibility>, <evolution>]
//!   note <name> [<visibility>, <evolution>]
//!   <name> -> <name> [<label>]
//!   <name> --> <name> [<label>]  (dashed)
//!   evolve <name> <target_evolution>
//!   pipeline <name>
//!   evolution <genesis>+<custom>+<product>+<commodity>
//!
//! Coordinates: visibility is Y (0=invisible, 1=visible), evolution is X (0=genesis, 1=commodity)
//! Both can be 0-1 decimal or 0-100 percentage.

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
    /// Visibility in 0-100 range (Y axis: 100=visible, 0=invisible)
    pub visibility: f64,
    /// Evolution in 0-100 range (X axis: 0=genesis, 100=commodity)
    pub evolution: f64,
    pub kind: WardleyNodeKind,
    pub sourcing: Sourcing,
    pub inertia: bool,
    /// Custom label x-offset in pixels (None = use default nodeLabelOffset)
    pub label_offset_x: Option<f64>,
    /// Custom label y-offset in pixels (None = use default -nodeLabelOffset)
    pub label_offset_y: Option<f64>,
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
    /// Visibility in 0-100 range
    pub visibility: f64,
    /// Evolution in 0-100 range
    pub evolution: f64,
}

/// An "evolve" trend: component moves toward a new evolution position.
#[derive(Debug, Clone)]
pub struct WardleyTrend {
    /// The node ID/label being evolved
    pub node_id: String,
    /// Target evolution in 0-100 range
    pub target_x: f64,
    /// Y position (same as source node) in 0-100 range
    pub target_y: f64,
}

#[derive(Debug, Clone)]
pub struct EvolutionStages {
    /// Each entry: (label, start_boundary 0-100)
    pub stages: Vec<(String, f64)>,
}

impl Default for EvolutionStages {
    fn default() -> Self {
        // Mermaid defaults: Genesis | Custom Built | Product | Commodity
        // Each stage occupies 25% of the evolution axis
        EvolutionStages {
            stages: vec![
                ("Genesis".into(), 0.0),
                ("Custom Built".into(), 25.0),
                ("Product".into(), 50.0),
                ("Commodity".into(), 75.0),
            ],
        }
    }
}

pub struct WardleyDiagram {
    pub title: Option<String>,
    pub nodes: Vec<WardleyNode>,
    pub links: Vec<WardleyLink>,
    pub annotations: Vec<WardleyAnnotation>,
    pub trends: Vec<WardleyTrend>,
    pub evolution: EvolutionStages,
    /// Canvas width hint (default 900)
    pub width: f64,
    /// Canvas height hint (default 600)
    pub height: f64,
}

impl Default for WardleyDiagram {
    fn default() -> Self {
        WardleyDiagram {
            title: None,
            nodes: Vec::new(),
            links: Vec::new(),
            annotations: Vec::new(),
            trends: Vec::new(),
            evolution: EvolutionStages::default(),
            width: 900.0,
            height: 600.0,
        }
    }
}

/// Convert a raw coordinate (0-1 or 0-100) to 0-100 range.
fn to_percent(v: f64) -> f64 {
    if v <= 1.0 {
        v * 100.0
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
            if lower == "wardley"
                || lower.starts_with("wardley ")
                || lower == "wardley-beta"
                || lower.starts_with("wardley-beta ")
            {
                header_seen = true;
            }
            continue;
        }

        // accTitle / accDescr — skip
        if trimmed.starts_with("accTitle") || trimmed.starts_with("accDescr") {
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
            let rest = trimmed[10..].trim();
            let parts: Vec<&str> = rest.split('+').collect();
            if !parts.is_empty() {
                diag.evolution.stages.clear();
                let step = 100.0 / parts.len() as f64;
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
            let rest = trimmed[5..].trim();
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

        // pipeline <name> — skip
        if trimmed.to_lowercase().starts_with("pipeline ") {
            continue;
        }

        // annotation <number> [vis, evo] <text>
        if trimmed.to_lowercase().starts_with("annotation ") {
            let rest = trimmed[11..].trim();
            if let Some(ann) = parse_annotation(rest, &mut annotation_counter) {
                diag.annotations.push(ann);
            }
            continue;
        }

        // annotations (box position line — skip)
        if trimmed.to_lowercase().starts_with("annotations ") {
            continue;
        }

        // evolve <name> <target_evolution>
        if trimmed.to_lowercase().starts_with("evolve ") {
            let rest = trimmed[7..].trim();
            if let Some(trend) = parse_evolve(rest, &diag.nodes) {
                diag.trends.push(trend);
            }
            continue;
        }

        // trend / accelerator / deaccelerator (skip)
        if trimmed.to_lowercase().starts_with("trend ")
            || trimmed.to_lowercase().starts_with("accelerator ")
            || trimmed.to_lowercase().starts_with("deaccelerator ")
        {
            continue;
        }

        // Links: name -> name or name --> name (with or without spaces around arrows)
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

/// Parse "Name [visibility, evolution]" with optional "label [dx, dy]" suffix.
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
    let (vis, evo, sourcing, inertia, label_offset_x, label_offset_y) =
        if let Some(bracket_content) = parse_first_brackets(remainder) {
            let after_brackets = skip_first_brackets(remainder);
            let mut parts = bracket_content.splitn(10, ',');
            let vis_raw: f64 = parts
                .next()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(50.0);
            let evo_raw: f64 = parts
                .next()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(50.0);
            let vis = to_percent(vis_raw);
            let evo = to_percent(evo_raw);

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

            // Parse "label [dx, dy]" from remaining text after the coordinate brackets
            let (lox, loy) = parse_label_offset(after_brackets.trim());

            (vis, evo, sourcing, inertia, lox, loy)
        } else {
            (50.0, 50.0, Sourcing::None, false, None, None)
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
        label_offset_x,
        label_offset_y,
    })
}

/// Parse "label [dx, dy]" returning (Some(dx), Some(dy)) or (None, None).
fn parse_label_offset(s: &str) -> (Option<f64>, Option<f64>) {
    // Look for "label [dx, dy]"
    let lower = s.to_lowercase();
    if let Some(pos) = lower.find("label") {
        let after = s[pos + 5..].trim();
        if let Some(content) = parse_first_brackets(after) {
            let mut parts = content.splitn(2, ',');
            let dx: Option<f64> = parts.next().and_then(|s| s.trim().parse().ok());
            let dy: Option<f64> = parts.next().and_then(|s| s.trim().parse().ok());
            return (dx, dy);
        }
    }
    (None, None)
}

/// Parse the content of the first `[...]` bracket pair.
fn parse_first_brackets(s: &str) -> Option<String> {
    let start = s.find('[')?;
    let end = s[start..].find(']')?;
    Some(s[start + 1..start + end].to_string())
}

/// Return the string after the first `[...]` bracket pair.
fn skip_first_brackets(s: &str) -> &str {
    if let Some(start) = s.find('[') {
        if let Some(end) = s[start..].find(']') {
            return &s[start + end + 1..];
        }
    }
    s
}

fn parse_link(s: &str) -> Option<WardleyLink> {
    // Normalize: collapse spaces around arrows
    // Support: "A -> B", "A --> B", "A->B", "A-->B"
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

    // The rest may be: "ToName" or "ToName: label" or "ToName [label]"
    let (to, label) = if let Some(colon) = rest.find(':') {
        (
            rest[..colon].trim().to_string(),
            Some(rest[colon + 1..].trim().to_string()),
        )
    } else if let Some(bracket) = rest.find('[') {
        (
            rest[..bracket].trim().to_string(),
            parse_first_brackets(&rest[bracket..]).map(|s| s.trim().to_string()),
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

/// Parse "evolve <component_name> <target_evolution>".
fn parse_evolve(rest: &str, nodes: &[WardleyNode]) -> Option<WardleyTrend> {
    // rest is e.g. "Kettle 0.62" or "Power 0.89"
    // Find the last whitespace-separated token as the number
    let last_space = rest.rfind(' ')?;
    let node_id = rest[..last_space].trim().to_string();
    let target_raw: f64 = rest[last_space..].trim().parse().ok()?;
    let target_x = to_percent(target_raw);

    // Look up the node's visibility (y coordinate)
    let target_y = nodes
        .iter()
        .find(|n| n.id == node_id || n.label == node_id)
        .map(|n| n.visibility)
        .unwrap_or(50.0);

    Some(WardleyTrend {
        node_id,
        target_x,
        target_y,
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
    let (vis, evo) = if let Some(bstart) = rest.find('[') {
        if let Some(bend) = rest[bstart..].find(']') {
            let coords = &rest[bstart + 1..bstart + bend];
            let mut parts = coords.splitn(2, ',');
            let vis: f64 = parts
                .next()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(50.0);
            let evo: f64 = parts
                .next()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(50.0);
            (to_percent(vis), to_percent(evo))
        } else {
            (50.0, 50.0)
        }
    } else {
        (50.0, 50.0)
    };

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
        // Stored as 0-100 range
        assert!((d.nodes[0].visibility - 90.0).abs() < 0.01);
        assert!((d.nodes[0].evolution - 10.0).abs() < 0.01);
        assert_eq!(d.links.len(), 1);
        assert_eq!(d.links[0].from, "UserNeed");
        assert_eq!(d.links[0].to, "Backend");
    }

    #[test]
    fn wardley_beta_header() {
        let input = "wardley-beta\ntitle Tea Shop\ncomponent Cup [0.73, 0.78]\n";
        let d = parse(input).diagram;
        assert_eq!(d.title.as_deref(), Some("Tea Shop"));
        assert_eq!(d.nodes.len(), 1);
    }

    #[test]
    fn dashed_link() {
        let input = "wardley\n    component A [0.5, 0.5]\n    component B [0.3, 0.8]\n    A-->B\n";
        let d = parse(input).diagram;
        assert!(d.links[0].dashed);
    }

    #[test]
    fn link_with_spaces() {
        let input = "wardley\ncomponent A [0.5, 0.5]\ncomponent B [0.3, 0.8]\nA -> B\n";
        let d = parse(input).diagram;
        assert_eq!(d.links.len(), 1);
        assert_eq!(d.links[0].from, "A");
        assert_eq!(d.links[0].to, "B");
    }

    #[test]
    fn evolve_keyword() {
        let input = "wardley\ncomponent Kettle [0.43, 0.35]\nevolve Kettle 0.62\n";
        let d = parse(input).diagram;
        assert_eq!(d.trends.len(), 1);
        assert_eq!(d.trends[0].node_id, "Kettle");
        assert!((d.trends[0].target_x - 62.0).abs() < 0.01);
    }

    #[test]
    fn label_offset() {
        let input = "wardley\ncomponent Kettle [0.43, 0.35] label [-57, 4]\n";
        let d = parse(input).diagram;
        assert_eq!(d.nodes.len(), 1);
        assert_eq!(d.nodes[0].label_offset_x, Some(-57.0));
        assert_eq!(d.nodes[0].label_offset_y, Some(4.0));
    }
}
