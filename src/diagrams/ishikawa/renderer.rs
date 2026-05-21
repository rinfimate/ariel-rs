use super::constants::*;
use super::parser::{IshikawaDiagram, IshikawaNode};
use super::templates::{self, esc, fmt};
/// Faithful Rust port of Mermaid's ishikawaRenderer.ts.
///
/// Key algorithm details (from TypeScript source):
/// - SPINE_BASE_LENGTH = 250, BONE_STUB = 30, BONE_BASE = 60, BONE_PER_CHILD = 5
/// - ANGLE = (82 * PI) / 180  (almost vertical bones)
/// - Causes alternate upper (i%2==0) / lower (i%2==1)
/// - Spine length is proportional to descendant counts per side
/// - Even-depth sub-bones are horizontal; odd-depth are diagonal
/// - Head is a kite/diamond shape at the right end of the spine
/// - Labels in boxes; lines with arrow markers pointing toward head
///
/// We skip Rough.js (hand-drawn mode) — we render clean SVG only.
///
/// Bounding box: since we lack DOM access we analytically track the min/max
/// of all rendered coordinates (branch endpoints, label boxes, head extent)
/// and derive the viewBox from those, matching what Mermaid's applyPaddedViewBox
/// produces after calling getBBox().
use crate::text::measure;
use crate::theme::Theme;

/// Wrap text to at most `max_chars` characters per line, splitting on whitespace.
/// Matches Mermaid's wrapText() exactly.
fn wrap_text(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        let last = lines.len().wrapping_sub(1);
        if !lines.is_empty() && lines[last].len() + 1 + word.len() <= max_chars {
            let n = lines.len() - 1;
            lines[n].push(' ');
            lines[n].push_str(word);
        } else {
            lines.push(word.to_string());
        }
    }
    lines.join("\n")
}

/// Split text on newlines or <br> tags (matches Mermaid's splitLines).
fn split_lines(text: &str) -> Vec<String> {
    // Split on \n or <br/> or <br />
    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            result.push(current.clone());
            current.clear();
            i += 1;
        } else if bytes[i] == b'<' {
            // check for <br .../>
            let rest = &text[i..];
            let lower = rest.to_lowercase();
            if lower.starts_with("<br/>")
                || lower.starts_with("<br />")
                || lower.starts_with("<br>")
            {
                result.push(current.clone());
                current.clear();
                let skip = if lower.starts_with("<br/>") {
                    5
                } else if lower.starts_with("<br />") {
                    6
                } else {
                    4
                };
                i += skip;
            } else {
                current.push(bytes[i] as char);
                i += 1;
            }
        } else {
            current.push(bytes[i] as char);
            i += 1;
        }
    }
    result.push(current);
    result
}

/// Measure the widest line in a (possibly multi-line) text block.
/// Returns (max_line_width, line_height, num_lines).
fn measure_text_block(text: &str, font_size: f64) -> (f64, f64, usize) {
    let lines = split_lines(text);
    let lh = font_size * 1.05;
    let max_w = lines
        .iter()
        .map(|l| measure(l, font_size).0)
        .fold(0.0_f64, f64::max);
    (max_w, lh, lines.len())
}

/// Like `measure_text_block` but scales the returned width by `TEXT_WIDTH_SCALE`
/// so that layout calculations match the Arial-based reference widths produced by
/// Mermaid's getBBox() calls.  Use this wherever a width drives a coordinate
/// calculation (bounding-box tracking, box sizing) rather than an SVG attribute.
fn measure_layout_width(text: &str, font_size: f64) -> (f64, f64, usize) {
    let (w, lh, n) = measure_text_block(text, font_size);
    (w * TEXT_WIDTH_SCALE, lh, n)
}

pub fn render(diag: &IshikawaDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let pc = vars.primary_color;
    let lc = vars.git_spine_color;
    let pt = match theme {
        crate::theme::Theme::Forest | crate::theme::Theme::Neutral => "#000000",
        _ => vars.text_color,
    };
    let root = match &diag.root {
        Some(r) => r,
        None => return templates::empty_svg().to_string(),
    };

    let causes = &root.children;

    // Split causes into upper (even index) and lower (odd index)
    let upper_causes: Vec<&IshikawaNode> = causes
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .map(|(_, n)| n)
        .collect();
    let lower_causes: Vec<&IshikawaNode> = causes
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 != 0)
        .map(|(_, n)| n)
        .collect();

    let upper_stats = side_stats(&upper_causes);
    let lower_stats = side_stats(&lower_causes);
    let descendant_total = upper_stats.total + lower_stats.total;

    let (mut upper_len, mut lower_len) = if descendant_total > 0 {
        let pool = SPINE_BASE_LENGTH * 2.0;
        let min_len = SPINE_BASE_LENGTH * 0.3;
        (
            (pool * (upper_stats.total as f64 / descendant_total as f64)).max(min_len),
            (pool * (lower_stats.total as f64 / descendant_total as f64)).max(min_len),
        )
    } else {
        (SPINE_BASE_LENGTH, SPINE_BASE_LENGTH)
    };

    let min_spacing = FONT_SIZE * 2.0;
    upper_len = upper_len.max(upper_stats.max as f64 * min_spacing);
    lower_len = lower_len.max(lower_stats.max as f64 * min_spacing);

    let spine_y = upper_len.max(SPINE_BASE_LENGTH);

    // Build SVG elements
    let mut elements: Vec<String> = Vec::new();

    // Arrow marker definition
    let marker_id = "ishikawa-arrow";
    elements.push(templates::arrowhead_marker(marker_id, lc));

    let marker_url = format!("url(#{marker_id})");

    // We work in a coordinate system where the fish HEAD is at (0, spine_y)
    // and the spine extends LEFT (negative X direction).
    // After collecting all elements we compute the bounding box and translate.

    let mut branch_elements: Vec<String> = Vec::new();
    let mut spine_x_left = 0.0_f64;

    // Track content bounding box in local coordinates
    let mut content_min_y = spine_y; // will shrink as we find upper content
    let mut content_max_y = spine_y; // will grow as we find lower content
    let mut content_min_x = 0.0_f64;

    // Draw branches pair by pair, starting 20px left of head
    let pair_count = causes.len().div_ceil(2);
    // cur_spine_x: current leftmost x where each pair's branch attaches.
    // Mermaid initialises spineX = 0 then subtracts 20 → −20.
    // After each pair, spineX = min of all text .getBBox().x in that pair.
    let mut cur_spine_x = -20.0_f64;

    for p in 0..pair_count {
        let upper = causes.get(p * 2);
        let lower = causes.get(p * 2 + 1);

        // leftmost x of all text labels in this pair (tracks Mermaid's getBBox approach)
        let mut pair_leftmost = cur_spine_x;

        for (cause_opt, dir) in [(&upper, -1i32), (&lower, 1i32)] {
            if let Some(cause) = cause_opt {
                let (elems, leftmost_x, min_y, max_y) = draw_branch(
                    cause,
                    cur_spine_x,
                    spine_y,
                    dir,
                    if dir < 0 { upper_len } else { lower_len },
                    &marker_url,
                    lc,
                    pc,
                    pt,
                );
                branch_elements.extend(elems);
                pair_leftmost = pair_leftmost.min(leftmost_x);
                content_min_y = content_min_y.min(min_y);
                content_max_y = content_max_y.max(max_y);
                content_min_x = content_min_x.min(leftmost_x);
            }
        }

        // Advance spine x to leftmost text in this pair (matches Mermaid JS)
        cur_spine_x = pair_leftmost;
        spine_x_left = spine_x_left.min(cur_spine_x);
    }

    // Also update content_min_x based on spine left edge
    content_min_x = content_min_x.min(spine_x_left);

    // Spine line (horizontal)
    elements.push(templates::spine_line(spine_x_left, spine_y, lc));

    elements.extend(branch_elements);

    // Head (kite/fish-head shape) at (0, spine_y)
    // Mermaid wraps the head text and measures the bounding box of the wrapped block.
    let head_label = &root.text;
    let head_font_size = FONT_SIZE; // measured at diagram font size; CSS overrides to 14px for display
    let max_chars_head = ((110.0 / (head_font_size * 0.6)).floor() as usize).max(6);
    let wrapped_head = wrap_text(head_label, max_chars_head);
    let lh = head_font_size * 1.05;
    // JS: w = max(60, tb.width+6) where tb.width is getBBox() at CSS-rendered 14px (Arial, SVG).
    // Liberation Sans at 14px × HEAD_TEXT_SCALE ≈ Arial 14px SVG getBBox (empirical from reference).
    let (tb_width_14_raw, _, n_lines) = measure_text_block(&wrapped_head, 14.0);
    let tb_width_14 = tb_width_14_raw * HEAD_TEXT_SCALE;
    let tb_height = n_lines as f64 * lh;
    // w and h used for both kite path and text centering (one consistent value, as in JS)
    let head_w = (tb_width_14 + 6.0).max(60.0);
    let head_h = (tb_height * 2.0 + 40.0).max(40.0);
    let head_right_x = head_w * 2.4;
    let head_path = format!(
        "M 0 {} L 0 {} Q {} 0 0 {} Z",
        fmt(-head_h / 2.0),
        fmt(head_h / 2.0),
        fmt(head_right_x),
        fmt(-head_h / 2.0),
    );

    // CSS `.ishikawa-head-label` overrides text-anchor to 'middle' and font-size to 14px.
    // With anchor=middle, getBBox().x = -tb.width/2, so JS formula gives:
    //   (w - tb.width)/2 - tb.x + 3 = (w - tb.width)/2 + tb.width/2 + 3 = w/2 + 3
    // → text center at x = head_w/2 + 3
    let head_text_x = head_w / 2.0 + 3.0;
    let head_text_svg = build_multiline_text_weighted(
        &wrapped_head,
        head_text_x,
        0.0,
        "ishikawa-head-label",
        "middle",
        14.0,
        "600",
        pt,
    );

    elements.push(format!(
        r##"<g class="ishikawa-head-group" transform="translate(0,{:.5})"><path class="ishikawa-head" fill="{pc}" stroke="{lc}" stroke-width="2" d="{hp}"/>{ht}</g>"##,
        spine_y,
        pc = pc,
        lc = lc,
        hp = head_path,
        ht = head_text_svg,
    ));

    // Head occupies from y = spine_y - head_h/2 to spine_y + head_h/2
    content_min_y = content_min_y.min(spine_y - head_h / 2.0);
    content_max_y = content_max_y.max(spine_y + head_h / 2.0);

    // Content extents (local coordinates).
    // The head path is "M 0 -h/2 L 0 h/2 Q ctrl_x 0 0 -h/2 Z".  The actual
    // rightmost point of that quadratic Bézier (P0=(0,h/2), P1=(ctrl_x,0),
    // P2=(0,-h/2)) occurs at t=0.5 and equals ctrl_x/2.  Mermaid obtains this
    // via getBBox(); we compute it analytically.
    let content_max_x = head_right_x / 2.0;

    // translate so content maps to padding-offset coords
    let translate_x = PADDING - content_min_x;
    let translate_y = PADDING - content_min_y;

    let content_w = content_max_x - content_min_x;
    let content_h = content_max_y - content_min_y;
    let total_w = content_w + PADDING * 2.0;
    let total_h = content_h + PADDING * 2.0;

    let content = elements.join("");
    templates::svg_root(
        total_w,
        total_h,
        total_w,
        "",
        "",
        translate_x,
        translate_y,
        &content,
    )
}

struct SideStats {
    total: usize,
    max: usize,
}

fn count_descendants(node: &IshikawaNode) -> usize {
    node.children.iter().map(|c| 1 + count_descendants(c)).sum()
}

fn side_stats(nodes: &[&IshikawaNode]) -> SideStats {
    let mut total = 0;
    let mut max = 0;
    for node in nodes {
        let d = count_descendants(node);
        total += d;
        max = max.max(d);
    }
    SideStats { total, max }
}

/// Draw a main branch (bone) from (start_x, start_y) in direction `dir` (-1=up, +1=down).
/// Returns (SVG elements, leftmost_text_x, min_y, max_y).
///
/// leftmost_text_x approximates the min getBBox().x of all text nodes in this branch,
/// which is what Mermaid uses to advance spine_x after each pair.
fn draw_branch(
    node: &IshikawaNode,
    start_x: f64,
    start_y: f64,
    dir: i32,
    length: f64,
    marker_url: &str,
    line_color: &str,
    primary_color: &str,
    primary_text: &str,
) -> (Vec<String>, f64, f64, f64) {
    let mut elements: Vec<String> = Vec::new();
    let children = &node.children;
    let has_children = !children.is_empty();
    let line_len = length * if has_children { 1.0 } else { 0.2 };

    let dx = -COS_A * line_len;
    let dy = SIN_A * line_len * dir as f64;
    let end_x = start_x + dx;
    let end_y = start_y + dy;

    // Track bounding box
    let mut min_y = start_y.min(end_y);
    let mut max_y = start_y.max(end_y);

    // Main branch line
    elements.push(templates::branch_line(
        start_x, start_y, end_x, end_y, marker_url, line_color,
    ));

    // Cause label at end of branch
    // Mermaid: drawCauseLabel(svg, node.text, endX, endY, direction, fontSize)
    // Text is placed at (endX, endY + 11*direction) with text-anchor=middle
    // A rect box is drawn around the text bbox.
    let cause_label_y = end_y + 11.0 * dir as f64;
    let cause_text_svg = build_multiline_text(
        node.text.as_str(),
        end_x,
        cause_label_y,
        "ishikawa-label cause",
        "middle",
        FONT_SIZE,
        primary_text,
    );
    let (tw, _, n_lines_cause) = measure_layout_width(node.text.as_str(), FONT_SIZE);
    let lh = FONT_SIZE * 1.05;
    let tb_h = n_lines_cause as f64 * lh;
    // Match Mermaid's getBBox()-based rect positioning:
    // The browser's getBBox().y for dominant-baseline:middle is approximately
    // 0.57 of the box height above the text y coordinate (more space below than above).
    let box_x = end_x - tw / 2.0 - 20.0;
    let box_w = tw + 40.0;
    let box_h = tb_h + 4.0;
    let box_y = cause_label_y - box_h * 0.57;
    elements.push(format!(
        r#"<g class="ishikawa-label-group">{}{}</g>"#,
        templates::cause_label_rect(box_x, box_y, box_w, box_h, primary_color, line_color),
        cause_text_svg,
    ));

    // The leftmost x from the cause label text
    let mut leftmost_x = box_x;
    min_y = min_y.min(box_y);
    max_y = max_y.max(box_y + box_h);

    if !has_children {
        return (elements, leftmost_x, min_y, max_y);
    }

    // Flatten the tree for sub-bones
    let (entries, y_order) = flatten_tree(children, dir);
    let entry_count = entries.len();
    let mut ys = vec![0.0f64; entry_count];
    for (slot, &entry_idx) in y_order.iter().enumerate() {
        ys[entry_idx] = start_y + dy * ((slot as f64 + 1.0) / (entry_count as f64 + 1.0));
    }

    struct BoneInfo {
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        child_count: usize,
        children_drawn: usize,
    }

    let mut bones: std::collections::HashMap<i32, BoneInfo> = std::collections::HashMap::new();
    bones.insert(
        -1,
        BoneInfo {
            x0: start_x,
            y0: start_y,
            x1: end_x,
            y1: end_y,
            child_count: children.len(),
            children_drawn: 0,
        },
    );

    let diag_x = -COS_A;
    let diag_y = SIN_A * dir as f64;

    for (i, entry) in entries.iter().enumerate() {
        let y = ys[i];
        let par = bones.get(&entry.parent_index).unwrap();

        let par_x0 = par.x0;
        let par_y0 = par.y0;
        let par_x1 = par.x1;
        let par_y1 = par.y1;
        let par_child_count = par.child_count;
        let par_children_drawn = par.children_drawn;

        let (bx0, by0, bx1);
        let grp_class;
        let sub_el;
        let text_el;
        let text_lx; // leftmost x of text for tracking

        if entry.depth.is_multiple_of(2) {
            // Even depth: horizontal sub-bone
            // Attach to parent diagonal at target Y
            let dy_p = par_y1 - par_y0;
            let t = if dy_p.abs() > 1e-9 {
                (y - par_y0) / dy_p
            } else {
                0.5
            };
            bx0 = lerp(par_x0, par_x1, t.clamp(0.0, 1.0));
            by0 = y;
            let stub_len = if entry.child_count > 0 {
                BONE_BASE + entry.child_count as f64 * BONE_PER_CHILD
            } else {
                BONE_STUB
            };
            bx1 = bx0 - stub_len;

            sub_el = templates::sub_branch_line(bx0, y, bx1, y, marker_url, line_color);

            // drawMultilineText at (bx1, y) text-anchor=end class=ishikawa-label align
            // Mermaid: drawMultilineText(grp, e.text, bx1, y, "ishikawa-label align", "end", fontSize)
            // text placed at x=bx1, y = y - (lines-1)*lh/2
            let (tw_sub, _, n_sub) = measure_layout_width(&entry.text, FONT_SIZE);
            let _ = n_sub;
            text_el = build_multiline_text(
                &entry.text,
                bx1,
                y,
                "ishikawa-label align",
                "end",
                FONT_SIZE,
                primary_text,
            );
            // leftmost text x: with text-anchor=end, text extends left of bx1
            text_lx = bx1 - tw_sub;
            grp_class = "ishikawa-sub-group";

            min_y = min_y.min(y - FONT_SIZE);
            max_y = max_y.max(y + FONT_SIZE);
        } else {
            // Odd depth: diagonal sub-bone
            let k = par_children_drawn as f64;
            let nc = par_child_count as f64;
            let frac = (nc - k) / (nc + 1.0);
            bx0 = lerp(par_x0, par_x1, frac);
            by0 = par_y0;
            bx1 = bx0 + diag_x * ((y - by0) / diag_y);

            sub_el = templates::sub_branch_line(bx0, by0, bx1, y, marker_url, line_color);

            // drawMultilineText at (bx1, y) text-anchor=end
            // class: "ishikawa-label up" if dir<0, "ishikawa-label down" if dir>0
            let odd_class = if dir < 0 {
                "ishikawa-label up"
            } else {
                "ishikawa-label down"
            };
            let (tw_sub, _, _) = measure_layout_width(&entry.text, FONT_SIZE);
            text_el = build_multiline_text(
                &entry.text,
                bx1,
                y,
                odd_class,
                "end",
                FONT_SIZE,
                primary_text,
            );
            text_lx = bx1 - tw_sub;
            grp_class = "ishikawa-sub-group";

            min_y = min_y.min(y - FONT_SIZE);
            max_y = max_y.max(y + FONT_SIZE);
        }

        leftmost_x = leftmost_x.min(text_lx);

        elements.push(format!(
            r#"<g class="{}">{}{}</g>"#,
            grp_class, sub_el, text_el
        ));

        if entry.child_count > 0 {
            bones.insert(
                i as i32,
                BoneInfo {
                    x0: bx0,
                    y0: by0,
                    x1: bx1,
                    y1: y,
                    child_count: entry.child_count,
                    children_drawn: 0,
                },
            );
        }
        // update parent's children_drawn
        if let Some(par_mut) = bones.get_mut(&entry.parent_index) {
            par_mut.children_drawn += 1;
        }
    }

    (elements, leftmost_x, min_y, max_y)
}

/// Build an SVG multiline text element matching Mermaid's drawMultilineText().
///
/// Mermaid places the text such that the first tspan is at:
///   y_base = y - (lines.length - 1) * lh / 2
/// and each subsequent tspan has dy = lh.
fn build_multiline_text(
    text: &str,
    x: f64,
    y: f64,
    cls: &str,
    anchor: &str,
    font_size: f64,
    fill: &str,
) -> String {
    build_multiline_text_weighted(text, x, y, cls, anchor, font_size, "", fill)
}

fn build_multiline_text_weighted(
    text: &str,
    x: f64,
    y: f64,
    cls: &str,
    anchor: &str,
    font_size: f64,
    font_weight: &str,
    fill: &str,
) -> String {
    let lines = split_lines(text);
    let lh = font_size * 1.05;
    let y_first = y - (lines.len() as f64 - 1.0) * lh / 2.0;
    let mut tspans = String::new();
    for (i, line) in lines.iter().enumerate() {
        let dy = if i == 0 {
            "0".to_string()
        } else {
            format!("{:.5}", lh)
        };
        tspans.push_str(&format!(
            r#"<tspan x="{:.5}" dy="{}">{}</tspan>"#,
            x,
            dy,
            esc(line)
        ));
    }
    let weight_attr = if font_weight.is_empty() {
        String::new()
    } else {
        format!(" font-weight=\"{}\"", font_weight)
    };
    format!(
        r#"<text class="{}" fill="{}" text-anchor="{}" x="{:.5}" y="{:.5}" font-size="{}"{} dominant-baseline="middle">{}</text>"#,
        cls, fill, anchor, x, y_first, font_size, weight_attr, tspans,
    )
}

#[derive(Debug)]
struct LabelEntry {
    text: String,
    depth: usize,
    parent_index: i32,
    child_count: usize,
}

/// Flatten the tree into a pre/post-order sequence matching ishikawaRenderer.ts flattenTree().
/// Each entry's text is wrapped at 15 chars (as done in Mermaid's flattenTree).
fn flatten_tree(children: &[IshikawaNode], dir: i32) -> (Vec<LabelEntry>, Vec<usize>) {
    let mut entries: Vec<LabelEntry> = Vec::new();
    let mut y_order: Vec<usize> = Vec::new();

    fn walk(
        nodes: &[IshikawaNode],
        pid: i32,
        depth: usize,
        dir: i32,
        entries: &mut Vec<LabelEntry>,
        y_order: &mut Vec<usize>,
    ) {
        let ordered: Vec<&IshikawaNode> = if dir < 0 {
            nodes.iter().rev().collect()
        } else {
            nodes.iter().collect()
        };
        for child in ordered {
            let idx = entries.len() as i32;
            let gc = &child.children;
            // Mermaid wraps at 15 chars in flattenTree
            let wrapped = wrap_text_static(&child.text, 15);
            entries.push(LabelEntry {
                depth,
                text: wrapped,
                parent_index: pid,
                child_count: gc.len(),
            });
            if depth.is_multiple_of(2) {
                // even: pre-order (push before children)
                y_order.push(idx as usize);
                if !gc.is_empty() {
                    walk(gc, idx, depth + 1, dir, entries, y_order);
                }
            } else {
                // odd: post-order (push after children)
                if !gc.is_empty() {
                    walk(gc, idx, depth + 1, dir, entries, y_order);
                }
                y_order.push(idx as usize);
            }
        }
    }

    walk(children, -1, 2, dir, &mut entries, &mut y_order);
    (entries, y_order)
}

/// Static version of wrap_text for use inside fn item (no captures).
fn wrap_text_static(text: &str, max_chars: usize) -> String {
    wrap_text(text, max_chars)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::ishikawa::parser;

    #[test]
    fn render_produces_svg() {
        let input = "fishbone\n    Equipment failure\n        Worn parts\n        Calibration\n    Human error\n        Training\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("ishikawa-spine"));
        // Head text may be wrapped into tspan elements, so check for a word fragment
        assert!(svg.contains("Equipment"));
    }

    #[test]
    fn empty_root_returns_empty_svg() {
        let diag = IshikawaDiagram {
            title: None,
            root: None,
        };
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("Empty Ishikawa"));
    }

    #[test]
    fn lower_branch_within_viewbox() {
        // Regression: lower cause label must not exceed the SVG viewBox height.
        let input = "ishikawa\n    Effect: [Quality Problem]\n    Cause1: [Materials]\n        SubCause1: [Bad input]\n    Cause2: [Methods]\n        SubCause2: [Wrong process]";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);

        // Extract viewBox height
        let vb_start = svg.find("viewBox=\"0 0 ").expect("viewBox not found");
        let rest = &svg[vb_start + 13..];
        let parts: Vec<f64> = rest
            .split_whitespace()
            .take(2)
            .filter_map(|s| s.trim_end_matches('"').parse().ok())
            .collect();
        assert_eq!(parts.len(), 2, "could not parse viewBox");
        let _total_w = parts[0];
        let total_h = parts[1];

        // The SVG height must be >= 500 to contain both the upper and lower branches
        // (each ~250px branch + labels).  Previous bug produced ~540 but clipped content.
        assert!(
            total_h >= 530.0,
            "SVG height {total_h} too small — lower content is clipped"
        );
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "ishikawa\n    Effect: [Quality Problem]\n    Cause1: [Materials]\n        SubCause1: [Bad input]\n    Cause2: [Methods]\n        SubCause2: [Wrong process]";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
