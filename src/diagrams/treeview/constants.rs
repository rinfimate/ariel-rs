//! Layout and styling constants for the treeView renderer.

// ---------------------------------------------------------------------------
// Font / text geometry
// ---------------------------------------------------------------------------

/// Font size used for node labels (px). Matches Mermaid treeView default.
pub const FONT_SIZE: f64 = 16.0;

/// Vertical distance between consecutive node centre-lines (px).
/// Derived from the reference SVG: node_y values differ by exactly this amount.
pub const ROW_HEIGHT: f64 = 27.015625;

// ---------------------------------------------------------------------------
// Horizontal layout
// ---------------------------------------------------------------------------

/// Left padding for the root node text (px). Text x = INDENT_STEP * depth + LEFT_PAD.
pub const LEFT_PAD: f64 = 5.0;

/// Horizontal indent added per tree depth level (px).
pub const INDENT_STEP: f64 = 15.0;

/// Length of the horizontal connector line between the vertical trunk and
/// the node text (px). The line runs from (text_x - INDENT_STEP) to (text_x - 5).
/// Equivalently: x1 = text_x - INDENT_STEP, x2 = text_x - 5.
pub const H_LINE_GAP: f64 = 5.0;

// ---------------------------------------------------------------------------
// ViewBox adjustments
// ---------------------------------------------------------------------------

/// viewBox min-x offset (px). Matches Mermaid reference SVG.
pub const VIEWBOX_X: f64 = -0.5;

/// viewBox min-y offset (px).
pub const VIEWBOX_Y: f64 = 0.0;

/// Right-side padding added to the maximum text right-edge to obtain the
/// viewBox width (px).
pub const VIEWBOX_RIGHT_PAD: f64 = 1.0;

// ---------------------------------------------------------------------------
// Vertical connector
// ---------------------------------------------------------------------------

/// The "half-stroke" tweak added to the bottom end of each vertical connector
/// line so it cleanly meets the centre of the last child's horizontal line.
pub const V_LINE_BOTTOM_TWEAK: f64 = 0.5;

// ---------------------------------------------------------------------------
// Text measurement
// ---------------------------------------------------------------------------

/// Chrome getBBox() includes a trailing advance of ~4.4375 px at 16 px Arial.
/// Added to the measured text width to obtain the actual right edge of each label.
pub const TRAILING_ADVANCE: f64 = 4.4375;
