/// Parser for Mermaid Gantt chart syntax.
///
/// Supported grammar (faithful to ganttDb.ts):
///   gantt
///       title <text>
///       dateFormat YYYY-MM-DD
///       excludes weekends
///       tickInterval 1week|1day|1month
///       section <name>
///       <task label>   :<modifiers...>, <id>, <start>, <end|duration>
///
/// Task modifiers: done, active, crit, milestone
/// Start: YYYY-MM-DD or "after <id>"
/// Duration: Nd (days), Nh (hours), Nw (weeks), Nm (months)
/// End: YYYY-MM-DD (absolute end date)

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Normal,
    Done,
    Active,
}

#[derive(Debug, Clone)]
pub struct Task {
    /// Display label (trimmed)
    pub label: String,
    /// Optional task id (for `after` dependencies)
    pub id: String,
    /// Section this task belongs to
    pub section: String,
    /// Section index (0-based, for colour cycling)
    pub section_index: usize,
    /// Task status
    pub status: TaskStatus,
    /// Whether this is a critical task
    pub crit: bool,
    /// Whether this is a milestone
    pub milestone: bool,
    /// Start in days-since-epoch (resolved from absolute date or `after` dep)
    pub start_day: f64,
    /// End in days-since-epoch (start + duration)
    pub end_day: f64,
}

#[derive(Debug, Clone, Default)]
pub struct GanttDiagram {
    pub title: Option<String>,
    pub date_format: String,
    pub excludes: Vec<String>,
    pub tick_interval: Option<String>,
    /// Pre-parsed tick interval in days (None = use auto-computed interval).
    pub tick_interval_days: Option<f64>,
    pub sections: Vec<String>,
    pub tasks: Vec<Task>,
}

/// Parse tickInterval string like "1week", "1day", "1month" → days.
pub fn parse_tick_interval(s: &str) -> f64 {
    let s = s.trim().to_lowercase();
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    let unit: String = s.chars().skip_while(|c| c.is_ascii_digit()).collect();
    let n: f64 = digits.parse().unwrap_or(1.0);
    match unit.as_str() {
        "day" | "days" => n,
        "week" | "weeks" => n * 7.0,
        "month" | "months" => n * 30.0,
        _ => 7.0, // default 1 week
    }
}

// ── Date helpers ─────────────────────────────────────────────────────────────

/// Parse "YYYY-MM-DD" → days since 2000-01-01 (arbitrary epoch, consistent).
/// Returns None if the string cannot be parsed.
pub fn parse_date(s: &str) -> Option<f64> {
    let s = s.trim();
    let parts: Vec<&str> = s.splitn(3, '-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i64 = parts[0].parse().ok()?;
    let month: i64 = parts[1].parse().ok()?;
    let day: i64 = parts[2].parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(date_to_days(year, month, day))
}

/// Convert calendar date to a day number (Julian Day Number style, but relative).
/// Uses the standard algorithm: days since 2000-01-01.
fn date_to_days(year: i64, month: i64, day: i64) -> f64 {
    // Algorithm: count days since year 0 using the Gregorian proleptic calendar
    let y = year;
    let m = month;
    let d = day;
    // Days from year 0 to start of year
    let ya = if m <= 2 { y - 1 } else { y };
    let era = ya.div_euclid(400);
    let yoe = ya.rem_euclid(400); // year of era [0, 399]
    let doy = (153 * (m + (if m > 2 { -3 } else { 9 })) + 2) / 5 + d - 1; // day of year [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // day of era [0, 146096]
    let jdn = era * 146097 + doe - 719468; // days since Unix epoch (1970-01-01)
    jdn as f64
}

/// Parse a duration string: "Nd", "Nh", "Nw", "Nm" → days.
fn parse_duration(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (num_str, unit) = if let Some(n) = s.strip_suffix('d') {
        (n, 'd')
    } else if let Some(n) = s.strip_suffix('h') {
        (n, 'h')
    } else if let Some(n) = s.strip_suffix('w') {
        (n, 'w')
    } else if let Some(n) = s.strip_suffix('m') {
        (n, 'm')
    } else {
        return None;
    };
    let n: f64 = num_str.trim().parse().ok()?;
    let days = match unit {
        'd' => n,
        'h' => n / 24.0,
        'w' => n * 7.0,
        'm' => n * 30.0, // approximate; Mermaid uses 30 days for months
        _ => return None,
    };
    Some(days)
}

// ── Parser ───────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> crate::error::ParseResult<GanttDiagram> {
    let mut diag = GanttDiagram {
        date_format: "YYYY-MM-DD".to_string(),
        ..Default::default()
    };

    let mut current_section = String::new();
    let mut section_index: usize = 0;
    let mut seen_first_section = false;

    // First pass: collect tasks with raw start/end info
    // We need two passes: one to collect, one to resolve `after` deps.

    struct RawTask {
        label: String,
        id: String,
        section: String,
        section_index: usize,
        status: TaskStatus,
        crit: bool,
        milestone: bool,
        start_spec: StartSpec,
        end_spec: EndSpec,
    }

    #[derive(Debug, Clone)]
    enum StartSpec {
        Date(f64),     // absolute date in days
        After(String), // "after <id>"
        None,          // use previous task's end
    }

    #[derive(Debug, Clone)]
    enum EndSpec {
        Date(f64),     // absolute end date
        Duration(f64), // duration in days
        None,
    }

    let mut raw_tasks: Vec<RawTask> = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();

        // Skip blank lines and comments
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        // Skip the "gantt" keyword line itself
        if trimmed == "gantt" {
            continue;
        }

        // Header directives
        if let Some(rest) = trimmed.strip_prefix("title") {
            let t = rest.trim();
            if !t.is_empty() {
                diag.title = Some(t.to_string());
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("dateFormat") {
            diag.date_format = rest.trim().to_string();
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("excludes") {
            for part in rest.split(',') {
                let p = part.trim();
                if !p.is_empty() {
                    diag.excludes.push(p.to_string());
                }
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("tickInterval") {
            let raw = rest.trim().to_string();
            diag.tick_interval_days = Some(parse_tick_interval(&raw));
            diag.tick_interval = Some(raw);
            continue;
        }

        // Section header
        if let Some(rest) = trimmed.strip_prefix("section") {
            let name = rest.trim().to_string();
            if seen_first_section {
                section_index += 1;
            }
            current_section = name.clone();
            seen_first_section = true;
            if !diag.sections.contains(&name) {
                diag.sections.push(name);
            }
            continue;
        }

        // Task line: must contain a colon
        if let Some(colon_pos) = trimmed.find(':') {
            let label = trimmed[..colon_pos].trim().to_string();
            let spec = trimmed[colon_pos + 1..].trim();

            // Parse comma-separated modifiers and date fields
            let parts: Vec<&str> = spec.split(',').map(|s| s.trim()).collect();

            let mut status = TaskStatus::Normal;
            let mut crit = false;
            let mut milestone = false;
            let mut task_id = String::new();
            let mut start_spec = StartSpec::None;
            let mut end_spec = EndSpec::None;

            // Process parts left-to-right
            // Mermaid ganttDb.ts logic (simplified):
            // Parts can be: modifiers (done|active|crit|milestone), id, start, end/duration
            // A modifier is a known keyword; an id is an identifier (no spaces, no date format);
            // a start is either a date or "after <id>"; an end is a date or duration.

            let remaining: Vec<&str> = parts.clone();
            let mut i = 0;

            // First consume modifier keywords
            while i < remaining.len() {
                match remaining[i] {
                    "done" => {
                        status = TaskStatus::Done;
                        i += 1;
                    }
                    "active" => {
                        status = TaskStatus::Active;
                        i += 1;
                    }
                    "crit" => {
                        crit = true;
                        i += 1;
                    }
                    "milestone" => {
                        milestone = true;
                        i += 1;
                    }
                    _ => break,
                }
            }

            // Remaining parts: [id?,] start, end|duration
            // Heuristic: if first remaining is NOT a date and NOT "after ...", it's an id.
            // A date matches YYYY-MM-DD pattern.
            // "after xxx" is a start spec.
            // Otherwise it's a duration or date.

            if i < remaining.len() {
                let tok = remaining[i];
                let is_date = looks_like_date(tok);
                let is_after = tok.starts_with("after ");
                let is_duration = parse_duration(tok).is_some();

                if !is_date && !is_after && !is_duration {
                    // It's a task id
                    task_id = tok.to_string();
                    i += 1;
                }
            }

            // Now parse start
            if i < remaining.len() {
                let tok = remaining[i];
                if let Some(stripped) = tok.strip_prefix("after ") {
                    let dep_id = stripped.trim().to_string();
                    start_spec = StartSpec::After(dep_id);
                    i += 1;
                } else if let Some(d) = parse_date(tok) {
                    start_spec = StartSpec::Date(d);
                    i += 1;
                } else if parse_duration(tok).is_some() {
                    // No explicit start - duration only
                    // Don't consume, leave for end_spec parsing
                }
            }

            // Now parse end/duration
            if i < remaining.len() {
                let tok = remaining[i];
                if let Some(d) = parse_date(tok) {
                    end_spec = EndSpec::Date(d);
                } else if let Some(dur) = parse_duration(tok) {
                    end_spec = EndSpec::Duration(dur);
                }
            }

            // Default duration for milestone is 0
            if milestone && matches!(end_spec, EndSpec::None) {
                end_spec = EndSpec::Duration(0.0);
            }

            // Default duration if none specified
            if matches!(end_spec, EndSpec::None) {
                end_spec = EndSpec::Duration(1.0); // 1 day default
            }

            if task_id.is_empty() {
                // Auto-generate id: "task" + index
                task_id = format!("task{}", raw_tasks.len() + 1);
            }

            let sec_name = if seen_first_section {
                current_section.clone()
            } else {
                String::new()
            };

            let sec_idx = if seen_first_section { section_index } else { 0 };

            raw_tasks.push(RawTask {
                label,
                id: task_id,
                section: sec_name,
                section_index: sec_idx,
                status,
                crit,
                milestone,
                start_spec,
                end_spec,
            });
        }
    }

    // Second pass: resolve dependencies
    // Build a map of id → end_day for resolved tasks
    let mut id_to_end: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

    // Determine if weekends should be excluded from duration counting
    let exclude_weekends = diag.excludes.iter().any(|e| e == "weekends");

    // Find the global start date (first absolute date)
    let global_start = raw_tasks
        .iter()
        .find_map(|t| {
            if let StartSpec::Date(d) = &t.start_spec {
                Some(*d)
            } else {
                None
            }
        })
        .unwrap_or(0.0);

    let mut last_end: f64 = global_start;

    for raw in &raw_tasks {
        // Resolve the start day; if excludes weekends and the resolved start lands on a
        // weekend (e.g. from an `after` dependency that ends on Friday, making the
        // successor start on Saturday), advance it to the next Monday.
        let start_day_raw = match &raw.start_spec {
            StartSpec::Date(d) => *d,
            StartSpec::After(dep_id) => id_to_end.get(dep_id).copied().unwrap_or(last_end),
            StartSpec::None => last_end,
        };

        let start_day = if exclude_weekends {
            skip_to_weekday(start_day_raw)
        } else {
            start_day_raw
        };

        let end_day = match &raw.end_spec {
            EndSpec::Date(d) => *d,
            EndSpec::Duration(dur) => {
                if exclude_weekends && *dur >= 1.0 {
                    add_working_days(start_day, *dur)
                } else {
                    start_day + dur
                }
            }
            EndSpec::None => start_day + 1.0,
        };

        id_to_end.insert(raw.id.clone(), end_day);
        last_end = end_day;

        diag.tasks.push(Task {
            label: raw.label.clone(),
            id: raw.id.clone(),
            section: raw.section.clone(),
            section_index: raw.section_index,
            status: raw.status.clone(),
            crit: raw.crit,
            milestone: raw.milestone,
            start_day,
            end_day,
        });
    }

    crate::error::ParseResult::ok(diag)
}

/// Format a day number (days since Unix epoch) as "YYYY-MM-DD".
/// Public so the renderer can use it for exclude-range labels.
pub fn format_date_public(days: f64) -> String {
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

/// Given a day number (days since Unix epoch 1970-01-01), return the day-of-week.
/// 0=Thursday, 1=Friday, 2=Saturday, 3=Sunday, 4=Monday, 5=Tuesday, 6=Wednesday.
/// So: Saturday = dow % 7 == 2, Sunday = dow % 7 == 3.
fn day_of_week(day: f64) -> i64 {
    (day as i64).rem_euclid(7)
}

/// Returns true if the given day is a Saturday or Sunday.
fn is_weekend(day: f64) -> bool {
    let dow = day_of_week(day);
    dow == 2 || dow == 3
}

/// If `day` falls on a weekend, advance it to the following Monday.
fn skip_to_weekday(day: f64) -> f64 {
    let mut d = day.floor();
    loop {
        if !is_weekend(d) {
            break;
        }
        d += 1.0;
    }
    d
}

/// Compute the end day given a start day and a duration in working days,
/// skipping Saturday and Sunday.
///
/// A "10-day" task starting on Monday Jan 1 spans Jan 1–Jan 12 (10 weekdays),
/// with the exclusive end being Jan 13 (the first day after the last working day).
/// The start day itself counts as working day #1.
fn add_working_days(start: f64, working_days: f64) -> f64 {
    // We count whole working days only. Fractional durations (hours) are calendar fractions.
    let whole_days = working_days.floor() as i64;
    let frac = working_days - working_days.floor();

    let mut d = start.floor() as i64;
    let mut counted = 0i64;
    // Count working days starting from (and including) `start`.
    // After the loop, `d` is the first calendar day AFTER the last working day (exclusive end).
    while counted < whole_days {
        if !is_weekend(d as f64) {
            counted += 1;
        }
        d += 1;
    }
    d as f64 + frac
}

/// Returns true if the string looks like a YYYY-MM-DD date.
fn looks_like_date(s: &str) -> bool {
    // Basic heuristic: 10 chars, has dashes at positions 4 and 7
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b[..4].iter().all(|c| c.is_ascii_digit())
        && b[5..7].iter().all(|c| c.is_ascii_digit())
        && b[8..10].iter().all(|c| c.is_ascii_digit())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        let input = "gantt\n    title A Gantt Diagram\n    dateFormat YYYY-MM-DD\n    section Section\n    A task          :a1, 2024-01-01, 30d\n    Another task    :after a1, 20d";
        let d = parse(input).diagram;
        assert_eq!(d.title.as_deref(), Some("A Gantt Diagram"));
        assert_eq!(d.tasks.len(), 2);
        assert_eq!(d.tasks[0].id, "a1");
        assert_eq!(d.tasks[0].label, "A task");
        // 30 day duration
        assert!((d.tasks[0].end_day - d.tasks[0].start_day - 30.0).abs() < 0.01);
        // Second task starts where first ends
        assert!((d.tasks[1].start_day - d.tasks[0].end_day).abs() < 0.01);
    }

    #[test]
    fn parse_date_fn() {
        let d1 = parse_date("2024-01-01").unwrap();
        let d2 = parse_date("2024-01-02").unwrap();
        assert!((d2 - d1 - 1.0).abs() < 0.01);
    }

    #[test]
    fn parse_duration_fn() {
        assert_eq!(parse_duration("30d"), Some(30.0));
        assert_eq!(parse_duration("1w"), Some(7.0));
        assert_eq!(parse_duration("24h"), Some(1.0));
        assert_eq!(parse_duration("1m"), Some(30.0));
    }

    #[test]
    fn parse_milestones() {
        let input = "gantt\n    dateFormat YYYY-MM-DD\n    title Adding GANTT milestones\n    section A\n    Completed task      :done, des1, 2024-01-06, 2024-01-08\n    Active task         :active, des2, 2024-01-09, 3d\n    Future task         :des3, after des2, 5d\n    section Critical\n    Crit done task      :crit, done, 2024-01-06, 24h\n    Crit active task    :crit, active, 3d\n    Crit task           :crit, 5d";
        let d = parse(input).diagram;
        assert_eq!(d.tasks.len(), 6);
        assert_eq!(d.tasks[0].status, TaskStatus::Done);
        assert!(!d.tasks[0].crit);
        assert!(d.tasks[3].crit);
        assert_eq!(d.tasks[3].status, TaskStatus::Done);
        assert!(d.tasks[4].crit);
        assert_eq!(d.tasks[4].status, TaskStatus::Active);
    }
}
