//! Layout and styling constants for the Ishikawa (fishbone) diagram renderer.
#![allow(dead_code)]

use std::f64::consts::PI;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for all Ishikawa labels (px).
pub const FONT_SIZE: f64 = 14.0;

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
