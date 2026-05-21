/// Parser for Mermaid pie chart syntax.
///
/// Supported grammar:
///   pie [showData] [title <text>]
///       "Label" : value
///       "Label2" : value2
///
/// Labels are kept in insertion order (IndexMap).
use indexmap::IndexMap;

#[derive(Debug, Clone, Default)]
pub struct PieDiagram {
    /// Optional diagram title.
    pub title: Option<String>,
    /// Whether to show raw data values in legend labels.
    pub show_data: bool,
    /// Ordered map of label → value (only non-negative values).
    pub sections: IndexMap<String, f64>,
}

pub fn parse(input: &str) -> crate::error::ParseResult<PieDiagram> {
    let mut diag = PieDiagram::default();

    for line in input.lines() {
        let trimmed = line.trim();

        // Skip blank lines and comments
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        // accTitle / accDescr — skip
        if trimmed.starts_with("accTitle") || trimmed.starts_with("accDescr") {
            continue;
        }

        // The first meaningful line must start with "pie"
        if let Some(stripped) = trimmed.strip_prefix("pie") {
            let rest = stripped.trim();
            parse_header(rest, &mut diag);
            continue;
        }

        // Data line: "Label" : value
        if trimmed.starts_with('"') {
            if let Some((label, value)) = parse_data_line(trimmed) {
                if value >= 0.0 {
                    diag.sections.insert(label, value);
                }
            }
        }
    }

    crate::error::ParseResult::ok(diag)
}

/// Parse the rest of the `pie` header line: optional `showData`, optional `title <text>`.
fn parse_header(rest: &str, diag: &mut PieDiagram) {
    let mut s = rest;

    // showData keyword
    if s.starts_with("showData") {
        diag.show_data = true;
        s = s["showData".len()..].trim_start();
    }

    // title keyword
    if let Some(stripped) = s.strip_prefix("title") {
        let title_text = stripped.trim();
        if !title_text.is_empty() {
            diag.title = Some(title_text.to_string());
        }
    }
}

/// Parse a data line of the form: `"Label" : 42.5`
fn parse_data_line(line: &str) -> Option<(String, f64)> {
    // Find the closing quote
    let start = line.find('"')?;
    let end = line[start + 1..].find('"')? + start + 1;
    let label = line[start + 1..end].to_string();

    // Find the colon after the closing quote
    let after_quote = &line[end + 1..];
    let colon_pos = after_quote.find(':')?;
    let value_str = after_quote[colon_pos + 1..].trim();

    let value: f64 = value_str.parse().ok()?;
    Some((label, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_pie() {
        let input = "pie\n    \"Dogs\" : 386\n    \"Cats\" : 85\n    \"Rats\" : 15";
        let d = parse(input).diagram;
        assert_eq!(d.title, None);
        assert!(!d.show_data);
        assert_eq!(d.sections["Dogs"], 386.0);
        assert_eq!(d.sections["Cats"], 85.0);
        assert_eq!(d.sections["Rats"], 15.0);
    }

    #[test]
    fn pie_with_title() {
        let input = "pie title Pets adopted by volunteers\n    \"Dogs\" : 386\n    \"Cats\" : 85";
        let d = parse(input).diagram;
        assert_eq!(d.title.as_deref(), Some("Pets adopted by volunteers"));
        assert_eq!(d.sections.len(), 2);
    }

    #[test]
    fn pie_showdata() {
        let input =
            "pie showData title Key Elements\n    \"Calcium\" : 42.96\n    \"Potassium\" : 50.05";
        let d = parse(input).diagram;
        assert!(d.show_data);
        assert_eq!(d.title.as_deref(), Some("Key Elements"));
        assert!((d.sections["Calcium"] - 42.96).abs() < 1e-9);
    }

    #[test]
    fn pie_many_slices() {
        let input = "pie title Browser Market Share\n    \"Chrome\" : 65.2\n    \"Safari\" : 19.2\n    \"Firefox\" : 4.0\n    \"Edge\" : 3.1\n    \"Samsung\" : 2.8\n    \"Opera\" : 1.2\n    \"Other\" : 4.5";
        let d = parse(input).diagram;
        assert_eq!(d.sections.len(), 7);
        assert_eq!(d.title.as_deref(), Some("Browser Market Share"));
    }
}
