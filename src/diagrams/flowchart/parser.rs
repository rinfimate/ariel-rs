use indexmap::IndexMap;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct NodeStyle {
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeShape {
    Rectangle,        // [text]
    RoundedRect,      // (text)
    Diamond,          // {text}
    Circle,           // ((text))
    Asymmetric,       // >text]
    Cylinder,         // [(text)]
    Subroutine,       // [[text]]
    Stadium,          // ([text])
    Hexagon,          // {{text}}
    Trapezoid,        // [/text\]
    TrapezoidAlt,     // [\text/]
    Parallelogram,    // [/text/]
    ParallelogramAlt, // [\text\]
    Default,          // bare id
}

#[derive(Debug, Clone)]
pub struct FlowNode {
    pub label: String,
    pub shape: NodeShape,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeStyle {
    Arrow,         // -->
    Line,          // ---
    DotArrow,      // -.->
    ThickArrow,    // ==>
    DotLine,       // -.-
    OpenArrow,     // --o
    CrossArrow,    // --x
    BiArrow,       // <-->
    BiDotArrow,    // <-.->
    BiThickArrow,  // <==>
    BiCrossArrow,  // <--x
    BiCircleArrow, // o--o
    Invisible,     // ~~~
}

#[derive(Debug, Clone)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub style: EdgeStyle,
}

#[derive(Debug, Clone, Default)]
pub struct Subgraph {
    pub id: String,
    pub label: Option<String>,
    pub direction: Option<String>,
    pub members: Vec<String>, // node IDs and nested subgraph IDs
}

#[derive(Debug)]
pub struct FlowchartDiagram {
    pub direction: String,
    pub nodes: IndexMap<String, FlowNode>,
    pub edges: Vec<FlowEdge>,
    pub subgraphs: Vec<Subgraph>,
    pub node_subgraph: HashMap<String, String>, // node_id -> subgraph_id
    pub node_styles: HashMap<String, NodeStyle>, // node_id -> style overrides
}

pub fn parse(input: &str) -> crate::error::ParseResult<FlowchartDiagram> {
    let mut diag = FlowchartDiagram {
        direction: "TB".to_string(),
        nodes: IndexMap::new(),
        edges: Vec::new(),
        subgraphs: Vec::new(),
        node_subgraph: HashMap::new(),
        node_styles: HashMap::new(),
    };
    let mut parse_errors: Vec<crate::error::ParseError> = Vec::new();

    // Stack for nested subgraphs
    let mut subgraph_stack: Vec<Subgraph> = Vec::new();

    for (line_number, raw_line) in input.lines().enumerate() {
        let line_number = line_number + 1; // 1-based
        let line = strip_comment(raw_line).trim().to_string();
        if line.is_empty() {
            continue;
        }

        // Flowchart direction declaration
        if line.starts_with("flowchart ") || line.starts_with("graph ") {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                diag.direction = parts[1].trim().to_uppercase();
            }
            continue;
        }

        // Subgraph open
        if let Some(stripped) = line.strip_prefix("subgraph") {
            let rest = stripped.trim();
            let (id, label) = parse_subgraph_header(rest);
            subgraph_stack.push(Subgraph {
                id,
                label,
                direction: None,
                members: Vec::new(),
            });
            continue;
        }

        // End of subgraph
        if line == "end" {
            if let Some(finished) = subgraph_stack.pop() {
                let sg_id = finished.id.clone();
                if let Some(parent) = subgraph_stack.last_mut() {
                    parent.members.push(sg_id.clone());
                }
                diag.subgraphs.push(finished);
            }
            continue;
        }

        // Direction inside subgraph
        if let Some(stripped) = line.strip_prefix("direction ") {
            let dir = stripped.trim().to_uppercase();
            if let Some(sg) = subgraph_stack.last_mut() {
                sg.direction = Some(dir);
            }
            continue;
        }

        // classDef / class / linkStyle / click / %% — skip
        if line.starts_with("classDef ")
            || line.starts_with("class ")
            || line.starts_with("linkStyle ")
            || line.starts_with("click ")
            || line.starts_with("%%")
        {
            continue;
        }

        // style id fill:#xxx,stroke:#yyy,...
        if let Some(stripped) = line.strip_prefix("style ") {
            let rest = stripped.trim();
            let mut parts = rest.splitn(2, ' ');
            if let (Some(node_id), Some(attrs)) = (parts.next(), parts.next()) {
                let style = parse_style_attrs(attrs);
                diag.node_styles.insert(node_id.trim().to_string(), style);
            }
            continue;
        }

        // Edge / node statement — detect unrecognizable lines
        let nodes_before = diag.nodes.len();
        let edges_before = diag.edges.len();
        let members_before: Vec<String> = diag.nodes.keys().cloned().collect();
        parse_statement(&line, &mut diag);

        // If nothing was produced for a non-trivial line, record an error.
        let produced_something =
            diag.nodes.len() != nodes_before || diag.edges.len() != edges_before;
        if !produced_something {
            // parse_statement returns no output when the first token is empty (e.g.
            // the line starts with a character that isn't a valid node-id character).
            // Only flag lines whose first char isn't a known node-id start character.
            let first_char = line.chars().next().unwrap_or(' ');
            let looks_like_node_start = first_char.is_alphanumeric() || first_char == '_';
            if !looks_like_node_start {
                parse_errors.push(crate::error::ParseError::at_line(
                    line_number,
                    format!("Unrecognized syntax: '{}'", line),
                ));
            }
        }

        // Track which subgraph new nodes belong to
        if let Some(sg) = subgraph_stack.last_mut() {
            let sg_id = sg.id.clone();
            for id in diag.nodes.keys() {
                if !members_before.contains(id) && !diag.node_subgraph.contains_key(id) {
                    sg.members.push(id.clone());
                    diag.node_subgraph.insert(id.clone(), sg_id.clone());
                }
            }
        }
    }

    crate::error::ParseResult::with_errors(diag, parse_errors)
}

fn strip_comment(line: &str) -> &str {
    if let Some(pos) = line.find("%%") {
        &line[..pos]
    } else {
        line
    }
}

fn parse_subgraph_header(rest: &str) -> (String, Option<String>) {
    // rest may be: "one", "one [Label]", "one[Label]", just whitespace
    let rest = rest.trim();
    if rest.is_empty() {
        return (format!("_sg_{}", rand_id()), None);
    }
    // Check for bracket label
    if let Some(bracket) = rest.find('[') {
        let id = rest[..bracket].trim().to_string();
        let label_raw = &rest[bracket + 1..];
        let label = label_raw.trim_end_matches(']').trim().to_string();
        let id = if id.is_empty() {
            format!("_sg_{}", rand_id())
        } else {
            id
        };
        return (id, Some(label));
    }
    (rest.to_string(), None)
}

static RAND_CTR: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
fn rand_id() -> u32 {
    RAND_CTR.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

// ─── Statement parser ─────────────────────────────────────────────────────────

fn parse_statement(line: &str, diag: &mut FlowchartDiagram) {
    let mut pos = 0;
    let chars: Vec<char> = line.chars().collect();

    // Collect left-side node list (may have & between them)
    let mut left_nodes: Vec<String> = Vec::new();

    loop {
        skip_ws(&chars, &mut pos);
        if pos >= chars.len() {
            break;
        }

        let (id, shape_opt, label_opt) = parse_node_token(&chars, &mut pos);
        if id.is_empty() {
            break;
        }

        // Register node
        ensure_node(diag, &id, shape_opt, label_opt);
        left_nodes.push(id);

        skip_ws(&chars, &mut pos);
        // Multiple left nodes with &
        if pos < chars.len() && chars[pos] == '&' {
            pos += 1;
            continue;
        }
        break;
    }

    if left_nodes.is_empty() {
        return;
    }

    // Check for edge
    skip_ws(&chars, &mut pos);
    if pos >= chars.len() {
        return;
    }

    // Try to parse edge
    if let Some((style, label, after_pos)) = try_parse_edge(&chars, pos) {
        pos = after_pos;

        // Right-side node list
        let mut right_nodes: Vec<String> = Vec::new();
        loop {
            skip_ws(&chars, &mut pos);
            if pos >= chars.len() {
                break;
            }

            let (id, shape_opt, label_opt) = parse_node_token(&chars, &mut pos);
            if id.is_empty() {
                break;
            }

            ensure_node(diag, &id, shape_opt, label_opt);
            right_nodes.push(id);

            skip_ws(&chars, &mut pos);
            if pos < chars.len() && chars[pos] == '&' {
                pos += 1;
                continue;
            }
            break;
        }

        // Create edges: each left × each right
        for l in &left_nodes {
            for r in &right_nodes {
                diag.edges.push(FlowEdge {
                    from: l.clone(),
                    to: r.clone(),
                    label: label.clone(),
                    style: style.clone(),
                });
            }
        }

        // If there's more on the line (chained edges), recurse on remaining
        skip_ws(&chars, &mut pos);
        if pos < chars.len() {
            let rest: String = chars[pos..].iter().collect();
            // The right_nodes become left_nodes for the next segment
            let mut chain = right_nodes
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(" & ");
            chain.push(' ');
            chain.push_str(&rest);
            parse_statement(&chain, diag);
        }
    }
}

fn ensure_node(
    diag: &mut FlowchartDiagram,
    id: &str,
    shape: Option<NodeShape>,
    label: Option<String>,
) {
    if !diag.nodes.contains_key(id) {
        let label = label.unwrap_or_else(|| id.to_string());
        let shape = shape.unwrap_or(NodeShape::Default);
        diag.nodes.insert(id.to_string(), FlowNode { label, shape });
    } else if let Some(node) = diag.nodes.get_mut(id) {
        // Update shape/label if explicitly provided
        if let Some(s) = shape {
            node.shape = s;
        }
        if let Some(l) = label {
            node.label = l;
        }
    }
}

fn skip_ws(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos].is_whitespace() {
        *pos += 1;
    }
}

/// Parse a node token: id + optional shape bracket
/// Returns (id, shape, label)
fn parse_node_token(
    chars: &[char],
    pos: &mut usize,
) -> (String, Option<NodeShape>, Option<String>) {
    skip_ws(chars, pos);
    if *pos >= chars.len() {
        return (String::new(), None, None);
    }

    // Parse the node ID
    let id = parse_node_id(chars, pos);
    if id.is_empty() {
        return (String::new(), None, None);
    }

    // Check for shape brackets immediately after id
    if *pos >= chars.len() {
        return (id, None, None);
    }

    let (shape, label) = parse_shape(chars, pos);
    (id, shape, label)
}

fn parse_node_id(chars: &[char], pos: &mut usize) -> String {
    let mut id = String::new();
    while *pos < chars.len() {
        let c = chars[*pos];
        let next = chars.get(*pos + 1).copied();
        // Stop at edge-starting sequences: -- (line/arrow), -. (dot), == (thick)
        if c == '-' && matches!(next, Some('-') | Some('.')) {
            break;
        }
        if c == '=' && next == Some('=') {
            break;
        }
        if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
            if id.is_empty() && c == '-' {
                break;
            } // no leading -
            id.push(c);
            *pos += 1;
        } else {
            break;
        }
    }
    id
}

fn parse_shape(chars: &[char], pos: &mut usize) -> (Option<NodeShape>, Option<String>) {
    if *pos >= chars.len() {
        return (None, None);
    }

    match chars[*pos] {
        '[' => {
            // [[ = Subroutine, [( = Cylinder/database, [/ or [\ = trapezoid/parallelogram, [ = Rectangle
            if *pos + 1 < chars.len() && chars[*pos + 1] == '[' {
                *pos += 2;
                let text = read_until(chars, pos, "]]");
                (Some(NodeShape::Subroutine), Some(text))
            } else if *pos + 1 < chars.len() && chars[*pos + 1] == '(' {
                // [(text)] = Cylinder / database
                *pos += 2;
                let text = read_until(chars, pos, ")]");
                (Some(NodeShape::Cylinder), Some(text))
            } else if *pos + 1 < chars.len() && chars[*pos + 1] == '/' {
                // [/text\] = Trapezoid or [/text/] = Parallelogram
                *pos += 2;
                let mut text = String::new();
                let mut shape = NodeShape::Parallelogram;
                while *pos < chars.len() {
                    if chars[*pos] == '\\' && *pos + 1 < chars.len() && chars[*pos + 1] == ']' {
                        shape = NodeShape::Trapezoid;
                        *pos += 2;
                        break;
                    }
                    if chars[*pos] == '/' && *pos + 1 < chars.len() && chars[*pos + 1] == ']' {
                        shape = NodeShape::Parallelogram;
                        *pos += 2;
                        break;
                    }
                    text.push(chars[*pos]);
                    *pos += 1;
                }
                (Some(shape), Some(text.trim().to_string()))
            } else if *pos + 1 < chars.len() && chars[*pos + 1] == '\\' {
                // [\text/] = TrapezoidAlt or [\text\] = ParallelogramAlt
                *pos += 2;
                let mut text = String::new();
                let mut shape = NodeShape::TrapezoidAlt;
                while *pos < chars.len() {
                    if chars[*pos] == '/' && *pos + 1 < chars.len() && chars[*pos + 1] == ']' {
                        shape = NodeShape::TrapezoidAlt;
                        *pos += 2;
                        break;
                    }
                    if chars[*pos] == '\\' && *pos + 1 < chars.len() && chars[*pos + 1] == ']' {
                        shape = NodeShape::ParallelogramAlt;
                        *pos += 2;
                        break;
                    }
                    text.push(chars[*pos]);
                    *pos += 1;
                }
                (Some(shape), Some(text.trim().to_string()))
            } else {
                *pos += 1;
                let text = read_bracket_text(chars, pos, ']');
                (Some(NodeShape::Rectangle), Some(text))
            }
        }
        '(' => {
            if *pos + 1 < chars.len() && chars[*pos + 1] == '(' {
                *pos += 2;
                let text = read_until(chars, pos, "))");
                (Some(NodeShape::Circle), Some(text))
            } else if *pos + 1 < chars.len() && chars[*pos + 1] == '[' {
                // ([text]) = Stadium / pill shape
                *pos += 2;
                let text = read_until(chars, pos, "])");
                (Some(NodeShape::Stadium), Some(text))
            } else {
                *pos += 1;
                let text = read_bracket_text(chars, pos, ')');
                (Some(NodeShape::RoundedRect), Some(text))
            }
        }
        '{' => {
            if *pos + 1 < chars.len() && chars[*pos + 1] == '{' {
                *pos += 2;
                let text = read_until(chars, pos, "}}");
                (Some(NodeShape::Hexagon), Some(text))
            } else {
                *pos += 1;
                let text = read_bracket_text(chars, pos, '}');
                (Some(NodeShape::Diamond), Some(text))
            }
        }
        '>' => {
            *pos += 1;
            let text = read_bracket_text(chars, pos, ']');
            (Some(NodeShape::Asymmetric), Some(text))
        }
        '@' => {
            // @{ shape: X, label: 'Y' } attribute syntax
            *pos += 1;
            if *pos < chars.len() && chars[*pos] == '{' {
                *pos += 1;
                let block = read_bracket_text(chars, pos, '}');
                let mut shape: Option<NodeShape> = None;
                let mut label: Option<String> = None;
                for part in block.split(',') {
                    let part = part.trim();
                    if let Some(val) = part.strip_prefix("shape:") {
                        shape = Some(match val.trim().trim_matches('\'').trim_matches('"') {
                            "circle" | "doublecircle" => NodeShape::Circle,
                            "rect" | "rectangle" => NodeShape::Rectangle,
                            "round" | "rounded" => NodeShape::RoundedRect,
                            "diamond" | "question" => NodeShape::Diamond,
                            "hexagon" => NodeShape::Hexagon,
                            "stadium" | "pill" => NodeShape::Stadium,
                            "cylinder" | "database" => NodeShape::Cylinder,
                            "subroutine" => NodeShape::Subroutine,
                            "parallelogram" => NodeShape::Parallelogram,
                            "parallelogram-alt" => NodeShape::ParallelogramAlt,
                            "trapezoid" => NodeShape::Trapezoid,
                            "trapezoid-alt" => NodeShape::TrapezoidAlt,
                            _ => NodeShape::Default,
                        });
                    } else if let Some(val) = part.strip_prefix("label:") {
                        label = Some(val.trim().trim_matches('\'').trim_matches('"').to_string());
                    }
                }
                (shape, label)
            } else {
                (None, None)
            }
        }
        _ => (None, None),
    }
}

fn read_bracket_text(chars: &[char], pos: &mut usize, close: char) -> String {
    let mut text = String::new();
    let mut depth = 1i32;
    // Handle quoted text "..."
    let open = match close {
        ']' => '[',
        ')' => '(',
        '}' => '{',
        c => c,
    };
    while *pos < chars.len() {
        let c = chars[*pos];
        *pos += 1;
        if c == '"' {
            // Consume quoted string
            while *pos < chars.len() {
                let q = chars[*pos];
                *pos += 1;
                if q == '"' {
                    break;
                }
                text.push(q);
            }
            continue;
        }
        if c == open {
            depth += 1;
        }
        if c == close {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        text.push(c);
    }
    text.trim().to_string()
}

fn read_until(chars: &[char], pos: &mut usize, end: &str) -> String {
    let end_chars: Vec<char> = end.chars().collect();
    let mut text = String::new();
    while *pos < chars.len() {
        // Check for end sequence
        if *pos + end_chars.len() <= chars.len()
            && chars[*pos..*pos + end_chars.len()] == end_chars[..]
        {
            *pos += end_chars.len();
            break;
        }
        text.push(chars[*pos]);
        *pos += 1;
    }
    text.trim().to_string()
}

// ─── Style parser ────────────────────────────────────────────────────────────

fn parse_style_attrs(attrs: &str) -> NodeStyle {
    let mut style = NodeStyle::default();
    for part in attrs.split(',') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix("fill:") {
            style.fill = Some(normalize_color(v.trim()));
        } else if let Some(v) = part.strip_prefix("stroke-width:") {
            style.stroke_width = Some(v.trim().to_string());
        } else if let Some(v) = part.strip_prefix("stroke:") {
            style.stroke = Some(normalize_color(v.trim()));
        } else if let Some(v) = part.strip_prefix("color:") {
            style.color = Some(normalize_color(v.trim()));
        }
    }
    style
}

fn normalize_color(c: &str) -> String {
    // Expand short hex (#f9f → #ff99ff)
    if c.starts_with('#') && c.len() == 4 {
        let chars: Vec<char> = c.chars().collect();
        return format!("#{0}{0}{1}{1}{2}{2}", chars[1], chars[2], chars[3]);
    }
    c.to_string()
}

// ─── Edge parser ─────────────────────────────────────────────────────────────

/// Returns (style, label, new_pos) or None if no edge at this position
fn try_parse_edge(chars: &[char], pos: usize) -> Option<(EdgeStyle, Option<String>, usize)> {
    let mut p = pos;

    // Skip whitespace
    while p < chars.len() && chars[p].is_whitespace() {
        p += 1;
    }
    if p >= chars.len() {
        return None;
    }

    // Arrow types — try longest match first
    let rest: String = chars[p..].iter().collect();

    // Invisible link ~~~
    if rest.starts_with("~~~") {
        return Some((EdgeStyle::Invisible, None, p + 3));
    }

    // Bidirectional arrows — must be checked before unidirectional
    if rest.starts_with("<==>") {
        let pp = p + 4;
        let (label, end) = try_pipe_label(chars, pp);
        return Some((EdgeStyle::BiThickArrow, label, end));
    }
    if rest.starts_with("<-.->") {
        let pp = p + 5;
        let (label, end) = try_pipe_label(chars, pp);
        return Some((EdgeStyle::BiDotArrow, label, end));
    }
    if rest.starts_with("<--x") {
        let pp = p + 4;
        let (label, end) = try_pipe_label(chars, pp);
        return Some((EdgeStyle::BiCrossArrow, label, end));
    }
    if rest.starts_with("<-->") {
        let pp = p + 4;
        let (label, end) = try_pipe_label(chars, pp);
        return Some((EdgeStyle::BiArrow, label, end));
    }

    // Bidirectional circle o--o
    if rest.starts_with("o--o") {
        let pp = p + 4;
        let (label, end) = try_pipe_label(chars, pp);
        return Some((EdgeStyle::BiCircleArrow, label, end));
    }

    // Thick arrow ==>
    if rest.starts_with("==>") {
        let pp = p + 3;
        let (label, end) = try_pipe_label(chars, pp);
        return Some((EdgeStyle::ThickArrow, label, end));
    }
    if rest.starts_with("==") {
        // ==text==> style
        let inner_start = p + 2;
        if let Some((text, end)) = parse_text_arrow(chars, inner_start, "==>") {
            return Some((EdgeStyle::ThickArrow, Some(text), end));
        }
    }

    // Dot arrow -.->
    if rest.starts_with("-.->") {
        let pp = p + 4;
        let (label, end) = try_pipe_label(chars, pp);
        return Some((EdgeStyle::DotArrow, label, end));
    }
    if rest.starts_with("-.") {
        let inner_start = p + 2;
        if let Some((text, end)) = parse_text_arrow(chars, inner_start, ".->") {
            return Some((EdgeStyle::DotArrow, Some(text), end));
        }
        // dot line -.-
        if let Some(pos2) = find_seq(chars, inner_start, ".-") {
            let text: String = chars[inner_start..pos2].iter().collect();
            return Some((EdgeStyle::DotLine, Some(text.trim().to_string()), pos2 + 2));
        }
    }

    // Open arrow --o
    if rest.starts_with("--o") {
        return Some((EdgeStyle::OpenArrow, None, p + 3));
    }
    // Cross arrow --x
    if rest.starts_with("--x") {
        return Some((EdgeStyle::CrossArrow, None, p + 3));
    }

    // Arrow --> or --text-->
    if rest.starts_with("-->") {
        let pp = p + 3;
        let (label, end) = try_pipe_label(chars, pp);
        return Some((EdgeStyle::Arrow, label, end));
    }
    if rest.starts_with("--") {
        let inner_start = p + 2;
        // Check for --text-->
        if let Some((text, end)) = parse_text_arrow(chars, inner_start, "-->") {
            return Some((EdgeStyle::Arrow, Some(text), end));
        }
        // Plain line ---
        if rest.starts_with("---") {
            return Some((EdgeStyle::Line, None, p + 3));
        }
        // Line with text --text--
        if let Some(pos2) = find_seq(chars, inner_start, "--") {
            let text: String = chars[inner_start..pos2].iter().collect();
            return Some((EdgeStyle::Line, Some(text.trim().to_string()), pos2 + 2));
        }
    }

    None
}

/// After an arrow like `-->`, check for `|label|`
fn try_pipe_label(chars: &[char], pos: usize) -> (Option<String>, usize) {
    let mut p = pos;
    while p < chars.len() && chars[p].is_whitespace() {
        p += 1;
    }
    if p < chars.len() && chars[p] == '|' {
        p += 1;
        let mut label = String::new();
        while p < chars.len() && chars[p] != '|' {
            label.push(chars[p]);
            p += 1;
        }
        if p < chars.len() {
            p += 1;
        } // closing |
        return (Some(label.trim().to_string()), p);
    }
    (None, pos)
}

/// Parse --text--> style: find the end sequence and extract text in between
fn parse_text_arrow(chars: &[char], start: usize, end_seq: &str) -> Option<(String, usize)> {
    if let Some(pos) = find_seq(chars, start, end_seq) {
        let text: String = chars[start..pos].iter().collect();
        Some((text.trim().to_string(), pos + end_seq.len()))
    } else {
        None
    }
}

fn find_seq(chars: &[char], start: usize, seq: &str) -> Option<usize> {
    let seq_chars: Vec<char> = seq.chars().collect();
    let n = seq_chars.len();
    for i in start..chars.len().saturating_sub(n - 1) {
        if chars[i..i + n] == seq_chars[..] {
            return Some(i);
        }
    }
    None
}
