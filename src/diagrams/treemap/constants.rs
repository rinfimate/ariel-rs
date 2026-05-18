//! Layout and styling constants for the treemap renderer.

// ---------------------------------------------------------------------------
// Canvas geometry
// ---------------------------------------------------------------------------

/// Canvas width (nodeWidth × SECTION_INNER_PADDING = 100 × 10, px).
pub const CANVAS_W: f64 = 1000.0;

/// Canvas height (nodeHeight × SECTION_INNER_PADDING = 40 × 10, px).
pub const CANVAS_H: f64 = 400.0;

// ---------------------------------------------------------------------------
// Padding / layout
// ---------------------------------------------------------------------------

/// Inner padding between leaf tiles (config.padding / paddingInner, px).
pub const INNER_PADDING: f64 = 10.0;

/// Padding at the top of branch (non-leaf) nodes for the section header (paddingTop, px).
pub const SECTION_HEADER_HEIGHT: f64 = 25.0;

/// Padding on left, right, and bottom of branch nodes (paddingLeft/Right/Bottom, px).
pub const SECTION_INNER_PADDING: f64 = 10.0;

/// Height reserved for the diagram title (px).
pub const TITLE_HEIGHT: f64 = 30.0;

/// Padding added around the content bounding box for the SVG viewBox (px).
pub const DIAGRAM_PADDING: f64 = 8.0;

// ---------------------------------------------------------------------------
// Squarify layout
// ---------------------------------------------------------------------------

/// Golden-ratio constant used by the D3 squarify algorithm (φ = (1+√5)/2).
pub const PHI: f64 = 1.618_033_988_749_895;

// ---------------------------------------------------------------------------
// Label colours (cScaleLabel from default Mermaid theme)
// ---------------------------------------------------------------------------

/// Ordinal label-colour palette (cScaleLabel0..11 from Mermaid's default theme).
/// Index 0 maps to white (dark tile), all others map to black.
pub const C_SCALE_LABEL: &[&str] = &[
    "#ffffff", // cScaleLabel0
    "black",   // cScaleLabel1
    "black",   // cScaleLabel2
    "black",   // cScaleLabel3
    "black",   // cScaleLabel4
    "black",   // cScaleLabel5
    "black",   // cScaleLabel6
    "black",   // cScaleLabel7
    "black",   // cScaleLabel8
    "black",   // cScaleLabel9
    "black",   // cScaleLabel10
    "black",   // cScaleLabel11
];
