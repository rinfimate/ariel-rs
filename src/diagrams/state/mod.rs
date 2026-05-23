pub mod constants;
pub mod parser;
pub mod renderer;
pub mod templates;

pub fn render(diag: &parser::StateDiagram, theme: crate::theme::Theme) -> String {
    renderer::render(diag, theme)
}
