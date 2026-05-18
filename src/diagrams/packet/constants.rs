//! Layout and styling constants for the packet renderer.

// ---------------------------------------------------------------------------
// Bit geometry
// ---------------------------------------------------------------------------

/// Number of bits displayed per row.
pub const BITS_PER_ROW: u32 = 32;

/// Width of each bit cell (px). Matches `mermaid defaultConfig.packet.bitWidth`.
pub const BIT_WIDTH: f64 = 32.0;

/// Height of each field box row (px). Matches `mermaid defaultConfig.packet.rowHeight`.
pub const ROW_HEIGHT: f64 = 32.0;

// ---------------------------------------------------------------------------
// Padding
// ---------------------------------------------------------------------------

/// Horizontal padding subtracted from the right edge of each block (px).
pub const PADDING_X: f64 = 5.0;

/// Vertical padding above each row for bit number labels (px). Base 5 + 10 for showBits.
pub const PADDING_Y: f64 = 15.0;

// ---------------------------------------------------------------------------
// SVG sizing
// ---------------------------------------------------------------------------

/// Total SVG width = BIT_WIDTH × BITS_PER_ROW + 2 (px).
pub const SVG_WIDTH: f64 = BIT_WIDTH * BITS_PER_ROW as f64 + 2.0; // 1026

/// Y offset subtracted from the bit-number label position (px).
pub const BIT_NUMBER_Y_OFFSET: f64 = 2.0;
