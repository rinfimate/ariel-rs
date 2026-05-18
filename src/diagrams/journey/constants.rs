//! Layout and styling constants for the journey renderer.

// ---------------------------------------------------------------------------
// Layout — task geometry
// ---------------------------------------------------------------------------

/// Horizontal gap between tasks (px). Matches `conf.journey.taskMargin`.
pub const TASK_MARGIN: f64 = 50.0;

/// Fixed width of each task box (px). Matches `conf.journey.width`.
pub const TASK_WIDTH: f64 = 150.0;

/// Height of each section header row (px). Matches `conf.journey.height`.
pub const SECTION_HEIGHT: f64 = 50.0;

/// Fixed left margin for the diagram (px). Not expanded for actor label widths.
pub const LEFT_MARGIN: f64 = 150.0;

/// Font size used in the journey diagram (px).
#[allow(dead_code)]
pub const FONT_SIZE: f64 = 14.0;

// ---------------------------------------------------------------------------
// Layout — vertical positions
// ---------------------------------------------------------------------------

/// Y-coordinate where task vertical dashed lines start (top of task box, px).
pub const TASK_LINE_TOP: f64 = 110.0;

/// Y-coordinate where task vertical dashed lines end (bottom of diagram area, px).
pub const TASK_LINE_BOTTOM: f64 = 450.0;

/// Y-coordinate of the horizontal activity arrow line (= SECTION_HEIGHT × 4, px).
pub const ACTIVITY_LINE_Y: f64 = 200.0;

/// Height of the SVG view area (above the -25 viewBox offset, px).
pub const VIEW_HEIGHT: f64 = 540.0;

// ---------------------------------------------------------------------------
// Actor legend
// ---------------------------------------------------------------------------

/// Starting Y position for the first actor circle in the left legend (px).
pub const ACTOR_LEGEND_START_Y: f64 = 60.0;

/// Vertical step between consecutive actor entries in the legend (px).
pub const ACTOR_LEGEND_STEP: f64 = 20.0;

/// Colour palette for actor circles. Matches Mermaid `actorColours` default.
pub const ACTOR_COLOURS: [&str; 4] = ["#8FBC8F", "#7CFC00", "#FF8C00", "#4169E1"];

// ---------------------------------------------------------------------------
// Section header colours
// ---------------------------------------------------------------------------

/// Background fill colours for section header rectangles (dark colours matching reference).
pub const SECTION_FILLS: [&str; 4] = [
    "#191970", // midnight blue  (section-type-0)
    "#8B008B", // dark magenta   (section-type-1)
    "#4B0082", // indigo         (section-type-2)
    "#006400", // dark green     (section-type-3)
];
