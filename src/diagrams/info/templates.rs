//! SVG templates for the info diagram.

pub use crate::diagrams::util::{esc, fmt};

use super::constants::*;

/// Full SVG for the info diagram showing Mermaid version and ariel-rs version.
pub fn render_svg(id: &str, mermaid_version: &str, ariel_version: &str) -> String {
    let sub_y = TEXT_Y + FONT_SIZE as f64 + 8.0;
    format!(
        "<svg id=\"{id}\" width=\"100%\" height=\"{h}\" \
         xmlns=\"http://www.w3.org/2000/svg\" \
         style=\"max-width: {w}px;\" \
         viewBox=\"0 0 {w} {h}\" \
         role=\"graphics-document document\" \
         aria-roledescription=\"info\">\
         <text x=\"{x}\" y=\"{y}\" class=\"ariel-version\" \
         font-size=\"{fs}\" \
         style=\"text-anchor: middle;\">ariel-rs v{aver}</text>\
         <text x=\"{x}\" y=\"{sy}\" class=\"version\" \
         font-size=\"16\" \
         style=\"text-anchor: middle; fill: #666;\">mermaid v{mver}</text>\
         </svg>",
        id = id,
        h = fmt(SVG_HEIGHT),
        w = fmt(SVG_WIDTH),
        x = fmt(TEXT_X),
        y = fmt(TEXT_Y),
        fs = FONT_SIZE,
        mver = esc(mermaid_version),
        sy = fmt(sub_y),
        aver = esc(ariel_version),
    )
}
