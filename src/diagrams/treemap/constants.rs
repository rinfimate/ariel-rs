//! Layout and styling constants for the treemap renderer.

// ---------------------------------------------------------------------------
// Canvas geometry
// ---------------------------------------------------------------------------

/// Canvas width (nodeWidth × 10 = 100 × 10, px).
pub(crate) const CANVAS_W: f64 = 1000.0;

/// Canvas height (nodeHeight × 10 = 40 × 10, px).
pub(crate) const CANVAS_H: f64 = 400.0;

// ---------------------------------------------------------------------------
// Padding / layout
// ---------------------------------------------------------------------------

/// Inner padding between adjacent tiles (paddingInner, px).
pub(crate) const PADDING_INNER: f64 = 10.0;

/// Height of the section header area inside branch nodes (paddingTop, px).
pub(crate) const SECTION_HEADER_HEIGHT: f64 = 25.0;

/// Side/bottom padding inside branch nodes (paddingLeft/Right/Bottom, px).
pub(crate) const SECTION_PADDING: f64 = 10.0;

/// Height reserved above the treemap for the diagram title (px).
pub(crate) const TITLE_HEIGHT: f64 = 30.0;

/// Padding added around the content bounding box for the SVG viewBox (px).
pub(crate) const DIAGRAM_PADDING: f64 = 8.0;

// ---------------------------------------------------------------------------
// Squarify layout
// ---------------------------------------------------------------------------

/// Golden-ratio constant used by the D3 squarify algorithm (φ = (1+√5)/2).
pub(crate) const PHI: f64 = 1.618_033_988_749_895;

// ---------------------------------------------------------------------------
// Font metric
// ---------------------------------------------------------------------------

/// Character-width scaling factor applied to raw font measurements.
/// Approximates an average 0.55 px/px ratio per character at any font size.
#[allow(dead_code)]
pub(crate) const CHAR_WIDTH_FACTOR: f64 = 0.55;

/// Maximum initial font size for leaf labels before shrinking (px).
pub(crate) const MAX_LABEL_FONT: f64 = 38.0;

/// Minimum available width/height for a leaf tile to render a label (px).
pub(crate) const MIN_TILE_AVAIL: f64 = 10.0;

/// Inner tile padding on each edge used to compute available label space (px).
pub(crate) const TILE_INNER_PAD: f64 = 4.0;

/// Gap between the leaf label and the value text (px).
pub(crate) const VALUE_Y_GAP: f64 = 2.0;

/// Scale factor from label font size to value font size.
pub(crate) const VALUE_FONT_SCALE: f64 = 0.6;

/// Minimum value font size (px).
pub(crate) const VALUE_FONT_MIN: u32 = 10;

// ---------------------------------------------------------------------------
// Section clip path
// ---------------------------------------------------------------------------

/// Horizontal clip margin inside section header clip paths (px).
pub(crate) const SECTION_CLIP_MARGIN: f64 = 12.0;

/// Horizontal clip margin inside leaf clip paths (px).
pub(crate) const LEAF_CLIP_MARGIN: f64 = 4.0;

/// X margin from right edge for section value text (px).
pub(crate) const SECTION_VALUE_X_MARGIN: f64 = 10.0;

// ---------------------------------------------------------------------------
// Color palettes (Mermaid default theme v11.15, double-updateColors)
// ---------------------------------------------------------------------------

/// cScale0..11 background colors from the Mermaid default theme.
pub(crate) const C_SCALE: &[&str] = &[
    "hsl(240, 100%, 76.2745098039%)",
    "hsl(60, 100%, 73.5294117647%)",
    "hsl(80, 100%, 76.2745098039%)",
    "hsl(270, 100%, 76.2745098039%)",
    "hsl(300, 100%, 76.2745098039%)",
    "hsl(330, 100%, 76.2745098039%)",
    "hsl(0, 100%, 76.2745098039%)",
    "hsl(30, 100%, 76.2745098039%)",
    "hsl(90, 100%, 76.2745098039%)",
    "hsl(150, 100%, 76.2745098039%)",
    "hsl(180, 100%, 76.2745098039%)",
    "hsl(210, 100%, 76.2745098039%)",
];

/// cScalePeer0..11 border/stroke colors from the Mermaid default theme.
pub(crate) const C_SCALE_PEER: &[&str] = &[
    "hsl(240, 100%, 61.2745098039%)",
    "hsl(60, 100%, 48.5294117647%)",
    "hsl(80, 100%, 56.2745098039%)",
    "hsl(270, 100%, 61.2745098039%)",
    "hsl(300, 100%, 61.2745098039%)",
    "hsl(330, 100%, 61.2745098039%)",
    "hsl(0, 100%, 61.2745098039%)",
    "hsl(30, 100%, 61.2745098039%)",
    "hsl(90, 100%, 61.2745098039%)",
    "hsl(150, 100%, 61.2745098039%)",
    "hsl(180, 100%, 61.2745098039%)",
    "hsl(210, 100%, 61.2745098039%)",
];

/// Label text colors (cScaleLabel): indices 0 and 3 are white, all others black.
pub(crate) fn c_scale_label(idx: usize) -> &'static str {
    match idx % 12 {
        0 | 3 => "#ffffff",
        _ => "black",
    }
}
