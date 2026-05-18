use super::constants::*;
use super::parser::JourneyDiagram;
use super::templates;
/// Faithful Rust port of Mermaid's journeyRenderer.ts + svgDraw.js (user journey).
///
/// Layout algorithm mirrors journeyRenderer.ts draw() exactly:
///
/// Constants (conf.journey defaults):
///   diagramMarginX = 50, diagramMarginY = 10
///   taskMargin = 50, width (task width) = 150
///   height (section head height) = 50
///   leftMargin = 150  (fixed, not expanded for actor labels)
///   boxTextMargin = 5
///
/// Layout (matches reference Mermaid output):
///   - viewBox: "0 -25 {totalWidth} 540", height=565
///   - Actor legend at left (circles + names), starting y=60, step=20
///   - Section headers: y=50, height=50
///     section_x   = task_start_index * (TASK_W+TASK_MARGIN) + LEFT_MARGIN
///     section_w   = count*(TASK_W+TASK_MARGIN) - TASK_MARGIN
///   - Task boxes: y=110, height=50
///     task_x = i * (TASK_W+TASK_MARGIN) + LEFT_MARGIN
///   - Task vertical lines: y1=110, y2=450
///   - Activity line: y=200 (=height*4), x1=LEFT_MARGIN, x2=total_content_width-4
///   - Score face: face_cy = 450 - score*30  (score 5→300, 3→360, 1→420)
///     smile mouth: translate(cx, face_cy+2)
///     neutral mouth: line at y=face_cy+7
///     frown mouth: translate(cx, face_cy+7)
///   - Title: x=LEFT_MARGIN, y=25
///   - total_width = num_tasks*(TASK_W+TASK_MARGIN) + 2*LEFT_MARGIN
use crate::theme::Theme;

// ── Main render function ───────────────────────────────────────────────────────

pub fn render(diag: &JourneyDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let id = "mermaid-journey";

    if diag.tasks.is_empty() {
        return templates::empty_svg(id);
    }

    // Assign actors their positions and colours
    struct ActorInfo {
        name: String,
        color: &'static str,
        position: usize,
    }
    let actor_infos: Vec<ActorInfo> = diag
        .actors
        .iter()
        .enumerate()
        .map(|(i, name)| ActorInfo {
            name: name.clone(),
            color: ACTOR_COLOURS[i % ACTOR_COLOURS.len()],
            position: i,
        })
        .collect();

    let num_tasks = diag.tasks.len();

    // Total SVG width = num_tasks*(TASK_W+TASK_MARGIN) + 2*LEFT_MARGIN
    let task_step = TASK_MARGIN + TASK_WIDTH; // 200
    let total_width = (num_tasks as f64) * task_step + 2.0 * LEFT_MARGIN;

    let mut out = String::new();

    // SVG root
    out.push_str(&templates::svg_root(
        id,
        total_width as i64,
        total_width as i64,
        VIEW_HEIGHT as i64,
        (VIEW_HEIGHT + 25.0) as i64,
    ));

    // Style block
    out.push_str(&templates::style_block(&build_style(id, ff)));

    // Empty g (matches reference)
    out.push_str("<g></g>");

    // Arrowhead marker defs
    out.push_str(&templates::arrowhead_marker(id));

    // ── Actor legend (left side) ─────────────────────────────────────────────
    let mut legend_y = ACTOR_LEGEND_START_Y;
    for actor in &actor_infos {
        out.push_str(&templates::actor_circle(
            legend_y as i64,
            actor.position,
            actor.color,
        ));
        out.push_str(&templates::actor_label(
            (legend_y + 7.0) as i64,
            &escape_text(&actor.name),
        ));
        legend_y += ACTOR_LEGEND_STEP;
    }

    // ── Section header boxes ─────────────────────────────────────────────────
    {
        let mut last_section: Option<(&str, usize, usize)> = None;
        let mut task_i = 0usize;

        let flush_section =
            |out: &mut String, name: &str, sec_idx: usize, task_start: usize, task_end: usize| {
                let fill = SECTION_FILLS[sec_idx % SECTION_FILLS.len()];
                let count = task_end - task_start;
                let sec_x = (task_start as f64) * task_step + LEFT_MARGIN;
                let sec_w = (count as f64) * task_step - TASK_MARGIN;
                let tx = (sec_x + sec_w / 2.0) as i64;
                let ty = (50.0 + SECTION_HEIGHT / 2.0) as i64; // 75
                let si = sec_idx % SECTION_FILLS.len();
                out.push_str(&templates::section_rect(
                    sec_x as i64,
                    fill,
                    sec_w as i64,
                    SECTION_HEIGHT as i64,
                    si,
                ));
                out.push_str(&templates::section_label(
                    sec_x as i64,
                    sec_w as i64,
                    SECTION_HEIGHT as i64,
                    si,
                    tx,
                    ty,
                    &escape_text(name),
                    ff,
                ));
            };

        for task in &diag.tasks {
            let same = last_section
                .as_ref()
                .map(|(n, _, _)| *n == task.section.as_str())
                .unwrap_or(false);
            if same {
                task_i += 1;
            } else {
                if let Some((n, si, start)) = last_section.take() {
                    flush_section(&mut out, n, si, start, task_i);
                }
                last_section = Some((task.section.as_str(), task.section_index, task_i));
                task_i += 1;
            }
        }
        if let Some((n, si, start)) = last_section {
            flush_section(&mut out, n, si, start, task_i);
        }
    }

    // ── Tasks (task line, face, task box, actor dots) ────────────────────────
    for (i, task) in diag.tasks.iter().enumerate() {
        let task_x = (i as f64) * task_step + LEFT_MARGIN;
        let task_cx = task_x + TASK_WIDTH / 2.0; // centre x of task
        let fill = SECTION_FILLS[task.section_index % SECTION_FILLS.len()];
        let si = task.section_index % SECTION_FILLS.len();

        // Score-to-face-y: score 5→300, score 3→360, score 1→420
        let face_cy = TASK_LINE_BOTTOM - (task.score as f64) * 30.0;

        out.push_str("<g>");

        // Vertical dashed task line
        out.push_str(&templates::task_line(
            id,
            i,
            task_cx as i64,
            TASK_LINE_TOP as i64,
            TASK_LINE_BOTTOM as i64,
        ));

        // Face (circle) at score-mapped y position
        out.push_str(&templates::face_circle(task_cx as i64, face_cy as i64));

        // Face features (eyes + mouth based on score)
        let eye_left_cx = task_cx - 5.0;
        let eye_right_cx = task_cx + 5.0;
        let eye_y = face_cy - 5.0;
        out.push_str(&templates::face_eyes(
            eye_left_cx as i64,
            eye_right_cx as i64,
            eye_y as i64,
        ));

        // Mouth: smile for score>=4, neutral for score==3, frown for score<=2
        if task.score >= 4 {
            out.push_str(&templates::mouth_smile(
                task_cx as i64,
                (face_cy + 2.0) as i64,
            ));
        } else if task.score == 3 {
            out.push_str(&templates::mouth_neutral(
                (task_cx - 5.0) as i64,
                (task_cx + 5.0) as i64,
                (face_cy + 7.0) as i64,
            ));
        } else {
            out.push_str(&templates::mouth_frown(
                task_cx as i64,
                (face_cy + 7.0) as i64,
            ));
        }
        out.push_str("</g>"); // close face-features g

        // Task box
        out.push_str(&templates::task_rect(
            task_x as i64,
            TASK_LINE_TOP as i64,
            fill,
            TASK_WIDTH as i64,
            SECTION_HEIGHT as i64,
            si,
        ));

        // Actor dots on top-left corner of task box
        for (ai, actor_name) in task.people.iter().enumerate() {
            if let Some(actor) = actor_infos.iter().find(|a| &a.name == actor_name) {
                let dot_x = task_x + 14.0 + (ai as f64) * 10.0;
                out.push_str(&templates::actor_dot(
                    dot_x as i64,
                    TASK_LINE_TOP as i64,
                    actor.position,
                    actor.color,
                    &escape_text(&actor.name),
                ));
            }
        }

        // Task label text (foreignObject + text fallback)
        let text_cx = task_cx as i64;
        let text_cy = (TASK_LINE_TOP + SECTION_HEIGHT / 2.0) as i64; // 135
        out.push_str(&templates::task_label(
            task_x as i64,
            TASK_LINE_TOP as i64,
            TASK_WIDTH as i64,
            SECTION_HEIGHT as i64,
            text_cx,
            text_cy,
            &escape_text(&task.task),
            ff,
        ));

        out.push_str("</g>"); // close task g
    }

    // ── Title ────────────────────────────────────────────────────────────────
    if let Some(ref title) = diag.title {
        out.push_str(&templates::title_text(
            LEFT_MARGIN as i64,
            ff,
            &escape_text(title),
        ));
    }

    // ── Activity line ────────────────────────────────────────────────────────
    let line_x1 = LEFT_MARGIN;
    let line_x2 = (num_tasks as f64) * task_step + LEFT_MARGIN - 4.0;
    out.push_str(&templates::activity_line(
        line_x1 as i64,
        ACTIVITY_LINE_Y as i64,
        line_x2 as i64,
        id,
    ));

    out.push_str("</svg>");
    out
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn build_style(id: &str, ff: &str) -> String {
    format!(
        "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}@keyframes edge-animation-frame{{from{{stroke-dashoffset:0;}}}}@keyframes dash{{to{{stroke-dashoffset:0;}}}}#{id} .mouth{{stroke:#666;}}#{id} line{{stroke:#333;}}#{id} .legend{{fill:#333;font-family:{ff};}}#{id} .label text{{fill:#333;}}#{id} .label{{color:#333;}}#{id} .face{{fill:#FFF8DC;stroke:#999;}}#{id} .task-type-0,#{id} .section-type-0{{fill:#ECECFF;}}#{id} .task-type-1,#{id} .section-type-1{{fill:#ffffde;}}#{id} .task-type-2,#{id} .section-type-2{{fill:hsl(304, 100%, 96.2745098039%);}}#{id} .task-type-3,#{id} .section-type-3{{fill:hsl(124, 100%, 93.5294117647%);}}#{id} .task-type-4,#{id} .section-type-4{{fill:hsl(176, 100%, 96.2745098039%);}}#{id} .task-type-5,#{id} .section-type-5{{fill:hsl(-4, 100%, 93.5294117647%);}}#{id} .task-type-6,#{id} .section-type-6{{fill:hsl(8, 100%, 96.2745098039%);}}#{id} .task-type-7,#{id} .section-type-7{{fill:hsl(188, 100%, 93.5294117647%);}}",
        id = id, ff = ff,
    )
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    #[test]
    fn basic_render_produces_svg() {
        let input = "journey\n    title My working day\n    section Go to work\n      Make tea: 5: Me\n      Go upstairs: 3: Me\n      Do work: 1: Me, Cat\n    section Go home\n      Go downstairs: 5: Me\n      Sit down: 3: Me";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        assert!(svg.contains("<svg"), "no <svg element");
        assert!(svg.contains("My working day"), "no title");
        assert!(svg.contains("Make tea"), "no task");
        assert!(svg.contains("Go to work"), "no section");
    }

    #[test]
    fn renders_no_tasks() {
        let input = "journey\n    title Empty";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        assert!(svg.contains("Empty Journey"));
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "journey\n    title My working day\n    section Go to work\n      Make tea: 5: Me\n      Go upstairs: 3: Me\n      Do work: 1: Me, Cat\n    section Go home\n      Go downstairs: 5: Me\n      Sit down: 5: Me";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
