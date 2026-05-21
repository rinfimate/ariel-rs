//! Layout and styling constants for the requirement renderer.

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size used throughout the requirement diagram (px).
/// defaultConfig says 14 but Mermaid renders at the global CSS font-size (16px effective).
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

/// Minimum box width (px). Natural sizing — Mermaid's rect_min_width=200 is overridden by theme.
#[allow(dead_code)]
pub const MIN_WIDTH: f64 = 0.0;
/// Minimum box height (px). Natural sizing — Mermaid's rect_min_height=200 is overridden by theme.
#[allow(dead_code)]
pub const MIN_HEIGHT: f64 = 0.0;

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
// Colours — resolved at render time from ThemeVars
// ---------------------------------------------------------------------------
// box_fill      = vars.primary_color   (#ECECFF default / #1f2020 dark)
// box_stroke    = vars.primary_border  (#9370DB default / #81B1DB dark)
// font_color    = vars.primary_text    (#333333 default / #cde498 dark)
// line_color    = vars.line_color      (#333333 default / #81B1DB dark)

// ---------------------------------------------------------------------------
// Text measurement
// ---------------------------------------------------------------------------

/// Effective space advance width at 16 px Arial in browser string measurements (px).
/// measure_browser() stores 0 for space, so tmw() adds this per space character.
/// Calibrated to 4.0555: matches browser measurement for multi-space requirement texts.
pub const SPACE_W_16: f64 = 4.0555;

/// Safety margin added to text width measurements to prevent last-letter clipping (px).
/// Set to 0: browser-measured character widths already account for glyph advance, so no
/// extra margin is needed (adding 6px caused boxes to be ~6px wider than the reference).
pub const TEXT_SAFETY_MARGIN: f64 = 0.0;
