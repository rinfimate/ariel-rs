//! Layout and styling constants for the event modeling renderer.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Swimlane geometry
// ---------------------------------------------------------------------------

/// Vertical padding inside each swimlane (above and below boxes, px).
pub const SWIMLANE_PADDING: f64 = 20.0;

/// Vertical gap between swimlane strips (px).
pub const SWIMLANE_GAP: f64 = 0.0;

/// Horizontal x position of swimlane labels (px).
pub const SWIMLANE_LABEL_X: f64 = 30.0;

/// Y offset from swimlane top to swimlane label baseline (px).
pub const SWIMLANE_LABEL_Y_OFFSET: f64 = 30.0;

// ---------------------------------------------------------------------------
// Box geometry
// ---------------------------------------------------------------------------

/// Horizontal padding inside each box (px).
pub const BOX_PADDING: f64 = 10.0;

/// Minimum box width (px).
pub const BOX_MIN_WIDTH: f64 = 100.0;

/// Maximum box width (px).
pub const BOX_MAX_WIDTH: f64 = 200.0;

/// Minimum box height (px).
pub const BOX_MIN_HEIGHT: f64 = 60.0;

/// Maximum box height (px).
pub const BOX_MAX_HEIGHT: f64 = 120.0;

// ---------------------------------------------------------------------------
// Layout geometry
// ---------------------------------------------------------------------------

/// Left margin for swimlane labels / content start x (px).
pub const CONTENT_START_X: f64 = 160.0;

/// Outer padding around the entire diagram (px).
pub const DIAGRAM_PADDING: f64 = 30.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for box labels and swimlane text (px).
pub const FONT_SIZE: f64 = 13.0;

// ---------------------------------------------------------------------------
// Box overlap (unused — kept for reference)
// ---------------------------------------------------------------------------

/// Boxes in adjacent swimlanes can overlap vertically by this amount (px).
#[allow(dead_code)]
pub const BOX_OVERLAP: f64 = 20.0;
