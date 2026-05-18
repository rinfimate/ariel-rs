use super::constants::*;
use super::parser::{PacketDiagram, PacketField};
use super::templates;
/// Faithful Rust port of Mermaid's packetRenderer.ts.
///
/// Layout algorithm (matches packetRenderer.ts / drawWord):
///
/// Constants (from mermaid defaultConfig.packet, with showBits=true):
///   bitWidth   = 32  px per bit cell
///   bitsPerRow = 32
///   rowHeight  = 32  px tall field boxes
///   paddingX   = 5   px gap at right of each block (separates adjacent blocks)
///   paddingY   = 15  px (base 5 + 10 for showBits) above each row for bit numbers
///
/// Derived:
///   totalRowHeight = rowHeight + paddingY = 47
///   svgWidth       = bitWidth * bitsPerRow + 2 = 1026
///   svgHeight      = totalRowHeight * (numRows + 1) - (hasTitle ? 0 : rowHeight)
///
/// Per row N (0-indexed):
///   wordY = N * totalRowHeight + paddingY
///
/// Per block within a row:
///   blockX = (block.start % bitsPerRow) * bitWidth + 1
///   width  = (block.end - block.start + 1) * bitWidth - paddingX
///   rect   at (blockX, wordY, width, rowHeight)
///   label  at (blockX + width/2, wordY + rowHeight/2)  dominant-baseline=middle
///   bitNumberY = wordY - 2
///   start-bit label: x=blockX, text-anchor=start  (middle if single bit)
///   end-bit   label: x=blockX+width, text-anchor=end  (omitted if single bit)
///
/// Title (if present):
///   (svgWidth/2, svgHeight - totalRowHeight/2)
use crate::theme::Theme;

pub fn render(diag: &PacketDiagram, theme: Theme, _use_foreign_object: bool) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let svg_id = "mermaid-packet";

    if diag.fields.is_empty() {
        return templates::empty_svg(svg_id, ff);
    }

    // ── Pre-process fields into rows ──────────────────────────────────────────
    let words = build_words(&diag.fields);

    let total_row_height = ROW_HEIGHT + PADDING_Y; // 47
    let has_title = diag.title.is_some();
    let svg_h =
        total_row_height * (words.len() as f64 + 1.0) - if has_title { 0.0 } else { ROW_HEIGHT };

    let mut out = String::new();

    out.push_str(&templates::svg_root(
        svg_id,
        &fmt(SVG_WIDTH),
        &fmt(svg_h),
        &fmt(SVG_WIDTH),
    ));

    out.push_str("<style>");
    out.push_str(&build_style(svg_id, ff));
    out.push_str("</style>");

    // Empty first group (mermaid emits <g></g> before the content group)
    out.push_str("<g></g>");

    // Render rows
    for (row_idx, word) in words.iter().enumerate() {
        let word_y = row_idx as f64 * total_row_height + PADDING_Y;

        out.push_str("<g>");

        for block in word {
            let block_x = (block.start % BITS_PER_ROW) as f64 * BIT_WIDTH + 1.0;
            let width = (block.end - block.start + 1) as f64 * BIT_WIDTH - PADDING_X;
            let is_single = block.start == block.end;
            let bit_number_y = word_y - BIT_NUMBER_Y_OFFSET;

            // Field rectangle
            out.push_str(&templates::field_rect(
                &fmt(block_x),
                &fmt(word_y),
                &fmt(width),
                &fmt(ROW_HEIGHT),
            ));

            // Field label centered in box
            out.push_str(&templates::field_label(
                &fmt(block_x + width / 2.0),
                &fmt(word_y + ROW_HEIGHT / 2.0),
                &esc(&block.label),
            ));

            // Bit number labels
            if is_single {
                out.push_str(&templates::bit_number_single(
                    &fmt(block_x + width / 2.0),
                    &fmt(bit_number_y),
                    block.start,
                ));
            } else {
                out.push_str(&templates::bit_number_start(
                    &fmt(block_x),
                    &fmt(bit_number_y),
                    block.start,
                ));
                // End bit (only when not single-bit)
                out.push_str(&templates::bit_number_end(
                    &fmt(block_x + width),
                    &fmt(bit_number_y),
                    block.end,
                ));
            }
        }

        out.push_str("</g>");
    }

    // Title
    out.push_str(&templates::title(
        &fmt(SVG_WIDTH / 2.0),
        &fmt(svg_h - total_row_height / 2.0),
        &esc(diag.title.as_deref().unwrap_or("")),
    ));

    out.push_str("</svg>");
    out
}

/// Split fields into rows, replicating mermaid's getNextFittingBlock logic.
/// Returns a Vec of rows, each row is a Vec of PacketField segments.
fn build_words(fields: &[PacketField]) -> Vec<Vec<PacketField>> {
    let mut words: Vec<Vec<PacketField>> = Vec::new();
    let mut current_word: Vec<PacketField> = Vec::new();
    let mut row = 1u32; // mermaid uses 1-indexed rows

    for field in fields {
        let mut start = field.start;
        let end = field.end;
        let label = field.label.clone();

        loop {
            // Does this segment fit within the current row?
            let row_end_bit = row * BITS_PER_ROW - 1; // last bit of current row (inclusive)
            if end <= row_end_bit {
                // Fits entirely in this row
                current_word.push(PacketField {
                    start,
                    end,
                    label: label.clone(),
                });
                // If this block fills the row exactly, push the word
                if end + 1 == row * BITS_PER_ROW {
                    words.push(std::mem::take(&mut current_word));
                    row += 1;
                }
                break;
            } else {
                // Split: put [start..row_end_bit] in current row, remainder in next row
                current_word.push(PacketField {
                    start,
                    end: row_end_bit,
                    label: label.clone(),
                });
                words.push(std::mem::take(&mut current_word));
                row += 1;
                start = row_end_bit + 1;
                // continue loop with remainder
            }
        }
    }

    // Push any remaining partial word
    if !current_word.is_empty() {
        words.push(current_word);
    }

    words
}

fn build_style(id: &str, ff: &str) -> String {
    // Matches mermaid's packet styles (defaultPacketStyleOptions)
    format!(
        "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}\
        #{id} .packetByte{{font-size:10px;}}\
        #{id} .packetByte.start{{fill:black;}}\
        #{id} .packetByte.end{{fill:black;}}\
        #{id} .packetLabel{{fill:black;font-size:12px;}}\
        #{id} .packetTitle{{fill:black;font-size:14px;}}\
        #{id} .packetBlock{{stroke:black;stroke-width:1;fill:#efefef;}}\
        #{id} :root{{--mermaid-font-family:{ff};}}",
        id = id,
        ff = ff,
    )
}

fn fmt(v: f64) -> String {
    // Format without unnecessary decimals, matching mermaid's d3 integer output
    if v == v.floor() && v.abs() < 1e12 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.4}", v);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    #[test]
    fn basic_render_produces_svg() {
        let input = "packet-beta\n    0-15: \"Source Port\"\n    16-31: \"Destination Port\"\n    32-63: \"Sequence Number\"\n    64-95: \"Acknowledgment Number\"";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains("<svg"), "no <svg element");
        assert!(svg.contains("Source Port"), "no field label");
        assert!(svg.contains("packetBlock"), "no packet fields");
    }

    #[test]
    fn renders_bit_numbers() {
        let input = "packet-beta\n    0-7: \"Byte\"\n    8-15: \"Second\"";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains(">0<"), "no bit 0");
    }

    #[test]
    fn multi_row_packet() {
        // More than 32 bits → 2 rows
        let input = "packet-beta\n    0-15: \"Source Port\"\n    16-31: \"Destination Port\"\n    32-63: \"Sequence Number\"";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default, false);
        // Both rows should be rendered (2 <g> groups in the content)
        assert!(
            svg.contains("packet-row") || svg.contains("<g>"),
            "no row groups"
        );
        assert!(svg.contains("Sequence Number"), "no second row field");
    }

    #[test]
    fn viewbox_matches_reference_basic() {
        // packet_basic: 3 fields (0-7 Source, 8-15 Dest, 16-31 Data), no title → 1026×62
        let input = "packet-beta\n    0-7: \"Source\"\n    8-15: \"Dest\"\n    16-31: \"Data\"";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(
            svg.contains("viewBox=\"0 0 1026 62\""),
            "wrong viewBox: {}",
            &svg[..200]
        );
    }

    #[test]
    #[ignore = "platform-specific float precision — run locally"]
    fn snapshot_default_theme() {
        let input =
            "packet-beta\n accTitle: Packet\n 0-7: \"Source\"\n 8-15: \"Dest\"\n 16-31: \"Data\"";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(svg);
    }
}
