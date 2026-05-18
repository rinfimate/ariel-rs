pub(crate) mod constants;
pub mod parser;
pub mod renderer;
pub(crate) mod templates;

use crate::error::ParseError;
use crate::theme::Theme;

pub(crate) fn validate(_diagram: &parser::QuadrantDiagram) -> Vec<ParseError> {
    vec![]
}

/// Render a quadrant chart from Mermaid syntax to SVG.
pub fn render(input: &str, theme: Theme) -> String {
    let mut result = parser::parse(input);
    result.errors.extend(validate(&result.diagram));
    renderer::render(&result.diagram, theme)
}

/// Render to SVG (plain SVG text elements, no foreignObject).
pub fn render_html(input: &str, theme: Theme) -> String {
    render(input, theme)
}
