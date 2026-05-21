// Faithful Rust port of mermaid/src/diagrams/packet/renderer.ts
//
// draw():
//   svgWidth  = bitWidth * bitsPerRow + 2                  = 1026
//   svgHeight = totalRowHeight * (words.length + 1) - (title ? 0 : rowHeight)
//   svg.attr("viewBox", `0 0 ${svgWidth} ${svgHeight}`)
//   configureSvgSize(svg, svgHeight, svgWidth, useMaxWidth=true)
//     → width="100%", style="max-width: {svgWidth}px;"
//
// drawWord(svg, word, rowNumber, config):
//   wordY      = rowNumber * (rowHeight + paddingY) + paddingY
//   blockX     = block.start % bitsPerRow * bitWidth + 1
//   width      = (block.end - block.start + 1) * bitWidth - paddingX
//   rect:  x=blockX, y=wordY, width, height=rowHeight, class="packetBlock"
//   label: x=blockX+width/2, y=wordY+rowHeight/2, class="packetLabel",
//          dominant-baseline="middle", text-anchor="middle"
//   if showBits:
//     isSingle = (end === start)
//     bitNumberY = wordY - 2
//     start-bit: x=blockX+(isSingle?width/2:0), text-anchor=(isSingle?"middle":"start")
//     end-bit (if !isSingle): x=blockX+width, text-anchor="end"
//
// title (always emitted, may be empty):
//   x=svgWidth/2, y=svgHeight - totalRowHeight/2,
//   dominant-baseline="middle", text-anchor="middle", class="packetTitle"
//
// Styling: Mermaid injects CSS for the class names. We use inline CSS-independent
// attributes with per-theme colors from ThemeVars.

use super::constants::*;
use super::parser::PacketDiagram;
use super::templates::{self, esc, fmt};
use crate::theme::Theme;

pub fn render(diag: &PacketDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let block_fill = vars.packet_block_fill;
    let block_stroke = vars.packet_block_stroke;
    let text_color = vars.packet_text_color;
    let svg_id = "mermaid-packet";

    let words = &diag.words;

    let total_row_height = ROW_HEIGHT + PADDING_Y;
    let has_title = diag
        .title
        .as_deref()
        .map(|t| !t.is_empty())
        .unwrap_or(false);
    let svg_h =
        total_row_height * (words.len() as f64 + 1.0) - if has_title { 0.0 } else { ROW_HEIGHT };

    let mut out = String::new();

    out.push_str(&templates::svg_root(
        svg_id,
        &fmt(SVG_WIDTH),
        &fmt(svg_h),
        &fmt(SVG_WIDTH),
        vars.font_family,
    ));

    // Empty first group (Mermaid emits `<g></g>` before content groups)
    out.push_str("<g></g>");

    // Render words (rows)
    for (row_number, word) in words.iter().enumerate() {
        let word_y = row_number as f64 * total_row_height + PADDING_Y;
        out.push_str("<g>");

        for block in word {
            let block_x = (block.start % BITS_PER_ROW) as f64 * BIT_WIDTH + 1.0;
            let width = (block.end - block.start + 1) as f64 * BIT_WIDTH - PADDING_X;
            let is_single = block.start == block.end;
            let bit_number_y = word_y - BIT_NUMBER_Y_OFFSET;

            // rect
            out.push_str(&templates::block_rect(
                block_x,
                word_y,
                width,
                ROW_HEIGHT,
                block_fill,
                block_stroke,
            ));

            // label
            out.push_str(&templates::block_label(
                block_x + width / 2.0,
                word_y + ROW_HEIGHT / 2.0,
                &esc(&block.label),
                text_color,
            ));

            // bit numbers (showBits = true by default)
            let start_x = block_x + if is_single { width / 2.0 } else { 0.0 };
            let start_anchor = if is_single { "middle" } else { "start" };
            out.push_str(&templates::bit_label(
                start_x,
                bit_number_y,
                block.start,
                "start",
                start_anchor,
                text_color,
            ));
            if !is_single {
                out.push_str(&templates::bit_label(
                    block_x + width,
                    bit_number_y,
                    block.end,
                    "end",
                    "end",
                    text_color,
                ));
            }
        }

        out.push_str("</g>");
    }

    // Title (always emitted; empty when no title, matching Mermaid)
    let title_text = esc(diag.title.as_deref().unwrap_or(""));
    out.push_str(&templates::diagram_title(
        SVG_WIDTH / 2.0,
        svg_h - total_row_height / 2.0,
        &title_text,
        text_color,
    ));

    out.push_str("</svg>");
    out
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    #[test]
    fn basic_render_produces_svg() {
        let input = "packet-beta\n    0-15: \"Source Port\"\n    16-31: \"Destination Port\"\n    32-63: \"Sequence Number\"\n    64-95: \"Acknowledgment Number\"";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "no <svg");
        assert!(svg.contains("Source Port"), "no field label");
        assert!(svg.contains("packetBlock"), "no packet fields");
    }

    #[test]
    fn renders_bit_numbers() {
        let input = "packet-beta\n    0-7: \"Byte\"\n    8-15: \"Second\"";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains(">0<"), "no bit 0");
    }

    #[test]
    fn multi_row_packet() {
        let input = "packet-beta\n    0-15: \"Source Port\"\n    16-31: \"Destination Port\"\n    32-63: \"Sequence Number\"";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("Sequence Number"), "no second row field");
    }

    #[test]
    fn viewbox_matches_reference_basic() {
        // 1 row, no title → svgH = totalRowHeight * 2 - rowHeight = 47*2 - 32 = 62
        let input = "packet-beta\n    0-7: \"Source\"\n    8-15: \"Dest\"\n    16-31: \"Data\"";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(
            svg.contains("viewBox=\"0 0 1026 62\""),
            "wrong viewBox: {}",
            &svg[..200]
        );
    }

    #[test]
    fn dark_theme() {
        let input = "packet-beta\n  0-15: \"A\"\n  16-31: \"B\"";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Dark);
        assert!(svg.contains("#333"), "dark theme block fill missing");
    }

    #[test]
    fn snapshot_default_theme() {
        let input =
            "packet-beta\n title TCP\n 0-15: \"Source Port\"\n 16-31: \"Destination Port\"\n 32-63: \"Sequence Number\"";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
