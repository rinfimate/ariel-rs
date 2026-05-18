//! Layout and styling constants for the requirement renderer.

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size used throughout the requirement diagram (px). Matches Mermaid CSS `font-size:16px`.
pub const FONT_SIZE: f64 = 16.0;

// ---------------------------------------------------------------------------
// Box geometry
// ---------------------------------------------------------------------------

/// Horizontal padding inside each requirement/element box (px).
pub const PAD_X: f64 = 10.0;

/// Vertical padding above the first body item inside a box (px).
pub const PAD_Y: f64 = 8.0;

/// Row height (vertical spacing between body items inside a box, px).
pub const ROW_H: f64 = 24.0;

/// Height of the header section inside each node box (px).
/// HEADER_H = PAD_Y + 2×ROW_H + ROW_H/2 = 8 + 48 + 12 = 68.
pub const HEADER_H: f64 = 68.0;

/// Minimum allowed box width (currently 0 — no artificial minimum).
pub const MIN_WIDTH: f64 = 0.0;

// ---------------------------------------------------------------------------
// Dagre layout parameters
// ---------------------------------------------------------------------------

/// Node separation used by dagre (px).
pub const NODE_SEP: f64 = 50.0;

/// Rank separation used by dagre (px). Dagre adds edge-label height (18) giving total gap of 74.
pub const RANK_SEP: f64 = 56.0;

/// Horizontal margin around the dagre graph (px).
pub const MARGIN_X: f64 = 8.0;

/// Vertical margin around the dagre graph (px).
pub const MARGIN_Y: f64 = 8.0;

// ---------------------------------------------------------------------------
// Colours
// ---------------------------------------------------------------------------

/// Fill colour for requirement boxes. Matches Mermaid neo/classic theme.
pub const BOX_FILL: &str = "#ECECFF";

/// Stroke colour for requirement box borders.
pub const BOX_STROKE: &str = "#9370DB";

/// Fill colour for element boxes.
pub const ELEM_FILL: &str = "#ECECFF";

/// Stroke colour for element box borders.
pub const ELEM_STROKE: &str = "#9370DB";

/// Text fill colour for all labels.
pub const FONT_COLOR: &str = "#333";

/// Stroke colour for relationship edges and markers.
pub const REL_COLOR: &str = "#333333";

// ---------------------------------------------------------------------------
// Text measurement
// ---------------------------------------------------------------------------

/// Effective space advance width at 16 px Arial in browser string measurements (px).
/// Actual isolate space = 4.453125 px; empirically calibrated to 4.0 to match reference.
pub const SPACE_W_16: f64 = 4.0;

/// Safety margin added to text width measurements to prevent last-letter clipping (px).
pub const TEXT_SAFETY_MARGIN: f64 = 6.0;
