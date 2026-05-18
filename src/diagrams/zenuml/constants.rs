//! Layout constants for the ZenUML sequence diagram renderer.

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Default font size for all ZenUML diagram text (px).
pub const FONT_SIZE: f64 = 14.0;

// ---------------------------------------------------------------------------
// Participant geometry
// ---------------------------------------------------------------------------

/// Width of each participant box (px).
pub const PART_WIDTH: f64 = 100.0;

/// Height of each participant box (px).
pub const PART_HEIGHT: f64 = 40.0;

/// Horizontal gap between adjacent participant boxes (px).
pub const PART_SPACING: f64 = 40.0;

// ---------------------------------------------------------------------------
// Lifeline / message geometry
// ---------------------------------------------------------------------------

/// Y coordinate where the lifeline starts (below the participant box, px).
/// Equals PART_HEIGHT + 10.
pub const LIFELINE_TOP: f64 = PART_HEIGHT + 10.0;

/// Vertical spacing between consecutive messages (px).
pub const MSG_SPACING: f64 = 40.0;

/// Width of an activation bar on a lifeline (px).
pub const ACTIVATION_W: f64 = 10.0;

// ---------------------------------------------------------------------------
// Block / control structure
// ---------------------------------------------------------------------------

/// Padding inside control-structure blocks (px).
pub const BLOCK_PADDING: f64 = 10.0;

// ---------------------------------------------------------------------------
// Diagram padding
// ---------------------------------------------------------------------------

/// General padding added around the diagram content (px).
pub const PADDING: f64 = 20.0;
