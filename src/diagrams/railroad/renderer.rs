use super::constants::*;
use super::parser::{AstNode, RailroadDiagram, RailroadRule};
use super::templates;
/// Faithful Rust port of Mermaid's railroadRenderer.ts.
///
/// Key algorithm details (from TypeScript source):
/// - PathBuilder utility for SVG path construction
/// - RailroadRenderer class with measureText, renderTerminal, renderNonTerminal,
///   renderSequence, renderChoice, renderOptional, renderRepetition, renderSpecial
/// - Layout parameters from DEFAULT_RAILROAD_CONFIG:
///   padding=10, verticalSeparation=8, horizontalSeparation=10, arcRadius=10
///   fontSize=14, fontFamily=monospace, markerRadius=5
/// - Terminal: rounded rect (rx=10), fill=#FFFFC0
/// - NonTerminal: plain rect, fill=#FFFFFF
/// - Sequence: horizontal concatenation with connecting lines
/// - Choice: vertical alternatives with arc connections to center line
/// - Optional: bypass path above element
/// - Repetition: loop-back path below (+ top bypass for *)
/// - RenderResult has: width, height, up (height above center), down (height below center)
use crate::text::measure;
use crate::theme::Theme;

/// Render dimensions (matching RenderResult interface)
#[derive(Debug, Clone, Copy)]
struct Dims {
    width: f64,
    height: f64,
    up: f64,   // height above center line
    down: f64, // height below center line
}

impl Dims {
    fn new(width: f64, up: f64, down: f64) -> Self {
        Dims {
            width,
            height: up + down,
            up,
            down,
        }
    }
}

/// Collected SVG elements + dimensions
struct RenderResult {
    elements: Vec<String>,
    dims: Dims,
}

pub fn render(diag: &RailroadDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    if diag.rules.is_empty() {
        return templates::empty_svg(&escape(diag.title.as_deref().unwrap_or("Railroad")));
    }

    let mut all_elements: Vec<String> = Vec::new();
    all_elements.push(templates::style_block(ff));

    let title_h = if diag.title.is_some() { 25.0 } else { 0.0 };

    if let Some(ref title) = diag.title {
        all_elements.push(templates::title_text(200.0, &escape(title)));
    }

    let mut y = PADDING + title_h;
    let mut max_width = 0.0_f64;

    let mut rule_elements: Vec<(String, f64, f64)> = Vec::new(); // (svg_content, y, height)

    for rule in &diag.rules {
        let result = render_rule(rule, y, ff);
        max_width = max_width.max(result.dims.width);
        let h = result.dims.height;
        rule_elements.push((result.elements.join(""), y, h));
        y += h + VERTICAL_SEP;
    }

    let total_w = max_width + PADDING * 2.0;
    let total_h = y + PADDING;

    // Update title centering now that we know total_w
    // (We'll just use total_w/2 for title x)
    if !rule_elements.is_empty() {
        all_elements.extend(rule_elements.iter().map(|(c, _, _)| c.clone()));
    }

    format!(
        "{}{}{}{}",
        templates::svg_root(total_w, total_h),
        all_elements.join(""),
        rule_elements
            .iter()
            .map(|(c, _, _)| c.as_str())
            .collect::<Vec<_>>()
            .join(""),
        "</svg>"
    )
}

fn render_rule(rule: &RailroadRule, y: f64, ff: &str) -> RenderResult {
    let rule_name = format!("{} =", rule.name);
    let (name_w, _) = measure(&rule_name, FONT_SIZE);
    let name_display_w = name_w + 20.0;
    let definition_x = name_display_w + 20.0;
    let baseline_y_local = PADDING.max(20.0);

    let def_result = render_expression(&rule.definition, ff);
    let baseline_y = baseline_y_local.max(def_result.dims.up);
    let definition_y = baseline_y - def_result.dims.up;

    let total_w = definition_x + def_result.dims.width + MARKER_RADIUS * 2.0 + 20.0;
    let total_h = (baseline_y + def_result.dims.down + PADDING * 2.0).max(40.0);

    let mut elems: Vec<String> = Vec::new();

    let abs_y = y;

    // Rule group
    elems.push(format!(
        r#"<g class="railroad-rule" transform="translate(0,{abs_y:.1})">"#,
    ));

    // Definition group (positioned)
    elems.push(format!(
        r#"<g transform="translate({definition_x:.1},{definition_y:.1})">"#,
    ));
    elems.extend(def_result.elements.clone());
    elems.push("</g>".to_string());

    // Rule name text
    elems.push(templates::rule_name_text(
        baseline_y,
        FONT_SIZE,
        RULE_NAME_COLOR,
        ff,
        &escape(&rule_name),
    ));

    // Start marker circle
    elems.push(templates::start_marker(
        name_display_w,
        baseline_y,
        MARKER_RADIUS,
        LINE_COLOR,
        LINE_COLOR,
        STROKE_WIDTH,
    ));

    // End marker circle (double circle)
    let end_cx = definition_x + def_result.dims.width + 10.0;
    elems.push(templates::end_marker_outer(
        end_cx,
        baseline_y,
        MARKER_RADIUS,
        LINE_COLOR,
        LINE_COLOR,
        STROKE_WIDTH,
    ));
    elems.push(templates::end_marker_inner(
        end_cx,
        baseline_y,
        MARKER_RADIUS - 2.0,
        LINE_COLOR,
        STROKE_WIDTH,
    ));

    // Line from start marker to definition
    elems.push(line(
        name_display_w + MARKER_RADIUS,
        baseline_y,
        definition_x,
        baseline_y,
    ));

    // Line from definition to end marker
    elems.push(line(
        definition_x + def_result.dims.width,
        baseline_y,
        end_cx - MARKER_RADIUS,
        baseline_y,
    ));

    // Comment
    if let Some(ref cmt) = rule.comment {
        elems.push(templates::comment_text(
            total_h - 4.0,
            FONT_SIZE - 2.0,
            ff,
            &escape(cmt),
        ));
    }

    elems.push("</g>".to_string());

    RenderResult {
        elements: elems,
        dims: Dims::new(total_w, total_h / 2.0, total_h / 2.0),
    }
}

/// Render an expression recursively (matches RailroadRenderer class methods).
fn render_expression(node: &AstNode, ff: &str) -> RenderResult {
    match node {
        AstNode::Terminal(val) => render_terminal(val, ff),
        AstNode::NonTerminal(name) => render_nonterminal(name, ff),
        AstNode::Sequence(elements) => render_sequence(elements, ff),
        AstNode::Choice(alternatives) => render_choice(alternatives, ff),
        AstNode::Optional(element) => render_optional(element, ff),
        AstNode::Repetition { element, min } => render_repetition(element, *min, ff),
        AstNode::Special(text) => render_special(text, ff),
    }
}

/// Render terminal (rounded rectangle, yellow fill) — railroadRenderer.ts renderTerminal()
fn render_terminal(value: &str, ff: &str) -> RenderResult {
    let (tw, th) = measure(value, FONT_SIZE);
    let width = tw + PADDING * 2.0;
    let height = th + PADDING * 2.0;
    let up = height / 2.0;
    let down = height / 2.0;

    let elems = vec![templates::terminal_node(
        width,
        height,
        TERMINAL_FILL,
        TERMINAL_STROKE,
        STROKE_WIDTH,
        width / 2.0,
        height / 2.0,
        FONT_SIZE,
        ff,
        &escape(value),
    )];

    RenderResult {
        elements: elems,
        dims: Dims::new(width, up, down),
    }
}

/// Render non-terminal (plain rectangle, white fill) — railroadRenderer.ts renderNonTerminal()
fn render_nonterminal(name: &str, ff: &str) -> RenderResult {
    let (tw, th) = measure(name, FONT_SIZE);
    let width = tw + PADDING * 2.0;
    let height = th + PADDING * 2.0;
    let up = height / 2.0;
    let down = height / 2.0;

    let elems = vec![templates::nonterminal_node(
        width,
        height,
        NONTERMINAL_FILL,
        TERMINAL_STROKE,
        STROKE_WIDTH,
        width / 2.0,
        height / 2.0,
        FONT_SIZE,
        ff,
        &escape(name),
    )];

    RenderResult {
        elements: elems,
        dims: Dims::new(width, up, down),
    }
}

/// Render sequence — railroadRenderer.ts renderSequence()
fn render_sequence(elements: &[AstNode], ff: &str) -> RenderResult {
    if elements.is_empty() {
        return RenderResult {
            elements: vec![],
            dims: Dims::new(0.0, 0.0, 0.0),
        };
    }

    let rendered: Vec<RenderResult> = elements.iter().map(|e| render_expression(e, ff)).collect();

    let max_up = rendered.iter().map(|r| r.dims.up).fold(0.0_f64, f64::max);
    let max_down = rendered.iter().map(|r| r.dims.down).fold(0.0_f64, f64::max);
    let total_w = rendered.iter().map(|r| r.dims.width).sum::<f64>()
        + (rendered.len().saturating_sub(1)) as f64 * HORIZONTAL_SEP;

    let mut elems: Vec<String> = Vec::new();
    elems.push(r#"<g class="railroad-sequence">"#.to_string());

    let mut x = 0.0_f64;
    for (i, r) in rendered.iter().enumerate() {
        let y_offset = max_up - r.dims.up;
        elems.push(format!(
            r#"<g transform="translate({x:.1},{y_offset:.1})">"#,
        ));
        elems.extend(r.elements.clone());
        elems.push("</g>".to_string());

        if i < rendered.len() - 1 {
            // Connecting line
            let lx1 = x + r.dims.width;
            let lx2 = lx1 + HORIZONTAL_SEP;
            elems.push(line(lx1, max_up, lx2, max_up));
        }
        x += r.dims.width + HORIZONTAL_SEP;
    }

    elems.push("</g>".to_string());

    RenderResult {
        elements: elems,
        dims: Dims::new(total_w, max_up, max_down),
    }
}

/// Render choice — railroadRenderer.ts renderChoice()
fn render_choice(alternatives: &[AstNode], ff: &str) -> RenderResult {
    if alternatives.is_empty() {
        return RenderResult {
            elements: vec![],
            dims: Dims::new(0.0, 0.0, 0.0),
        };
    }

    let rendered: Vec<RenderResult> = alternatives
        .iter()
        .map(|e| render_expression(e, ff))
        .collect();

    let max_w = rendered
        .iter()
        .map(|r| r.dims.width)
        .fold(0.0_f64, f64::max);
    let total_h = rendered.iter().map(|r| r.dims.height).sum::<f64>()
        + (rendered.len().saturating_sub(1)) as f64 * VERTICAL_SEP;

    let arc_w = ARC_RADIUS * 4.0;
    let total_w = max_w + arc_w;
    let center_y = total_h / 2.0;

    let mut elems: Vec<String> = Vec::new();
    elems.push(r#"<g class="railroad-choice">"#.to_string());

    let mut y = 0.0_f64;
    for r in &rendered {
        let elem_center_y = y + r.dims.up;
        let elem_x = ARC_RADIUS * 2.0 + (max_w - r.dims.width) / 2.0;

        elems.push(format!(r#"<g transform="translate({elem_x:.1},{y:.1})">"#,));
        elems.extend(r.elements.clone());
        elems.push("</g>".to_string());

        // Left arc from center_y to elem_center_y
        let left_path = choice_arc(0.0, center_y, elem_x, elem_center_y, true);
        elems.push(templates::path_el(&left_path, LINE_COLOR, STROKE_WIDTH));

        // Right arc from elem_center_y to center_y
        let right_path = choice_arc(
            elem_x + r.dims.width,
            elem_center_y,
            total_w,
            center_y,
            false,
        );
        elems.push(templates::path_el(&right_path, LINE_COLOR, STROKE_WIDTH));

        y += r.dims.height + VERTICAL_SEP;
    }

    elems.push("</g>".to_string());

    RenderResult {
        elements: elems,
        dims: Dims::new(total_w, center_y, total_h - center_y),
    }
}

/// Build arc path for choice connections.
fn choice_arc(x1: f64, y1: f64, x2: f64, y2: f64, is_left: bool) -> String {
    if (y1 - y2).abs() < 0.1 {
        return format!("M {x1:.1} {y1:.1} L {x2:.1} {y2:.1}");
    }
    let r = ARC_RADIUS;
    let going_down = y2 > y1;

    if is_left {
        // Left side: from center to alternative
        if going_down {
            format!(
                "M {:.1} {:.1} A {r} {r} 0 0 1 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 0 {:.1} {:.1} L {:.1} {:.1}",
                x1, y1,
                x1 + r, y1 + r,
                x1 + r, y2 - r,
                x1 + r * 2.0, y2,
                x2, y2
            )
        } else {
            format!(
                "M {:.1} {:.1} A {r} {r} 0 0 0 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 1 {:.1} {:.1} L {:.1} {:.1}",
                x1, y1,
                x1 + r, y1 - r,
                x1 + r, y2 + r,
                x1 + r * 2.0, y2,
                x2, y2
            )
        }
    } else {
        // Right side: from alternative to center
        let rx = x2 - r * 2.0;
        if going_down {
            // alternative is above center
            format!(
                "M {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 0 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 1 {:.1} {:.1}",
                x1, y1,
                rx, y1,
                rx + r, y1 + r,
                rx + r, y2 - r,
                x2, y2,
            )
        } else {
            format!(
                "M {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 1 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 0 {:.1} {:.1}",
                x1, y1,
                rx, y1,
                rx + r, y1 - r,
                rx + r, y2 + r,
                x2, y2,
            )
        }
    }
}

/// Render optional — railroadRenderer.ts renderOptional()
fn render_optional(element: &AstNode, ff: &str) -> RenderResult {
    let inner = render_expression(element, ff);
    let r = ARC_RADIUS;
    let arc_h = r * 2.0;
    let total_w = inner.dims.width + r * 4.0;
    let total_h = inner.dims.height + arc_h;
    let elem_x = r * 2.0;
    let elem_y = arc_h;
    let center_y = elem_y + inner.dims.up;

    let mut elems: Vec<String> = Vec::new();
    elems.push(r#"<g class="railroad-optional">"#.to_string());

    elems.push(format!(
        r#"<g transform="translate({elem_x:.1},{elem_y:.1})">"#,
    ));
    elems.extend(inner.elements.clone());
    elems.push("</g>".to_string());

    // Lower through-path
    elems.push(line(0.0, center_y, elem_x, center_y));
    elems.push(line(elem_x + inner.dims.width, center_y, total_w, center_y));

    // Upper bypass
    let bypass = format!(
        "M {:.1} {:.1} A {r} {r} 0 0 0 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 1 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 1 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 0 {:.1} {:.1}",
        0.0, center_y,
        r, center_y - r,
        r, r,
        r * 2.0, 0.0,
        total_w - r * 2.0, 0.0,
        total_w - r, r,
        total_w - r, center_y - r,
        total_w, center_y,
    );
    elems.push(templates::path_el(&bypass, LINE_COLOR, STROKE_WIDTH));

    elems.push("</g>".to_string());

    RenderResult {
        elements: elems,
        dims: Dims::new(total_w, center_y, total_h - center_y),
    }
}

/// Render repetition — railroadRenderer.ts renderRepetition()
fn render_repetition(element: &AstNode, min: u32, ff: &str) -> RenderResult {
    let inner = render_expression(element, ff);
    let r = ARC_RADIUS;
    let arc_h = r * 2.0;
    let total_w = inner.dims.width + r * 4.0;
    let has_bypass = min == 0;
    let elem_y = if has_bypass { arc_h } else { 0.0 };
    let total_h = inner.dims.height + arc_h + if has_bypass { arc_h } else { 0.0 };
    let elem_x = r * 2.0;
    let center_y = elem_y + inner.dims.up;

    let mut elems: Vec<String> = Vec::new();
    elems.push(r#"<g class="railroad-repetition">"#.to_string());

    elems.push(format!(
        r#"<g transform="translate({elem_x:.1},{elem_y:.1})">"#,
    ));
    elems.extend(inner.elements.clone());
    elems.push("</g>".to_string());

    // Forward paths
    elems.push(line(0.0, center_y, elem_x, center_y));
    elems.push(line(elem_x + inner.dims.width, center_y, total_w, center_y));

    // Loop-back path (below)
    let loop_y = elem_y + inner.dims.height + r;
    let loop_path = format!(
        "M {:.1} {:.1} A {r} {r} 0 0 1 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 1 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 1 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 1 {:.1} {:.1}",
        elem_x + inner.dims.width, center_y,
        elem_x + inner.dims.width + r, center_y + r,
        elem_x + inner.dims.width + r, loop_y,
        elem_x + inner.dims.width, loop_y + r,
        r * 2.0, loop_y + r,
        r, loop_y,
        r, center_y + r,
        r * 2.0, center_y,
    );
    elems.push(templates::path_el(&loop_path, LINE_COLOR, STROKE_WIDTH));

    // Bypass (for *)
    if has_bypass {
        let bypass = format!(
            "M {:.1} {:.1} A {r} {r} 0 0 0 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 1 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 1 {:.1} {:.1} L {:.1} {:.1} A {r} {r} 0 0 0 {:.1} {:.1}",
            0.0, center_y,
            r, center_y - r,
            r, r,
            r * 2.0, 0.0,
            total_w - r * 2.0, 0.0,
            total_w - r, r,
            total_w - r, center_y - r,
            total_w, center_y,
        );
        elems.push(templates::path_el(&bypass, LINE_COLOR, STROKE_WIDTH));
    }

    elems.push("</g>".to_string());

    RenderResult {
        elements: elems,
        dims: Dims::new(total_w, center_y, total_h - center_y),
    }
}

/// Render special — railroadRenderer.ts renderSpecial()
fn render_special(text: &str, ff: &str) -> RenderResult {
    let label = format!("? {} ?", text);
    let (tw, th) = measure(&label, FONT_SIZE);
    let width = tw + PADDING * 2.0;
    let height = th + PADDING * 2.0;

    let elems = vec![templates::special_node(
        width,
        height,
        STROKE_WIDTH,
        width / 2.0,
        height / 2.0,
        FONT_SIZE,
        ff,
        &escape(&label),
    )];

    RenderResult {
        elements: elems,
        dims: Dims::new(width, height / 2.0, height / 2.0),
    }
}

fn line(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    templates::connector_line(x1, y1, x2, y2, LINE_COLOR, STROKE_WIDTH)
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::railroad::parser;

    #[test]
    fn render_produces_svg() {
        let input = "railroad\n    title Grammar\n    digit ::= \"0\" | \"1\" | \"2\"\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("railroad-terminal"));
    }

    #[test]
    fn empty_rules_renders() {
        let input = "railroad\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
    }

    #[test]
    #[ignore = "platform-specific float precision — run locally"]
    fn snapshot_default_theme() {
        let input = "railroad\n    title Grammar\n    digit ::= \"0\" | \"1\" | \"2\"\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(svg);
    }
}
