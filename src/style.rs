//! Shared styling types and constants for diagram renderers.
//!
//! This module centralises CSS colour values, stroke specifications, and
//! font-size constants that are used by multiple diagram renderers.  Nothing
//! in this module is wired into the renderers yet; that migration will happen
//! in a later phase.  The `dead_code` lint is suppressed at the module level
//! because every item here will be used once individual renderers migrate.
#![allow(dead_code)]

// ─────────────────────────────────────────────────────────────────────────────
// Color type
// ─────────────────────────────────────────────────────────────────────────────

/// A CSS colour value (hex, hsl, named, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub &'static str);

impl Color {
    /// Return the colour value as a string slice.
    pub fn as_str(self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Stroke type
// ─────────────────────────────────────────────────────────────────────────────

/// A stroke specification (colour + width).
#[derive(Debug, Clone, Copy)]
pub struct Stroke {
    /// The stroke colour.
    pub color: Color,
    /// The stroke width in pixels.
    pub width: f64,
}

impl Stroke {
    /// Create a new `Stroke` from a colour and width.
    pub const fn new(color: Color, width: f64) -> Self {
        Self { color, width }
    }

    /// Render as an inline SVG style string, e.g. `"stroke:#333333;stroke-width:2px;"`.
    pub fn as_style(self) -> String {
        format!("stroke:{};stroke-width:{}px;", self.color, self.width)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Universal / utility colours
// ─────────────────────────────────────────────────────────────────────────────

/// Fully transparent.  Used where SVG `fill="none"` or CSS `transparent` is needed.
pub const TRANSPARENT: Color = Color("transparent");

/// Pure white — `#ffffff`.  Background of many diagram elements and edge labels.
pub const WHITE: Color = Color("#ffffff");

/// Pure black — `#000000`.  Used for terminal strokes, sequence crosshead markers, etc.
pub const BLACK: Color = Color("#000000");

// ─────────────────────────────────────────────────────────────────────────────
// Mermaid default-theme primary palette
// ─────────────────────────────────────────────────────────────────────────────

/// Primary node fill — `#ECECFF`.
///
/// Used by: requirement boxes, entity boxes, block cylinders, pie slice 1,
/// quadrant-1 fill, git tag background, block nodes, flowchart nodes.
pub const PRIMARY_COLOR: Color = Color("#ECECFF");

/// Primary node border / stroke — `#9370DB` (medium purple).
///
/// Used by: requirement boxes, entity boxes, block cylinders, flowchart node
/// strokes, neo-look node strokes.
pub const PRIMARY_BORDER: Color = Color("#9370DB");

/// Secondary colour — `#ffffde` (pale yellow).
///
/// Used by: cluster backgrounds, commit-label backgrounds, pie slice 2,
/// note-related cluster fills (some contexts).
pub const SECONDARY_COLOR: Color = Color("#ffffde");

/// Secondary border — `#aaaa33` (dark yellow-green).
///
/// Used by: cluster borders, note strokes.
pub const SECONDARY_BORDER: Color = Color("#aaaa33");

/// Tertiary colour — `#fff0f0` (very light pink).  Default theme tertiary fill.
pub const TERTIARY_COLOR: Color = Color("#fff0f0");

/// Tertiary border — `#ff0000` (red).  Default theme tertiary stroke.
pub const TERTIARY_BORDER: Color = Color("#ff0000");

// ─────────────────────────────────────────────────────────────────────────────
// Common line / text colours
// ─────────────────────────────────────────────────────────────────────────────

/// Standard connector / edge line colour — `#333333`.
///
/// Used by: flowchart edges, arrowheads, markers, ER relationship lines,
/// requirement edges, architecture edges, block edges.
pub const LINE_COLOR: Color = Color("#333333");

/// Standard font / text colour — `#333333`.
///
/// Used by: label text, cluster label text, title text, flowchart labels,
/// block labels, architecture node labels, requirement text.
pub const FONT_COLOR: Color = Color("#333333");

/// Abbreviated dark text — `#333` (identical to `#333333` in CSS).
///
/// Some renderers use the 3-digit shorthand; this constant captures it exactly.
pub const FONT_COLOR_SHORT: Color = Color("#333");

/// Near-black text used in quadrant and xy-chart renderers — `#131300`.
///
/// Used by: quadrant label fills, xy-chart axis/title colours,
/// class-diagram edge-label text.
pub const DARK_TEXT: Color = Color("#131300");

// ─────────────────────────────────────────────────────────────────────────────
// Note colours
// ─────────────────────────────────────────────────────────────────────────────

/// Note box fill — `#fff5ad` (pale yellow).
///
/// Used by: sequence diagram notes, state diagram notes/annotations.
pub const NOTE_FILL: Color = Color("#fff5ad");

/// Note box stroke — `#aaaa33` (same as `SECONDARY_BORDER`).
///
/// Used by: sequence diagram notes, state diagram notes/annotations.
pub const NOTE_STROKE: Color = Color("#aaaa33");

// ─────────────────────────────────────────────────────────────────────────────
// Error / diagnostic colours
// ─────────────────────────────────────────────────────────────────────────────

/// Error icon / text fill — `#552222` (dark red).
///
/// Used by: flowchart and block `.error-icon` and `.error-text` CSS rules.
pub const ERROR_COLOR: Color = Color("#552222");

// ─────────────────────────────────────────────────────────────────────────────
// Edge-label background
// ─────────────────────────────────────────────────────────────────────────────

/// Edge-label rectangle background — semi-transparent light grey.
///
/// Used as `fill` in flowchart / block edge-label `<rect>` elements:
/// `rgba(232,232,232, 0.8)`.
pub const EDGE_LABEL_BG: Color = Color("rgba(232,232,232, 0.8)");

// ─────────────────────────────────────────────────────────────────────────────
// C4 diagram colours
// ─────────────────────────────────────────────────────────────────────────────

/// C4 Person element fill — `#08427B` (dark navy blue).
pub const C4_PERSON_FILL: Color = Color("#08427B");

/// C4 Person element stroke — `#073B6F`.
pub const C4_PERSON_STROKE: Color = Color("#073B6F");

/// C4 external element fill — `#999999` (medium grey).
pub const C4_EXT_FILL: Color = Color("#999999");

/// C4 external element stroke — `#8A8A8A`.
pub const C4_EXT_STROKE: Color = Color("#8A8A8A");

/// C4 internal (system/container/component) element fill — `#1168BD` (medium blue).
pub const C4_INTERNAL_FILL: Color = Color("#1168BD");

/// C4 internal element stroke — `#3C7FC0`.
pub const C4_INTERNAL_STROKE: Color = Color("#3C7FC0");

/// C4 boundary stroke and dark text colour — `#444444`.
pub const C4_BOUNDARY_STROKE: Color = Color("#444444");

/// White text used on dark C4 element backgrounds — `#FFFFFF`.
pub const C4_LIGHT_TEXT: Color = Color("#FFFFFF");

// ─────────────────────────────────────────────────────────────────────────────
// Railroad diagram colours
// ─────────────────────────────────────────────────────────────────────────────

/// Railroad terminal node fill — `#FFFFC0` (pale yellow).
pub const RAILROAD_TERMINAL_FILL: Color = Color("#FFFFC0");

/// Railroad non-terminal node fill — `#FFFFFF` (white).
pub const RAILROAD_NONTERMINAL_FILL: Color = Color("#FFFFFF");

/// Railroad rule-name label colour — `#000066` (dark blue).
pub const RAILROAD_RULE_NAME_COLOR: Color = Color("#000066");

/// Railroad special node fill — `#F0E0FF` (pale lavender).
pub const RAILROAD_SPECIAL_FILL: Color = Color("#F0E0FF");

/// Railroad special node stroke — `#8800CC` (purple).
pub const RAILROAD_SPECIAL_STROKE: Color = Color("#8800CC");

// ─────────────────────────────────────────────────────────────────────────────
// Architecture diagram colours
// ─────────────────────────────────────────────────────────────────────────────

/// Architecture service-icon background fill — `#087ebf` (blue).
pub const ARCH_ICON_FILL: Color = Color("#087ebf");

/// Architecture node-group border — `hsl(240, 60%, 86.2745098039%)`.
pub const ARCH_GROUP_STROKE: Color = Color("hsl(240, 60%, 86.2745098039%)");

// ─────────────────────────────────────────────────────────────────────────────
// Wardley map colours
// ─────────────────────────────────────────────────────────────────────────────

/// Wardley even-stage background fill — `#f9f9fb` (near white).
pub const WARDLEY_STAGE_EVEN: Color = Color("#f9f9fb");

/// Wardley odd-stage background fill — `#f0f0f5` (very light lavender).
pub const WARDLEY_STAGE_ODD: Color = Color("#f0f0f5");

// ─────────────────────────────────────────────────────────────────────────────
// Cynefin diagram colours
// ─────────────────────────────────────────────────────────────────────────────

/// Cynefin Complex quadrant background — `#e8f4f8`.
pub const CYNEFIN_COMPLEX_BG: Color = Color("#e8f4f8");

/// Cynefin Complicated quadrant background — `#f0ffe0`.
pub const CYNEFIN_COMPLICATED_BG: Color = Color("#f0ffe0");

/// Cynefin Chaotic quadrant background — `#fff0f0`.
pub const CYNEFIN_CHAOTIC_BG: Color = Color("#fff0f0");

/// Cynefin Clear quadrant background — `#fffff0`.
pub const CYNEFIN_CLEAR_BG: Color = Color("#fffff0");

/// Cynefin Confusion ellipse background — `#f5f5f5`.
pub const CYNEFIN_CONFUSION_BG: Color = Color("#f5f5f5");

// ─────────────────────────────────────────────────────────────────────────────
// Mindmap root colour
// ─────────────────────────────────────────────────────────────────────────────

/// Mindmap root node fill — `hsl(240, 100%, 46.2745098039%)` (deep blue).
pub const MINDMAP_ROOT_FILL: Color = Color("hsl(240, 100%, 46.2745098039%)");

/// Mindmap root node text colour — `#ffffff`.
pub const MINDMAP_ROOT_TEXT: Color = Color("#ffffff");

// ─────────────────────────────────────────────────────────────────────────────
// Common stroke-width constants (px)
// ─────────────────────────────────────────────────────────────────────────────

/// Thin stroke used for node borders in most diagrams (px).
pub const STROKE_WIDTH_THIN: f64 = 1.0;

/// Standard stroke width for railroad shapes and outer pie border (px).
pub const STROKE_WIDTH_STANDARD: f64 = 2.0;

/// Thick stroke used for architecture edges (px).
pub const STROKE_WIDTH_THICK: f64 = 3.0;

// ─────────────────────────────────────────────────────────────────────────────
// Pre-built Stroke values for frequently repeated combinations
// ─────────────────────────────────────────────────────────────────────────────

/// The default primary node stroke: `#9370DB` at 1 px.
pub const PRIMARY_NODE_STROKE: Stroke = Stroke::new(PRIMARY_BORDER, STROKE_WIDTH_THIN);

/// The default line / edge stroke: `#333333` at 1 px.
pub const EDGE_STROKE: Stroke = Stroke::new(LINE_COLOR, STROKE_WIDTH_THIN);

/// The standard railroad stroke: `#000000` at 2 px.
pub const RAILROAD_STROKE: Stroke = Stroke::new(BLACK, STROKE_WIDTH_STANDARD);

// ─────────────────────────────────────────────────────────────────────────────
// Common font-size constants (px)
// ─────────────────────────────────────────────────────────────────────────────

/// Standard body font size used across most diagram renderers (px).
pub const FONT_SIZE_DEFAULT: f64 = 16.0;

/// Smaller font size used by gantt, railroad, journey, zenuml, cynefin, ishikawa (px).
pub const FONT_SIZE_SMALL: f64 = 14.0;

/// Compact font size used by gantt axis labels (px).
pub const FONT_SIZE_TINY: f64 = 10.0;
