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

// Root and section colours are now in ThemeVars:
//   vars.mindmap_root_fill, vars.mindmap_root_text,
//   vars.mindmap_section_fills[i%11], vars.mindmap_section_text[i%11],
//   vars.mindmap_section_lines[i%11]

/// Rounded corner radius for rectangular mindmap node shapes (px).
pub const NODE_RECT_RX: f64 = 5.0;
