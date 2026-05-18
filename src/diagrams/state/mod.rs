pub(crate) mod constants;
pub mod parser;
pub mod renderer;
pub(crate) mod templates;

use crate::error::ParseError;
use crate::theme::Theme;

pub(crate) fn validate(diagram: &parser::StateDiagram) -> Vec<ParseError> {
    let mut errors = Vec::new();

    // Collect all declared state IDs (recursively through composite states)
    let mut known: std::collections::HashSet<String> = std::collections::HashSet::new();
    collect_state_ids(&diagram.stmts, &mut known);

    for stmt in &diagram.stmts {
        check_transition(stmt, &known, &mut errors);
    }

    errors
}

fn collect_state_ids(stmts: &[parser::StateStmt], known: &mut std::collections::HashSet<String>) {
    for stmt in stmts {
        if let parser::StateStmt::State(node) = stmt {
            known.insert(node.id.clone());
            collect_state_ids(&node.doc, known);
        }
    }
}

fn check_transition(
    stmt: &parser::StateStmt,
    known: &std::collections::HashSet<String>,
    errors: &mut Vec<ParseError>,
) {
    match stmt {
        parser::StateStmt::Transition(t) => {
            // [*] is the pseudo-state — always valid
            for id in [&t.from, &t.to] {
                if id != "[*]" && !known.contains(id.as_str()) {
                    errors.push(ParseError::new(format!(
                        "Transition references unknown state '{}'",
                        id
                    )));
                }
            }
        }
        parser::StateStmt::State(node) => {
            for inner in &node.doc {
                check_transition(inner, known, errors);
            }
        }
        parser::StateStmt::Direction(_) => {}
    }
}

/// Render with <foreignObject> HTML labels (matches Mermaid reference output).
pub fn render_html(input: &str, theme: Theme) -> String {
    let mut result = parser::parse(input);
    result.errors.extend(validate(&result.diagram));
    renderer::render(&result.diagram, theme, true)
}
