use super::constants::*;
use super::parser::{
    Commit, DiagramDirection, GitGraphDiagram, COMMIT_CHERRY_PICK, COMMIT_HIGHLIGHT, COMMIT_MERGE,
    COMMIT_REVERSE,
};
/// Faithful Rust port of Mermaid's gitGraphRenderer.ts
///
/// Key design:
/// - LR (left-to-right, default): branches are horizontal lanes stacked vertically.
///   Each commit advances x by COMMIT_STEP+LAYOUT_OFFSET. Lane y = branchPos.
/// - TB/BT: branches are vertical lanes side-by-side; commits advance y.
/// - Commits are circles; merge commits have an inner circle; REVERSE has an X cross;
///   HIGHLIGHT uses a square; CHERRY_PICK has eye-like circles.
/// - Arrows connect each commit to its parents using arc paths.
/// - Branch labels rendered as rectangles with text at left margin.
/// - Colors cycle through THEME_COLOR_LIMIT (8) css classes.
#[allow(unused_imports)]
use super::templates::{self, build_style, esc};
use crate::text::measure;
use crate::theme::Theme;
use std::collections::HashMap;

// ── Position bookkeeping ─────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
struct BranchPosition {
    pos: f64,
    index: usize,
}

#[derive(Clone, Copy, Debug)]
struct CommitPos {
    x: f64,
    y: f64,
}

fn color_index(raw: usize) -> usize {
    raw % THEME_COLOR_LIMIT
}

// ── Main render function ─────────────────────────────────────────────────────

pub fn render(diag: &GitGraphDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let dir = diag.direction;

    // Build a commit lookup by id for arrow color resolution
    let commit_by_id: HashMap<&str, &Commit> =
        diag.commits.iter().map(|c| (c.id.as_str(), c)).collect();

    // ── Step 1: Branch positions ──────────────────────────────────────────────
    // LR mode: each branch is a horizontal lane separated by 90 units vertically
    // (matching Mermaid's gitGraphRenderer lane spacing).
    let mut branch_pos_map: HashMap<String, BranchPosition> = HashMap::new();
    {
        let mut pos: f64 = 0.0;
        for (idx, branch) in diag.branches.iter().enumerate() {
            let label_w = measure(&branch.name, BRANCH_FONT_SIZE).0 * BRANCH_FONT_SCALE;
            branch_pos_map.insert(branch.name.clone(), BranchPosition { pos, index: idx });
            let tb_extra = if dir == DiagramDirection::TB || dir == DiagramDirection::BT {
                label_w / 2.0
            } else {
                0.0
            };
            pos += 90.0 + (if ROTATE_COMMIT_LABEL { 40.0 } else { 0.0 }) + tb_extra;
        }
    }

    // ── Step 2: Commit positions ───────────────────────────────────────────────
    let mut sorted: Vec<&Commit> = diag.commits.iter().collect();
    sorted.sort_by_key(|c| c.seq);

    let mut cpos_map: HashMap<String, CommitPos> = HashMap::new();
    let mut max_pos: f64 = 0.0;
    let mut pos: f64 = if dir == DiagramDirection::TB || dir == DiagramDirection::BT {
        DEFAULT_POS
    } else {
        0.0
    };

    for commit in &sorted {
        let bp = branch_pos_map
            .get(&commit.branch)
            .copied()
            .unwrap_or(BranchPosition { pos: 0.0, index: 0 });

        let (x, y) = match dir {
            DiagramDirection::TB | DiagramDirection::BT => (bp.pos, pos + LAYOUT_OFFSET),
            DiagramDirection::LR => (pos + LAYOUT_OFFSET, bp.pos - 2.0),
        };
        cpos_map.insert(commit.id.clone(), CommitPos { x, y });

        let advance = match dir {
            DiagramDirection::TB | DiagramDirection::BT => y,
            DiagramDirection::LR => x,
        };
        if advance > max_pos {
            max_pos = advance;
        }
        pos += COMMIT_STEP + LAYOUT_OFFSET;
    }
    max_pos += COMMIT_STEP;

    // ── Step 3: Canvas size ───────────────────────────────────────────────────
    // For LR mode with tags: tags render above y=-2 (main branch), extending
    // up ~29 local units.  Add a top margin so they are not clipped.
    let has_tags = diag.commits.iter().any(|c| !c.tags.is_empty());
    // Tag badges render above the main branch (y=-2), extending up ~29 local units.
    // Reference: viewBox y-start shifts from -20.5 to -36.7 (Δ=16.2) for tags.
    // So translate_y for tagged = 36.7 (non-tagged = 20).  tag_top_margin = 16.7.
    let tag_top_margin: f64 = if has_tags && dir == DiagramDirection::LR {
        16.7
    } else {
        0.0
    };

    let (svg_w, svg_h, x_offset, translate_y) = match dir {
        DiagramDirection::LR => {
            // x_offset formula: empirically tuned to match Mermaid's reference x_offset.
            // Reference: x_offset = label_w_browser + 63, where label_w_browser ≈ our_measure * 1.075.
            let max_label_w = diag
                .branches
                .iter()
                .map(|b| measure(&b.name, BRANCH_FONT_SIZE).0 * BRANCH_FONT_SCALE + 63.0)
                .fold(0.0_f64, f64::max);
            let left = max_label_w;
            // Height: matches the Mermaid reference viewBox height.
            // Reference uses rotated commit labels which extend below the last branch.
            // Empirically: h ≈ translate_y + (n-1)*90 + bottom_margin
            //   where bottom_margin = 62.5 - (n-2)*13  (n=2 → 62.5, n=3 → 49.5, etc.)
            let n = diag.branches.len() as f64;
            // For n≥3, the bottom margin also depends on the longest commit label
            // at the lowest branch — longer labels extend further when rotated 45°.
            // Baseline rect_w ≈31 (short label, e.g. "Hotfix") is baked into 49.5.
            // Extra extension = (max_rect_w - baseline) * inv_sqrt2.
            let label_bottom_extra = if n >= 3.0 {
                let lowest_branch = diag.branches.last().map(|b| b.name.as_str()).unwrap_or("");
                let max_rect_w = diag
                    .commits
                    .iter()
                    .filter(|c| c.branch == lowest_branch)
                    .map(|c| measure(&c.id, 10.0).0 * COMMIT_LABEL_FONT_SCALE + 4.0)
                    .fold(0.0_f64, f64::max);
                ((max_rect_w - 31.0).max(0.0)) * std::f64::consts::FRAC_1_SQRT_2
            } else {
                0.0
            };
            let bottom_margin = 62.5 - (n - 2.0) * 13.0 + label_bottom_extra;
            // When tags are present, height is slightly larger (extra 1.47 from Mermaid)
            let tag_h_extra = if has_tags { 1.47 } else { 0.0 };
            let h = (20.0 + tag_top_margin) + (n - 1.0) * 90.0 + bottom_margin + tag_h_extra;
            // Width: left_margin + branch_extent + right_margin (8 units, matching reference)
            (left + max_pos + 8.0, h, left, 20.0 + tag_top_margin)
        }
        DiagramDirection::TB => {
            let w = diag.branches.len() as f64 * 60.0 + 80.0;
            (w, max_pos + 80.0, 40.0, 20.0)
        }
        DiagramDirection::BT => {
            let w = diag.branches.len() as f64 * 60.0 + 80.0;
            (w, max_pos + 80.0, 40.0, 20.0)
        }
    };

    // ── Step 4: Emit SVG ──────────────────────────────────────────────────────
    let id = "mermaid-gitgraph";
    let mut out = String::new();

    out += &format!(
        concat!(
            r#"<svg id="{id}" xmlns="http://www.w3.org/2000/svg""#,
            r#" width="100%" height="{h:.1}" viewBox="0 0 {w:.1} {h:.1}""#,
            r#" style="max-width: {w:.1}px;""#,
            r#" role="graphics-document document" aria-roledescription="git-graph">"#
        ),
        id = id,
        w = svg_w,
        h = svg_h
    );

    out += &format!("<style>{}</style>", build_style(id, ff));
    out += &build_defs(id);
    out += &format!(
        r#"<g transform="translate({:.1},{:.1})">"#,
        x_offset, translate_y
    );

    // ── Branch spines & labels ────────────────────────────────────────────────
    out += r#"<g class="branches">"#;
    for branch in &diag.branches {
        let bp = match branch_pos_map.get(&branch.name) {
            Some(b) => *b,
            None => continue,
        };
        let ci = color_index(bp.index);
        let (fill, stroke) = GIT_COLORS[ci];
        let label_w = measure(&branch.name, BRANCH_FONT_SIZE).0 * BRANCH_FONT_SCALE;
        let box_w = label_w + BRANCH_LABEL_PADDING * 2.0;

        let text_color = BRANCH_LABEL_TEXT_COLORS[ci % THEME_COLOR_LIMIT];
        match dir {
            DiagramDirection::LR => {
                let y = bp.pos - 2.0;
                // Branch spine: dark grey line with dash-2 pattern (matches reference .branch CSS)
                out += &format!(
                    "<line x1=\"0\" y1=\"{y:.1}\" x2=\"{mx:.1}\" y2=\"{y:.1}\" stroke=\"#333333\" stroke-width=\"1\" stroke-dasharray=\"2\"/>",
                    y = y, mx = max_pos
                );
                // Position box: right edge at x=-35 to match reference spacing
                let bx = -(box_w + 35.0);
                let by = y - 10.0;
                // Branch label background: filled with branch color, no stroke
                out += &format!(
                    concat!(
                        r#"<rect x="{bx:.1}" y="{by:.1}" width="{bw:.1}" height="20""#,
                        r#" rx="4" ry="4" fill="{f}"/>"#
                    ),
                    bx = bx,
                    by = by,
                    bw = box_w,
                    f = fill
                );
                // Branch label text: color depends on branch index
                out += &format!(
                    "<text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"16\" fill=\"{tc}\" font-family=\"{ff}\" text-anchor=\"start\">{name}</text>",
                    tx = bx + BRANCH_LABEL_PADDING,
                    ty = y + 5.0,
                    tc = text_color,
                    name = esc(&branch.name)
                );
            }
            DiagramDirection::TB => {
                let x = bp.pos;
                out += &format!(
                    concat!(
                        r#"<line x1="{x:.1}" y1="{dp:.1}" x2="{x:.1}" y2="{mx:.1}""#,
                        r#" stroke="{s}" stroke-width="2" stroke-dasharray="4 2"/>"#
                    ),
                    x = x,
                    dp = DEFAULT_POS,
                    mx = max_pos,
                    s = stroke
                );
                let bx = x - box_w / 2.0;
                out += &format!(
                    concat!(
                        r#"<rect x="{bx:.1}" y="0" width="{bw:.1}" height="20""#,
                        r#" rx="4" fill="{f}" stroke="{s}" stroke-width="1"/>"#
                    ),
                    bx = bx,
                    bw = box_w,
                    f = fill,
                    s = stroke
                );
                out += &format!(
                    "<text x=\"{tx:.1}\" y=\"14\" font-size=\"14\" fill=\"#333\" font-family=\"{ff}\" text-anchor=\"start\">{name}</text>",
                    tx = bx + BRANCH_LABEL_PADDING,
                    name = esc(&branch.name)
                );
            }
            DiagramDirection::BT => {
                let x = bp.pos;
                let my = max_pos + 5.0;
                out += &format!(
                    concat!(
                        r#"<line x1="{x:.1}" y1="{dp:.1}" x2="{x:.1}" y2="{mx:.1}""#,
                        r#" stroke="{s}" stroke-width="2" stroke-dasharray="4 2"/>"#
                    ),
                    x = x,
                    dp = DEFAULT_POS,
                    mx = max_pos,
                    s = stroke
                );
                let bx = x - box_w / 2.0;
                out += &format!(
                    concat!(
                        r#"<rect x="{bx:.1}" y="{my:.1}" width="{bw:.1}" height="20""#,
                        r#" rx="4" fill="{f}" stroke="{s}" stroke-width="1"/>"#
                    ),
                    bx = bx,
                    my = my,
                    bw = box_w,
                    f = fill,
                    s = stroke
                );
                out += &format!(
                    "<text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"14\" fill=\"#333\" font-family=\"{ff}\" text-anchor=\"start\">{name}</text>",
                    tx = bx + BRANCH_LABEL_PADDING,
                    ty = my + 14.0,
                    name = esc(&branch.name)
                );
            }
        }
    }
    out += "</g>"; // branches

    // ── Arrows ────────────────────────────────────────────────────────────────
    out += r#"<g class="commit-arrows">"#;
    for commit in &sorted {
        if commit.parents.is_empty() {
            continue;
        }
        let p2 = match cpos_map.get(&commit.id) {
            Some(p) => *p,
            None => continue,
        };
        let branch_idx = branch_pos_map
            .get(&commit.branch)
            .map(|b| b.index)
            .unwrap_or(0);
        let ci = color_index(branch_idx);
        let default_stroke = GIT_COLORS[ci].1;

        for (pidx, parent_id) in commit.parents.iter().enumerate() {
            let p1 = match cpos_map.get(parent_id) {
                Some(p) => *p,
                None => continue,
            };

            // For merge commits the second parent arrow uses the source branch color
            let arrow_stroke = if commit.commit_type == COMMIT_MERGE && pidx > 0 {
                if let Some(pc) = commit_by_id.get(parent_id.as_str()) {
                    let pci =
                        color_index(branch_pos_map.get(&pc.branch).map(|b| b.index).unwrap_or(0));
                    GIT_COLORS[pci].1
                } else {
                    default_stroke
                }
            } else {
                default_stroke
            };

            let d = draw_arrow(p1, p2, dir, commit);
            // Reference uses CSS class .arrow{stroke-width:8;stroke-linecap:round;fill:none;}
            // so arrows are thick rounded lines without arrowhead markers.
            out += &format!(
                "<path d=\"{d}\" class=\"arrow arrow{ci}\" fill=\"none\" stroke=\"{s}\" stroke-width=\"8\" stroke-linecap=\"round\"/>",
                d = d, s = arrow_stroke, ci = branch_idx % THEME_COLOR_LIMIT
            );
        }
    }
    out += "</g>"; // commit-arrows

    // ── Commit bullets ────────────────────────────────────────────────────────
    out += r#"<g class="commit-bullets">"#;
    for commit in &sorted {
        let cp = match cpos_map.get(&commit.id) {
            Some(p) => *p,
            None => continue,
        };
        let ci = color_index(
            branch_pos_map
                .get(&commit.branch)
                .map(|b| b.index)
                .unwrap_or(0),
        );
        let (fill, stroke) = GIT_COLORS[ci];
        let sym = commit.custom_type.unwrap_or(commit.commit_type);
        draw_commit_bullet(&mut out, commit, cp, sym, fill, stroke);
    }
    out += "</g>"; // commit-bullets

    // ── Commit labels ─────────────────────────────────────────────────────────
    if SHOW_COMMIT_LABEL {
        out += r#"<g class="commit-labels">"#;
        for commit in &sorted {
            let cp = match cpos_map.get(&commit.id) {
                Some(p) => *p,
                None => continue,
            };
            draw_commit_label(&mut out, commit, cp, dir, ff);
            draw_commit_tags(&mut out, commit, cp, dir, ff);
        }
        out += "</g>"; // commit-labels
    }

    out += "</g>"; // main translate group
    out += "</svg>";
    out
}

// ── Arrow path construction ───────────────────────────────────────────────────
// Faithful port of drawArrow() from gitGraphRenderer.ts.
// For LR cross-lane arrows Mermaid uses a single corner-arc of radius 20:
//   - Downward (p1.y < p2.y): go vertical from p1 to (p1.x, p2.y-20),
//     arc r=20 to (p1.x+20, p2.y), then horizontal to p2.
//   - Upward   (p1.y > p2.y): go horizontal from p1 to (p2.x-20, p1.y),
//     arc r=20 to (p2.x, p1.y-20), then vertical to p2.
// Variables `arc` and `arc2` in the TS source are embedded SVG arc command strings.
// We inline them as literals in the Rust format strings.

fn draw_arrow(p1: CommitPos, p2: CommitPos, dir: DiagramDirection, commit_b: &Commit) -> String {
    // Determine if we need the rerouting path (cross-lane arrow)
    let needs_reroute = match dir {
        DiagramDirection::LR => (p1.y - p2.y).abs() > 1.0,
        DiagramDirection::TB | DiagramDirection::BT => (p1.x - p2.x).abs() > 1.0,
    };

    if needs_reroute {
        match dir {
            DiagramDirection::LR => {
                if p1.y < p2.y {
                    // Downward: vertical then arc then horizontal
                    // M p1.x p1.y  L p1.x (p2.y-20)  A 20 20 0 0 0 (p1.x+20) p2.y  L p2.x p2.y
                    format!(
                        "M {x1:.1} {y1:.1} L {x1:.1} {y2r:.1} A 20 20, 0, 0, 0, {x1o:.1} {y2:.1} L {x2:.1} {y2:.1}",
                        x1 = p1.x, y1 = p1.y,
                        y2r = p2.y - 20.0,
                        x1o = p1.x + 20.0,
                        y2 = p2.y,
                        x2 = p2.x
                    )
                } else {
                    // Upward: horizontal then arc then vertical
                    // M p1.x p1.y  L (p2.x-20) p1.y  A 20 20 0 0 0 p2.x (p1.y-20)  L p2.x p2.y
                    format!(
                        "M {x1:.1} {y1:.1} L {x2r:.1} {y1:.1} A 20 20, 0, 0, 0, {x2:.1} {y1o:.1} L {x2:.1} {y2:.1}",
                        x1 = p1.x, y1 = p1.y,
                        x2r = p2.x - 20.0,
                        x2 = p2.x,
                        y1o = p1.y - 20.0,
                        y2 = p2.y
                    )
                }
            }
            DiagramDirection::TB => {
                let line_x = if p1.x < p2.x {
                    p1.x + (p2.x - p1.x) / 2.0
                } else {
                    p2.x + (p1.x - p2.x) / 2.0
                };
                if p1.x < p2.x {
                    format!(
                        "M {x1:.1} {y1:.1} L {lx_r:.1} {y1:.1} A 10 10, 0, 0, 1, {lx:.1} {y1o:.1} L {lx:.1} {y2r:.1} A 10 10, 0, 0, 0, {lxo:.1} {y2:.1} L {x2:.1} {y2:.1}",
                        x1 = p1.x, y1 = p1.y,
                        lx_r = line_x - 10.0, lx = line_x,
                        y1o = p1.y + 10.0,
                        y2r = p2.y - 10.0,
                        lxo = line_x + 10.0, y2 = p2.y, x2 = p2.x
                    )
                } else {
                    format!(
                        "M {x1:.1} {y1:.1} L {lx_r:.1} {y1:.1} A 10 10, 0, 0, 0, {lx:.1} {y1o:.1} L {lx:.1} {y2r:.1} A 10 10, 0, 0, 1, {lxo:.1} {y2:.1} L {x2:.1} {y2:.1}",
                        x1 = p1.x, y1 = p1.y,
                        lx_r = line_x + 10.0, lx = line_x,
                        y1o = p1.y + 10.0,
                        y2r = p2.y - 10.0,
                        lxo = line_x - 10.0, y2 = p2.y, x2 = p2.x
                    )
                }
            }
            DiagramDirection::BT => {
                let line_x = if p1.x < p2.x {
                    p1.x + (p2.x - p1.x) / 2.0
                } else {
                    p2.x + (p1.x - p2.x) / 2.0
                };
                if p1.x < p2.x {
                    format!(
                        "M {x1:.1} {y1:.1} L {lx_r:.1} {y1:.1} A 10 10, 0, 0, 0, {lx:.1} {y1o:.1} L {lx:.1} {y2r:.1} A 10 10, 0, 0, 1, {lxo:.1} {y2:.1} L {x2:.1} {y2:.1}",
                        x1 = p1.x, y1 = p1.y,
                        lx_r = line_x - 10.0, lx = line_x,
                        y1o = p1.y - 10.0,
                        y2r = p2.y + 10.0,
                        lxo = line_x + 10.0, y2 = p2.y, x2 = p2.x
                    )
                } else {
                    format!(
                        "M {x1:.1} {y1:.1} L {lx_r:.1} {y1:.1} A 10 10, 0, 0, 1, {lx:.1} {y1o:.1} L {lx:.1} {y2r:.1} A 10 10, 0, 0, 0, {lxo:.1} {y2:.1} L {x2:.1} {y2:.1}",
                        x1 = p1.x, y1 = p1.y,
                        lx_r = line_x + 10.0, lx = line_x,
                        y1o = p1.y - 10.0,
                        y2r = p2.y + 10.0,
                        lxo = line_x - 10.0, y2 = p2.y, x2 = p2.x
                    )
                }
            }
        }
    } else {
        // Simple arc: radius=20, offset=20
        // arc="A 20 20, 0, 0, 0,"  arc2="A 20 20, 0, 0, 1,"
        match dir {
            DiagramDirection::LR => {
                // Same branch lane: straight line
                format!("M {:.1} {:.1} L {:.1} {:.1}", p1.x, p1.y, p2.x, p2.y)
            }
            DiagramDirection::TB => {
                if (p1.x - p2.x).abs() < 1.0 {
                    format!("M {:.1} {:.1} L {:.1} {:.1}", p1.x, p1.y, p2.x, p2.y)
                } else if p1.x < p2.x {
                    if commit_b.commit_type == COMMIT_MERGE {
                        format!(
                            "M {x1:.1} {y1:.1} L {x1:.1} {y2r:.1} A 20 20, 0, 0, 0, {x1o:.1} {y2:.1} L {x2:.1} {y2:.1}",
                            x1 = p1.x, y1 = p1.y, y2r = p2.y - 20.0, x1o = p1.x + 20.0, y2 = p2.y, x2 = p2.x
                        )
                    } else {
                        format!(
                            "M {x1:.1} {y1:.1} L {x2r:.1} {y1:.1} A 20 20, 0, 0, 1, {x2:.1} {y1o:.1} L {x2:.1} {y2:.1}",
                            x1 = p1.x, y1 = p1.y, x2r = p2.x - 20.0, x2 = p2.x, y1o = p1.y + 20.0, y2 = p2.y
                        )
                    }
                } else {
                    if commit_b.commit_type == COMMIT_MERGE {
                        format!(
                            "M {x1:.1} {y1:.1} L {x1:.1} {y2r:.1} A 20 20, 0, 0, 1, {x1o:.1} {y2:.1} L {x2:.1} {y2:.1}",
                            x1 = p1.x, y1 = p1.y, y2r = p2.y - 20.0, x1o = p1.x - 20.0, y2 = p2.y, x2 = p2.x
                        )
                    } else {
                        format!(
                            "M {x1:.1} {y1:.1} L {x2r:.1} {y1:.1} A 20 20, 0, 0, 0, {x2:.1} {y1o:.1} L {x2:.1} {y2:.1}",
                            x1 = p1.x, y1 = p1.y, x2r = p2.x + 20.0, x2 = p2.x, y1o = p1.y + 20.0, y2 = p2.y
                        )
                    }
                }
            }
            DiagramDirection::BT => {
                if (p1.x - p2.x).abs() < 1.0 {
                    format!("M {:.1} {:.1} L {:.1} {:.1}", p1.x, p1.y, p2.x, p2.y)
                } else if p1.x < p2.x {
                    if commit_b.commit_type == COMMIT_MERGE {
                        format!(
                            "M {x1:.1} {y1:.1} L {x1:.1} {y2r:.1} A 20 20, 0, 0, 1, {x1o:.1} {y2:.1} L {x2:.1} {y2:.1}",
                            x1 = p1.x, y1 = p1.y, y2r = p2.y + 20.0, x1o = p1.x + 20.0, y2 = p2.y, x2 = p2.x
                        )
                    } else {
                        format!(
                            "M {x1:.1} {y1:.1} L {x2r:.1} {y1:.1} A 20 20, 0, 0, 0, {x2:.1} {y1o:.1} L {x2:.1} {y2:.1}",
                            x1 = p1.x, y1 = p1.y, x2r = p2.x - 20.0, x2 = p2.x, y1o = p1.y - 20.0, y2 = p2.y
                        )
                    }
                } else {
                    if commit_b.commit_type == COMMIT_MERGE {
                        format!(
                            "M {x1:.1} {y1:.1} L {x1:.1} {y2r:.1} A 20 20, 0, 0, 0, {x1o:.1} {y2:.1} L {x2:.1} {y2:.1}",
                            x1 = p1.x, y1 = p1.y, y2r = p2.y + 20.0, x1o = p1.x - 20.0, y2 = p2.y, x2 = p2.x
                        )
                    } else {
                        format!(
                            "M {x1:.1} {y1:.1} L {x2r:.1} {y1:.1} A 20 20, 0, 0, 1, {x2:.1} {y1o:.1} L {x2:.1} {y2:.1}",
                            x1 = p1.x, y1 = p1.y, x2r = p2.x + 20.0, x2 = p2.x, y1o = p1.y - 20.0, y2 = p2.y
                        )
                    }
                }
            }
        }
    }
}

// ── Commit bullet rendering ───────────────────────────────────────────────────

fn draw_commit_bullet(
    out: &mut String,
    commit: &Commit,
    cp: CommitPos,
    sym: u8,
    fill: &str,
    stroke: &str,
) {
    match sym {
        COMMIT_HIGHLIGHT => {
            *out += &format!(
                r#"<rect x="{:.1}" y="{:.1}" width="20" height="20" rx="2" fill="{}" stroke="{}" stroke-width="2"/>"#,
                cp.x - 10.0,
                cp.y - 10.0,
                fill,
                stroke
            );
            *out += &format!(
                r#"<rect x="{:.1}" y="{:.1}" width="12" height="12" rx="1" fill="{}" stroke="{}" stroke-width="1"/>"#,
                cp.x - 6.0,
                cp.y - 6.0,
                fill,
                stroke
            );
        }
        COMMIT_CHERRY_PICK => {
            *out += &format!(
                r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{}" stroke="{}" stroke-width="2"/>"#,
                cp.x, cp.y, COMMIT_RADIUS, fill, stroke
            );
            *out += &format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"2.75\" fill=\"#fff\"/>",
                cp.x - 3.0,
                cp.y + 2.0
            );
            *out += &format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"2.75\" fill=\"#fff\"/>",
                cp.x + 3.0,
                cp.y + 2.0
            );
            *out += &format!(
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#fff\" stroke-width=\"1.5\"/>",
                cp.x + 3.0, cp.y + 1.0, cp.x, cp.y - 5.0
            );
            *out += &format!(
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#fff\" stroke-width=\"1.5\"/>",
                cp.x - 3.0, cp.y + 1.0, cp.x, cp.y - 5.0
            );
        }
        _ => {
            // NORMAL, MERGE, REVERSE
            *out += &format!(
                r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{}" stroke="{}" stroke-width="2" class="commit commit-{}"/>"#,
                cp.x,
                cp.y,
                COMMIT_RADIUS,
                fill,
                stroke,
                esc(&commit.id)
            );
            if sym == COMMIT_MERGE {
                *out += &format!(
                    "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"6\" class=\"commit-merge commit{}\" stroke=\"#ECECFF\" fill=\"#ECECFF\"/>",
                    cp.x, cp.y, esc(&commit.id)
                );
            }
            if sym == COMMIT_REVERSE {
                let c = 5.0;
                *out += &format!(
                    "<path d=\"M {:.1},{:.1}L{:.1},{:.1}M{:.1},{:.1}L{:.1},{:.1}\" stroke=\"#fff\" stroke-width=\"2\"/>",
                    cp.x - c, cp.y - c, cp.x + c, cp.y + c,
                    cp.x - c, cp.y + c, cp.x + c, cp.y - c,
                );
            }
        }
    }
}

// ── Commit label rendering ────────────────────────────────────────────────────

fn draw_commit_label(
    out: &mut String,
    commit: &Commit,
    cp: CommitPos,
    dir: DiagramDirection,
    ff: &str,
) {
    if commit.commit_type == COMMIT_CHERRY_PICK {
        return;
    }
    if commit.commit_type == COMMIT_MERGE && !commit.custom_id {
        return;
    }

    let label = &commit.id;
    let (label_w_raw, label_h) = measure(label, 10.0);
    // Apply scale factor to match browser font metrics (browser Arial is ~14% wider at 10px)
    let label_w = label_w_raw * COMMIT_LABEL_FONT_SCALE;

    match dir {
        DiagramDirection::LR => {
            // Mermaid gitGraphRenderer rotates commit labels -45° around the raw commit pos.
            // rx = cp.x - LAYOUT_OFFSET = the commit's sequence position (before +10 offset).
            // ry = cp.y (branch lane y).
            //
            // Label geometry (in the group's local / parent SVG coordinate space):
            //   rect_x    = cp.x - label_w/2
            //   rect_y    = cp.y + 13.5   (COMMIT_RADIUS + 3.5)
            //   rect_h    = 15            (fixed, matching Mermaid line-height)
            //   text_x    = rect_x + 2
            //   text_y    = cp.y + 25     (baseline ≈ rect_y + 11.5)
            //
            // The group's translate (tx, ty) matches the Mermaid reference formula:
            //   tx = -(label_w/2 + 15.7) * (1/√2)
            //   ty = (label_w/2 + 11.27) * (1/√2)
            // which is derived empirically from the reference SVG data.
            let rx = cp.x - LAYOUT_OFFSET;
            let ry = cp.y;
            // Rect includes 2px padding on each side; text starts 2px inside rect.
            let rect_w = label_w + 4.0;
            let rect_x = cp.x - rect_w / 2.0;
            let rect_y = cp.y + 13.5;
            let text_x = rect_x + 2.0;
            let text_y = cp.y + 25.0;
            // Mermaid translate formula based on empirical analysis of the reference SVG.
            let inv_sqrt2 = std::f64::consts::FRAC_1_SQRT_2;
            let tx = -(rect_w / 2.0 + 15.7) * inv_sqrt2;
            let ty = (rect_w / 2.0 + 11.27) * inv_sqrt2;
            *out += &format!(
                "<g transform=\"translate({tx:.3},{ty:.3}) rotate(-45,{rx:.3},{ry:.3})\">",
                tx = tx,
                ty = ty,
                rx = rx,
                ry = ry
            );
            *out += &format!(
                "<rect class=\"commit-label-bkg\" x=\"{:.3}\" y=\"{:.3}\" width=\"{:.3}\" height=\"15\"/>",
                rect_x, rect_y, rect_w
            );
            *out += &format!(
                "<text x=\"{:.3}\" y=\"{:.3}\" class=\"commit-label\">{}</text>",
                text_x,
                text_y,
                esc(label)
            );
            *out += "</g>";
        }
        DiagramDirection::TB | DiagramDirection::BT => {
            let lx = cp.x - COMMIT_RADIUS - label_w - 8.0;
            let ly = cp.y - label_h / 2.0;
            *out += &format!(
                "<rect class=\"commit-label-bkg\" x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\"/>",
                lx, ly, label_w, label_h
            );
            *out += &format!(
                "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"#000021\" font-family=\"{ff}\" class=\"commit-label\">{}</text>",
                lx, ly + label_h - 2.0,
                esc(label),
                ff = ff
            );
        }
    }
}

fn draw_commit_tags(
    out: &mut String,
    commit: &Commit,
    cp: CommitPos,
    dir: DiagramDirection,
    ff: &str,
) {
    if commit.tags.is_empty() {
        return;
    }
    let mut y_off: f64 = 0.0;
    for tag in &commit.tags {
        let (tw_raw, _) = measure(tag, 10.0);
        let tw = tw_raw * TAG_TEXT_FONT_SCALE;
        match dir {
            DiagramDirection::LR => {
                // Classic Mermaid tag-badge shape: a rectangle with a left-pointing
                // notch (like a price tag / git tag label) and a hole circle.
                //
                // Geometry (all relative to commit centre cp.x, cp.y):
                //   body half-width  = tw/2 + 4
                //   body_left        = cp.x - body_half
                //   body_right       = cp.x + body_half
                //   pointer_x        = body_left - 8   (flat left edge)
                //   badge_bottom     = cp.y - COMMIT_RADIUS - 2  (just above circle)
                //   badge_top        = badge_bottom - 15
                //   badge_mid_y      = badge_bottom - 7.5
                //   pointer arrow: two pts at pointer_x, at badge_mid_y ± 2
                let body_half = tw / 2.0 + 4.0;
                let body_left = cp.x - body_half;
                let body_right = cp.x + body_half;
                let pointer_x = body_left - 8.0;
                let badge_bottom = cp.y - COMMIT_RADIUS - 1.7 - y_off;
                let badge_top = badge_bottom - 15.0;
                let badge_mid = badge_bottom - 7.5;
                // 6-point polygon: bottom-pointer, top-pointer,
                //   top-body-left, top-right, bottom-right, bottom-body-left
                *out += &format!(
                    "<polygon class=\"tag-label-bkg\" points=\"{x0:.3},{yb:.3} {x0:.3},{yt:.3} {bl:.3},{bt:.3} {br:.3},{bt:.3} {br:.3},{bb:.3} {bl:.3},{bb:.3}\" fill=\"#ECECFF\" stroke=\"hsl(240, 60%, 86.2745098039%)\" stroke-width=\"1\"/>",
                    x0 = pointer_x,
                    yb = badge_mid + 2.0,
                    yt = badge_mid - 2.0,
                    bl = body_left,
                    br = body_right,
                    bt = badge_top,
                    bb = badge_bottom,
                );
                // Hole circle at pointer_x + 4
                *out += &format!(
                    "<circle cy=\"{:.3}\" cx=\"{:.3}\" r=\"1.5\" class=\"tag-hole\" fill=\"#333\"/>",
                    badge_mid,
                    pointer_x + 4.0,
                );
                // Tag text: start at body_left + 4, baseline ≈ badge_mid + 3
                *out += &format!(
                    "<text y=\"{:.3}\" class=\"tag-label\" x=\"{:.3}\" font-size=\"10\" fill=\"#131300\" font-family=\"{ff}\">{}</text>",
                    badge_mid + 3.2,
                    body_left + 4.0,
                    esc(tag),
                    ff = ff
                );
            }
            DiagramDirection::TB | DiagramDirection::BT => {
                let tx = cp.x + COMMIT_RADIUS + 8.0;
                let ty = cp.y + y_off;
                *out += &format!(
                    "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"14\" rx=\"2\" fill=\"#ffffe0\" stroke=\"#cc9900\" stroke-width=\"1\"/>",
                    tx - 2.0, ty - 7.0, tw + 4.0
                );
                *out += &format!(
                    "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"#333\" font-family=\"{ff}\">{}</text>",
                    tx, ty + 4.0, esc(tag),
                    ff = ff
                );
            }
        }
        y_off += 20.0;
    }
}

// ── SVG helpers ───────────────────────────────────────────────────────────────

fn build_defs(id: &str) -> String {
    format!(
        "<defs><marker id=\"{id}-arrowhead\" markerWidth=\"10\" markerHeight=\"7\" refX=\"10\" refY=\"3.5\" orient=\"auto\"><polygon points=\"0 0, 10 3.5, 0 7\" fill=\"#333\"/></marker></defs>",
        id = id
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::git::parser;

    #[test]
    fn basic_render() {
        let input = "gitGraph\n    commit\n    branch develop\n    commit\n    checkout main\n    merge develop";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "missing <svg tag");
        assert!(svg.contains("</svg>"), "missing </svg> tag");
        assert!(svg.contains("circle"), "missing commit circles");
    }

    #[test]
    fn sample_diagram_renders() {
        let input = r"gitGraph
    commit
    branch develop
    commit
    commit
    checkout main
    merge develop
    commit
    branch feature
    commit
    checkout develop
    merge feature";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("commit-bullets"));
        assert!(svg.contains("commit-arrows"));
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "gitGraph\n   commit\n   branch develop\n   checkout develop\n   commit\n   commit\n   checkout main\n   merge develop\n   commit";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
