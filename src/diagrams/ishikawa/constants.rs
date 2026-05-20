//! Layout and styling constants for the Ishikawa (fishbone) diagram renderer.
#![allow(dead_code)]

use std::f64::consts::PI;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for all Ishikawa labels (px).
pub const FONT_SIZE: f64 = 16.0;

/// Line height for Ishikawa text (= FONT_SIZE × 1.05, px).
#[allow(dead_code)]
pub const LINE_HEIGHT: f64 = FONT_SIZE * 1.05;

// ---------------------------------------------------------------------------
// Spine / branch geometry
// ---------------------------------------------------------------------------

/// Base length of each half-spine (px).
pub const SPINE_BASE_LENGTH: f64 = 250.0;

/// Length of stub sub-bones with no children (px).
pub const BONE_STUB: f64 = 30.0;

/// Base length of sub-bones with children (px).
pub const BONE_BASE: f64 = 60.0;

/// Extra length added per child node for sub-bones (px).
pub const BONE_PER_CHILD: f64 = 5.0;

/// The angle of main branches relative to the spine (radians).
/// Equals 82° in radians — almost vertical.
#[allow(dead_code)]
pub const ANGLE: f64 = (82.0 * PI) / 180.0;

/// cos(82°) — x-component of the unit vector along a main branch.
pub const COS_A: f64 = 0.13917310096006544;

/// sin(82°) — y-component of the unit vector along a main branch.
pub const SIN_A: f64 = 0.9902680687415704;

// ---------------------------------------------------------------------------
// Padding / margins
// ---------------------------------------------------------------------------

/// Outer padding around the diagram (px).
pub const PADDING: f64 = 20.0;

// ---------------------------------------------------------------------------
// Head geometry
// ---------------------------------------------------------------------------

/// Minimum head width before text padding is applied (px).
pub const HEAD_MIN_WIDTH: f64 = 60.0;

/// Minimum head height (px).
pub const HEAD_MIN_HEIGHT: f64 = 80.0;

/// Font size used inside the fish head label (px).
pub const HEAD_LABEL_FONT_SIZE: f64 = FONT_SIZE;

/// X-scale factor for the head bezier control point (horizontal kite extension).
pub const HEAD_CTRL_X_SCALE: f64 = 2.4;

/// X-scale factor for the head label x position.
pub const HEAD_LABEL_X_SCALE: f64 = 0.8;

// ---------------------------------------------------------------------------
// Spine starting offset
// ---------------------------------------------------------------------------

/// Initial x offset from head x=0 where branch placement starts (px).
pub const SPINE_START_OFFSET: f64 = 20.0;

// ---------------------------------------------------------------------------
// Font metric correction
// ---------------------------------------------------------------------------

/// Scale factor applied to measured text widths for layout purposes.
///
/// Mermaid JS uses Arial (measured in a real browser via getBBox()), but we
/// bundle LiberationSans for portability. LiberationSans runs approximately
/// 89.6% of Arial's advance widths at the same point size, so multiplying our
/// measurements by this factor brings computed text-bounding-box positions
/// (and therefore the diagram's horizontal extent) into agreement with the
/// reference SVGs.
///
/// Empirically derived from comparing reference getBBox() values (Arial 16 px)
/// to our LiberationSans measurements across several strings:
///   "SubCause1: [Bad"  ref=125.42 / ours=112.27 → ratio 1.1171
///   "Inappropriate"    ref=94.30  / ours=84.40  → ratio 1.1173
pub const TEXT_WIDTH_SCALE: f64 = 1.117;

/// Scale factor for the head label text at 14px.
/// Liberation Sans at 14px × 1.21 ≈ Arial 14px SVG getBBox (empirical ratio from reference).
pub const HEAD_TEXT_SCALE: f64 = 1.21;
