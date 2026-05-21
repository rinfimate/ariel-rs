// Faithful Rust port of the Mermaid packet Langium grammar + db.ts `populate()`.
//
// Grammar (from PacketGrammarGrammar JSON):
//
//   Packet:
//     YAML?                              — hidden: /---\r?\n[\S\s]*?\r?\n---/
//     ("packet" | "packet-beta")
//     (TitleAndAccessibilities | PacketBlock | NEWLINE)*
//
//   PacketBlock:
//     ( start=INT ("-" end=INT)?         — explicit start, optional end
//     | "+" bits=INT )                   — relative bits, auto start
//     ":" label=STRING
//
//   TitleAndAccessibilities: title | accTitle | accDescr
//
// populate() validation (mirrored faithfully):
//   • end < start                  → error (block skipped)
//   • bits === 0                   → error (block skipped)
//   • start !== lastBit + 1        → error "not contiguous" (block + remainder skipped)
//   • start defaults to lastBit + 1  (when "+bits" form used)
//   • end   defaults to start + bits - 1  (or start when only start given)
//
// getNextFittingBlock: splits a block that spans a 32-bit row boundary into two.

use super::constants::{BITS_PER_ROW, MAX_PACKET_SIZE};

// ─── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PacketBlock {
    pub start: u32,
    pub end: u32,
    pub label: String,
}

#[derive(Debug, Default)]
pub struct PacketDiagram {
    pub title: Option<String>,
    /// Rows of blocks — already split at 32-bit row boundaries.
    pub words: Vec<Vec<PacketBlock>>,
}

// ─── Parser ───────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<PacketDiagram> {
    let mut title: Option<String> = None;

    // State mirroring populate()
    let mut last_bit: i64 = -1;
    let mut current_word: Vec<PacketBlock> = Vec::new();
    let mut row: u32 = 1;
    let mut words: Vec<Vec<PacketBlock>> = Vec::new();

    let mut in_yaml = false;
    let mut found_keyword = false;

    for raw in input.lines() {
        let line = strip_comment(raw);
        let trimmed = line.trim();

        // ── YAML frontmatter ──────────────────────────────────────────────────
        if trimmed == "---" {
            in_yaml = !in_yaml;
            continue;
        }
        if in_yaml {
            continue;
        }

        // ── Keyword ───────────────────────────────────────────────────────────
        if !found_keyword {
            if trimmed == "packet"
                || trimmed == "packet-beta"
                || trimmed.starts_with("packet ")
                || trimmed.starts_with("packet\t")
                || trimmed.starts_with("packet-beta ")
                || trimmed.starts_with("packet-beta\t")
            {
                found_keyword = true;
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        // ── Directives ────────────────────────────────────────────────────────
        if let Some(rest) = trimmed
            .strip_prefix("title ")
            .or_else(|| trimmed.strip_prefix("title\t"))
        {
            title = Some(rest.trim().trim_matches('"').to_string());
            continue;
        }
        if trimmed == "title" {
            title = Some(String::new());
            continue;
        }
        if trimmed.starts_with("accTitle") || trimmed.starts_with("accDescr") {
            continue;
        }

        // ── PacketBlock ───────────────────────────────────────────────────────
        let Some((raw_start, raw_end, raw_bits, label)) = parse_block_line(trimmed) else {
            continue;
        };

        // Validate bits ≠ 0
        if raw_bits == Some(0) {
            continue;
        }

        // Resolve start
        let start = raw_start.unwrap_or((last_bit + 1) as u32);

        // Validate contiguity
        if start as i64 != last_bit + 1 {
            // non-contiguous: skip this block entirely (Mermaid throws an error)
            continue;
        }

        // Resolve end
        let end = raw_end.unwrap_or_else(|| {
            let bits = raw_bits.unwrap_or(1);
            start + bits - 1
        });

        // Validate end >= start
        if end < start {
            continue;
        }

        last_bit = end as i64;

        // ── getNextFittingBlock loop (mirrors populate's while loop) ───────────
        let mut cur_start = start;
        let mut cur_end = end;
        loop {
            if words.len() >= MAX_PACKET_SIZE {
                break;
            }
            let (block, maybe_next) =
                get_next_fitting_block(cur_start, cur_end, &label, row, BITS_PER_ROW);

            current_word.push(block);

            // If the block's end is exactly at the row boundary, commit the word
            if current_word.last().map(|b| b.end + 1) == Some(row * BITS_PER_ROW) {
                words.push(std::mem::take(&mut current_word));
                row += 1;
            }

            match maybe_next {
                None => break,
                Some((ns, ne)) => {
                    cur_start = ns;
                    cur_end = ne;
                }
            }
        }
    }

    // Push any trailing partial word
    if !current_word.is_empty() {
        words.push(current_word);
    }

    crate::error::ParseResult::ok(PacketDiagram { title, words })
}

// ─── getNextFittingBlock ─────────────────────────────────────────────────────
//
// Direct port of the JS function:
//   if block.end + 1 <= row * bitsPerRow → fits, return [block, undefined]
//   else → split at row_end; return [first_half, next_half]

fn get_next_fitting_block(
    start: u32,
    end: u32,
    label: &str,
    row: u32,
    bits_per_row: u32,
) -> (PacketBlock, Option<(u32, u32)>) {
    if end + 1 <= row * bits_per_row {
        (
            PacketBlock {
                start,
                end,
                label: label.to_string(),
            },
            None,
        )
    } else {
        let row_end = row * bits_per_row - 1;
        let row_start = row * bits_per_row;
        (
            PacketBlock {
                start,
                end: row_end,
                label: label.to_string(),
            },
            Some((row_start, end)),
        )
    }
}

// ─── Line parser ─────────────────────────────────────────────────────────────

/// Parse a `PacketBlock` line.
/// Returns `(start, end, bits, label)` — any of start/end/bits may be None.
fn parse_block_line(line: &str) -> Option<(Option<u32>, Option<u32>, Option<u32>, String)> {
    // "+N: label" — relative bits form
    if let Some(rest) = line.strip_prefix('+') {
        let colon = rest.find(':')?;
        let bits: u32 = rest[..colon].trim().parse().ok()?;
        let label = parse_label(&rest[colon + 1..]);
        if label.is_empty() {
            return None;
        }
        return Some((None, None, Some(bits), label));
    }

    // "N-M: label" or "N: label"
    let colon = line.find(':')?;
    let range_part = line[..colon].trim();
    let label = parse_label(&line[colon + 1..]);

    if let Some(dash) = range_part.find('-') {
        let start: u32 = range_part[..dash].trim().parse().ok()?;
        let end: u32 = range_part[dash + 1..].trim().parse().ok()?;
        Some((Some(start), Some(end), None, label))
    } else {
        let start: u32 = range_part.trim().parse().ok()?;
        Some((Some(start), None, None, label))
    }
}

fn parse_label(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn strip_comment(line: &str) -> &str {
    if let Some(pos) = line.find("%%") {
        &line[..pos]
    } else {
        line
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_packet() {
        let input =
            "packet-beta\n    0-15: \"Source Port\"\n    16-31: \"Destination Port\"\n    32-63: \"Sequence Number\"\n    64-95: \"Acknowledgment Number\"";
        let diag = parse(input).diagram;
        assert_eq!(diag.words.len(), 3, "expected 3 rows");
        assert_eq!(diag.words[0][0].start, 0);
        assert_eq!(diag.words[0][0].end, 15);
        assert_eq!(diag.words[0][0].label, "Source Port");
    }

    #[test]
    fn parse_single_bit() {
        let input = "packet-beta\n    0: \"Flag\"";
        let diag = parse(input).diagram;
        assert_eq!(diag.words.len(), 1);
        assert_eq!(diag.words[0][0].start, 0);
        assert_eq!(diag.words[0][0].end, 0);
    }

    #[test]
    fn parse_packet_keyword_no_beta() {
        let input = "packet\n  0-7: \"A\"\n  8-15: \"B\"";
        let diag = parse(input).diagram;
        assert_eq!(diag.words.len(), 1);
        assert_eq!(diag.words[0].len(), 2);
    }

    #[test]
    fn parse_yaml_frontmatter() {
        let input = "---\ntitle: \"TCP Packet\"\n---\npacket-beta\n  0-15: \"Source Port\"\n  16-31: \"Destination Port\"";
        let diag = parse(input).diagram;
        assert_eq!(diag.words.len(), 1);
        assert_eq!(diag.words[0][0].label, "Source Port");
    }

    #[test]
    fn rejects_non_contiguous_blocks() {
        // Blocks 0-99 are valid, then 106: "URG" skips 100-105 → non-contiguous → skipped.
        // Mirrors the live_editor_packet error: "Packet block 106-106 not contiguous, start from 100".
        let input = "packet-beta\n  0-99: \"A\"\n  106: \"URG\"";
        let diag = parse(input).diagram;
        let all: Vec<&PacketBlock> = diag.words.iter().flatten().collect();
        // Only the first block (split across rows) is rendered; 106 is skipped.
        assert!(
            all.iter().all(|b| b.end <= 99),
            "non-contiguous block should be absent"
        );
        assert!(
            all.iter().any(|b| b.start == 0),
            "first block should be rendered"
        );
    }

    #[test]
    fn relative_bits_form() {
        let input = "packet-beta\n  0-7: \"A\"\n  +8: \"B\"";
        let diag = parse(input).diagram;
        let all: Vec<&PacketBlock> = diag.words.iter().flatten().collect();
        assert_eq!(all[1].start, 8);
        assert_eq!(all[1].end, 15);
        assert_eq!(all[1].label, "B");
    }

    #[test]
    fn block_spans_row_boundary() {
        // 24-39 crosses bit 31 → splits into 24-31 (row 0) and 32-39 (row 1)
        let input = "packet-beta\n  0-23: \"A\"\n  24-39: \"B\"";
        let diag = parse(input).diagram;
        assert_eq!(diag.words.len(), 2, "expected 2 rows after split");
        assert_eq!(diag.words[0].last().unwrap().end, 31);
        assert_eq!(diag.words[1][0].start, 32);
        assert_eq!(diag.words[1][0].end, 39);
    }
}
