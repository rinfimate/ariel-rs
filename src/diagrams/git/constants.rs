//! Layout and styling constants for the git graph renderer.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Layout geometry
// ---------------------------------------------------------------------------

/// Offset added to each commit's axis position (px).
pub const LAYOUT_OFFSET: f64 = 10.0;

/// Distance between consecutive commits on the primary axis (px).
pub const COMMIT_STEP: f64 = 40.0;

/// Default starting position for TB/BT diagrams (px).
pub const DEFAULT_POS: f64 = 30.0;

// ---------------------------------------------------------------------------
// Commit visual geometry
// ---------------------------------------------------------------------------

/// Radius of the commit circle (px).
pub const COMMIT_RADIUS: f64 = 10.0;

/// Number of CSS colour classes cycling through the git theme palette.
pub const THEME_COLOR_LIMIT: usize = 8;

// ---------------------------------------------------------------------------
// Branch labels
// ---------------------------------------------------------------------------

/// Horizontal padding inside a branch label box on each side (px).
pub const BRANCH_LABEL_PADDING: f64 = 10.0;

/// Font size for branch name labels (px).
pub const BRANCH_FONT_SIZE: f64 = 16.0;

/// Scale factor to compensate for difference between bundled Arial and browser-rendered Arial (LR branches).
/// Browser metrics are ~7.5% wider per character at 16px.
pub const BRANCH_FONT_SCALE: f64 = 1.075;

// ---------------------------------------------------------------------------
// Commit labels
// ---------------------------------------------------------------------------

/// Scale factor for commit label text (hex-like strings at 10px, ~14% wider in browser).
pub const COMMIT_LABEL_FONT_SCALE: f64 = 1.14;

/// Scale factor for tag text (short labels at 10px, ~27% wider in browser).
pub const TAG_TEXT_FONT_SCALE: f64 = 1.27;

/// Whether to render commit ID labels below each commit.
pub const SHOW_COMMIT_LABEL: bool = true;

/// Whether to rotate commit labels 45°. Matches defaultConfig.gitGraph.rotateCommitLabel = true.
pub const ROTATE_COMMIT_LABEL: bool = true;

// ---------------------------------------------------------------------------
// Arrow arcs
// ---------------------------------------------------------------------------

/// Arc radius used for cross-lane arrows in LR mode (px).
pub const LR_ARC_RADIUS: f64 = 20.0;

/// Arc radius used for cross-lane arrows in TB/BT mode (px).
pub const TB_ARC_RADIUS: f64 = 10.0;

// ---------------------------------------------------------------------------
// Branch colours are now served from ThemeVars:
//   vars.git_branch_colors[i % 8]  — (fill, stroke) tuples
// ---------------------------------------------------------------------------

// Text colours for branch label text also from ThemeVars:
//   vars.git_branch_label_text_colors[i % 8]
