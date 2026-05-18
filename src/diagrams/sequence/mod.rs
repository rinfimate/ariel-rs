pub(crate) mod constants;
pub mod parser;
pub mod renderer;
pub(crate) mod templates;

use crate::error::ParseError;
use crate::theme::Theme;

pub(crate) fn validate(diagram: &parser::SequenceDiagram) -> Vec<ParseError> {
    use parser::SeqItem;
    let mut errors = Vec::new();

    // Collect explicitly declared participant aliases
    let declared: std::collections::HashSet<String> = diagram
        .items
        .iter()
        .filter_map(|item| {
            if let SeqItem::Participant(p) = item {
                Some(p.alias.clone())
            } else {
                None
            }
        })
        .collect();

    // Only validate if there are explicit declarations
    if declared.is_empty() {
        return errors;
    }

    for item in &diagram.items {
        if let SeqItem::Message(msg) = item {
            if !declared.contains(&msg.from) {
                errors.push(ParseError::new(format!(
                    "Message sender '{}' is not a declared participant",
                    msg.from
                )));
            }
            if !declared.contains(&msg.to) {
                errors.push(ParseError::new(format!(
                    "Message receiver '{}' is not a declared participant",
                    msg.to
                )));
            }
        }
    }

    errors
}

/// Render a Mermaid sequence diagram to SVG with foreignObject HTML labels.
pub fn render_html(input: &str, theme: Theme) -> String {
    let mut result = parser::parse(input);
    result.errors.extend(validate(&result.diagram));
    renderer::render(&result.diagram, theme, true)
}
