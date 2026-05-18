//! Layout and styling constants for the C4 diagram renderer.
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Element geometry
// ---------------------------------------------------------------------------

/// Width of each element box (px).
pub const ELEMENT_W: f64 = 216.0;

/// Horizontal gap between elements in the same row (px).
pub const H_GAP: f64 = 100.0;

/// Vertical gap between element rows (px).
pub const V_GAP: f64 = 100.0;

/// X coordinate of the first element's left edge when no boundary is present (px).
pub const SVG_LEFT: f64 = 150.0;

/// Boundary padding on left/right/bottom sides (px).
pub const BOUND_PAD: f64 = 50.0;

/// Space from boundary top edge to the first element row (px).
pub const BOUND_TOP: f64 = 100.0;

/// Y coordinate of the first boundary rect top (px).
pub const BOUNDARY_FIRST_Y: f64 = 122.0;

/// Y coordinate of the first ungrouped element row (px).
pub const UNGROUPED_FIRST_Y: f64 = 166.0;

/// Maximum number of element columns per row.
pub const COLS: usize = 2;

// ---------------------------------------------------------------------------
// Element heights
// ---------------------------------------------------------------------------

/// Base height of a Person element box (no description, px).
pub const PERSON_BASE_H: f64 = 103.0;

/// Height added per description line for a Person element (px).
pub const PERSON_DESCR_LINE_H: f64 = 31.0;

/// Base height of a System/Container/Component element box (no description, px).
pub const SYSTEM_BASE_H: f64 = 60.0;

/// Height added per description line for a System element (px).
pub const SYSTEM_DESCR_LINE_H: f64 = 26.0;

// ---------------------------------------------------------------------------
// ViewBox / SVG dimensions
// ---------------------------------------------------------------------------

/// Extra bottom padding below the last element in the SVG height computation (px).
pub const SVG_BOTTOM_PAD: f64 = 100.0;

/// ViewBox y-offset applied to the SVG (negative — shifts content up, px).
pub const VIEWBOX_Y_OFFSET: f64 = 70.0;

// ---------------------------------------------------------------------------
// Relationship geometry
// ---------------------------------------------------------------------------

/// Y-offset applied to a curved relationship path's control point (px).
pub const REL_CURVE_Y_OFFSET: f64 = 74.0;

/// Vertical distance above the midpoint for straight-line labels (px).
pub const REL_LABEL_STRAIGHT_OFFSET: f64 = 12.0;

/// Y-distance between relationship label and tech label (px).
pub const REL_TECHN_Y_GAP: f64 = 14.0;

// ---------------------------------------------------------------------------
// Colours
// ---------------------------------------------------------------------------

/// Fill colour for Person element backgrounds.
pub const PERSON_FILL: &str = "#08427B";

/// Stroke colour for Person element borders.
pub const PERSON_STROKE: &str = "#073B6F";

/// Fill colour for external element backgrounds.
pub const EXT_FILL: &str = "#999999";

/// Stroke colour for external element borders.
pub const EXT_STROKE: &str = "#8A8A8A";

/// Fill colour for internal (non-person, non-external) element backgrounds.
pub const INTERNAL_FILL: &str = "#1168BD";

/// Stroke colour for internal element borders.
pub const INTERNAL_STROKE: &str = "#3C7FC0";

/// Stroke colour for boundary rectangles.
pub const BOUNDARY_STROKE: &str = "#444444";

/// Fill colour for text on white-background elements (relationship labels, etc.).
pub const DARK_TEXT: &str = "#444444";

// ---------------------------------------------------------------------------
// Person icon
// ---------------------------------------------------------------------------

/// Base64-encoded PNG icon used for Person elements.
pub const PERSON_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAIAAADYYG7QAAACD0lEQVR4Xu2YoU4EMRCGT+4j8Ai8AhaH4QHgAUjQuFMECUgMIUgwJAgMhgQsAYUiJCiQIBBY+EITsjfTdme6V24v4c8vyGbb+ZjOtN0bNcvjQXmkH83WvYBWto6PLm6v7p7uH1/w2fXD+PBycX1Pv2l3IdDm/vn7x+dXQiAubRzoURa7gRZWd0iGRIiJbOnhnfYBQZNJjNbuyY2eJG8fkDE3bbG4ep6MHUAsgYxmE3nVs6VsBWJSGccsOlFPmLIViMzLOB7pCVO2AtHJMohH7Fh6zqitQK7m0rJvAVYgGcEpe//PLdDz65sM4pF9N7ICcXDKIB5Nv6j7tD0NoSdM2QrU9Gg0ewE1LqBhHR3BBdvj2vapnidjHxD/q6vd7Pvhr31AwcY8eXMTXAKECZZJFXuEq27aLgQK5uLMohCenGGuGewOxSjBvYBqeG6B+Nqiblggdjnc+ZXDy+FNFpFzw76O3UBAROuXh6FoiAcf5g9eTvUgzy0nWg6I8cXHRUpg5bOVBCo+KDpFajOf23GgPme7RSQ+lacIENUgJ6gg1k6HjgOlqnLqip4tEuhv0hNEMXUD0clyXE3p6pZA0S2nnvTlXwLJEZWlb7cTQH1+USgTN4VhAenm/wea1OCAOmqo6fE1WCb9WSKBah+rbUWPWAmE2Rvk0ApiB45eOyNAzU8xcTvj8KvkKEoOaIYeHNA3ZuygAvFMUO0AAAAASUVORK5CYII=";
