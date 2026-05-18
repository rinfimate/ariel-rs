//! Layout and styling constants for the mindmap renderer.

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Default font size for node labels (px).
pub const FONT_SIZE: f64 = 16.0;

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

/// Vertical slot height allocated per leaf node (px). Controls spacing between branches.
pub const NODE_SLOT: f64 = 71.47;

/// Horizontal gap between a parent node edge and a child node edge (px).
pub const NODE_H_GAP: f64 = 20.0;

/// Margin around the bounding box of all nodes used for the SVG viewBox (px).
pub const MARGIN: f64 = 20.0;

/// Fixed height of rectangular node shapes (px).
pub const NODE_SHAPE_H: f64 = 24.0;

// ---------------------------------------------------------------------------
// Root node colours
// ---------------------------------------------------------------------------

/// Fill colour for the root node.
pub const ROOT_FILL: &str = "hsl(240, 100%, 46.2745098039%)";

/// Text colour for the root node label.
pub const ROOT_TEXT_COLOR: &str = "#ffffff";

// ---------------------------------------------------------------------------
// Section colour palettes (11 colours, indexed mod 11)
// ---------------------------------------------------------------------------

/// Fill colours for non-root section nodes (section 0..10).
pub const SECTION_FILLS: [&str; 11] = [
    "hsl(60, 100%, 73.5294117647%)",
    "hsl(80, 100%, 76.2745098039%)",
    "hsl(270, 100%, 76.2745098039%)",
    "hsl(300, 100%, 76.2745098039%)",
    "hsl(330, 100%, 76.2745098039%)",
    "hsl(0, 100%, 76.2745098039%)",
    "hsl(30, 100%, 76.2745098039%)",
    "hsl(90, 100%, 76.2745098039%)",
    "hsl(150, 100%, 76.2745098039%)",
    "hsl(180, 100%, 76.2745098039%)",
    "hsl(210, 100%, 76.2745098039%)",
];

/// Text fill colours for non-root section nodes (section 0..10).
pub const SECTION_TEXT_COLORS: [&str; 11] = [
    "black", "black", "#ffffff", "black", "black", "black", "black", "black", "black", "black",
    "black",
];

/// Edge/line colours for each section (section 0..10).
pub const SECTION_LINE_COLORS: [&str; 11] = [
    "hsl(240, 100%, 83.5294117647%)",
    "hsl(260, 100%, 86.2745098039%)",
    "hsl(90, 100%, 86.2745098039%)",
    "hsl(120, 100%, 86.2745098039%)",
    "hsl(150, 100%, 86.2745098039%)",
    "hsl(180, 100%, 86.2745098039%)",
    "hsl(210, 100%, 86.2745098039%)",
    "hsl(270, 100%, 86.2745098039%)",
    "hsl(330, 100%, 86.2745098039%)",
    "hsl(0, 100%, 86.2745098039%)",
    "hsl(30, 100%, 86.2745098039%)",
];

/// Rounded corner radius for rectangular mindmap node shapes (px).
pub const NODE_RECT_RX: f64 = 5.0;
