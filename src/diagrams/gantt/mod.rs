pub(crate) mod constants;
pub mod parser;
pub mod renderer;
pub(crate) mod templates;

use crate::error::ParseError;
use crate::theme::Theme;

pub(crate) fn validate(diagram: &parser::GanttDiagram) -> Vec<ParseError> {
    let mut errors = Vec::new();

    for task in &diagram.tasks {
        if task.label.is_empty() {
            errors.push(ParseError::new(format!(
                "Task '{}' has an empty label",
                task.id
            )));
        }
    }

    errors
}

/// Render a Gantt diagram from Mermaid syntax to SVG with foreignObject HTML labels.
pub fn render_html(input: &str, theme: Theme) -> String {
    let mut result = parser::parse(input);
    result.errors.extend(validate(&result.diagram));
    renderer::render(&result.diagram, theme, true)
}
