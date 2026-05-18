/// Parser for Mermaid packet diagram syntax.
///
/// Faithful port of packetDB.ts.
///
/// Syntax:
///   packet-beta
///       0-15: "Source Port"
///       16-31: "Destination Port"
///       32-63: "Sequence Number"
///       64-95: "Acknowledgment Number"
///       96-99: "Data Offset"
///       100-105: "Reserved"
///       106: "URG"
///       107: "ACK"
///
/// Each field is defined by a bit range `start-end: "label"` or a single bit `N: "label"`.
/// The bit fields are rendered as horizontal boxes proportional to their width in bits.

#[derive(Debug, Clone)]
pub struct PacketField {
    pub start: u32,
    pub end: u32, // inclusive
    pub label: String,
}

#[derive(Debug, Default)]
pub struct PacketDiagram {
    pub title: Option<String>,
    pub fields: Vec<PacketField>,
}

pub fn parse(input: &str) -> crate::error::ParseResult<PacketDiagram> {
    let mut diag = PacketDiagram::default();

    for raw_line in input.lines() {
        let line = strip_comment(raw_line).trim().to_string();
        if line.is_empty() {
            continue;
        }

        // Diagram type declaration
        if line == "packet-beta" || line.starts_with("packet-beta") {
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

        // Field: N-M: "label" or N: "label"
        if let Some(colon_pos) = line.find(':') {
            let range_part = line[..colon_pos].trim();
            let label_part = line[colon_pos + 1..].trim().trim_matches('"').to_string();

            if let Some(dash_pos) = range_part.find('-') {
                // Range: start-end
                let start_str = range_part[..dash_pos].trim();
                let end_str = range_part[dash_pos + 1..].trim();
                if let (Ok(start), Ok(end)) = (start_str.parse::<u32>(), end_str.parse::<u32>()) {
                    diag.fields.push(PacketField {
                        start,
                        end: end.max(start),
                        label: label_part,
                    });
                    continue;
                }
            }
            // Single bit
            if let Ok(bit) = range_part.parse::<u32>() {
                diag.fields.push(PacketField {
                    start: bit,
                    end: bit,
                    label: label_part,
                });
                continue;
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_packet() {
        let input = "packet-beta\n    0-15: \"Source Port\"\n    16-31: \"Destination Port\"\n    32-63: \"Sequence Number\"\n    64-95: \"Acknowledgment Number\"";
        let diag = parse(input).diagram;
        assert_eq!(diag.fields.len(), 4);
        assert_eq!(diag.fields[0].start, 0);
        assert_eq!(diag.fields[0].end, 15);
        assert_eq!(diag.fields[0].label, "Source Port");
        assert_eq!(diag.fields[0].end - diag.fields[0].start + 1, 16);
        assert_eq!(diag.fields[3].end - diag.fields[3].start + 1, 32);
    }

    #[test]
    fn parse_single_bit() {
        let input = "packet-beta\n    0: \"Flag\"";
        let diag = parse(input).diagram;
        assert_eq!(diag.fields.len(), 1);
        assert_eq!(diag.fields[0].end - diag.fields[0].start + 1, 1);
    }

    #[test]
    fn bit_width_calculation() {
        let f = PacketField {
            start: 0,
            end: 15,
            label: "x".into(),
        };
        assert_eq!(f.end - f.start + 1, 16);
        let f2 = PacketField {
            start: 32,
            end: 63,
            label: "y".into(),
        };
        assert_eq!(f2.end - f2.start + 1, 32);
    }
}
