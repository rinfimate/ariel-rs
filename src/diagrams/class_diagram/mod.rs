pub(crate) mod constants;
pub mod parser;
pub mod renderer;
pub(crate) mod templates;

use crate::error::ParseError;
use crate::theme::Theme;

pub(crate) fn validate(diagram: &parser::ClassDiagram) -> Vec<ParseError> {
    let mut errors = Vec::new();

    for rel in &diagram.relations {
        if !diagram.classes.contains_key(&rel.id1) {
            errors.push(ParseError::new(format!(
                "Relation references unknown class '{}'",
                rel.id1
            )));
        }
        if !diagram.classes.contains_key(&rel.id2) {
            errors.push(ParseError::new(format!(
                "Relation references unknown class '{}'",
                rel.id2
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
