use super::constants::*;
use super::parser::{TimelineDiagram, TimelineTask};
use super::templates;
/// Faithful Rust port of Mermaid's timelineRenderer.ts + svgDraw.js.
///
/// Layout algorithm (mirrors the JS draw() function exactly):
///
/// Constants from Mermaid defaults:
///   LEFT_MARGIN = 50   (conf.timeline?.leftMargin ?? 50)
///   masterX starts at 50 + LEFT_MARGIN = 100
///   masterY / sectionBeginY = 50
///   Node width = 150, padding = 20  → rendered width = 150 + 2*20 = 190
///   Step between tasks = 200px (masterX += 200)
///   Step between sections = 200 * max(tasksForSection.length, 1)
///
/// Per-node height: text_height + fontSize*1.1*0.5 + padding, min = maxHeight
///   (faithful to getVirtualNodeHeight / drawNode in svgDraw.js)
///
/// Sections are drawn above tasks; tasks are drawn at masterY + maxSectionHeight + 50.
/// Events are drawn below their task with a dashed vertical line.
///
/// The horizontal activity line is drawn at:
///   depthY = (hasSections) ? maxSectionHeight + maxTaskHeight + 150
///           :                maxTaskHeight + 100
///
/// Section colour palette: 12 colours cycling by section index.
/// Task colour = section colour (same index for all tasks in that section).
/// isWithoutSections: each task gets its own colour index (incremented).
use crate::text::measure;
use crate::theme::Theme;

// ── Layout constants (faithful to Mermaid timelineRenderer.ts defaults) ────────
// All layout constants are imported from super::constants via `use super::constants::*`.

// ── Mermaid timeline section colour palette (HSL, mirrors JS cScale0..10) ──────
//
// Mermaid generates CSS like:
//   .section--1 path { fill: hsl(240, 100%, 76.27%) }
//   .section-0  path { fill: hsl(60,  100%, 73.53%) }
//   etc.
//
// The hue steps follow Mermaid's cScaleLabel[] / SCALE constant:
//   cScale0 = hsl(240,100%,76.27%)  section--1 (index 0 → class -1)
//   cScale1 = hsl(60, 100%,73.53%)  section-0  (index 1 → class  0)
//   cScale2 = hsl(80, 100%,76.27%)  section-1
//   cScale3 = hsl(270,100%,76.27%)  section-2
//   cScale4 = hsl(300,100%,76.27%)  section-3
//   cScale5 = hsl(330,100%,76.27%)  section-4
//   cScale6 = hsl(0,  100%,76.27%)  section-5
//   cScale7 = hsl(30, 100%,76.27%)  section-6
//   cScale8 = hsl(90, 100%,76.27%)  section-7
//   cScale9 = hsl(150,100%,76.27%)  section-8
//   cScale10= hsl(180,100%,76.27%)  section-9
//   cScale11= hsl(210,100%,76.27%)  section-10
//
// Line colours (hue + 180 for complementary, +10% lightness):
//   cScale0 line: hsl(60,  100%, 86.27%)
//   cScale1 line: hsl(240, 100%, 83.53%)
//   cScale2 line: hsl(260, 100%, 86.27%)
//   cScale3 line: hsl(90,  100%, 86.27%)
//   cScale4 line: hsl(120, 100%, 86.27%)
//   cScale5 line: hsl(150, 100%, 86.27%)
//   cScale6 line: hsl(180, 100%, 86.27%)
//   cScale7 line: hsl(210, 100%, 86.27%)
//   cScale8 line: hsl(270, 100%, 86.27%)
//   cScale9 line: hsl(330, 100%, 86.27%)
//   cScale10 line: hsl(0,  100%, 86.27%)
//   cScale11 line: hsl(30, 100%, 86.27%)
//
// Text colours (white for dark sections, black for light):
//   cScale0 text: #ffffff  (240° blue = dark)
//   cScale1 text: black    (60° yellow = light)
//   cScale2 text: black    (80° lime = light)
//   cScale3 text: #ffffff  (270° purple = dark)
//   cScale4 text: black    (300° pink = medium)
//   cScale5 text: black    (330° rose = medium)
//   cScale6 text: black    (0° red = medium)
//   cScale7 text: black    (30° orange = medium)
//   cScale8 text: black    (90° green = light)
//   cScale9 text: black    (150° teal = medium)
//   cScale10 text: black   (180° cyan = light)
//   cScale11 text: black   (210° blue = medium)

struct SectionStyle {
    fill: &'static str, // hsl fill for path/circle/rect
    line: &'static str, // hsl stroke for line separator
    text: &'static str, // fill for text
}

const SECTION_STYLES: [SectionStyle; 12] = [
    SectionStyle {
        fill: "hsl(240, 100%, 76.2745098039%)",
        line: "hsl(60, 100%, 86.2745098039%)",
        text: "#ffffff",
    },
    SectionStyle {
        fill: "hsl(60, 100%, 73.5294117647%)",
        line: "hsl(240, 100%, 83.5294117647%)",
        text: "black",
    },
    SectionStyle {
        fill: "hsl(80, 100%, 76.2745098039%)",
        line: "hsl(260, 100%, 86.2745098039%)",
        text: "black",
    },
    SectionStyle {
        fill: "hsl(270, 100%, 76.2745098039%)",
        line: "hsl(90, 100%, 86.2745098039%)",
        text: "#ffffff",
    },
    SectionStyle {
        fill: "hsl(300, 100%, 76.2745098039%)",
        line: "hsl(120, 100%, 86.2745098039%)",
        text: "black",
    },
    SectionStyle {
        fill: "hsl(330, 100%, 76.2745098039%)",
        line: "hsl(150, 100%, 86.2745098039%)",
        text: "black",
    },
    SectionStyle {
        fill: "hsl(0, 100%, 76.2745098039%)",
        line: "hsl(180, 100%, 86.2745098039%)",
        text: "black",
    },
    SectionStyle {
        fill: "hsl(30, 100%, 76.2745098039%)",
        line: "hsl(210, 100%, 86.2745098039%)",
        text: "black",
    },
    SectionStyle {
        fill: "hsl(90, 100%, 76.2745098039%)",
        line: "hsl(270, 100%, 86.2745098039%)",
        text: "black",
    },
    SectionStyle {
        fill: "hsl(150, 100%, 76.2745098039%)",
        line: "hsl(330, 100%, 86.2745098039%)",
        text: "black",
    },
    SectionStyle {
        fill: "hsl(180, 100%, 76.2745098039%)",
        line: "hsl(0, 100%, 86.2745098039%)",
        text: "black",
    },
    SectionStyle {
        fill: "hsl(210, 100%, 76.2745098039%)",
        line: "hsl(30, 100%, 86.2745098039%)",
        text: "black",
    },
];

fn section_fill(idx: usize) -> &'static str {
    SECTION_STYLES[idx % SECTION_STYLES.len()].fill
}

#[allow(dead_code)]
fn section_line(idx: usize) -> &'static str {
    SECTION_STYLES[idx % SECTION_STYLES.len()].line
}

#[allow(dead_code)]
fn section_text(idx: usize) -> &'static str {
    SECTION_STYLES[idx % SECTION_STYLES.len()].text
}

// ── Node height computation ────────────────────────────────────────────────────

/// Raw height computation — mirrors JS getVirtualNodeHeight exactly:
///   bbox.height + fontSize * 1.1 * 0.5 + node.padding
/// Does NOT apply node.maxHeight (that is done separately, like JS's drawNode).
fn raw_node_height(text: &str) -> f64 {
    let text_height = wrapped_text_height(text, NODE_WIDTH);
    text_height + FONT_SIZE * 1.1 * 0.5 + NODE_PADDING
}

/// Height for drawing: max(raw_height, max_height).
/// Mirrors drawNode line 978: node.height = Math.max(node.height, node.maxHeight).
fn virtual_node_height(text: &str, max_height: f64) -> f64 {
    raw_node_height(text).max(max_height)
}

/// Text wrapping height, calibrated to match Chromium's SVG getBBox() behaviour.
///
/// Chromium's getBBox for `<text dy="1em"><tspan dy="1em">text</tspan></text>` returns:
///   - 1 line:  FONT_SIZE × 1.0634765625  (= 17.015625 at 16px)
///   - Each additional line adds: FONT_SIZE × 1.1  (= 17.6 at 16px)
///
/// WRAPPING: Mermaid's wrap() uses split(/(\s+|<br>)/) which keeps whitespace
/// tokens. When building a tspan text via line.join(" "), each word boundary
/// gets 3 spaces (1 join + 1 original_space + 1 join), making the effective
/// text wider than single-space joining. We simulate this by using 3 spaces
/// between words when measuring line widths.
///
/// Derivation:
///   Single-line reference task height = 65.815625 = bbox.h + 8.8 + 20 → bbox.h = 17.015625
///   Two-line reference event height = 63.415625 = bbox.h + 8.8 + 20 → bbox.h = 34.615625
///   34.615625 = 17.015625 + 17.6 = first_line + FONT_SIZE × 1.1
fn wrapped_text_height(text: &str, max_width: f64) -> f64 {
    // Calibrated browser heights (empirically derived from reference SVG coordinates)
    let first_line_h = FONT_SIZE * 1.0634765625; // = 17.015625 at 16px
    let additional_line_h = FONT_SIZE * 1.1; // = 17.6 at 16px

    let lines = count_lines(text, max_width);

    if lines == 1 {
        first_line_h
    } else {
        first_line_h + (lines - 1) as f64 * additional_line_h
    }
}

/// Count the number of wrapped lines for `text` at the given `max_width`.
/// Uses 3 spaces between words (simulating Mermaid's wrap() whitespace token behaviour).
fn count_lines(text: &str, max_width: f64) -> usize {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return 1;
    }

    let mut lines = 1usize;
    let mut current_words: Vec<&str> = Vec::new();

    for word in &words {
        let candidate_words: Vec<&str> = current_words
            .iter()
            .copied()
            .chain(std::iter::once(*word))
            .collect();
        // Mermaid joins with 3 spaces: each boundary = join_space + whitespace_token + join_space
        let candidate_text = candidate_words.join("   ");
        let (w, _) = measure(&candidate_text, FONT_SIZE);
        if w > max_width && !current_words.is_empty() {
            lines += 1;
            current_words = vec![word];
        } else {
            current_words = candidate_words;
        }
    }

    lines
}

// ── SVG helpers ───────────────────────────────────────────────────────────────

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Build the SVG path for a node background.
/// Mirrors defaultBkg in svgDraw.js (r=5, rd=5).
fn node_bg_path(width: f64, height: f64) -> String {
    let r = NODE_CORNER_R;
    let rd = NODE_CORNER_R;
    format!(
        "M0 {h_rd} v{neg_hm2rd} q0,-{r},{r},-{r} h{wm2rd} q{r},0,{r},{r} v{hrd} H0 Z",
        h_rd = height - rd,
        neg_hm2rd = -(height - 2.0 * rd),
        r = r,
        wm2rd = width - 2.0 * rd,
        hrd = height - rd,
    )
}

/// Draw one timeline node (period or event) as SVG.
/// Returns (svg_string, rendered_height).
/// Mirrors drawNode() in svgDraw.js.
///
/// Styling: NO inline fill/stroke on path/line — CSS class selectors handle colors,
/// exactly as Mermaid's reference output does.
fn draw_node(
    label: &str,
    section_idx: usize,
    node_id: &mut usize,
    max_height: f64,
    _is_event: bool,
) -> (String, f64) {
    let width = RENDERED_WIDTH;
    let height = virtual_node_height(label, max_height);
    // JS: (fullSection % maxSections) - 1  where maxSections=12
    let section_class = (section_idx % 12) as i64 - 1;

    let path_d = node_bg_path(width, height);
    let id_val = *node_id;
    *node_id += 1;

    // Text is vertically centered in the top area (10px from top)
    let text_ty = 10.0;
    let text_tx = width / 2.0;

    let tspans = build_tspans(label, NODE_WIDTH);

    let mut svg = String::new();
    svg.push_str(&templates::node_group_open(section_class));
    svg.push_str("  <g>\n");
    // No inline style — CSS class `.section-N path` sets fill/stroke
    svg.push_str(&templates::node_bg_path(id_val, &path_d));
    svg.push_str(&templates::node_separator_line(
        section_class,
        height,
        width,
    ));
    svg.push_str("  </g>\n");
    svg.push_str(&templates::node_text_group(text_tx, text_ty, &tspans));
    svg.push_str("</g>");

    (svg, height)
}

/// Build SVG text element with tspan wrapping for the given text.
/// Uses 3-space joining to match Mermaid's wrap() whitespace token behaviour.
fn build_tspans(text: &str, max_width: f64) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    let line_height_em = 1.1_f64;
    let mut lines: Vec<String> = Vec::new();
    let mut current_words: Vec<&str> = Vec::new();

    for word in &words {
        let candidate_words: Vec<&str> = current_words
            .iter()
            .copied()
            .chain(std::iter::once(*word))
            .collect();
        // Use 3 spaces for width check (matches Mermaid's wrap() join behavior)
        let candidate_3sp = candidate_words.join("   ");
        let (w, _) = measure(&candidate_3sp, FONT_SIZE);
        if w > max_width && !current_words.is_empty() {
            // Push current line with single space (visual rendering)
            lines.push(current_words.join(" "));
            current_words = vec![word];
        } else {
            current_words = candidate_words;
        }
    }
    if !current_words.is_empty() {
        lines.push(current_words.join(" "));
    }

    let mut out = String::new();
    out.push_str("<text dy=\"1em\" alignment-baseline=\"middle\" dominant-baseline=\"middle\" text-anchor=\"middle\">");
    for (i, line) in lines.iter().enumerate() {
        let dy = if i == 0 {
            "1em".to_string()
        } else {
            format!("{:.1}em", line_height_em)
        };
        out.push_str(&format!(
            "<tspan x=\"0\" dy=\"{dy}\">{text}</tspan>",
            dy = dy,
            text = escape(line),
        ));
    }
    out.push_str("</text>");
    out
}

// ── Arrowhead marker ──────────────────────────────────────────────────────────

fn arrowhead_marker(id: &str) -> String {
    templates::arrowhead_marker(id)
}

// ── CSS style ─────────────────────────────────────────────────────────────────
//
// Mirrors the full Mermaid timeline CSS block, using the same HSL values.
// Important: CSS class selectors (not inline styles) set fills/strokes so
// the SVG appearance exactly matches the Mermaid reference.

fn build_style(id: &str, ff: &str) -> String {
    let mut s = format!(
        "#{id}{{font-family:{ff};font-size:{fs}px;fill:#333;}}",
        id = id,
        ff = ff,
        fs = FONT_SIZE,
    );

    // section--1 (index 0) … section-10 (index 11)
    for (i, st) in SECTION_STYLES.iter().enumerate() {
        let idx: i64 = (i as i64) - 1;
        // The CSS selectors match Mermaid's generated stylesheet exactly:
        //   .section-N rect, .section-N path, .section-N circle, .section-N path { fill: ... }
        //   .section-N text { fill: black|#ffffff }
        //   .section-N line { stroke: ...; stroke-width: 3; }
        s.push_str(&format!(
            "#{id} .section-{idx} rect,#{id} .section-{idx} path,#{id} .section-{idx} circle,#{id} .section-{idx} path{{fill:{fill};}}",
            id = id, idx = idx, fill = st.fill,
        ));
        s.push_str(&format!(
            "#{id} .section-{idx} text{{fill:{text};}}",
            id = id,
            idx = idx,
            text = st.text,
        ));
        s.push_str(&format!(
            "#{id} .section-edge-{idx}{{stroke:{fill};}}",
            id = id,
            idx = idx,
            fill = st.fill,
        ));
        s.push_str(&format!(
            "#{id} .section-{idx} line{{stroke:{line};stroke-width:3;}}",
            id = id,
            idx = idx,
            line = st.line,
        ));
        // node-line-N class (used on the horizontal separator below node text)
        s.push_str(&format!(
            "#{id} .node-line-{idx}{{stroke:{line};stroke-width:3;}}",
            id = id,
            idx = idx,
            line = st.line,
        ));
    }

    s.push_str(&format!(
        concat!(
            "#{id} .edge{{stroke-width:3;}}",
            "#{id} .timeline-node{{fill:none;}}",
            "#{id} .node-bkg{{opacity:1;}}",
            "#{id} p{{margin:0;}}",
            "#{id} svg{{font-family:{ff};font-size:{fs}px;}}",
            "#{id} text{{font-family:{ff};fill:#333;}}",
            "#{id} .section-label{{text-anchor:middle;dominant-baseline:middle;}}",
            "#{id} .title-text{{font-size:24px;font-weight:bold;fill:#333;text-anchor:middle;}}",
            "#{id} .activity-line{{stroke:#333;stroke-width:4px;}}",
            "#{id} .eventWrapper{{filter:brightness(120%);}}",
            "#{id} .lineWrapper line{{stroke:#ffffff;}}",
        ),
        id = id,
        ff = ff,
        fs = FONT_SIZE,
    ));

    s
}

// ── Main render ───────────────────────────────────────────────────────────────

pub fn render(diag: &TimelineDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let diagram_id = DIAGRAM_ID;
    let mut node_id: usize = 0;

    let tasks = &diag.tasks;
    let sections = &diag.sections;
    let has_sections = !sections.is_empty();

    // ── Pre-calculate max heights (mirrors the JS height pre-pass) ────────────
    let mut max_section_height: f64 = 0.0;
    let mut max_task_height: f64 = 0.0;
    let mut max_event_line_length: f64 = 0.0;
    let mut max_event_count: usize = 0;

    // Pre-pass: use RAW heights (no max applied), then add 20.
    // Mirrors JS: sectionHeight = getVirtualNodeHeight(...) [no max]; maxSectionHeight = max(max, h+20)
    for section in sections {
        let h = raw_node_height(section);
        max_section_height = max_section_height.max(h + 20.0);
    }

    for task in tasks {
        let h = raw_node_height(&task.task);
        max_task_height = max_task_height.max(h + 20.0);

        max_event_count = max_event_count.max(task.events.len());

        let mut event_line_len: f64 = 0.0;
        for event in &task.events {
            // JS pre-pass uses getVirtualNodeHeight (raw, no maxHeight applied):
            event_line_len += raw_node_height(event);
        }
        if !task.events.is_empty() {
            event_line_len += (task.events.len() - 1) as f64 * 10.0;
        }
        max_event_line_length = max_event_line_length.max(event_line_len);
    }

    // Suppress unused warning — max_event_count mirrors JS but isn't used in layout
    let _ = max_event_count;

    // ── SVG body ─────────────────────────────────────────────────────────────
    let mut parts: Vec<String> = Vec::new();
    parts.push(arrowhead_marker(diagram_id));

    let mut master_x = MASTER_START_X;
    let mut master_y = MASTER_START_Y;
    let section_begin_y = SECTION_START_Y;

    if has_sections {
        for (section_number, section) in sections.iter().enumerate() {
            let tasks_for_section: Vec<&TimelineTask> =
                tasks.iter().filter(|t| t.section == *section).collect();

            // Width spans from the first task center to the last task's right edge:
            // (n-1)*TASK_STEP + RENDERED_WIDTH = 200*(n-1) + 190 = 200*n - 10
            let section_width = 200.0 * (tasks_for_section.len().max(1) as f64) - 10.0;
            let sec_h = virtual_node_height(section, max_section_height).max(max_section_height);

            let sec_svg = draw_section_box(
                section,
                section_number,
                section_width,
                sec_h,
                master_x,
                section_begin_y,
            );
            parts.push(sec_svg);

            master_y = section_begin_y + max_section_height + 50.0;

            let task_svgs = draw_tasks(
                tasks_for_section.as_slice(),
                section_number,
                master_x,
                master_y,
                max_task_height,
                max_event_line_length,
                diagram_id,
                &mut node_id,
                false,
            );
            parts.extend(task_svgs);

            master_x += 200.0 * (tasks_for_section.len().max(1) as f64);
            master_y = section_begin_y; // mirrors JS: masterY = sectionBeginY (reset for next section)
            let _ = master_y; // suppress unused-assignment warning
        }
    } else {
        let all_tasks: Vec<&TimelineTask> = tasks.iter().collect();
        let task_svgs = draw_tasks(
            all_tasks.as_slice(),
            0,
            master_x,
            master_y,
            max_task_height,
            max_event_line_length,
            diagram_id,
            &mut node_id,
            true,
        );
        parts.extend(task_svgs);
        master_x += 200.0 * tasks.len() as f64;
    }

    // ── Activity line ─────────────────────────────────────────────────────────
    // depthY = max_section_height + max_task_height + 150 (with sections)
    //          max_task_height + 100 (no sections)
    let depth_y = if has_sections {
        max_section_height + max_task_height + 150.0
    } else {
        max_task_height + 100.0
    };

    // Mirrors JS: box = getBBox before line; x2 = box.width + 3*LEFT_MARGIN.
    // box.width (before line) = content right edge - content left edge
    //   = (master_x - TASK_STEP + RENDERED_WIDTH) - MASTER_START_X
    //   = master_x - TASK_STEP + RENDERED_WIDTH - MASTER_START_X
    //   = master_x - 200 + 190 - 200 = master_x - 210
    let box_width_before_line = master_x - TASK_STEP + RENDERED_WIDTH - MASTER_START_X;
    let activity_line_x2 = box_width_before_line + 3.0 * LEFT_MARGIN;

    // Activity line (horizontal timeline spine)
    parts.push(templates::activity_line(
        LEFT_MARGIN,
        depth_y,
        activity_line_x2,
        diagram_id,
    ));

    // ── Title ─────────────────────────────────────────────────────────────────
    // Mirrors JS: title.x = box.width / 2 - LEFT_MARGIN (for non-neo look)
    let title_x = box_width_before_line / 2.0 - LEFT_MARGIN;
    let title_svg = if let Some(title) = &diag.title {
        templates::title_text(title_x, &escape(title))
    } else {
        String::new()
    };

    // ── Viewbox ───────────────────────────────────────────────────────────────
    // Mirrors setupGraphViewbox with padding=50:
    //   After activity line, getBBox: x=LEFT_MARGIN=150 to x=activity_line_x2
    //   getBBox.width = activity_line_x2 - LEFT_MARGIN
    //   vb_x = LEFT_MARGIN - padding = 150 - 50 = 100
    //   vb_w = getBBox.width + 2*padding = (activity_line_x2 - LEFT_MARGIN) + 100
    //   vb_y = svgBounds.y - padding ≈ -60 (title text top in browser + 8px body margin ≈ -10; -10-50=-60)
    //   vb_h = svgBounds.height + 2*padding; svgBounds.height = connector_y2 - svgBounds.y ≈ connector_y2+10
    //   connector_y2 = MASTER_START_Y + max_task_height + 100 + max_event_line_length + 100 (no sections)
    //                  OR section_begin_y + max_section_height + 50 + max_task_height + 200 + max_event_line_length (with sections)
    let vb_x = LEFT_MARGIN - VIEWBOX_PADDING; // 150 - 50 = 100
    let vb_y = -60.0; // = svgBounds.y - padding ≈ -10 - 50 (includes browser body margin ~8px + title ascent ~2px)
    let vb_w = (activity_line_x2 - LEFT_MARGIN) + 2.0 * VIEWBOX_PADDING; // = activity_line_x2 - 150 + 100
                                                                         // connector_y2 = max task_y + maxTaskHeight + 200 + maxEventLineLength
    let task_y = if has_sections {
        SECTION_START_Y + max_section_height + 50.0
    } else {
        MASTER_START_Y
    };
    let connector_y2 = task_y + max_task_height + 200.0 + max_event_line_length;
    // vb_h = (connector_y2 - svgBounds.y) + 2*padding = (connector_y2 + 10) + 100 = connector_y2 + 110
    let vb_h = connector_y2 + 110.0;

    let style = build_style(diagram_id, ff);

    let mut svg = templates::svg_root(diagram_id, vb_w, vb_x, vb_y, vb_w, vb_h, &style);

    for part in &parts {
        svg.push_str(part);
        svg.push('\n');
    }

    if !title_svg.is_empty() {
        svg.push_str(&title_svg);
        svg.push('\n');
    }

    svg.push_str("</svg>");
    svg
}

// ── Draw a section header box ─────────────────────────────────────────────────

fn draw_section_box(
    label: &str,
    section_idx: usize,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
) -> String {
    let color = section_fill(section_idx);
    let text_color = section_text(section_idx);
    let r = NODE_CORNER_R;
    let rd = NODE_CORNER_R;
    let path = format!(
        "M0 {h_rd} v{neg} q0,-{r},{r},-{r} h{wm} q{r},0,{r},{r} v{hrd} H0 Z",
        h_rd = height - rd,
        neg = -(height - 2.0 * rd),
        r = r,
        wm = width - 2.0 * rd,
        hrd = height - rd,
    );

    let text_x = width / 2.0;
    let text_y = height / 2.0;

    // Bottom line class mirrors the node-line CSS (e.g. node-line--1, node-line-0)
    let line_class_idx = (section_idx % 12) as i64 - 1;
    let line_color = section_line(section_idx);

    templates::section_box(
        x,
        y,
        &path,
        color,
        line_class_idx,
        height,
        width,
        line_color,
        text_x,
        text_y,
        text_color,
        &escape(label),
    )
}

// ── Draw tasks ────────────────────────────────────────────────────────────────

/// Mirrors drawTasks() in timelineRenderer.ts.
#[allow(clippy::too_many_arguments)]
fn draw_tasks(
    tasks: &[&TimelineTask],
    section_color_start: usize,
    mut master_x: f64,
    master_y: f64,
    max_task_height: f64,
    max_event_line_length: f64,
    diagram_id: &str,
    node_id: &mut usize,
    is_without_sections: bool,
) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut section_color_idx = section_color_start;

    for task in tasks {
        let (task_svg, task_h) = draw_node(
            &task.task,
            section_color_idx,
            node_id,
            max_task_height,
            false,
        );

        parts.push(templates::task_wrapper(master_x, master_y, &task_svg));

        if !task.events.is_empty() {
            // Connector line: mirrors JS line 1215
            //   y1 = masterY + maxTaskHeight  (masterY = task y = master_y)
            //   y2 = masterY + maxTaskHeight + 100 + maxEventLineLength + 100
            let line_x = master_x + RENDERED_WIDTH / 2.0;
            let line_y1 = master_y + task_h;
            let line_y2 = master_y + max_task_height + 100.0 + max_event_line_length + 100.0;

            parts.push(templates::connector_line(
                line_x, line_y1, line_y2, diagram_id,
            ));

            // Events: JS does masterY += 100, passes to drawEvents, which does another += 100
            // So first event y = master_y + 100 + 100 = master_y + 200
            let event_start_y = master_y + 100.0; // = JS's (masterY + 100) passed to drawEvents
            draw_events_append(
                &task.events,
                section_color_idx,
                master_x,
                event_start_y,
                node_id,
                &mut parts,
            );
        }

        master_x += TASK_STEP;

        if is_without_sections {
            section_color_idx += 1;
        }
    }

    parts
}

// ── Draw events ───────────────────────────────────────────────────────────────

/// Mirrors drawEvents() in timelineRenderer.ts.
fn draw_events_append(
    events: &[String],
    section_idx: usize,
    master_x: f64,
    start_y: f64,
    node_id: &mut usize,
    parts: &mut Vec<String>,
) {
    // JS drawEvents: masterY = masterY + 100 at start
    let mut current_y = start_y + 100.0;

    for event in events {
        let (event_svg, event_h) = draw_node(event, section_idx, node_id, 50.0, true);

        parts.push(templates::event_wrapper(master_x, current_y, &event_svg));

        current_y += 10.0 + event_h;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::timeline::parser;

    #[test]
    fn basic_render_produces_svg() {
        let input = concat!(
            "timeline\n",
            "    title History of Social Media\n",
            "    2002 : LinkedIn\n",
            "    2004 : Facebook\n",
            "         : Google\n",
        );
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "missing <svg tag");
        assert!(svg.contains("LinkedIn"));
        assert!(svg.contains("Facebook"));
        assert!(svg.contains("Google"));
        assert!(svg.contains("History of Social Media"));
        assert!(svg.contains("2002"));
        assert!(svg.contains("2004"));
    }

    #[test]
    fn with_sections_renders() {
        let input = concat!(
            "timeline\n",
            "    title History of Social Media Platform\n",
            "    section ICT and Internet\n",
            "        1978 : first commercial social network\n",
            "        1994 : GeoCities\n",
        );
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("ICT and Internet"));
        assert!(svg.contains("1978"));
        assert!(svg.contains("GeoCities"));
    }

    #[test]
    fn activity_line_present() {
        let input = "timeline\n    2002 : LinkedIn\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("stroke-width:4")); // inline style format
    }

    #[test]
    fn node_bg_path_roundtrip() {
        let p = node_bg_path(190.0, 60.0);
        assert!(p.starts_with("M0 "));
        assert!(p.contains('q'));
        assert!(p.ends_with('Z'));
    }

    #[test]
    fn full_example_with_sections() {
        // Tasks that have a section keyword get drawn under their section header.
        // Tasks without a section (before any 'section' keyword) are in section=""
        // which is not in the sections list, so only the sectioned tasks are drawn.
        let input = concat!(
            "timeline\n",
            "    title History of Social Media Platform\n",
            "    section ICT and Internet\n",
            "        1978 : first commercial social network\n",
            "        1994 : GeoCities\n",
            "    section Social Media\n",
            "        2002 : LinkedIn\n",
            "        2004 : Facebook\n",
            "             : Google\n",
        );
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("ICT and Internet"));
        assert!(svg.contains("GeoCities"));
        assert!(svg.contains("LinkedIn"));
        assert!(svg.contains("Facebook"));
        assert!(svg.contains("Google"));
    }

    #[test]
    fn full_example_no_sections() {
        // All tasks without any section keyword.
        let input = concat!(
            "timeline\n",
            "    title History of Social Media Platform\n",
            "    2002 : LinkedIn\n",
            "    2004 : Facebook\n",
            "         : Google\n",
            "    2005 : YouTube\n",
            "    2006 : Twitter\n",
        );
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("YouTube"));
        assert!(svg.contains("Twitter"));
        assert!(svg.contains("LinkedIn"));
        assert!(svg.contains("Facebook"));
    }

    #[test]
    fn snapshot_default_theme() {
        let input =
            "timeline\n    title History of Social Media\n    2002 : LinkedIn\n    2004 : Facebook";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
