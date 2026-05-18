//! Layout and color constants for the sankey renderer.

// ---------------------------------------------------------------------------
// Colour palette
// ---------------------------------------------------------------------------

/// Tableau10 colour palette, mirrors d3-scale schemeTableau10.
/// Used to assign a distinct colour to each node by insertion index.
pub const TABLEAU10: [&str; 10] = [
    "#4e79a7", "#f28e2b", "#e15759", "#76b7b2", "#59a14f", "#edc948", "#b07aa1", "#ff9da7",
    "#9c755f", "#bab0ac",
];

// ---------------------------------------------------------------------------
// SVG / layout identifiers
// ---------------------------------------------------------------------------

/// Fixed SVG element id for the sankey diagram root.
pub const SVG_ID: &str = "mermaid-sankey";

// ---------------------------------------------------------------------------
// CSS / label sizing
// ---------------------------------------------------------------------------

/// Font size (px) used for node labels.
#[allow(dead_code)]
pub const LABEL_FONT_SIZE: &str = "14px";

/// Font size attribute value used on the node-labels group.
pub const LABEL_FONT_SIZE_ATTR: &str = "14";

/// Pixel gap between a node edge and its label text (label offset from node, px).
pub const LABEL_OFFSET: f64 = 6.0;
