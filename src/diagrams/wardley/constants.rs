//! Canvas and styling constants for the wardley renderer.

// ---------------------------------------------------------------------------
// Canvas scaling
// ---------------------------------------------------------------------------

/// Scale factor converting percentage coordinates to SVG pixels (px/%).
pub const SCALE: f64 = 6.0;

/// Axis padding added around the plot area on each side (px).
pub const PADDING: f64 = 30.0;

// ---------------------------------------------------------------------------
// Node geometry
// ---------------------------------------------------------------------------

/// Radius of component and anchor node circles (px).
pub const NODE_RADIUS: f64 = 8.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for node labels and link labels (px).
pub const FONT_SIZE: f64 = 12.0;

/// Font size for axis labels and stage labels (px).
pub const AXIS_FONT: f64 = 11.0;

/// Font size for the diagram title (px).
pub const TITLE_FONT: f64 = 16.0;

// ---------------------------------------------------------------------------
// Stage background colours
// ---------------------------------------------------------------------------

/// Fill colour for even-indexed evolution stage backgrounds.
pub const STAGE_FILL_EVEN: &str = "#f9f9fb";

/// Fill colour for odd-indexed evolution stage backgrounds.
pub const STAGE_FILL_ODD: &str = "#f0f0f5";
