/// Parser for Mermaid sankey-beta diagram syntax.
///
/// Faithful port of sankeyDB.ts.
///
/// Grammar:
///   [--- YAML front matter ---]
///   sankey-beta
///   source,target,value
///   source,target,value
///   ...
///
/// The DB maintains:
///   - nodes: ordered array of unique node IDs (preserving insertion order)
///   - links: array of (source_id, target_id, value) triples
///
/// Node IDs are de-duplicated: findOrCreateNode mirrors sankeyDB.ts.
///
/// YAML front matter may contain:
///   config:
///     sankey:
///       showValues: false
///       width: 600
///       height: 400
///       nodeAlignment: left | right | center | justify
///       prefix: ""
///       suffix: ""
///       nodeWidth: 10
///       nodePadding: 12
///       linkColor: gradient | source | target | <hex>

#[derive(Debug, Clone)]
pub struct SankeyConfig {
    pub show_values: bool,
    pub width: f64,
    pub height: f64,
    pub node_alignment: NodeAlignment,
    pub prefix: String,
    pub suffix: String,
    pub node_width: f64,
    pub node_padding: f64,
    pub link_color: LinkColor,
}

impl Default for SankeyConfig {
    fn default() -> Self {
        SankeyConfig {
            show_values: true,
            width: 600.0,
            height: 400.0,
            node_alignment: NodeAlignment::Justify,
            prefix: String::new(),
            suffix: String::new(),
            node_width: 10.0,
            node_padding: 12.0,
            link_color: LinkColor::Gradient,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeAlignment {
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinkColor {
    Gradient,
    Source,
    Target,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct SankeyNode {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct SankeyLink {
    pub source: String, // node ID
    pub target: String, // node ID
    pub value: f64,
}

pub struct SankeyDiagram {
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<SankeyLink>,
    pub config: SankeyConfig,
}

/// Parse a sankey-beta diagram from Mermaid syntax.
/// Mirrors the logic of sankeyDB.ts clear/addLink/findOrCreateNode.
pub fn parse(input: &str) -> crate::error::ParseResult<SankeyDiagram> {
    let mut nodes: Vec<SankeyNode> = Vec::new();
    let mut links: Vec<SankeyLink> = Vec::new();

    // Track node uniqueness by ID (mirrors sankeyDB.ts nodesMap)
    let mut nodes_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    // Strip and parse YAML front matter
    let (body, yaml_config, _yaml_title) = parse_frontmatter(input);
    let config = yaml_config.unwrap_or_default();

    let mut header_seen = false;

    for raw_line in body.lines() {
        let trimmed = raw_line.trim();

        // Skip blank lines and comments
        if trimmed.is_empty() || trimmed.starts_with("%%") || trimmed.starts_with('#') {
            continue;
        }

        // Detect and skip the "sankey-beta" header line
        if !header_seen {
            if trimmed.eq_ignore_ascii_case("sankey-beta") {
                header_seen = true;
                continue;
            }
            // If we haven't seen the header yet, keep looking
            continue;
        }

        // Each data line is: source,target,value
        // Source and target may be quoted: "source","target",value
        if let Some((source, target, value)) = parse_csv_line(trimmed) {
            // findOrCreateNode for source
            find_or_create(&mut nodes, &mut nodes_map, &source);
            find_or_create(&mut nodes, &mut nodes_map, &target);

            links.push(SankeyLink {
                source,
                target,
                value,
            });
        }
    }

    crate::error::ParseResult::ok(SankeyDiagram {
        nodes,
        links,
        config,
    })
}

/// Mirrors sankeyDB.ts findOrCreateNode — ensure a node with the given ID exists.
fn find_or_create(
    nodes: &mut Vec<SankeyNode>,
    nodes_map: &mut std::collections::HashMap<String, usize>,
    id: &str,
) {
    if !nodes_map.contains_key(id) {
        let idx = nodes.len();
        nodes.push(SankeyNode { id: id.to_string() });
        nodes_map.insert(id.to_string(), idx);
    }
}

/// Parse a CSV line of the form: source,target,value
/// Handles optional quoting of source and target fields.
/// Mirrors the Mermaid sankey grammar which uses CSV parsing.
fn parse_csv_line(line: &str) -> Option<(String, String, f64)> {
    let fields = split_csv(line);
    if fields.len() < 3 {
        return None;
    }

    let source = unquote(&fields[0]);
    let target = unquote(&fields[1]);
    let value_str = fields[2].trim();

    let value: f64 = value_str.parse().ok()?;

    if source.is_empty() || target.is_empty() {
        return None;
    }

    Some((source, target, value))
}

/// Split a CSV line respecting quoted fields (RFC 4180 subset).
fn split_csv(line: &str) -> Vec<String> {
    let mut fields: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if in_quotes {
            if c == '"' {
                // Check for escaped quote ""
                if chars.get(i + 1) == Some(&'"') {
                    current.push('"');
                    i += 2;
                    continue;
                }
                in_quotes = false;
            } else {
                current.push(c);
            }
        } else {
            match c {
                '"' => {
                    in_quotes = true;
                }
                ',' => {
                    fields.push(current.clone());
                    current.clear();
                }
                _ => {
                    current.push(c);
                }
            }
        }
        i += 1;
    }
    fields.push(current);
    fields
}

/// Remove surrounding quotes from a field value.
fn unquote(s: &str) -> String {
    let t = s.trim();
    if (t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')) {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

/// Parse YAML front matter and return (body_without_frontmatter, config, title).
/// The front matter is delimited by `---` lines.
fn parse_frontmatter(input: &str) -> (&str, Option<SankeyConfig>, Option<String>) {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("---") {
        return (input, None, None);
    }

    let after_open = &trimmed[3..];
    // Skip rest of the opening --- line (it may have content, but typically doesn't)
    let body_start = if let Some(nl) = after_open.find('\n') {
        &after_open[nl + 1..]
    } else {
        return (input, None, None);
    };

    // Find closing ---
    if let Some(close_pos) = body_start.find("\n---") {
        let yaml_str = &body_start[..close_pos];
        let remainder = &body_start[close_pos + 4..];
        // Skip the rest of the closing --- line
        let body = if let Some(nl) = remainder.find('\n') {
            &remainder[nl + 1..]
        } else {
            remainder.trim_start_matches('-').trim_start_matches('\n')
        };

        let (config, title) = parse_yaml_config(yaml_str);
        return (body, config, title);
    }

    (input, None, None)
}

/// Parse the YAML config block for sankey settings.
/// Very simple key-value parser (not a full YAML parser).
fn parse_yaml_config(yaml: &str) -> (Option<SankeyConfig>, Option<String>) {
    let mut config = SankeyConfig::default();
    let mut title: Option<String> = None;
    let mut found_sankey = false;

    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Detect `title: foo`
        if let Some(rest) = try_key(trimmed, "title") {
            title = Some(rest.to_string());
            continue;
        }

        // Detect `sankey:` block
        if trimmed == "sankey:" || trimmed.starts_with("sankey:") {
            found_sankey = true;
            continue;
        }

        if found_sankey {
            // Parse sankey sub-keys
            if let Some(rest) = try_key(trimmed, "showValues") {
                config.show_values = rest.trim().eq_ignore_ascii_case("true");
            } else if let Some(rest) = try_key(trimmed, "width") {
                if let Ok(v) = rest.trim().parse::<f64>() {
                    config.width = v;
                }
            } else if let Some(rest) = try_key(trimmed, "height") {
                if let Ok(v) = rest.trim().parse::<f64>() {
                    config.height = v;
                }
            } else if let Some(rest) = try_key(trimmed, "nodeAlignment") {
                config.node_alignment = match rest.trim().to_lowercase().as_str() {
                    "left" => NodeAlignment::Left,
                    "right" => NodeAlignment::Right,
                    "center" => NodeAlignment::Center,
                    _ => NodeAlignment::Justify,
                };
            } else if let Some(rest) = try_key(trimmed, "prefix") {
                config.prefix = unquote(rest.trim());
            } else if let Some(rest) = try_key(trimmed, "suffix") {
                config.suffix = unquote(rest.trim());
            } else if let Some(rest) = try_key(trimmed, "nodeWidth") {
                if let Ok(v) = rest.trim().parse::<f64>() {
                    config.node_width = v;
                }
            } else if let Some(rest) = try_key(trimmed, "nodePadding") {
                if let Ok(v) = rest.trim().parse::<f64>() {
                    config.node_padding = v;
                }
            } else if let Some(rest) = try_key(trimmed, "linkColor") {
                config.link_color = match rest.trim().to_lowercase().as_str() {
                    "gradient" => LinkColor::Gradient,
                    "source" => LinkColor::Source,
                    "target" => LinkColor::Target,
                    other => LinkColor::Custom(other.to_string()),
                };
            } else if !trimmed.starts_with(' ') && !trimmed.starts_with('\t') {
                // We've exited the sankey block
                found_sankey = false;
            }
        }
    }

    (Some(config), title)
}

/// Try to parse `key: value` from a YAML line, returning Some(value_str) on match.
fn try_key<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{}:", key);
    if line.starts_with(&prefix) {
        Some(line[prefix.len()..].trim())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_sankey() {
        let input = "sankey-beta\nA,B,10\nA,C,20\nB,D,5\n";
        let d = parse(input).diagram;
        assert_eq!(d.nodes.len(), 4);
        assert_eq!(d.links.len(), 3);
        assert_eq!(d.nodes[0].id, "A");
        assert_eq!(d.nodes[1].id, "B");
        assert_eq!(d.links[0].source, "A");
        assert_eq!(d.links[0].target, "B");
        assert_eq!(d.links[0].value, 10.0);
        assert_eq!(d.links[2].value, 5.0);
    }

    #[test]
    fn node_deduplication() {
        let input = "sankey-beta\nA,B,10\nA,C,20\n";
        let d = parse(input).diagram;
        // A appears as source twice but should only be one node
        assert_eq!(d.nodes.len(), 3);
        assert_eq!(d.nodes[0].id, "A");
    }

    #[test]
    fn quoted_fields() {
        let input = "sankey-beta\n\"Node A\",\"Node B\",15\n";
        let d = parse(input).diagram;
        assert_eq!(d.nodes[0].id, "Node A");
        assert_eq!(d.nodes[1].id, "Node B");
        assert_eq!(d.links[0].value, 15.0);
    }

    #[test]
    fn frontmatter_config() {
        let input = "---\nconfig:\n  sankey:\n    showValues: false\n    width: 800\n---\nsankey-beta\nA,B,10\n";
        let d = parse(input).diagram;
        assert!(!d.config.show_values);
        assert_eq!(d.config.width, 800.0);
        assert_eq!(d.nodes.len(), 2);
    }

    #[test]
    fn comments_skipped() {
        let input = "sankey-beta\n%% This is a comment\nA,B,10\n";
        let d = parse(input).diagram;
        assert_eq!(d.links.len(), 1);
    }

    #[test]
    fn csv_split() {
        let fields = split_csv("\"A,B\",C,10");
        assert_eq!(fields[0], "A,B");
        assert_eq!(fields[1], "C");
        assert_eq!(fields[2], "10");
    }
}
