//! Layout and styling constants for the railroad renderer.

// ---------------------------------------------------------------------------
// Layout geometry (DEFAULT_RAILROAD_CONFIG)
// ---------------------------------------------------------------------------

/// Padding inside each terminal/non-terminal box (px).
pub const PADDING: f64 = 10.0;

/// Vertical separation between stacked choice/repetition alternatives (px).
pub const VERTICAL_SEP: f64 = 8.0;

/// Horizontal separation between consecutive sequence elements (px).
pub const HORIZONTAL_SEP: f64 = 10.0;

/// Arc radius used for choice, optional and repetition paths (px).
pub const ARC_RADIUS: f64 = 10.0;

/// Font size for all text in railroad diagrams (px).
pub const FONT_SIZE: f64 = 14.0;

/// Radius of the start/end marker circles (px).
pub const MARKER_RADIUS: f64 = 5.0;

// ---------------------------------------------------------------------------
// Colours
// ---------------------------------------------------------------------------

/// Fill colour for terminal (rounded rectangle) nodes.
pub const TERMINAL_FILL: &str = "#FFFFC0";

/// Fill colour for non-terminal (plain rectangle) nodes.
pub const NONTERMINAL_FILL: &str = "#FFFFFF";

/// Stroke colour for terminal boxes and lines.
pub const TERMINAL_STROKE: &str = "#000000";

/// Stroke colour for connector lines.
pub const LINE_COLOR: &str = "#000000";

/// Stroke width for all shapes and lines (px).
pub const STROKE_WIDTH: f64 = 2.0;

/// Fill colour for rule-name text labels.
pub const RULE_NAME_COLOR: &str = "#000066";

/// Fill colour for special node boxes.
#[allow(dead_code)]
pub const SPECIAL_FILL: &str = "#F0E0FF";

/// Stroke colour for special node boxes.
#[allow(dead_code)]
pub const SPECIAL_STROKE: &str = "#8800CC";
