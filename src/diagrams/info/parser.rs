//! Parser for the info diagram.
//! Grammar: `info` optionally followed by `showInfo`

/// Parsed info diagram (no fields needed — version comes from the renderer).
#[derive(Debug, Default)]
pub struct InfoDiagram {
    /// Whether `showInfo` was present (currently unused in rendering).
    #[allow(dead_code)]
    pub show_info: bool,
}

/// Parse an info diagram from Mermaid text.
pub fn parse(input: &str) -> InfoDiagram {
    let show_info = input.lines().any(|l| l.trim() == "showInfo");
    InfoDiagram { show_info }
}
