// Faithful Rust port of mermaid/src/diagrams/radar/parser.ts + db.ts
//
// Grammar (radar-beta):
//   radar-beta
//   [title <text>]
//   [options]
//     showLegend: true/false
//     ticks: <n>
//     max: <n>
//     min: <n>
//     graticule: circle|polygon
//   axes <name> [label], <name> [label], ...  (or one per line)
//   curve <name> [label]: v1, v2, v3, ...

#[derive(Debug, Clone)]
pub struct RadarAxis {
    pub name: String,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct RadarCurve {
    pub label: String,
    pub entries: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct RadarOptions {
    pub show_legend: bool,
    pub ticks: usize,
    pub max: Option<f64>,
    pub min: f64,
    pub graticule: GraticuleType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GraticuleType {
    Circle,
    Polygon,
}

impl Default for RadarOptions {
    fn default() -> Self {
        RadarOptions {
            show_legend: true,
            ticks: 5,
            max: None,
            min: 0.0,
            graticule: GraticuleType::Circle,
        }
    }
}

#[derive(Debug)]
pub struct RadarDiagram {
    pub title: Option<String>,
    pub axes: Vec<RadarAxis>,
    pub curves: Vec<RadarCurve>,
    pub options: RadarOptions,
}

// ─── Parser ───────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<RadarDiagram> {
    let mut title: Option<String> = None;
    let mut axes: Vec<RadarAxis> = Vec::new();
    let mut curves: Vec<RadarCurve> = Vec::new();
    let mut options = RadarOptions::default();

    let mut in_header = true;
    let mut in_options = false;

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

        // Wait for header keyword
        if in_header {
            if trimmed.starts_with("radar") {
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
            in_options = false;
            continue;
        }

        // accTitle / accDescr – skip
        if trimmed.starts_with("accTitle") || trimmed.starts_with("accDescr") {
            in_options = false;
            continue;
        }

        // options block
        if trimmed == "options" {
            in_options = true;
            continue;
        }
        // end options
        if in_options
            && (trimmed == "end" || trimmed.starts_with("axes") || trimmed.starts_with("curve"))
        {
            in_options = false;
            // fall-through to handle current line
        } else if in_options {
            parse_option_line(trimmed, &mut options);
            continue;
        }

        // axes declaration: "axes name1 ["label1"], name2 ["label2"], ..."
        if let Some(stripped) = trimmed.strip_prefix("axes") {
            let rest = stripped.trim();
            for part in rest.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                let (name, label) = parse_name_label(part);
                axes.push(RadarAxis {
                    name: name.clone(),
                    label: label.unwrap_or(name),
                });
            }
            continue;
        }

        // curve declaration: "curve name ["label"]: v1, v2, v3"
        //   or reference-style: "curve name ["label"]: axis1: v1, axis2: v2"
        if let Some(stripped) = trimmed.strip_prefix("curve") {
            let rest = stripped.trim();
            if let Some(colon_pos) = rest.find(':') {
                let head = rest[..colon_pos].trim();
                let vals_str = rest[colon_pos + 1..].trim();
                let (name, label) = parse_name_label(head);

                // Parse values — either plain numbers or "axis: value" pairs
                let entries = parse_curve_values(vals_str, &axes);

                curves.push(RadarCurve {
                    label: label.unwrap_or(name),
                    entries,
                });
            }
            continue;
        }
    }

    crate::error::ParseResult::ok(RadarDiagram {
        title,
        axes,
        curves,
        options,
    })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn parse_option_line(line: &str, opts: &mut RadarOptions) {
    // "key: value" or "key : value"
    if let Some(pos) = line.find(':') {
        let key = line[..pos].trim();
        let val = line[pos + 1..].trim();
        match key {
            "showLegend" => {
                opts.show_legend = val == "true" || val == "1";
            }
            "ticks" => {
                if let Ok(n) = val.parse::<usize>() {
                    opts.ticks = n;
                }
            }
            "max" => {
                opts.max = val.parse::<f64>().ok();
            }
            "min" => {
                if let Ok(v) = val.parse::<f64>() {
                    opts.min = v;
                }
            }
            "graticule" => {
                opts.graticule = if val == "polygon" {
                    GraticuleType::Polygon
                } else {
                    GraticuleType::Circle
                };
            }
            _ => {}
        }
    }
}

/// Parse "name" or `name ["label"]` or `name [label]`
fn parse_name_label(s: &str) -> (String, Option<String>) {
    if let Some(bracket) = s.find('[') {
        let name = s[..bracket].trim().to_string();
        let rest = &s[bracket + 1..];
        let label_raw = rest
            .trim_end_matches(']')
            .trim()
            .trim_matches('"')
            .to_string();
        let label = if label_raw.is_empty() {
            None
        } else {
            Some(label_raw)
        };
        (name, label)
    } else {
        (s.trim().trim_matches('"').to_string(), None)
    }
}

/// Parse a comma-separated list of values or "axis_name: value" pairs.
/// If pairs, orders them according to the axes list.
fn parse_curve_values(vals_str: &str, axes: &[RadarAxis]) -> Vec<f64> {
    let parts: Vec<&str> = vals_str.split(',').collect();

    // Check if entries are axis-reference style (contain ':')
    if parts.iter().any(|p| p.contains(':')) {
        // Build map axis_name -> value
        let mut map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for part in &parts {
            let part = part.trim();
            if let Some(cp) = part.find(':') {
                let axis_name = part[..cp].trim().to_string();
                let value: f64 = part[cp + 1..].trim().parse().unwrap_or(0.0);
                map.insert(axis_name, value);
            }
        }
        // Order by axes
        axes.iter()
            .map(|a| *map.get(&a.name).unwrap_or(&0.0))
            .collect()
    } else {
        // Plain values in order
        parts
            .iter()
            .map(|p| p.trim().parse::<f64>().unwrap_or(0.0))
            .collect()
    }
}
