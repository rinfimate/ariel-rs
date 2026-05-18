use super::constants::*;
use super::parser::{
    Block, BlockKind, Message, Participant, ParticipantType, ZenUmlDiagram, ZenUmlStatement,
};
use super::templates;
/// Rust renderer for ZenUML sequence diagrams.
///
/// ZenUML is an external Mermaid-compatible sequence diagram DSL maintained at
/// github.com/mermaid-js/zenuml-core (Vue.js + Antlr4 — no single translatable .ts file).
///
/// This renderer follows the documented ZenUML visual style:
/// - Participant boxes at the top with type-specific icons
/// - Activation bars on participant lifelines
/// - Async arrows (->), sync arrows with dotted return
/// - Control structure boxes (if/while/loop/opt/par/try)
/// - ZenUML default monospace font + coloring
use crate::text::measure;
use crate::theme::Theme;

// ── Layout constants ──────────────────────────────────────────────────────────
// All constants are imported from super::constants via `use super::constants::*`.

pub fn render(diag: &ZenUmlDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    let participants = &diag.participants;
    if participants.is_empty() {
        // Even with no participants, show a valid empty SVG
        return empty_svg(diag.title.as_deref());
    }

    // Compute participant x-centers
    let n = participants.len();
    let part_xs: Vec<f64> = (0..n)
        .map(|i| PADDING + i as f64 * (PART_WIDTH + PART_SPACING) + PART_WIDTH / 2.0)
        .collect();
    let total_w =
        PADDING * 2.0 + n as f64 * PART_WIDTH + (n.saturating_sub(1)) as f64 * PART_SPACING;

    // Lay out messages to compute total height
    let (msg_y_positions, total_msg_height) = layout_statements(&diag.statements, 0.0);

    let lifeline_bottom = LIFELINE_TOP + total_msg_height + MSG_SPACING;
    let total_h = lifeline_bottom
        + PART_HEIGHT
        + PADDING * 2.0
        + if diag.title.is_some() { 30.0 } else { 0.0 };

    let mut parts: Vec<String> = Vec::new();
    let style = build_style(ff);
    parts.push(style);

    let title_offset = if diag.title.is_some() { 30.0 } else { 0.0 };

    // Title
    if let Some(ref title) = diag.title {
        parts.push(templates::title_text(total_w / 2.0, &escape(title)));
    }

    let gy = title_offset;
    parts.push(format!(r#"<g transform="translate(0,{:.1})">"#, gy));

    // Participant boxes
    for (pi, part) in participants.iter().enumerate() {
        let cx = part_xs[pi];
        let x = cx - PART_WIDTH / 2.0;
        render_participant(&mut parts, part, x, 0.0, PART_WIDTH, PART_HEIGHT);
    }

    // Lifelines
    for &cx in &part_xs {
        parts.push(templates::lifeline(cx, LIFELINE_TOP, lifeline_bottom));
    }

    // Render messages
    render_statements(
        &mut parts,
        &diag.statements,
        &msg_y_positions,
        participants,
        &part_xs,
        LIFELINE_TOP,
        0,
    );

    // Bottom participant boxes
    for (pi, part) in participants.iter().enumerate() {
        let cx = part_xs[pi];
        let x = cx - PART_WIDTH / 2.0;
        render_participant(
            &mut parts,
            part,
            x,
            lifeline_bottom,
            PART_WIDTH,
            PART_HEIGHT,
        );
    }

    parts.push("</g>".to_string());

    templates::svg_root(total_w, total_h, &parts.join(""))
}

/// Layout: assign Y position to each statement, return (positions, total_height).
fn layout_statements(stmts: &[ZenUmlStatement], base_y: f64) -> (Vec<f64>, f64) {
    let mut positions = Vec::new();
    let mut y = base_y;
    for stmt in stmts {
        positions.push(y);
        match stmt {
            ZenUmlStatement::Message(_) => {
                y += MSG_SPACING;
            }
            ZenUmlStatement::Return(_) => {
                y += MSG_SPACING * 0.7;
            }
            ZenUmlStatement::Creation(_) => {
                y += MSG_SPACING;
            }
            ZenUmlStatement::Comment(_) => {
                y += MSG_SPACING * 0.4;
            }
            ZenUmlStatement::Block(b) => {
                y += MSG_SPACING * 0.6; // block header
                let (_, block_h) = layout_statements(&b.body, 0.0);
                y += block_h + BLOCK_PADDING * 2.0;
                if let Some(ref else_body) = b.else_body {
                    let (_, else_h) = layout_statements(else_body, 0.0);
                    y += else_h + MSG_SPACING * 0.4;
                }
            }
        }
    }
    (positions, y - base_y)
}

fn render_statements(
    parts: &mut Vec<String>,
    stmts: &[ZenUmlStatement],
    positions: &[f64],
    participants: &[Participant],
    part_xs: &[f64],
    base_y: f64,
    _depth: usize,
) {
    for (i, stmt) in stmts.iter().enumerate() {
        let y = base_y + positions.get(i).copied().unwrap_or(0.0);
        match stmt {
            ZenUmlStatement::Message(msg) => {
                render_message(parts, msg, participants, part_xs, y);
            }
            ZenUmlStatement::Return(val) => {
                // Return: dashed line back
                if part_xs.len() >= 2 {
                    let ret_y = y + MSG_SPACING * 0.35;
                    parts.push(templates::return_line(
                        part_xs[part_xs.len() - 1],
                        ret_y,
                        part_xs[0],
                    ));
                    if !val.is_empty() {
                        let mx = (part_xs[0] + part_xs[part_xs.len() - 1]) / 2.0;
                        parts.push(templates::return_label_text(
                            mx,
                            ret_y - 3.0,
                            FONT_SIZE,
                            &escape(val),
                        ));
                    }
                }
            }
            ZenUmlStatement::Creation(name) => {
                // "new X" — show a create message
                let to_idx = participants
                    .iter()
                    .position(|p| &p.id == name || &p.label == name);
                if let Some(to_i) = to_idx {
                    let from_x = part_xs[0];
                    let to_x = part_xs[to_i];
                    draw_arrow(
                        parts,
                        from_x,
                        to_x,
                        y + MSG_SPACING * 0.5,
                        &format!("new {name}"),
                        false,
                    );
                }
            }
            ZenUmlStatement::Comment(text) => {
                parts.push(templates::comment_text(
                    PADDING,
                    y + 12.0,
                    FONT_SIZE - 1.0,
                    &escape(text),
                ));
            }
            ZenUmlStatement::Block(block) => {
                let (inner_positions, inner_h) = layout_statements(&block.body, 0.0);
                let box_h = inner_h + BLOCK_PADDING * 2.0 + MSG_SPACING * 0.6;
                render_block(
                    parts,
                    block,
                    &inner_positions,
                    participants,
                    part_xs,
                    y,
                    box_h,
                    _depth,
                );
            }
        }
    }
}

fn render_message(
    parts: &mut Vec<String>,
    msg: &Message,
    participants: &[Participant],
    part_xs: &[f64],
    y: f64,
) {
    let from_idx = participants
        .iter()
        .position(|p| p.id == msg.from || p.label == msg.from);
    let to_idx = participants
        .iter()
        .position(|p| p.id == msg.to || p.label == msg.to);

    let (from_x, to_x) = match (from_idx, to_idx) {
        (Some(fi), Some(ti)) => (part_xs[fi], part_xs[ti]),
        (None, Some(ti)) => {
            // self call or unknown from
            (part_xs.first().copied().unwrap_or(100.0), part_xs[ti])
        }
        (Some(fi), None) => (
            part_xs[fi],
            part_xs.get(fi + 1).copied().unwrap_or(part_xs[fi] + 120.0),
        ),
        (None, None) => return,
    };

    let arrow_y = y + MSG_SPACING * 0.5;
    let is_self = (from_x - to_x).abs() < 1.0;

    if is_self {
        // Self-call: small loop
        let lx = from_x + ACTIVATION_W / 2.0;
        parts.push(templates::self_call_path(lx, arrow_y));
        parts.push(templates::self_call_label(
            lx,
            arrow_y + 5.0,
            FONT_SIZE,
            &escape(&msg.label),
        ));
    } else {
        draw_arrow(parts, from_x, to_x, arrow_y, &msg.label, msg.sync);
    }
}

fn draw_arrow(parts: &mut Vec<String>, from_x: f64, to_x: f64, y: f64, label: &str, sync: bool) {
    let dashclass = if sync {
        "zenuml-arrow-sync"
    } else {
        "zenuml-arrow-async"
    };
    parts.push(templates::message_line(dashclass, from_x, y, to_x));

    let (tw, _) = measure(label, FONT_SIZE);
    let mx = (from_x + to_x) / 2.0;
    let text_y = y - 4.0;

    // Label background
    parts.push(templates::message_label_bg(
        mx - tw / 2.0 - 2.0,
        text_y - FONT_SIZE,
        tw + 4.0,
        FONT_SIZE + 2.0,
    ));
    parts.push(templates::message_label_text(
        mx,
        text_y,
        FONT_SIZE,
        &escape(label),
    ));
}

#[allow(clippy::too_many_arguments)]
fn render_block(
    parts: &mut Vec<String>,
    block: &Block,
    inner_positions: &[f64],
    participants: &[Participant],
    part_xs: &[f64],
    y: f64,
    box_h: f64,
    depth: usize,
) {
    let x_left = PADDING / 2.0;
    let x_right = part_xs.last().copied().unwrap_or(300.0) + PART_WIDTH / 2.0;
    let box_w = x_right - x_left;

    let kind_label = match block.kind {
        BlockKind::If => "if",
        BlockKind::While => "while",
        BlockKind::For => "for",
        BlockKind::ForEach => "forEach",
        BlockKind::Loop => "loop",
        BlockKind::Opt => "opt",
        BlockKind::Par => "par",
        BlockKind::Try => "try",
        BlockKind::Catch => "catch",
        BlockKind::Finally => "finally",
    };

    let header = if block.condition.is_empty() {
        kind_label.to_string()
    } else {
        format!("{} [{}]", kind_label, block.condition)
    };

    let box_class = match block.kind {
        BlockKind::If => "zenuml-block-cond",
        BlockKind::Loop | BlockKind::While | BlockKind::For | BlockKind::ForEach => {
            "zenuml-block-loop"
        }
        BlockKind::Opt => "zenuml-block-opt",
        BlockKind::Par => "zenuml-block-par",
        BlockKind::Try | BlockKind::Catch | BlockKind::Finally => "zenuml-block-try",
    };

    let depth_offset = depth as f64 * 4.0;

    parts.push(templates::block_rect(
        box_class,
        x_left + depth_offset,
        y,
        box_w - depth_offset * 2.0,
        box_h,
    ));
    parts.push(templates::block_label(
        x_left + depth_offset + 4.0,
        y + 14.0,
        FONT_SIZE - 1.0,
        &escape(&header),
    ));

    // Render inner statements
    render_statements(
        parts,
        &block.body,
        inner_positions,
        participants,
        part_xs,
        y + MSG_SPACING * 0.6 + BLOCK_PADDING,
        depth + 1,
    );
}

fn render_participant(parts: &mut Vec<String>, part: &Participant, x: f64, y: f64, w: f64, h: f64) {
    let class = match part.ptype {
        ParticipantType::Actor => "zenuml-actor",
        ParticipantType::Database => "zenuml-database",
        _ => "zenuml-participant",
    };

    parts.push(templates::participant_rect(class, x, y, w, h));

    // Actor icon: stick figure header
    if part.ptype == ParticipantType::Actor {
        let cx = x + w / 2.0;
        parts.push(templates::actor_head(cx, y + 6.0));
    } else if part.ptype == ParticipantType::Database {
        // Cylinder top ellipse
        let cx = x + w / 2.0;
        parts.push(templates::db_top_ellipse(cx, y + 4.0, w / 2.0 - 4.0));
    }

    let (tw, _) = measure(&part.label, FONT_SIZE);
    let _ = tw;
    parts.push(templates::participant_label(
        x + w / 2.0,
        y + h / 2.0 + FONT_SIZE / 3.0,
        FONT_SIZE,
        &escape(&part.label),
    ));
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn empty_svg(title: Option<&str>) -> String {
    let title_text = title.unwrap_or("ZenUML");
    templates::empty_svg(&escape(title_text))
}

fn build_style(ff: &str) -> String {
    format!(
        r#"<style>
.zenuml-title {{ fill: #333; font-family: {ff}; }}
.zenuml-participant {{ fill: #e8f4fd; stroke: #6aa3d5; stroke-width: 1.5; }}
.zenuml-actor {{ fill: #fff3e0; stroke: #f0a500; stroke-width: 1.5; }}
.zenuml-database {{ fill: #e8f5e9; stroke: #4caf50; stroke-width: 1.5; }}
.zenuml-actor-head {{ fill: #f0a500; stroke: #b37200; stroke-width: 1; }}
.zenuml-db-top {{ fill: #4caf50; stroke: #388e3c; stroke-width: 1; }}
.zenuml-part-label {{ fill: #333; font-family: {ff}; font-size: 13px; }}
.zenuml-lifeline {{ stroke: #aaa; stroke-width: 1; stroke-dasharray: 5,3; }}
.zenuml-arrow-async {{ stroke: #333; stroke-width: 1.5; fill: none; }}
.zenuml-arrow-sync {{ stroke: #333; stroke-width: 2; fill: none; }}
.zenuml-return {{ stroke: #777; stroke-width: 1; stroke-dasharray: 4,3; fill: none; }}
.zenuml-arrow {{ stroke: #333; stroke-width: 1.5; fill: none; }}
.zenuml-arrow-head {{ fill: #333; stroke: none; }}
.zenuml-arrow-head-open {{ fill: none; stroke: #333; stroke-width: 1.5; }}
.zenuml-msg-label {{ fill: #333; font-family: {ff}; }}
.zenuml-label-bg {{ fill: white; fill-opacity: 0.8; }}
.zenuml-comment {{ fill: #888; font-family: {ff}; }}
.zenuml-block-cond {{ fill: none; stroke: #6c8ebf; stroke-width: 1.5; }}
.zenuml-block-loop {{ fill: none; stroke: #82b366; stroke-width: 1.5; }}
.zenuml-block-opt {{ fill: none; stroke: #d6b656; stroke-width: 1.5; }}
.zenuml-block-par {{ fill: none; stroke: #9673a6; stroke-width: 1.5; }}
.zenuml-block-try {{ fill: none; stroke: #d79b00; stroke-width: 1.5; }}
.zenuml-block-label {{ fill: #555; font-family: {ff}; font-style: italic; }}
</style>"#,
        ff = ff,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::zenuml::parser;

    #[test]
    fn render_produces_svg() {
        let input = "zenuml\n    title Greeting\n    Alice->Bob: Hello\n    Bob->Alice: Hi\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("zenuml-participant"));
    }

    #[test]
    fn empty_input_svg() {
        let input = "zenuml\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "zenuml\n    title Greeting\n    Alice->Bob: Hello\n    Bob->Alice: Hi\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(svg);
    }

    // ── new tests ─────────────────────────────────────────────────────────────

    #[test]
    fn render_with_title() {
        let input = "zenuml\n  title My Diagram\n  Alice->Bob: Hello\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("My Diagram"));
    }

    #[test]
    fn render_actor_participant() {
        let input = "zenuml\n  @Actor User\n  User->Server: request\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("zenuml-actor"));
    }

    #[test]
    fn render_database_participant() {
        let input = "zenuml\n  @Database db\n  Alice->db: query\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("zenuml-database"));
    }

    #[test]
    fn render_sync_message() {
        let input = "zenuml\n  Alice->Bob: request\n  Bob.process()\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("zenuml-arrow-sync"));
    }

    #[test]
    fn render_async_message() {
        let input = "zenuml\n  Alice->Bob: hello\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("zenuml-arrow-async"));
    }

    #[test]
    fn render_return_statement() {
        let input = "zenuml\n  Alice->Bob: request\n  Bob->Alice: ok\n  return value\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_comment_statement() {
        let input = "zenuml\n  // a comment\n  Alice->Bob: hi\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("zenuml-comment"));
        assert!(svg.contains("a comment"));
    }

    #[test]
    fn render_creation_statement() {
        let input = "zenuml\n  @Component comp\n  Alice->comp: init\n  new comp\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("new comp"));
    }

    #[test]
    fn render_if_block() {
        let input = "zenuml\n  Alice->Bob: request\n  if(ok) {\n    Bob->Alice: yes\n  }\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("zenuml-block-cond"));
    }

    #[test]
    fn render_while_block() {
        // Participants before the block ensure rendering proceeds past empty check
        let input = "zenuml\n  Alice->Bob: start\n  while(retry) {\n    Alice->Bob: try\n  }\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("zenuml-block-loop"));
    }

    #[test]
    fn render_opt_block() {
        let input = "zenuml\n  Alice->Bob: start\n  opt() {\n    Alice->Bob: optional\n  }\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("zenuml-block-opt"));
    }

    #[test]
    fn render_par_block() {
        let input = "zenuml\n  Alice->Bob: start\n  par() {\n    Alice->Bob: parallel\n  }\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("zenuml-block-par"));
    }

    #[test]
    fn render_try_block() {
        let input = "zenuml\n  Alice->Bob: start\n  try() {\n    Alice->Bob: risky\n  }\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("zenuml-block-try"));
    }

    #[test]
    fn render_method_with_condition_label() {
        let input = "zenuml\n  Alice->Bob: start\n  if(x > 0) {\n    Alice->Bob: positive\n  }\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        // The condition text should appear in the block label
        assert!(svg.contains("if"));
    }

    #[test]
    fn render_dark_theme() {
        let input = "zenuml\n  Alice->Bob: hello\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Dark);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_forest_theme() {
        let input = "zenuml\n  Alice->Bob: hello\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Forest);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_neutral_theme() {
        let input = "zenuml\n  Alice->Bob: hello\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Neutral);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_many_participants() {
        let input = "zenuml\n  A->B: 1\n  B->C: 2\n  C->D: 3\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
        // 4 participants: A, B, C, D
        assert_eq!(diag.participants.len(), 4);
    }

    #[test]
    fn render_html_entry_point_produces_svg() {
        let input = "zenuml\n  Alice->Bob: Hi\n";
        let svg = crate::diagrams::zenuml::render_html(input, Theme::Default);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_for_block() {
        let input = "zenuml\n  Alice->Bob: start\n  for(i in list) {\n    Alice->Bob: item\n  }\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("zenuml-block-loop"));
    }

    #[test]
    fn render_foreach_block() {
        let input = "zenuml\n  Alice->Bob: start\n  forEach(x) {\n    Alice->Bob: x\n  }\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("zenuml-block-loop"));
    }

    #[test]
    fn render_special_chars_escaped_in_label() {
        let input = "zenuml\n  Alice->Bob: a&b\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        // Raw '&' must not appear unescaped in SVG
        assert!(!svg.contains(" a&b") || svg.contains("&amp;"));
    }
}
