//! Renderer for the info diagram.
//! Source: src/diagrams/info/infoRenderer.ts
//!
//! draw(text, id, version) creates a 400×100 SVG with the version string
//! centred (text-anchor:middle) at (100, 40) in 32px font.

use super::constants::{ARIEL_VERSION, MERMAID_VERSION};
use super::parser::InfoDiagram;
use super::templates::render_svg;
use crate::theme::Theme;

pub fn render(diag: &InfoDiagram, _theme: Theme) -> String {
    let _ = diag;
    render_svg("mermaid-info", MERMAID_VERSION, ARIEL_VERSION)
}
