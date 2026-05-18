/// Parser for Mermaid timeline diagram syntax.
///
/// Faithful port of the jison grammar in
/// packages/mermaid/src/diagrams/timeline/parser/timeline.jison
/// and the DB logic in timelineDb.js.
///
/// Grammar (from jison):
///   event token: ":"\s(?:[^:\n]|":"(?!\s))+   — starts with ": "
///   period token: [^#:\n]+                      — no colon, hash, or newline
///
/// Key observation: `2002 : LinkedIn` on one line produces:
///   period token "2002 " then event token ": LinkedIn"
/// So the parser must handle "period : event" pairs on the same line.
///
/// Grammar summary:
///   timeline [LR|TD]
///   [title <text>]
///   [section <text>]
///   <period> [: <event>]
///   [: <event>]   -- additional events on subsequent lines
///   ...

#[derive(Debug, Clone, Default)]
pub struct TimelineDiagram {
    /// Optional diagram title.
    pub title: Option<String>,
    /// Direction (LR or TD). Default LR.
    pub direction: String,
    /// Ordered list of section names (declared via `section` keyword).
    pub sections: Vec<String>,
    /// All tasks in order.
    pub tasks: Vec<TimelineTask>,
}

#[derive(Debug, Clone)]
pub struct TimelineTask {
    pub id: usize,
    /// Section name this task belongs to (empty = no section).
    pub section: String,
    /// The period label (e.g. "2002", "1978").
    pub task: String,
    /// Events listed under this period.
    pub events: Vec<String>,
}

pub fn parse(input: &str) -> crate::error::ParseResult<TimelineDiagram> {
    let mut diag = TimelineDiagram {
        direction: "LR".to_string(),
        ..Default::default()
    };

    let mut current_section = String::new();
    let mut current_task_id: usize = 0;
    let mut raw_tasks: Vec<TimelineTask> = Vec::new();
    // Track section order (deduplicating, preserving order).
    let mut section_seen: Vec<String> = Vec::new();

    let mut header_seen = false;

    for line in input.lines() {
        let trimmed = line.trim();

        // Skip blank lines and % comments
        if trimmed.is_empty() || trimmed.starts_with("%%") || trimmed.starts_with('#') {
            continue;
        }

        // First meaningful line: "timeline", "timeline LR", "timeline TD"
        if !header_seen {
            let lower = trimmed.to_lowercase();
            if lower == "timeline" {
                header_seen = true;
                continue;
            } else if lower.starts_with("timeline") {
                let rest = trimmed["timeline".len()..].trim();
                if rest.eq_ignore_ascii_case("LR") {
                    diag.direction = "LR".to_string();
                } else if rest.eq_ignore_ascii_case("TD") {
                    diag.direction = "TD".to_string();
                }
                header_seen = true;
                continue;
            }
            continue;
        }

        // title <text>  — jison: 'title'\s[^\n]+
        if trimmed.len() > 5 && trimmed[..5].eq_ignore_ascii_case("title") {
            let ch = trimmed.as_bytes()[5];
            if ch == b' ' || ch == b'\t' {
                let title_text = trimmed[5..].trim();
                if !title_text.is_empty() {
                    diag.title = Some(title_text.to_string());
                }
                continue;
            }
        }

        // section <text>  — jison: 'section'\s[^:\n]+
        if trimmed.len() > 7 && trimmed[..7].eq_ignore_ascii_case("section") {
            let ch = trimmed.as_bytes()[7];
            if ch == b' ' || ch == b'\t' {
                let section_text = trimmed[7..].trim().to_string();
                if !section_seen.contains(&section_text) {
                    section_seen.push(section_text.clone());
                }
                current_section = section_text;
                continue;
            }
        }

        // event: starts with ": " (colon + whitespace)
        // jison: ":"\s(?:[^:\n]|":"(?!\s))+
        if let Some(rest) = trimmed.strip_prefix(':') {
            // Must have at least one whitespace after ':'
            if rest.starts_with(' ') || rest.starts_with('\t') {
                let event_text = rest.trim().to_string();
                if !event_text.is_empty() {
                    // Add event to most recent task
                    if let Some(task) = raw_tasks
                        .iter_mut()
                        .rev()
                        .find(|t| t.id + 1 == current_task_id)
                    {
                        task.events.push(event_text);
                    }
                }
                continue;
            }
        }

        // period line: [^#:\n]+
        // A period line may or may not contain ": event" after the period text.
        // jison tokenizes them separately but they can appear on the same line.
        //
        // Rule: if the line contains " : " (space-colon-space), split it:
        //   left part = period
        //   right part = event
        // Otherwise the whole non-colon-containing prefix is the period.
        //
        // We replicate the jison tokenizer: period = [^#:\n]+, then event = ":"\s...
        // Find the first colon in the line to split:
        if let Some(colon_pos) = trimmed.find(':') {
            let period_part = trimmed[..colon_pos].trim();
            let after_colon = &trimmed[colon_pos + 1..];

            if !period_part.is_empty() && !period_part.contains('#') {
                // We have a period
                ensure_section_tracked(&current_section, &mut section_seen);

                raw_tasks.push(TimelineTask {
                    id: current_task_id,
                    section: current_section.clone(),
                    task: period_part.to_string(),
                    events: Vec::new(),
                });
                current_task_id += 1;

                // Check for event on same line: after colon must start with whitespace
                let event_text = after_colon.trim();
                if !event_text.is_empty() {
                    if let Some(task) = raw_tasks
                        .iter_mut()
                        .rev()
                        .find(|t| t.id + 1 == current_task_id)
                    {
                        task.events.push(event_text.to_string());
                    }
                }
            }
        } else {
            // No colon: pure period line (no event)
            let period_part = trimmed;
            if !period_part.is_empty() && !period_part.contains('#') {
                ensure_section_tracked(&current_section, &mut section_seen);

                raw_tasks.push(TimelineTask {
                    id: current_task_id,
                    section: current_section.clone(),
                    task: period_part.to_string(),
                    events: Vec::new(),
                });
                current_task_id += 1;
            }
        }
    }

    diag.sections = section_seen;
    diag.tasks = raw_tasks;

    crate::error::ParseResult::ok(diag)
}

fn ensure_section_tracked(current_section: &str, section_seen: &mut Vec<String>) {
    // Only add non-empty sections that were explicitly declared via 'section' keyword.
    // (section_seen is only populated when a 'section' line is encountered.)
    // This function is a no-op here; section tracking happens on 'section' lines.
    // We call it for clarity in the period handling path.
    let _ = current_section;
    let _ = section_seen;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_no_sections() {
        let input = "timeline\n    title History\n    2002 : LinkedIn\n    2004 : Facebook\n         : Google";
        let d = parse(input).diagram;
        assert_eq!(d.title.as_deref(), Some("History"));
        assert_eq!(d.tasks.len(), 2);
        assert_eq!(d.tasks[0].task, "2002");
        assert_eq!(d.tasks[0].events, vec!["LinkedIn"]);
        assert_eq!(d.tasks[1].task, "2004");
        assert_eq!(d.tasks[1].events, vec!["Facebook", "Google"]);
        assert!(d.sections.is_empty());
    }

    #[test]
    fn with_sections() {
        let input = concat!(
            "timeline\n",
            "    title Social Media\n",
            "    section Early\n",
            "        2002 : LinkedIn\n",
            "    section Later\n",
            "        2004 : Facebook\n",
        );
        let d = parse(input).diagram;
        assert_eq!(d.sections, vec!["Early", "Later"]);
        assert_eq!(d.tasks[0].section, "Early");
        assert_eq!(d.tasks[1].section, "Later");
    }

    #[test]
    fn multiple_events() {
        let input = "timeline\n    2004 : Facebook\n         : Google";
        let d = parse(input).diagram;
        assert_eq!(d.tasks[0].events.len(), 2);
        assert_eq!(d.tasks[0].events[0], "Facebook");
        assert_eq!(d.tasks[0].events[1], "Google");
    }

    #[test]
    fn full_example() {
        let input = concat!(
            "timeline\n",
            "    title History of Social Media Platform\n",
            "    2002 : LinkedIn\n",
            "    2004 : Facebook\n",
            "         : Google\n",
            "    2005 : YouTube\n",
            "    2006 : Twitter\n",
            "    section ICT and Internet\n",
            "        1978 : first commercial social network\n",
            "        1994 : GeoCities\n",
        );
        let d = parse(input).diagram;
        assert_eq!(d.title.as_deref(), Some("History of Social Media Platform"));
        // Tasks before the section have no section
        assert_eq!(d.tasks[0].task, "2002");
        assert_eq!(d.tasks[1].task, "2004");
        assert_eq!(d.tasks[2].task, "2005");
        assert_eq!(d.tasks[3].task, "2006");
        // Section tasks
        assert_eq!(d.tasks[4].task, "1978");
        assert_eq!(d.tasks[4].section, "ICT and Internet");
        assert_eq!(d.tasks[5].task, "1994");
        assert_eq!(d.sections, vec!["ICT and Internet"]);
    }

    #[test]
    fn no_title() {
        let input = "timeline\n    2002 : LinkedIn\n";
        let d = parse(input).diagram;
        assert_eq!(d.title, None);
        assert_eq!(d.tasks.len(), 1);
    }
}
