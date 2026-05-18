//! Layout and styling constants for the Cynefin diagram renderer.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Diagram geometry
// ---------------------------------------------------------------------------

/// Default diagram width (px).
pub const DIAGRAM_WIDTH: f64 = 800.0;

/// Default diagram height (px).
pub const DIAGRAM_HEIGHT: f64 = 600.0;

/// Outer padding around the diagram content (px).
pub const PADDING: f64 = 24.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for domain labels and item text (px).
pub const FONT_SIZE: f64 = 14.0;

// ---------------------------------------------------------------------------
// Boundary behaviour
// ---------------------------------------------------------------------------

/// Default boundary amplitude; 0.0 triggers automatic (1.5% of width/height).
pub const BOUNDARY_AMPLITUDE_DEFAULT: f64 = 0.0;

// ---------------------------------------------------------------------------
// Confusion domain
// ---------------------------------------------------------------------------

/// Maximum number of items rendered inside the Confusion region.
pub const MAX_CONFUSION_ITEMS: usize = 3;

// ---------------------------------------------------------------------------
// Domain background colours
// ---------------------------------------------------------------------------

/// Background fill for the Complex quadrant.
pub const COMPLEX_BG: &str = "#e8f4f8";

/// Background fill for the Complicated quadrant.
pub const COMPLICATED_BG: &str = "#f0ffe0";

/// Background fill for the Chaotic quadrant.
pub const CHAOTIC_BG: &str = "#fff0f0";

/// Background fill for the Clear quadrant.
pub const CLEAR_BG: &str = "#fffff0";

/// Background fill for the Confusion ellipse.
pub const CONFUSION_BG: &str = "#f5f5f5";

// ---------------------------------------------------------------------------
// Config flags
// ---------------------------------------------------------------------------

/// Whether to show model/practice subtitle lines under domain names.
pub const SHOW_DOMAIN_DESCRIPTIONS: bool = true;

// ---------------------------------------------------------------------------
// Item badge geometry
// ---------------------------------------------------------------------------

/// Height of an item badge rectangle (px).
pub const ITEM_HEIGHT: f64 = 26.0;

/// Horizontal padding inside an item badge (px).
pub const ITEM_PADDING_X: f64 = 10.0;

// ---------------------------------------------------------------------------
// Cynefin boundary parameters
// ---------------------------------------------------------------------------

/// Horizontal fraction of diagram width used for the Confusion ellipse rx.
pub const CONFUSION_RX_FRAC: f64 = 0.15;

/// Vertical fraction of diagram height used for the Confusion ellipse ry.
pub const CONFUSION_RY_FRAC: f64 = 0.15;

/// Fraction of diagram width used for cliff control point x-offset leftward (cp1).
pub const CLIFF_CP1X_FRAC: f64 = 0.08;

/// Fraction of diagram height used for cliff control point y (cp1).
pub const CLIFF_CP1Y_FRAC: f64 = 0.25;

/// Fraction of diagram width used for cliff control point x-offset rightward (cp2).
pub const CLIFF_CP2X_FRAC: f64 = 0.08;

/// Fraction of diagram height used for cliff control point y (cp2).
pub const CLIFF_CP2Y_FRAC: f64 = 0.1;

/// Number of cubic bezier segments used for each wavy boundary path.
pub const BOUNDARY_SEGMENTS: usize = 6;
