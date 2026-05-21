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

/// Font size used in the journey diagram (px). Matches defaultConfig.journey.taskFontSize = 14.
#[allow(dead_code)]
pub const FONT_SIZE: f64 = 14.0;

/// Font family for task label text. Matches defaultConfig.journey.taskFontFamily.
#[allow(dead_code)]
pub const TASK_FONT_FAMILY: &str = r#""Open Sans", sans-serif"#;

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

/// Actor legend circle colours. Matches defaultConfig.journey.actorColours exactly.
pub const ACTOR_COLOURS: [&str; 6] = [
    "#8FBC8F", "#7CFC00", "#00FFFF", "#20B2AA", "#B0E0E6", "#FFFFE0",
];

// ---------------------------------------------------------------------------
// Section header colours
// ---------------------------------------------------------------------------

/// Section header fill colours. Matches defaultConfig.journey.sectionFills exactly.
#[allow(dead_code)]
pub const SECTION_FILLS: [&str; 7] = [
    "#191970", // midnight blue
    "#8B008B", // dark magenta
    "#4B0082", // indigo
    "#2F4F4F", // dark slate gray
    "#800000", // maroon
    "#8B4513", // saddle brown
    "#00008B", // dark blue
];

/// Fallback fill colours for task boxes — types 0 and 1 are overridden at render time
/// with vars.primary_color and vars.secondary_color respectively.
pub const TASK_FILLS: [&str; 7] = [
    "#ECECFF",                        // type-0 (overridden by primary_color)
    "#ffffde",                        // type-1 (overridden by secondary_color)
    "hsl(304, 100%, 96.2745098039%)", // type-2
    "hsl(124, 100%, 93.5294117647%)", // type-3
    "hsl(176, 100%, 96.2745098039%)", // type-4
    "hsl(-4, 100%, 93.5294117647%)",  // type-5
    "hsl(8, 100%, 96.2745098039%)",   // type-6
];
