//! Layout and styling constants for the packet renderer.
//!
//! All values faithfully ported from Mermaid JS `defaultConfig.packet` +
//! `PacketDB.getConfig()` (adds 10 to paddingY when showBits=true).
#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Bit geometry  (defaultConfig.packet)
// ---------------------------------------------------------------------------

/// Number of bits displayed per row (`bitsPerRow`).
pub const BITS_PER_ROW: u32 = 32;

/// Width of each bit cell in px (`bitWidth`).
pub const BIT_WIDTH: f64 = 32.0;

/// Height of each field box row in px (`rowHeight`).
pub const ROW_HEIGHT: f64 = 32.0;

// ---------------------------------------------------------------------------
// Padding  (defaultConfig.packet + showBits adjustment)
// ---------------------------------------------------------------------------

/// Horizontal gap subtracted from the right edge of each block in px (`paddingX`).
pub const PADDING_X: f64 = 5.0;

/// Vertical padding above each row for bit-number labels in px.
/// Base value from config is 5; `showBits=true` (the default) adds 10 → 15.
pub const PADDING_Y: f64 = 15.0;

// ---------------------------------------------------------------------------
// SVG geometry
// ---------------------------------------------------------------------------

/// Total SVG canvas width = `bitWidth × bitsPerRow + 2` px.
pub const SVG_WIDTH: f64 = BIT_WIDTH * BITS_PER_ROW as f64 + 2.0; // 1026

/// Y offset from wordY down to the bit-number label baseline (`bitNumberY = wordY - 2`).
pub const BIT_NUMBER_Y_OFFSET: f64 = 2.0;

// ---------------------------------------------------------------------------
// Typography  (defaultPacketStyleOptions)
// ---------------------------------------------------------------------------

/// Font size for field label text (`labelFontSize`).
pub const LABEL_FONT_SIZE: &str = "12px";

/// Font size for bit-number text (`byteFontSize`).
pub const BYTE_FONT_SIZE: &str = "10px";

/// Font size for diagram title text (`titleFontSize`).
pub const TITLE_FONT_SIZE: &str = "14px";

// ---------------------------------------------------------------------------
// Parser limits
// ---------------------------------------------------------------------------

/// Maximum total number of packed words the parser will accept (`maxPacketSize`).
pub const MAX_PACKET_SIZE: usize = 10_000;
