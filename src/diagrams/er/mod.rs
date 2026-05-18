pub(crate) mod constants;
pub mod parser;
pub mod renderer;
pub(crate) mod templates;

use crate::error::ParseError;
use crate::theme::Theme;

pub(crate) fn validate(diagram: &parser::ErDiagram) -> Vec<ParseError> {
    let entity_names: std::collections::HashSet<&str> =
        diagram.entities.iter().map(|e| e.name.as_str()).collect();
    let mut errors = Vec::new();

    for rel in &diagram.relationships {
        if !entity_names.contains(rel.entity_a.as_str()) {
            errors.push(ParseError::new(format!(
                "Relationship references unknown entity '{}'",
                rel.entity_a
            )));
        }
        if !entity_names.contains(rel.entity_b.as_str()) {
            errors.push(ParseError::new(format!(
                "Relationship references unknown entity '{}'",
                rel.entity_b
            )));
        }
    }

    errors
}

/// Render with <foreignObject> HTML labels (matches Mermaid reference output).
pub fn render_html(input: &str, theme: Theme) -> String {
    let mut result = parser::parse(input);
    result.errors.extend(validate(&result.diagram));
    renderer::render(&result.diagram, theme, true)
}
