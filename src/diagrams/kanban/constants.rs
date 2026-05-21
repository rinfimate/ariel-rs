//! Layout and styling constants for the kanban renderer.

// ---------------------------------------------------------------------------
// Column geometry
// ---------------------------------------------------------------------------

/// Fixed width of each kanban column (px). Matches Mermaid `sectionWidth`.
pub const SECTION_WIDTH: f64 = 200.0;

/// Horizontal gap between adjacent columns (px).
pub const SECTION_GAP: f64 = 5.0;

/// Fixed height of each item card (px).
pub const ITEM_HEIGHT: f64 = 44.0;

/// Fixed width of each item card (px). Equal to SECTION_WIDTH − 15.
pub const ITEM_WIDTH: f64 = 185.0;

/// Column header height (px). Matches Mermaid `LABEL_HEIGHT_DEFAULT`.
pub const LABEL_HEIGHT: f64 = 25.0;

/// Vertical gap between item cards within a column (px).
pub const ITEM_GAP: f64 = 5.0;

/// Padding below the last item to the column bottom edge (px).
#[allow(dead_code)]
pub const COL_BOTTOM_PAD: f64 = 10.0;

/// Y-coordinate of the top edge of all columns (px, negative = above origin).
pub const COL_TOP: f64 = -300.0;

// ---------------------------------------------------------------------------
// ViewBox / SVG sizing
// ---------------------------------------------------------------------------

/// viewBox min-x offset (px).
pub const VIEWBOX_X: f64 = 90.0;

/// viewBox min-y offset (px).
pub const VIEWBOX_Y: f64 = -310.0;

/// Margin added around the content for SVG sizing (px).
pub const MARGIN: f64 = 10.0;

// ---------------------------------------------------------------------------
// Section colour palette (mirrors Mermaid kanban CSS generation)
// ---------------------------------------------------------------------------

/// HSL hue values for section-0 through section-10 (matching Mermaid CSS output exactly).
/// section-0 uses hue 60 with a distinct lightness (SECTION_L_0).
pub const SECTION_HUES: [u32; 11] = [60, 80, 270, 300, 330, 0, 30, 90, 150, 180, 210];

/// Lightness percentage for section-0 (special case, slightly darker than the rest).
pub const SECTION_L_0: &str = "83.5294117647%";

/// Lightness percentage for section-1 through section-10.
pub const SECTION_L: &str = "86.2745098039%";

/// Darker lightness used for stroke/edge colours.
#[allow(dead_code)]
pub const SECTION_L_DARK: &str = "76.2745098039%";

/// Darker lightness for section-0 stroke/edge colour.
#[allow(dead_code)]
pub const SECTION_L_0_DARK: &str = "73.5294117647%";

// ---------------------------------------------------------------------------
// Column geometry helpers (column left-edge X offset, px)
// ---------------------------------------------------------------------------

/// X offset of the leftmost column edge (px). All column positions derive from this.
pub const COL_LEFT_BASE: f64 = 100.0;

// ---------------------------------------------------------------------------
// Typography (item-height calculation)
// ---------------------------------------------------------------------------

/// Font size used for rendering kanban item labels (px).
pub const FONT_SIZE: f64 = 16.0;

/// Line height = font_size × 1.5 (CSS default line-height).
pub const LINE_HEIGHT: f64 = FONT_SIZE * 1.5;

/// Available width for text in each item card (px). Matches the foreignObject width.
pub const AVAILABLE_WIDTH: f64 = ITEM_WIDTH - 10.0;

/// Vertical padding (top + bottom) added to wrapped text in each card (px).
pub const V_PADDING: f64 = 20.0;

/// Scale factor: browser renders item text slightly wider than ab_glyph metrics.
pub const TEXT_SCALE: f64 = 1.13;
