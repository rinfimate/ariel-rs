use super::constants::*;
use super::parser::{IshikawaDiagram, IshikawaNode};
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
#[allow(unused_imports)]
use super::templates;
use crate::text::measure;
use crate::theme::Theme;

pub fn render(diag: &IshikawaDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let root = match &diag.root {
        Some(r) => r,
        None => return empty_svg(),
    };

    let causes = &root.children;
    let title_text = diag.title.as_deref().unwrap_or("");

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
    elements.push(format!(
        r#"<defs><marker id="{mid}" viewBox="0 0 10 10" refX="0" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M 10 0 L 0 5 L 10 10 Z" class="ishikawa-arrow"/></marker></defs>"#,
        mid = marker_id
    ));

    let marker_url = format!("url(#{marker_id})");

    // We work in a coordinate system where the fish HEAD is at (0, spine_y)
    // and the spine extends LEFT (negative X direction).
    // After collecting all elements we compute the bounding box and translate.

    let mut branch_elements: Vec<String> = Vec::new();
    let mut label_min_x = 0.0_f64;

    // Draw branches pair by pair
    let pair_count = causes.len().div_ceil(2);
    let mut cur_spine_x = 0.0_f64 - 20.0; // starts slightly left of head x=0

    for p in 0..pair_count {
        let upper = causes.get(p * 2);
        let lower = causes.get(p * 2 + 1);

        for (cause_opt, dir) in [(&upper, -1i32), (&lower, 1i32)] {
            if let Some(cause) = cause_opt {
                let (elems, leftmost) = draw_branch(
                    cause,
                    cur_spine_x,
                    spine_y,
                    dir,
                    if dir < 0 { upper_len } else { lower_len },
                    &marker_url,
                );
                branch_elements.extend(elems);
                label_min_x = label_min_x.min(leftmost);
            }
        }
        // Advance spine_x roughly based on typical branch width
        // In Mermaid this is computed from getBBox; we approximate
        cur_spine_x = label_min_x - 10.0;
    }

    let spine_x_left = cur_spine_x;

    // Spine line (horizontal, from leftmost to head x=0)
    elements.push(format!(
        r#"<line class="ishikawa-spine" x1="{:.1}" y1="{:.1}" x2="0" y2="{:.1}"/>"#,
        spine_x_left, spine_y, spine_y
    ));

    elements.extend(branch_elements);

    // Head (kite/fish-head shape) at (0, spine_y)
    let head_label = &root.text;
    let (head_w_text, _) = measure(head_label, FONT_SIZE);
    let head_w = (head_w_text + 6.0).max(60.0);
    let head_h = 80.0_f64.max(FONT_SIZE * 3.0 + 40.0);
    let head_path = format!(
        "M 0 {} L 0 {} Q {} 0 0 {} Z",
        fmt(-head_h / 2.0),
        fmt(head_h / 2.0),
        fmt(head_w * 2.4),
        fmt(-head_h / 2.0),
    );
    elements.push(format!(
        r#"<g class="ishikawa-head-group" transform="translate(0,{:.1})"><path class="ishikawa-head" d="{}"/><text class="ishikawa-head-label" text-anchor="middle" x="{:.1}" y="{:.1}" font-size="{}">{}</text></g>"#,
        spine_y,
        head_path,
        head_w * 0.8,
        0.0,
        FONT_SIZE,
        escape(head_label),
    ));

    // Compute bounding box for viewBox
    // We translate so that spine_x_left is at x=PADDING
    let translate_x = PADDING - spine_x_left;
    let translate_y = PADDING + upper_len;

    let total_w = -spine_x_left + head_w * 2.4 + PADDING * 2.0;
    let total_h = upper_len + lower_len + PADDING * 2.0;

    // Title
    let title_part = if !title_text.is_empty() {
        format!(
            r#"<text class="ishikawa-title" x="{:.1}" y="20" text-anchor="middle" font-size="16" font-weight="bold">{}</text>"#,
            total_w / 2.0,
            escape(title_text),
        )
    } else {
        String::new()
    };

    let style = build_style(ff);
    let content = elements.join("");
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {vw:.1} {vh:.1}" width="100%" style="max-width:{mw:.0}px"><style>{style}</style>{title_part}<g transform="translate({tx:.1},{ty:.1})">{content}</g></svg>"#,
        vw = total_w,
        vh = total_h + if title_text.is_empty() { 0.0 } else { 30.0 },
        mw = total_w,
        title_part = title_part,
        tx = translate_x,
        ty = translate_y,
        content = content,
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
/// Returns SVG element strings and the leftmost x coordinate used.
fn draw_branch(
    node: &IshikawaNode,
    start_x: f64,
    start_y: f64,
    dir: i32,
    length: f64,
    marker_url: &str,
) -> (Vec<String>, f64) {
    let mut elements: Vec<String> = Vec::new();
    let children = &node.children;
    let has_children = !children.is_empty();
    let line_len = length * if has_children { 1.0 } else { 0.2 };

    let dx = -COS_A * line_len;
    let dy = SIN_A * line_len * dir as f64;
    let end_x = start_x + dx;
    let end_y = start_y + dy;

    // Main branch line
    elements.push(format!(
        r#"<line class="ishikawa-branch" x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" marker-start="{}"/>"#,
        start_x, start_y, end_x, end_y, marker_url
    ));

    // Cause label box at end of branch
    let (label_elems, label_x) = draw_cause_label(&node.text, end_x, end_y, dir);
    elements.extend(label_elems);
    let mut leftmost = label_x;

    if !has_children {
        return (elements, leftmost);
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
        if entry.depth % 2 == 0 {
            // Horizontal bone: attach to parent diagonal at target Y
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

            elements.push(format!(
                r#"<line class="ishikawa-sub-branch" x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" marker-start="{}"/>"#,
                bx0, by0, bx1, by0, marker_url
            ));
            let label_anchor = "end";
            elements.push(format!(
                r#"<text class="ishikawa-label align" text-anchor="{}" x="{:.2}" y="{:.2}" font-size="{}">{}</text>"#,
                label_anchor, bx1 - 2.0, by0 + FONT_SIZE * 0.35, FONT_SIZE,
                escape(&entry.text)
            ));
            leftmost = leftmost.min(bx1 - measure(&entry.text, FONT_SIZE).0 - 4.0);
        } else {
            // Diagonal bone
            let k = par_children_drawn as f64;
            let nc = par_child_count as f64;
            let frac = (nc - k) / (nc + 1.0);
            bx0 = lerp(par_x0, par_x1, frac);
            by0 = par_y0;
            bx1 = bx0 + diag_x * ((y - by0) / diag_y);

            elements.push(format!(
                r#"<line class="ishikawa-sub-branch" x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" marker-start="{}"/>"#,
                bx0, by0, bx1, y, marker_url
            ));
            let y_lbl = if dir < 0 {
                y - 2.0
            } else {
                y + FONT_SIZE + 2.0
            };
            elements.push(format!(
                r#"<text class="ishikawa-label" text-anchor="end" x="{:.2}" y="{:.2}" font-size="{}">{}</text>"#,
                bx1, y_lbl, FONT_SIZE, escape(&entry.text)
            ));
            leftmost = leftmost.min(bx1 - measure(&entry.text, FONT_SIZE).0 - 4.0);
        }

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

    (elements, leftmost)
}

fn draw_cause_label(text: &str, x: f64, y: f64, dir: i32) -> (Vec<String>, f64) {
    let (tw, _) = measure(text, FONT_SIZE);
    let box_x = x - tw / 2.0 - 20.0;
    let box_y = if dir < 0 {
        y - FONT_SIZE - 4.0
    } else {
        y + 4.0
    };
    let box_w = tw + 40.0;
    let box_h = FONT_SIZE + 4.0;
    let text_y = if dir < 0 {
        y - 2.0
    } else {
        y + FONT_SIZE + 4.0
    };

    let elements = vec![
        format!(
            r#"<rect class="ishikawa-label-box" x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}"/>"#,
            box_x, box_y, box_w, box_h
        ),
        format!(
            r#"<text class="ishikawa-label cause" text-anchor="middle" x="{:.2}" y="{:.2}" font-size="{}">{}</text>"#,
            x,
            text_y,
            FONT_SIZE,
            escape(text)
        ),
    ];
    (elements, box_x)
}

#[derive(Debug)]
struct LabelEntry {
    text: String,
    depth: usize,
    parent_index: i32,
    child_count: usize,
}

/// Flatten the tree into a pre/post-order sequence matching ishikawaRenderer.ts flattenTree().
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
            entries.push(LabelEntry {
                depth,
                text: child.text.clone(),
                parent_index: pid,
                child_count: gc.len(),
            });
            if depth.is_multiple_of(2) {
                // even: pre-order
                y_order.push(idx as usize);
                if !gc.is_empty() {
                    walk(gc, idx, depth + 1, dir, entries, y_order);
                }
            } else {
                // odd: post-order
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

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn fmt(v: f64) -> String {
    format!("{:.2}", v)
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn empty_svg() -> String {
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50"><text x="10" y="30" font-size="14">Empty Ishikawa</text></svg>"#.to_string()
}

fn build_style(ff: &str) -> String {
    format!(
        r#"
.ishikawa-spine {{ stroke: #333; stroke-width: 3; fill: none; }}
.ishikawa-branch {{ stroke: #333; stroke-width: 2; fill: none; }}
.ishikawa-sub-branch {{ stroke: #555; stroke-width: 1.5; fill: none; }}
.ishikawa-head {{ fill: #fff; stroke: #333; stroke-width: 2; }}
.ishikawa-head-label {{ fill: #333; font-family: {ff}; font-weight: bold; }}
.ishikawa-label {{ fill: #333; font-family: {ff}; }}
.ishikawa-label-box {{ fill: #fff; stroke: #333; stroke-width: 1; rx: 3; ry: 3; }}
.ishikawa-arrow {{ fill: #333; }}
.ishikawa-title {{ fill: #333; font-family: {ff}; }}
"#,
        ff = ff,
    )
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
        assert!(svg.contains("Equipment failure"));
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
    #[ignore = "platform-specific float precision — run locally"]
    fn snapshot_default_theme() {
        let input = "ishikawa\n    Effect: [Quality Problem]\n    Cause1: [Materials]\n        SubCause1: [Bad input]\n    Cause2: [Methods]\n        SubCause2: [Wrong process]";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(svg);
    }
}
