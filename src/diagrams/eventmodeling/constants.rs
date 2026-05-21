//! Layout and styling constants for the event modeling renderer.
//!
//! All values are faithfully ported from Mermaid JS `db.ts` → `diagramProps`
//! and `renderer.ts` → default config.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Swimlane geometry (diagramProps)
// ---------------------------------------------------------------------------

/// Minimum swimlane height in px (`swimlaneMinHeight`).
pub const SWIMLANE_MIN_HEIGHT: f64 = 70.0;

/// Vertical padding inside each swimlane strip, above and below boxes (`swimlanePadding`).
pub const SWIMLANE_PADDING: f64 = 15.0;

/// Vertical gap between swimlane strips (`swimlaneGap`).
pub const SWIMLANE_GAP: f64 = 10.0;

// ---------------------------------------------------------------------------
// Box geometry (diagramProps)
// ---------------------------------------------------------------------------

/// Padding between boxes and inside boxes (`boxPadding`).
pub const BOX_PADDING: f64 = 10.0;

/// Horizontal overlap between a new box and the previous swimlane's last box (`boxOverlap`).
pub const BOX_OVERLAP: f64 = 90.0;

/// Default box Y offset (unused; swimlanePadding drives placement) (`boxDefaultY`).
pub const BOX_DEFAULT_Y: f64 = 0.0;

/// Minimum box width in px (`boxMinWidth`).
pub const BOX_MIN_WIDTH: f64 = 80.0;

/// Maximum box width in px (`boxMaxWidth`).
pub const BOX_MAX_WIDTH: f64 = 450.0;

/// Minimum box height in px (`boxMinHeight`).
pub const BOX_MIN_HEIGHT: f64 = 80.0;

/// Maximum box height in px (`boxMaxHeight`).
pub const BOX_MAX_HEIGHT: f64 = 750.0;

// ---------------------------------------------------------------------------
// Content layout (diagramProps)
// ---------------------------------------------------------------------------

/// X position of the first box (left margin for swimlane labels) (`contentStartX`).
pub const CONTENT_START_X: f64 = 250.0;

/// Maximum text width for word-wrapping inside a box (`textMaxWidth = boxMaxWidth - 2*boxPadding`).
pub const TEXT_MAX_WIDTH: f64 = 430.0;

/// Inner text padding inside each box (`boxTextPadding`).
pub const BOX_TEXT_PADDING: f64 = 10.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for box labels (px). Mermaid uses 16 px bold for box text.
pub const FONT_SIZE: f64 = 16.0;

/// Font size for swimlane labels (px). Mermaid renders swimlane text at the same size.
pub const SWIMLANE_FONT_SIZE: f64 = 16.0;

/// X position of swimlane label text.
pub const SWIMLANE_LABEL_X: f64 = 30.0;

/// Y offset from swimlane.y to swimlane label baseline.
pub const SWIMLANE_LABEL_Y_OFFSET: f64 = 30.0;

// ---------------------------------------------------------------------------
// Diagram padding (setupGraphViewbox → config.padding)
// ---------------------------------------------------------------------------

/// Outer padding around the full diagram (`config.padding`).
pub const DIAGRAM_PADDING: f64 = 30.0;

// ---------------------------------------------------------------------------
// Swimlane indices (calculateSwimlaneProps)
// ---------------------------------------------------------------------------

/// Swimlane index for UI / processor frames (top row).
pub const SWIMLANE_UI_IDX: i64 = 0;

/// Swimlane index for Command / ReadModel frames (middle row).
pub const SWIMLANE_CMD_IDX: i64 = 100;

/// Swimlane index for Event frames (bottom row).
pub const SWIMLANE_EVT_IDX: i64 = 200;

/// Boundary between namespace sub-swimlanes inside the UI band.
pub const SWIMLANE_UI_MAX: i64 = 100;

/// Boundary between namespace sub-swimlanes inside the CMD band.
pub const SWIMLANE_CMD_MAX: i64 = 200;

/// Boundary between namespace sub-swimlanes inside the EVT band.
pub const SWIMLANE_EVT_MAX: i64 = 300;

// ---------------------------------------------------------------------------
// Swimlane default labels (diagramProps)
// ---------------------------------------------------------------------------

pub const LABEL_UI_AUTOMATION: &str = "UI/Automation";
pub const LABEL_UI_AUTOMATION_PREFIX: &str = "UI/A: ";
pub const LABEL_COMMAND_READMODEL: &str = "Command/Read Model";
pub const LABEL_COMMAND_READMODEL_PREFIX: &str = "C/RM: ";
pub const LABEL_EVENTS: &str = "Events";
pub const LABEL_EVENTS_PREFIX: &str = "Stream: ";
