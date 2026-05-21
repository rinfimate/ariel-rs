pub(crate) mod constants;
pub mod parser;
pub mod renderer;
pub(crate) mod templates;

use crate::error::ParseError;
use crate::theme::Theme;

pub(crate) fn validate(diagram: &parser::FlowchartDiagram) -> Vec<ParseError> {
    let mut errors = Vec::new();
    let node_ids: std::collections::HashSet<&str> =
        diagram.nodes.keys().map(String::as_str).collect();

    // Check for duplicate node IDs (IndexMap preserves insertion order; duplicates can't exist
    // in the map itself, but we can detect edges referencing undeclared nodes).
    for edge in &diagram.edges {
        if !node_ids.contains(edge.from.as_str()) {
            errors.push(ParseError::new(format!(
                "Edge references unknown node '{}'",
                edge.from
            )));
        }
        if !node_ids.contains(edge.to.as_str()) {
            errors.push(ParseError::new(format!(
                "Edge references unknown node '{}'",
                edge.to
            )));
        }
    }

    errors
}

/// Render using native SVG `<text>` labels.
pub fn render_html(input: &str, theme: Theme) -> String {
    let mut result = parser::parse(input);
    result.errors.extend(validate(&result.diagram));
    renderer::render(&result.diagram, theme)
}
