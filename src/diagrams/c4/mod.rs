pub(crate) mod constants;
pub mod parser;
pub mod renderer;
pub(crate) mod templates;

use crate::error::ParseError;
use crate::theme::Theme;

pub(crate) fn validate(_diagram: &parser::C4Diagram) -> Vec<ParseError> {
    vec![]
}

/// Render a C4 architecture diagram from Mermaid syntax to SVG with foreignObject HTML labels.
pub fn render_html(input: &str, theme: Theme) -> String {
    let mut result = parser::parse(input);
    result.errors.extend(validate(&result.diagram));
    renderer::render(&result.diagram, theme, true)
}
