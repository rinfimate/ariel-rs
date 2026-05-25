//! Layout and styling constants for the sequence diagram renderer.

// ---------------------------------------------------------------------------
// Actor geometry
// ---------------------------------------------------------------------------

/// Default horizontal margin between adjacent actors (conf.actorMargin, px).
pub const ACTOR_MARGIN: f64 = 50.0;

/// Default actor box width (conf.width, px).
pub const ACTOR_WIDTH: f64 = 150.0;

/// Default actor box height for layout purposes (conf.height, px).
/// Used for spacing regardless of participant type.
pub const ACTOR_HEIGHT: f64 = 65.0;

/// Visual height of the actor-man (stick-figure) participant type (px).
pub const ACTOR_MAN_HEIGHT: f64 = 80.0;

// ---------------------------------------------------------------------------
// Box / control structure margins
// ---------------------------------------------------------------------------

/// Outer box margin around loop/alt/opt/par control structures (conf.boxMargin, px).
pub const BOX_MARGIN: f64 = 10.0;

/// Inner text margin inside control-structure label boxes (conf.boxTextMargin, px).
pub const BOX_TEXT_MARGIN: f64 = 5.0;

// ---------------------------------------------------------------------------
// Note geometry
// ---------------------------------------------------------------------------

/// Margin around note text (conf.noteMargin, px).
pub const NOTE_MARGIN: f64 = 10.0;

// ---------------------------------------------------------------------------
// Control-structure label box
// ---------------------------------------------------------------------------

/// Width of the label-badge pentagon on loop/alt/opt/par boxes (conf.labelBoxWidth, px).
pub const LABEL_BOX_WIDTH: f64 = 50.0;

/// Height of the label-badge pentagon on loop/alt/opt/par boxes (conf.labelBoxHeight, px).
pub const LABEL_BOX_HEIGHT: f64 = 20.0;

// ---------------------------------------------------------------------------
// Text / wrapping
// ---------------------------------------------------------------------------

/// Wrap-padding added on each side of a message text measurement (conf.wrapPadding, px).
pub const WRAP_PADDING: f64 = 10.0;

// ---------------------------------------------------------------------------
// Diagram margins
// ---------------------------------------------------------------------------

/// Horizontal margin on each side of the diagram viewBox (px).
pub const DIAGRAM_MARGIN_X: f64 = 50.0;

/// Vertical margin on each side of the diagram viewBox (px).
pub const DIAGRAM_MARGIN_Y: f64 = 10.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Default font size used for all text in sequence diagrams (px).
pub const FONT_SIZE: f64 = 16.0;

// ---------------------------------------------------------------------------
// Activation boxes
// ---------------------------------------------------------------------------

/// Width of an activation bar on a participant lifeline (conf.activationWidth, px).
pub const ACTIVATION_WIDTH: f64 = 10.0;

// ---------------------------------------------------------------------------
// SVG identifiers
// ---------------------------------------------------------------------------

/// Fixed id attribute for the sequence diagram SVG root element.
pub const DIAGRAM_ID: &str = "mermaid-seq";
