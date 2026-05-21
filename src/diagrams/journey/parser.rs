/// Parser for Mermaid User Journey diagram syntax.
///
/// Faithful port of journeyDb.ts.
///
/// Grammar:
///   journey
///       title <text>
///       section <name>
///       <task label>: <score>: <actor1>[, <actor2>, ...]
///
/// Example:
///   journey
///       title My working day
///       section Go to work
///         Make tea: 5: Me
///         Go upstairs: 3: Me, Cat
///       section Go home
///         Go downstairs: 5: Me
///         Sit down: 3: Me
/// A single task in the journey diagram.
#[derive(Debug, Clone)]
pub struct JourneyTask {
    /// Display label of the task
    pub task: String,
    /// Score (0–5, or any integer given in source)
    pub score: i32,
    /// Actors involved in this task
    pub people: Vec<String>,
    /// Section this task belongs to
    pub section: String,
    /// 0-based index of the section (for colour cycling)
    pub section_index: usize,
}

/// The full journey diagram (output of parsing).
#[derive(Debug, Default)]
pub struct JourneyDiagram {
    pub title: Option<String>,
    pub tasks: Vec<JourneyTask>,
    /// Ordered, deduplicated actor names (in order of first appearance)
    pub actors: Vec<String>,
    /// Section names in order
    pub sections: Vec<String>,
}

pub fn parse(input: &str) -> crate::error::ParseResult<JourneyDiagram> {
    let mut diag = JourneyDiagram::default();
    let mut current_section = String::new();
    let mut section_index: usize = 0;
    let mut header_seen = false;

    // Track actor order (de-duplicated)
    let mut actor_set: Vec<String> = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();

        // Skip blank lines and comments
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        // The first non-blank, non-comment line must be "journey"
        if !header_seen {
            if trimmed == "journey" || trimmed.starts_with("journey ") {
                header_seen = true;
            }
            continue;
        }

        // accTitle / accDescr — skip
        if trimmed.starts_with("accTitle") || trimmed.starts_with("accDescr") {
            continue;
        }

        // title directive
        if let Some(rest) = trimmed.strip_prefix("title") {
            let t = rest.trim();
            if !t.is_empty() {
                diag.title = Some(t.to_string());
            }
            continue;
        }

        // section directive
        if let Some(rest) = trimmed.strip_prefix("section") {
            let name = rest.trim().to_string();
            if !diag.sections.contains(&name) {
                diag.sections.push(name.clone());
            }
            if current_section != name {
                if !current_section.is_empty() {
                    section_index += 1;
                }
                current_section = name;
            }
            continue;
        }

        // Task line: must contain at least one colon.
        // Format: <label>: <score>: <actors>
        // OR:     <label>: <score>   (no actors)
        if let Some(first_colon) = trimmed.find(':') {
            let label = trimmed[..first_colon].trim().to_string();
            if label.is_empty() {
                continue;
            }

            let rest_after_label = trimmed[first_colon + 1..].trim();

            // Parse score and optional actors
            let (score, people) = if let Some(second_colon) = rest_after_label.find(':') {
                let score_str = rest_after_label[..second_colon].trim();
                let actors_str = rest_after_label[second_colon + 1..].trim();
                let score: i32 = score_str.parse().unwrap_or(0);
                let people: Vec<String> = actors_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                (score, people)
            } else {
                // Score only, no actors
                let score: i32 = rest_after_label.parse().unwrap_or(0);
                (score, vec![])
            };

            // Register actors
            for actor in &people {
                if !actor_set.contains(actor) {
                    actor_set.push(actor.clone());
                }
            }

            diag.tasks.push(JourneyTask {
                task: label,
                score,
                people,
                section: current_section.clone(),
                section_index,
            });
        }
    }

    // Mermaid sorts actors alphabetically for consistent legend ordering.
    actor_set.sort();
    diag.actors = actor_set;
    crate::error::ParseResult::ok(diag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        let input = "journey\n    title My working day\n    section Go to work\n      Make tea: 5: Me\n      Go upstairs: 3: Me\n      Do work: 1: Me, Cat\n    section Go home\n      Go downstairs: 5: Me\n      Sit down: 3: Me";
        let d = parse(input).diagram;
        assert_eq!(d.title.as_deref(), Some("My working day"));
        assert_eq!(d.sections, vec!["Go to work", "Go home"]);
        assert_eq!(d.tasks.len(), 5);
        assert_eq!(d.tasks[0].task, "Make tea");
        assert_eq!(d.tasks[0].score, 5);
        assert_eq!(d.tasks[0].people, vec!["Me"]);
        assert_eq!(d.tasks[2].people, vec!["Me", "Cat"]);
        assert_eq!(d.tasks[0].section, "Go to work");
        assert_eq!(d.tasks[3].section, "Go home");
        assert_eq!(d.tasks[3].section_index, 1);
        assert_eq!(d.actors, vec!["Cat", "Me"]); // alphabetically sorted
    }

    #[test]
    fn parse_no_actors() {
        let input = "journey\n    title Simple\n    section A\n      Task one: 3";
        let d = parse(input).diagram;
        assert_eq!(d.tasks[0].people.len(), 0);
        assert_eq!(d.tasks[0].score, 3);
    }

    #[test]
    fn parse_no_title() {
        let input = "journey\n    section S\n      Task: 5: X";
        let d = parse(input).diagram;
        assert_eq!(d.title, None);
        assert_eq!(d.tasks.len(), 1);
    }
}
