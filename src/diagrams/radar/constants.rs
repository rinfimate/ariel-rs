//! Layout and styling constants for the radar renderer.

// ---------------------------------------------------------------------------
// Colour palette
// ---------------------------------------------------------------------------

/// Data curve colour palette. Mirrors cScale0-11 from the Mermaid default theme.
pub const CURVE_COLORS: &[&str] = &[
    "#4C8BF5", "#FF6B6B", "#48C9B0", "#F5A623", "#9B59B6", "#3498DB", "#E67E22", "#1ABC9C",
    "#E74C3C", "#2ECC71", "#F39C12", "#8E44AD",
];

// ---------------------------------------------------------------------------
// SVG dimensions
// ---------------------------------------------------------------------------

/// Total SVG width (px).
pub const SVG_WIDTH: f64 = 600.0;

/// Total SVG height (px).
pub const SVG_HEIGHT: f64 = 450.0;

// ---------------------------------------------------------------------------
// Margins
// ---------------------------------------------------------------------------

/// Top margin between the SVG edge and the chart area (px).
pub const MARGIN_TOP: f64 = 40.0;

/// Right margin (px). Extended by LEGEND_WIDTH + gap when a legend is shown.
pub const MARGIN_RIGHT: f64 = 40.0;

/// Bottom margin (px).
pub const MARGIN_BOTTOM: f64 = 40.0;

/// Left margin (px).
pub const MARGIN_LEFT: f64 = 40.0;

// ---------------------------------------------------------------------------
// Legend
// ---------------------------------------------------------------------------

/// Width of the legend column (px).
pub const LEGEND_WIDTH: f64 = 120.0;

/// Side length of each legend colour swatch (px).
pub const LEGEND_BOX: f64 = 12.0;

/// Font size for legend labels (px).
pub const LEGEND_FONT: f64 = 12.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for axis labels (px).
pub const AXIS_LABEL_FONT: f64 = 12.0;

/// Font size for the diagram title (px).
pub const TITLE_FONT: f64 = 18.0;

// ---------------------------------------------------------------------------
// Catmull-Rom spline
// ---------------------------------------------------------------------------

/// Tension factor for the closed Catmull-Rom spline (α = 0.5, standard).
pub const CATMULL_ROM_ALPHA: f64 = 0.5;

// ---------------------------------------------------------------------------
// Axis label placement
// ---------------------------------------------------------------------------

/// Distance beyond the radius tip at which axis labels are placed (px).
pub const AXIS_LABEL_RADIUS_OFFSET: f64 = 18.0;

/// Cosine threshold for classifying a label as left- or right-aligned vs centred.
pub const AXIS_LABEL_ANCHOR_THRESHOLD: f64 = 0.1;
