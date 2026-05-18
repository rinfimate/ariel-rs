/// Parser for Mermaid quadrantChart syntax.
///
/// Supported grammar (faithful to quadrantDb.ts):
///   quadrantChart
///       title <text>
///       x-axis <left> --> <right>
///       y-axis <bottom> --> <top>
///       quadrant-1 <text>
///       quadrant-2 <text>
///       quadrant-3 <text>
///       quadrant-4 <text>
///       <Point Label>: [<x>, <y>]

#[derive(Debug, Clone, Default)]
pub struct QuadrantPoint {
    pub text: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Default)]
pub struct QuadrantDiagram {
    pub title: String,
    pub quadrant1_text: String,
    pub quadrant2_text: String,
    pub quadrant3_text: String,
    pub quadrant4_text: String,
    pub x_axis_left_text: String,
    pub x_axis_right_text: String,
    pub y_axis_bottom_text: String,
    pub y_axis_top_text: String,
    pub points: Vec<QuadrantPoint>,
}

pub fn parse(input: &str) -> crate::error::ParseResult<QuadrantDiagram> {
    let mut diag = QuadrantDiagram::default();

    for line in input.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        // First line: quadrantChart keyword
        if trimmed == "quadrantChart" || trimmed.starts_with("quadrantChart ") {
            // could have inline title: "quadrantChart title foo"
            let rest = trimmed["quadrantChart".len()..].trim();
            if let Some(t) = rest.strip_prefix("title") {
                let t = t.trim();
                if !t.is_empty() {
                    diag.title = t.to_string();
                }
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("title") {
            let t = rest.trim();
            if !t.is_empty() {
                diag.title = t.to_string();
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("x-axis") {
            let rest = rest.trim();
            parse_axis(
                rest,
                &mut diag.x_axis_left_text,
                &mut diag.x_axis_right_text,
            );
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("y-axis") {
            let rest = rest.trim();
            parse_axis(
                rest,
                &mut diag.y_axis_bottom_text,
                &mut diag.y_axis_top_text,
            );
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("quadrant-1") {
            diag.quadrant1_text = rest.trim().to_string();
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("quadrant-2") {
            diag.quadrant2_text = rest.trim().to_string();
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("quadrant-3") {
            diag.quadrant3_text = rest.trim().to_string();
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("quadrant-4") {
            diag.quadrant4_text = rest.trim().to_string();
            continue;
        }

        // Data point: "Label: [x, y]"
        if let Some(point) = parse_point(trimmed) {
            diag.points.push(point);
        }
    }

    crate::error::ParseResult::ok(diag)
}

/// Parse "LeftLabel --> RightLabel" or "SingleLabel" axis definitions.
fn parse_axis(rest: &str, left: &mut String, right: &mut String) {
    if let Some(idx) = rest.find("-->") {
        let l = rest[..idx].trim().to_string();
        let r = rest[idx + 3..].trim().to_string();
        *left = l;
        *right = r;
    } else {
        *left = rest.trim().to_string();
    }
}

/// Parse a data point line: "Label text: [0.3, 0.6]"
fn parse_point(line: &str) -> Option<QuadrantPoint> {
    // Find the colon that separates label from coordinates
    let colon_pos = line.find(':')?;
    let label = line[..colon_pos].trim().to_string();

    // Remainder after colon should contain [x, y]
    let rest = line[colon_pos + 1..].trim();
    let rest = rest.trim_start_matches('[').trim_end_matches(']').trim();
    let parts: Vec<&str> = rest.split(',').collect();
    if parts.len() < 2 {
        return None;
    }

    let x: f64 = parts[0].trim().parse().ok()?;
    let y: f64 = parts[1].trim().parse().ok()?;

    // Validate range [0, 1]
    if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
        return None;
    }

    Some(QuadrantPoint { text: label, x, y })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parse() {
        let input = r#"quadrantChart
    title Reach and engagement of campaigns
    x-axis Influence --> High Influence
    y-axis Low Reach --> High Reach
    quadrant-1 We should expand
    quadrant-2 Need to promote
    quadrant-3 Re-evaluate
    quadrant-4 May be improved
    Campaign A: [0.3, 0.6]
    Campaign B: [0.45, 0.23]"#;

        let d = parse(input).diagram;
        assert_eq!(d.title, "Reach and engagement of campaigns");
        assert_eq!(d.x_axis_left_text, "Influence");
        assert_eq!(d.x_axis_right_text, "High Influence");
        assert_eq!(d.y_axis_bottom_text, "Low Reach");
        assert_eq!(d.y_axis_top_text, "High Reach");
        assert_eq!(d.quadrant1_text, "We should expand");
        assert_eq!(d.quadrant2_text, "Need to promote");
        assert_eq!(d.quadrant3_text, "Re-evaluate");
        assert_eq!(d.quadrant4_text, "May be improved");
        assert_eq!(d.points.len(), 2);
        assert_eq!(d.points[0].text, "Campaign A");
        assert!((d.points[0].x - 0.3).abs() < 1e-9);
        assert!((d.points[0].y - 0.6).abs() < 1e-9);
        assert_eq!(d.points[1].text, "Campaign B");
    }

    #[test]
    fn parse_axis_single_label() {
        let input = "quadrantChart\n    x-axis Low Influence\n    y-axis Low Reach";
        let d = parse(input).diagram;
        assert_eq!(d.x_axis_left_text, "Low Influence");
        assert_eq!(d.x_axis_right_text, "");
        assert_eq!(d.y_axis_bottom_text, "Low Reach");
    }
}
