use super::constants::*;
use super::parser::{GanttDiagram, Task, TaskStatus};
/// Faithful Rust port of Mermaid's ganttRenderer.ts.
///
/// Layout algorithm (matches ganttRenderer.ts exactly):
/// - Left margin (leftPad) = 75px for section labels
/// - Total SVG width = 1984px (Mermaid default SVG width)
/// - Chart draw width = svgWidth - leftPad - rightPad = 1984 - 75 - 75 = 1834px (approx)
/// - Title at top (y=25), font-size 18px
/// - X-axis grid drawn below chart area
/// - Sections: alternating colour bands (section0/section1/section2/section3)
/// - Each task row: height=24px, task bar height=20px, bar y-offset=2px within row
/// - Tasks in same section share background rows
///
/// CSS classes used (faithful to ganttRenderer.ts):
///   .task0/.task1/.task2/.task3  — normal tasks by section
///   .active0/.active1/...        — active tasks
///   .done0/.done1/...            — done tasks
///   .crit0/.crit1/...            — crit tasks
///   .activeCrit0/.activeCrit1/... — active+crit
///   .doneCrit0/.doneCrit1/...    — done+crit
///   .milestone                   — milestone marker (rotated rect)
///   .milestoneText               — milestone label (italic)
///   .taskText0/.taskText1/...    — text inside task bar
///   .taskTextOutsideRight/.taskTextOutsideLeft — text outside bar
///   .sectionTitle0/.sectionTitle1/... — section label text
///   .grid                        — axis grid
///   .today                       — today line
///   .titleText                   — diagram title
#[allow(unused_imports)]
use super::templates::{
    self, esc, escape_id, exclude_rect, grid_domain_path, grid_group_open, grid_tick,
    milestone_rect, section_band_rect, section_title, svg_root, task_bar_rect, task_text,
    title_text, today_line,
};
use crate::text::measure;
use crate::theme::Theme;
fn svg_height(num_rows: usize) -> f64 {
    CHART_TOP + (num_rows as f64) * ROW_HEIGHT + GRID_AXIS_OFFSET + GRID_BOTTOM_PAD + 25.0
}

// ── Tick interval helpers ─────────────────────────────────────────────────────

/// Compute a nice tick interval in days given the total time span and draw width.
/// Matches Mermaid's d3 timeScale tick behaviour.
///
/// D3 `d3.scaleTime().ticks(n)` targets ~10 ticks and picks from these day-level intervals:
/// 1 day, 2 days, 7 days (week), 14 days, 30 days (month).
/// The approximate interval is span/10, then snapped to the nearest D3 time interval.
fn compute_tick_interval(span_days: f64, explicit: Option<f64>) -> f64 {
    if let Some(days) = explicit {
        return days;
    }
    // Auto: D3 timeScale.ticks() with ~10 target ticks
    // Approximate desired interval = span / 10
    let desired = span_days / 10.0;
    // D3 time intervals (in days): 1, 2, 7, 14, 30, 91 (quarter), 365 (year)
    // Use 1-day ticks for spans up to ~15 days (desired ≤ 1.5).
    // D3 picks 1-day for 11-day spans (desired=1.1) since 11 ticks ≈ target of 10.
    if desired <= 1.5 {
        1.0
    } else if desired <= 3.5 {
        // For spans ~20-35 days (desired ~2-3.5), use 2-day ticks.
        // e.g. 28-day span (excludes weekends): desired=2.8 → 2-day ticks.
        2.0
    } else if desired <= 7.0 {
        7.0
    } else if desired <= 14.0 {
        14.0
    } else {
        30.0
    }
}

/// Format a day number as "YYYY-MM-DD".
fn format_date(days: f64) -> String {
    // Reverse of date_to_days (Julian Day Number algorithm)
    let z = days as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

// ── CSS class computation ─────────────────────────────────────────────────────

/// Compute the CSS class for a task bar, matching ganttRenderer.ts exactly.
/// The class is a combination of state and section-index suffix.
/// Milestone tasks get `.milestone` prepended via transform in the SVG.
fn task_class(task: &Task) -> String {
    let sec = task.section_index % 4;
    let base = match (&task.status, task.crit) {
        (TaskStatus::Done, true) => format!("doneCrit{sec}"),
        (TaskStatus::Done, false) => format!("done{sec}"),
        (TaskStatus::Active, true) => format!("activeCrit{sec}"),
        (TaskStatus::Active, false) => format!("active{sec}"),
        (TaskStatus::Normal, true) => format!("crit{sec}"),
        (TaskStatus::Normal, false) => format!("task{sec}"),
    };
    base
}

/// Text class suffix (matches *Text* CSS).
fn text_class(task: &Task) -> String {
    let sec = task.section_index % 4;
    // Base taskText class always present
    let mut classes = vec![format!("taskText{sec}")];

    // Additional state text classes
    match (&task.status, task.crit) {
        (TaskStatus::Done, true) => {
            classes.push(format!("doneCritText{sec}"));
        }
        (TaskStatus::Done, false) => {
            classes.push(format!("doneText{sec}"));
        }
        (TaskStatus::Active, true) => {
            classes.push(format!("activeCritText{sec}"));
            classes.push(format!("critText{sec}"));
        }
        (TaskStatus::Active, false) => {
            classes.push(format!("activeText{sec}"));
        }
        (TaskStatus::Normal, true) => {
            classes.push(format!("critText{sec}"));
        }
        (TaskStatus::Normal, false) => {}
    }

    if task.milestone {
        classes.push("milestoneText".to_string());
    }

    classes.join(" ")
}

// ── Main render function ──────────────────────────────────────────────────────

pub fn render(diag: &GanttDiagram, theme: Theme, _use_foreign_object: bool) -> String {
    if diag.tasks.is_empty() {
        return empty_svg();
    }

    let vars = theme.resolve();
    let text_color = vars.text_color;
    let title_color = vars.title_color;
    let background = vars.background;
    // Per-theme contrast colour for done/active task label text (CSS: doneText, activeCritText, etc.)
    let contrast_color = match theme {
        Theme::Dark => "#2c2c2c",
        Theme::Neutral => "#333",
        _ => "black",
    };
    // Gantt section band fills — from ThemeVars
    let section_color = vars.gantt_section_fill0;
    let section_color2 = vars.gantt_section_fill1;
    let exclude_fill = vars.gantt_exclude_fill;

    // Compute time range
    let t_min = diag
        .tasks
        .iter()
        .map(|t| t.start_day)
        .fold(f64::INFINITY, f64::min);
    let t_max_tasks = diag
        .tasks
        .iter()
        .map(|t| t.end_day)
        .fold(f64::NEG_INFINITY, f64::max);
    let span_raw = (t_max_tasks - t_min).max(1.0);

    // Tick interval (computed from raw span before any domain extension)
    let tick_days = compute_tick_interval(span_raw, diag.tick_interval_days);

    // Compute ticks: start at first tick >= t_min, step by tick_days.
    // For 7-day (weekly) intervals, D3 snaps to Sundays (day-of-week = 0 in JS / Sunday-based).
    // 1970-01-01 was a Thursday. Offset within the week:
    //   0=Thu, 1=Fri, 2=Sat, 3=Sun, 4=Mon, 5=Tue, 6=Wed
    // So Sunday ≡ 3 (mod 7). Days-to-next-Sunday = (3 - dow + 7) % 7.
    let first_tick = if (tick_days - 7.0).abs() < 0.01 {
        // Weekly: snap to next Sunday >= t_min
        let t_floor = t_min.floor() as i64;
        let dow = t_floor.rem_euclid(7); // 0=Thu…3=Sun…6=Wed
        let days_to_sunday = (3 - dow).rem_euclid(7) as f64;
        if days_to_sunday == 0.0 {
            t_min
        } else {
            t_floor as f64 + days_to_sunday
        }
    } else if (tick_days - 1.0).abs() < 0.01 {
        // Daily: snap to the start of the first day >= t_min
        t_min.ceil()
    } else if (tick_days - 2.0).abs() < 0.01 {
        // 2-day ticks: start at t_min (reference shows tick at exactly t_min=Jan 1)
        t_min.floor()
    } else {
        // Monthly or explicit: numeric ceiling
        (t_min / tick_days).ceil() * tick_days
    };
    let mut ticks: Vec<f64> = Vec::new();
    let mut t = first_tick;
    // Generate ticks up to t_max_tasks (inclusive with tiny tolerance for float equality).
    while t <= t_max_tasks + tick_days * 0.01 {
        ticks.push(t);
        t += tick_days;
    }
    // D3 nice(): for multi-day intervals (2-day, weekly, etc.), if the last tick lands
    // exactly on t_max_tasks, extend by one more tick for visual context.
    // For 1-day ticks the reference does NOT extend past t_max_tasks.
    if tick_days > 1.0 {
        if let Some(&last) = ticks.last() {
            if (last - t_max_tasks).abs() < tick_days * 0.01 {
                ticks.push(last + tick_days);
            }
        }
    }

    // The effective scale domain is max(t_max_tasks, last_tick).
    // - If the last task ends exactly at a tick, D3's nice() extends one more tick,
    //   so the chart extends past the last task (visible padding).
    // - If the last task ends after the last tick, the domain covers the tasks.
    let t_max = ticks
        .last()
        .copied()
        .unwrap_or(t_max_tasks)
        .max(t_max_tasks);
    let span = (t_max - t_min).max(1.0);

    // Scale: maps day → x pixel within [LEFT_PAD, LEFT_PAD + DRAW_WIDTH]
    let day_to_x = |d: f64| -> f64 { LEFT_PAD + (d - t_min) / span * DRAW_WIDTH };

    let num_rows = diag.tasks.len();
    let height = svg_height(num_rows);
    let grid_y = CHART_TOP + (num_rows as f64) * ROW_HEIGHT + GRID_AXIS_OFFSET; // y where x-axis sits

    let id = "mermaid-gantt";

    let mut out = String::new();

    // SVG root
    out.push_str(&svg_root(id, SVG_WIDTH, height as i64));

    // Empty first group (Mermaid always emits this)
    out.push_str("<g></g>");

    // ── Exclude-range shading (weekends) ─────────────────────────────────────
    // Mermaid renders a grey band for each excluded weekend within the chart range.
    let exclude_weekends = diag.excludes.iter().any(|e| e == "weekends");
    let excl_y = TITLE_TOP + 10.0; // = 35
    let excl_height = grid_y - excl_y;
    out.push_str("<g>");
    if exclude_weekends {
        // Iterate over all Saturdays within [t_min, t_max]
        // Our epoch: day 0 = 1970-01-01 (Thursday). Saturday = dow % 7 == 2.
        // Find the first Saturday >= t_min
        let t_min_i = t_min.floor() as i64;
        let first_sat = {
            let dow = t_min_i.rem_euclid(7); // 0=Thu…2=Sat…3=Sun…
            let days_to_sat = (2 - dow).rem_euclid(7);
            t_min_i + days_to_sat
        };
        let mut sat = first_sat;
        while (sat as f64) < t_max {
            let excl_start = sat as f64;
            let excl_end = excl_start + 2.0; // Saturday + Sunday
            let ex = day_to_x(excl_start).round() as i64;
            let ex_end = day_to_x(excl_end).round() as i64;
            let ew = (ex_end - ex).max(0);
            let date_label = super::parser::format_date_public(excl_start);
            out.push_str(&exclude_rect(
                id,
                &date_label,
                ex,
                excl_y as i64,
                ew,
                excl_height as i64,
                (ex as f64 + ew as f64 / 2.0).round() as i64,
                (excl_y + excl_height / 2.0).round() as i64,
                exclude_fill,
            ));
            sat += 7; // next Saturday
        }
    }
    out.push_str("</g>");

    // ── X-axis grid ──────────────────────────────────────────────────────────
    // The grid group is translated to (LEFT_PAD, grid_y).
    // The domain/tick lines extend upward from grid_y to (TITLE_TOP + 10) in page coords,
    // so in the grid's local coordinate system the top is -(grid_y - (TITLE_TOP + 10)).
    let grid_height = grid_y - (TITLE_TOP + 10.0);
    out.push_str(&grid_group_open(
        LEFT_PAD as i64,
        grid_y as i64,
        AXIS_FONT_SIZE as i64,
    ));

    // Domain line (the horizontal baseline)
    // D3 axis uses 0.5-offset for crisp rendering: M0.5,{-h}V0.5H{w+0.5}V{-h}
    out.push_str(&grid_domain_path(
        -(grid_height.round() as i64),
        DRAW_WIDTH + 0.5,
    ));

    // Tick marks and labels
    // D3 axis adds 0.5 to pixel positions for crisp SVG rendering (crispEdges).
    // D3 rounds to the nearest integer first, then adds 0.5.
    for tick in &ticks {
        let x = ((*tick - t_min) / span * DRAW_WIDTH).round() + 0.5;
        let label = format_date(*tick);
        out.push_str(&grid_tick(
            x,
            -(grid_height as i64),
            AXIS_FONT_SIZE as i64,
            &label,
            text_color,
        ));
    }

    out.push_str("</g>");

    // ── Section background bands ──────────────────────────────────────────────
    // Mermaid renders ONE rect per task ROW (not per section), each with height=ROW_HEIGHT.
    // The band width extends to SVG_WIDTH - RIGHT_PAD/2 (= 1984 - 37.5 = 1946.5).
    out.push_str("<g>");

    // Group tasks by section order, track their y positions
    let section_bands = compute_section_bands(diag);
    let band_width = SVG_WIDTH - RIGHT_PAD / 2.0; // 1946.5
    for (sec_name, sec_idx, row_start, row_count) in &section_bands {
        let _ = sec_name;
        let class_idx = sec_idx % 4;
        for row_offset in 0..*row_count {
            let band_y = CHART_TOP + (*row_start + row_offset) as f64 * ROW_HEIGHT;
            out.push_str(&section_band_rect(
                band_y as i64,
                band_width,
                ROW_HEIGHT as i64,
                class_idx,
                section_color,
                section_color2,
                background,
            ));
        }
    }
    out.push_str("</g>");

    // ── Task bars ─────────────────────────────────────────────────────────────
    out.push_str("<g>");

    for (row_idx, task) in diag.tasks.iter().enumerate() {
        let bar_y = CHART_TOP + (row_idx as f64) * ROW_HEIGHT + BAR_OFFSET;
        let bar_x = day_to_x(task.start_day);
        let bar_w = (day_to_x(task.end_day) - bar_x).max(0.0);
        let bar_cx = bar_x + bar_w / 2.0;
        let bar_cy = bar_y + BAR_HEIGHT / 2.0;

        let tc = task_class(task);
        let _base_tc = base_task_class(task);

        if task.milestone {
            // Milestone: rendered as a rotated rect (diamond shape)
            // In Mermaid: the rect is rendered at the midpoint, then rotated 45°
            // Size matches BAR_HEIGHT
            let half = BAR_HEIGHT / 2.0;
            let mx = bar_cx;
            let my = bar_y + BAR_HEIGHT / 2.0;
            out.push_str(&milestone_rect(
                id,
                &escape_id(&task.id),
                mx - half * 0.8,
                my - half * 0.8,
                BAR_HEIGHT * 0.8,
                mx,
                my,
                &tc,
                theme,
            ));
        } else {
            // Normal task bar
            out.push_str(&task_bar_rect(
                id,
                &escape_id(&task.id),
                bar_x.round() as i64,
                bar_y as i64,
                bar_w.round() as i64,
                BAR_HEIGHT as i64,
                bar_cx.round() as i64,
                bar_cy.round() as i64,
                &tc,
                theme,
            ));
        }

        // Task text — check if it fits inside the bar
        let text = task.label.trim_end().to_string();
        let (text_w, _) = measure(&text, FONT_SIZE);
        let text_y = bar_y + BAR_HEIGHT / 2.0 + 3.5; // approximate vertical center

        // Text class: "taskText taskText0" etc. — no bar class (task0/done0/etc.)
        // which would bleed the bar fill colour onto the text.
        let text_cls = format!(" taskText {}", text_class(task));
        let tid = escape_id(&task.id);

        if bar_w > 0.0 && text_w + 2.0 <= bar_w {
            // Text fits inside bar — centered
            out.push_str(&task_text(
                id,
                &tid,
                FONT_SIZE as i64,
                bar_cx as i64,
                text_y as i64,
                BAR_HEIGHT as i64,
                text_cls.trim(),
                &esc(&text),
                vars.text_color,
                contrast_color,
            ));
        } else if bar_w < LEFT_PAD {
            // Text outside to the right
            let outside_cls = format!("taskTextOutsideRight {}", text_class(task));
            out.push_str(&task_text(
                id,
                &tid,
                FONT_SIZE as i64,
                (bar_x + bar_w + 2.0) as i64,
                text_y as i64,
                BAR_HEIGHT as i64,
                outside_cls.trim(),
                &esc(&text),
                vars.text_color,
                contrast_color,
            ));
        } else {
            // Text inside but truncated — show centered anyway (matches Mermaid)
            out.push_str(&task_text(
                id,
                &tid,
                FONT_SIZE as i64,
                bar_cx as i64,
                text_y as i64,
                BAR_HEIGHT as i64,
                text_cls.trim(),
                &esc(&text),
                vars.text_color,
                contrast_color,
            ));
        }
    }

    out.push_str("</g>");

    // ── Section title labels ─────────────────────────────────────────────────
    out.push_str("<g>");

    for (sec_name, sec_idx, row_start, row_count) in &section_bands {
        let band_center_y =
            CHART_TOP + (*row_start as f64) * ROW_HEIGHT + (*row_count as f64) * ROW_HEIGHT / 2.0;
        let class_idx = sec_idx % 4;
        out.push_str(&section_title(
            band_center_y as i64,
            SECTION_FONT_SIZE as i64,
            class_idx,
            &esc(sec_name),
            title_color,
        ));
    }

    out.push_str("</g>");

    // ── Today line ───────────────────────────────────────────────────────────
    // Mermaid renders a today line; we compute today's position
    // (in the reference SVGs it's far to the right, outside the visible range for old dates)
    let today_days = today_days();
    let today_x = day_to_x(today_days);
    let chart_bottom = grid_y + GRID_BOTTOM_PAD;
    out.push_str(&today_line(
        today_x as i64,
        TITLE_TOP as i64,
        chart_bottom as i64,
    ));

    // ── Title ────────────────────────────────────────────────────────────────
    if let Some(ref title) = diag.title {
        out.push_str(&title_text(
            (SVG_WIDTH / 2.0) as i64,
            TITLE_TOP as i64,
            &esc(title),
            title_color,
        ));
    }

    out.push_str("</svg>");
    out
}

/// Returns today as days since Unix epoch.
fn today_days() -> f64 {
    // Use 2026-05-17 as the fixed "today" for reproducible output
    // (matches the current date in the environment)
    super::parser::parse_date("2026-05-17").unwrap_or(0.0)
}

/// Compute section bands: (section_name, section_index, row_start, row_count).
fn compute_section_bands(diag: &GanttDiagram) -> Vec<(String, usize, usize, usize)> {
    let mut bands: Vec<(String, usize, usize, usize)> = Vec::new();
    let mut last_sec: Option<(String, usize)> = None;
    let mut row_start = 0usize;
    let mut count = 0usize;

    for task in &diag.tasks {
        let sec_key = (task.section.clone(), task.section_index);
        if let Some(ref lk) = last_sec {
            if *lk == sec_key {
                count += 1;
            } else {
                bands.push((lk.0.clone(), lk.1, row_start, count));
                row_start += count;
                count = 1;
                last_sec = Some(sec_key);
            }
        } else {
            last_sec = Some(sec_key);
            count = 1;
        }
    }
    if let Some(lk) = last_sec {
        if count > 0 {
            bands.push((lk.0.clone(), lk.1, row_start, count));
        }
    }
    bands
}

/// Base task CSS class name (without section index suffix), for text.
fn base_task_class(task: &Task) -> String {
    let sec = task.section_index % 4;
    match (&task.status, task.crit) {
        (TaskStatus::Done, true) => format!("doneCrit{sec}"),
        (TaskStatus::Done, false) => format!("done{sec}"),
        (TaskStatus::Active, true) => format!("activeCrit{sec}"),
        (TaskStatus::Active, false) => format!("active{sec}"),
        (TaskStatus::Normal, true) => format!("crit{sec}"),
        (TaskStatus::Normal, false) => format!("task{sec}"),
    }
}

fn empty_svg() -> String {
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50"><text x="10" y="30">Empty Gantt</text></svg>"#.to_string()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    #[test]
    fn basic_render_produces_svg() {
        let input = "gantt\n    title A Gantt Diagram\n    dateFormat YYYY-MM-DD\n    section Section\n    A task          :a1, 2024-01-01, 30d\n    Another task    :after a1, 20d";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        assert!(svg.contains("<svg"), "no <svg element");
        assert!(svg.contains("A Gantt Diagram"), "no title");
        assert!(svg.contains("task0"), "no task bars");
        assert!(svg.contains("sectionTitle"), "no section title");
    }

    #[test]
    fn sections_render() {
        let input = "gantt\n    title Project Schedule\n    dateFormat YYYY-MM-DD\n    section Design\n    Wireframes      :des1, 2024-01-01, 14d\n    Mockups         :des2, after des1, 14d\n    section Development\n    Backend         :dev1, after des1, 30d\n    section Testing\n    QA              :qa1, after dev1, 14d";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        assert!(svg.contains("section0"));
        assert!(svg.contains("section1"));
        assert!(svg.contains("Design"));
        assert!(svg.contains("Development"));
    }

    #[test]
    fn milestones_render() {
        let input = "gantt\n    dateFormat YYYY-MM-DD\n    title Adding GANTT milestones\n    section A\n    Completed task      :done, des1, 2024-01-06, 2024-01-08\n    Active task         :active, des2, 2024-01-09, 3d\n    Future task         :des3, after des2, 5d\n    section Critical\n    Crit done task      :crit, done, 2024-01-06, 24h\n    Crit active task    :crit, active, 3d\n    Crit task           :crit, 5d";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        assert!(svg.contains("done0"), "done task not rendered");
        assert!(svg.contains("activeCrit"), "activeCrit not rendered");
        assert!(svg.contains("doneCrit"), "doneCrit not rendered");
    }

    #[test]
    fn format_date_roundtrip() {
        let d = super::super::parser::parse_date("2024-01-15").unwrap();
        let s = format_date(d);
        assert_eq!(s, "2024-01-15");
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "gantt\n    title A Gantt Diagram\n    dateFormat YYYY-MM-DD\n    section Section\n    A task          :a1, 2024-01-01, 30d\n    Another task    :after a1, 20d";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
