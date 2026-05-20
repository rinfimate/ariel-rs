//! Canvas and styling constants for the wardley renderer.
//!
//! These match the Mermaid JS wardleyRenderer.ts getConfigValues() defaults.

// ---------------------------------------------------------------------------
// Canvas dimensions (matching wardleyRenderer.ts defaults)
// ---------------------------------------------------------------------------

/// Padding on all four sides of the chart area (px).
pub const PADDING: f64 = 48.0;

// ---------------------------------------------------------------------------
// Node geometry
// ---------------------------------------------------------------------------

/// Radius of component and anchor node circles (px).
pub const NODE_RADIUS: f64 = 6.0;

/// Default pixel offset from node center to label (px).
pub const NODE_LABEL_OFFSET: f64 = 8.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for node labels and link labels (px).
pub const LABEL_FONT_SIZE: f64 = 10.0;

/// Font size for axis labels and stage labels (px).
pub const AXIS_FONT_SIZE: f64 = 12.0;

/// Font size for stage labels (axis_font_size - 2).
pub const STAGE_FONT_SIZE: f64 = 10.0;

/// Font size for the diagram title (axis_font_size * 1.05).
pub const TITLE_FONT_SIZE: f64 = 12.6;
