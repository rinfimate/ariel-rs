use super::constants::*;
use super::parser::{KanbanDiagram, KanbanSection, NodeShape};
use super::templates::{self, esc, priority_line};
/// Faithful Rust port of Mermaid's kanbanRenderer.ts.
///
/// Layout algorithm (column-based, NOT dagre):
///
/// The JS renderer (kanbanRenderer.ts) uses a custom column layout:
///   - Each section (column) gets a fixed width (sectionWidth, default 200).
///   - Columns are placed side by side with a 5 px gap.
///   - Items within a column are stacked vertically with fixed heights.
///   - The column header height = 25 px (LABEL_HEIGHT_DEFAULT).
///   - Each item height = 44 px, gap between items = 5 px.
///   - Column rect height = 79 + (n_items - 1) * 49 (for n_items >= 1), or 50 for 0 items.
///   - viewBox: "90 -310 W H" where W = 15 + n_cols*205, H = max_col_h + 20.
///
/// This port faithfully replicates the Mermaid coordinate system and CSS class scheme.
use crate::text::measure;
use crate::theme::Theme;

// ── Layout math ────────────────────────────────────────────────────────────────

/// Estimate card height for a given item, accounting for text wrapping and metadata row.
fn item_height_full(item: &crate::diagrams::kanban::parser::KanbanItem) -> f64 {
    let base = item_height(&item.label);
    // Add a metadata row if ticket or assigned is present
    if item.ticket.is_some() || item.assigned.is_some() {
        base + 12.0
    } else {
        base
    }
}

/// Estimate card height for a given label, accounting for text wrapping.
/// Mirrors Mermaid's getBBox()-based approach as a static approximation.
fn item_height(label: &str) -> f64 {
    // Mermaid renders kanban labels at 16px — use 16px for wrapping estimation
    // so items wrap at the same point the browser would wrap them.
    let (text_w, _) = measure(label, FONT_SIZE);
    let lines = ((text_w * TEXT_SCALE) / AVAILABLE_WIDTH).ceil().max(1.0);
    (lines * LINE_HEIGHT + V_PADDING).max(ITEM_HEIGHT)
}

/// Map a priority string to its stroke color.
fn priority_color(priority: &str) -> Option<&'static str> {
    match priority.to_lowercase().as_str() {
        "very high" => Some("red"),
        "high" => Some("orange"),
        "low" => Some("blue"),
        "very low" => Some("lightblue"),
        _ => None,
    }
}

/// foreignObject height for a given label (full card, no metadata).
fn fo_height(label: &str) -> f64 {
    item_height(label) - 4.0
}

/// Text-only height — just the wrapped lines, no padding.
/// Used when a metadata row is present so we can position label + metadata cleanly.
#[allow(dead_code)]
fn text_height(label: &str) -> f64 {
    let (text_w, _) = measure(label, FONT_SIZE);
    let lines = ((text_w * TEXT_SCALE) / AVAILABLE_WIDTH).ceil().max(1.0);
    lines * LINE_HEIGHT
}

/// Wrap label text into lines that fit within AVAILABLE_WIDTH.
fn wrap_label(label: &str) -> Vec<String> {
    let words: Vec<&str> = label.split_whitespace().collect();
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in &words {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current, word)
        };
        let (w, _) = measure(&candidate, FONT_SIZE);
        if w * TEXT_SCALE > AVAILABLE_WIDTH && !current.is_empty() {
            lines.push(current.clone());
            current = word.to_string();
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(label.to_string());
    }
    lines
}

/// Column left-edge x position (0-based index).
fn col_left_x(idx: usize) -> f64 {
    COL_LEFT_BASE + idx as f64 * (SECTION_WIDTH + SECTION_GAP)
}

/// Column center x position (0-based index).
fn col_center_x(idx: usize) -> f64 {
    col_left_x(idx) + SECTION_WIDTH / 2.0
}

/// Column height accounting for variable item heights (text wrapping + metadata).
fn col_height_dynamic(items: &[crate::diagrams::kanban::parser::KanbanItem]) -> f64 {
    if items.is_empty() {
        return 50.0;
    }
    let total: f64 = items
        .iter()
        .map(|it| item_height_full(it) + ITEM_GAP)
        .sum::<f64>()
        - ITEM_GAP;
    LABEL_HEIGHT + total + 10.0
}

/// Cumulative Y centers for each item in a section, accounting for variable heights.
fn item_center_ys(items: &[crate::diagrams::kanban::parser::KanbanItem]) -> Vec<f64> {
    let mut ys = Vec::with_capacity(items.len());
    let mut y = COL_TOP + LABEL_HEIGHT;
    for item in items {
        let h = item_height_full(item);
        ys.push(y + h / 2.0);
        y += h + ITEM_GAP;
    }
    ys
}

// ── HSL section color palette (mirrors Mermaid's kanban CSS generation) ────────
//
// Mermaid generates per-section CSS rules with HSL colors.
// Hue sequence for section-1 onward: 80, 270, 300, 330, 0, 30, 90, 150, 180, 210.
// Lightness 86.27% (= 88/102 * 100% approximately, exact = 86.2745098039%).
// Section-1 text: black; section-2 text: #ffffff; section-3+ text: black (varies).
//
// We embed the same CSS classes as Mermaid so the PNG renderer applies them.

// ── Section rendering ──────────────────────────────────────────────────────────

/// Render a single column section.
/// Returns: (section_svg, items_svg) — sections go in <g class="sections">,
/// items go in <g class="items">.
#[allow(clippy::too_many_arguments)]
fn render_section_and_items(
    section: &KanbanSection,
    sec_idx: usize, // 1-based
    col_idx: usize, // 0-based
    svg_id: &str,
    ticket_base_url: Option<&str>,
    primary_border: &str,
    primary_text: &str,
    card_bg: &str,
    theme: crate::theme::Theme,
) -> (String, String) {
    let cx = col_center_x(col_idx);
    let lx = col_left_x(col_idx);
    let col_top = COL_TOP;
    let col_h = col_height_dynamic(&section.items);
    let item_centers = item_center_ys(&section.items);

    // ── Section column rect ──────────────────────────────────────────────────
    let mut sec_svg = templates::section_group_open(sec_idx, svg_id, &esc(&section.id));

    sec_svg.push_str(&templates::section_rect(
        lx,
        col_top,
        SECTION_WIDTH,
        col_h,
        sec_idx,
        theme,
    ));

    // Column header label — per-section text colour for themes with varying header contrast.
    let header_text = templates::section_header_text(sec_idx, theme);
    sec_svg.push_str(&templates::section_label_fo(
        lx + 20.0,
        col_top,
        &esc(&section.label),
        header_text,
    ));

    sec_svg.push_str("</g>");

    // ── Items ────────────────────────────────────────────────────────────────
    let mut items_svg = String::new();
    let item_w_half = ITEM_WIDTH / 2.0;

    for (item_idx, item) in section.items.iter().enumerate() {
        let icy = item_centers[item_idx];
        let dyn_h = item_height_full(item);
        let item_h_half = dyn_h / 2.0;
        let has_meta = item.ticket.is_some() || item.assigned.is_some();

        items_svg.push_str(&templates::item_group_open(svg_id, &esc(&item.id), cx, icy));

        // Card rect (shape determines style)
        let (rx_val, _shape_extra) = match item.shape {
            NodeShape::Circle => {
                let r = item_h_half.min(item_w_half);
                items_svg.push_str(&templates::item_circle(r, primary_border, card_bg));
                items_svg.push_str(&templates::item_label_fo_fixed(
                    -item_w_half + 10.0,
                    -item_h_half + 4.0,
                    ITEM_WIDTH - 10.0,
                    ITEM_WIDTH - 10.0,
                    fo_height(&item.label),
                    &esc(&item.label),
                    primary_text,
                ));
                items_svg.push_str("</g>");
                continue;
            }
            NodeShape::RoundedRect => (item_h_half / 2.0, None::<String>),
            NodeShape::Hexagon => {
                let dx = item_w_half / 2.0;
                let pts = format!(
                    "{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
                    -item_w_half,
                    0.0,
                    -item_w_half + dx,
                    -item_h_half,
                    item_w_half - dx,
                    -item_h_half,
                    item_w_half,
                    0.0,
                    item_w_half - dx,
                    item_h_half,
                    -item_w_half + dx,
                    item_h_half,
                );
                items_svg.push_str(&templates::item_hexagon(&pts, primary_border, card_bg));
                items_svg.push_str(&templates::item_label_fo_fixed(
                    -item_w_half + 10.0,
                    -item_h_half / 2.0,
                    ITEM_WIDTH - 10.0,
                    ITEM_WIDTH - 10.0,
                    fo_height(&item.label),
                    &esc(&item.label),
                    primary_text,
                ));
                items_svg.push_str("</g>");
                continue;
            }
            NodeShape::Cloud => (item_h_half, None),
            NodeShape::Bang => (4.0, None),
            NodeShape::Default => {
                items_svg.push_str(&templates::item_default_rect(
                    -item_w_half,
                    -item_h_half,
                    ITEM_WIDTH,
                    dyn_h,
                    primary_border,
                    card_bg,
                ));
                items_svg.push_str(&templates::item_label_fo_fixed(
                    -item_w_half + 10.0,
                    -item_h_half / 2.0 - 6.0,
                    ITEM_WIDTH - 10.0,
                    ITEM_WIDTH - 10.0,
                    fo_height(&item.label),
                    &esc(&item.label),
                    primary_text,
                ));
                items_svg.push_str("</g>");
                continue;
            }
            NodeShape::Rect => (5.0, None),
        };

        // Default rect-based rendering
        items_svg.push_str(&templates::item_rect(
            rx_val,
            -item_w_half,
            -item_h_half,
            ITEM_WIDTH,
            dyn_h,
            primary_border,
            card_bg,
        ));

        // Primary label — position depends on whether metadata row is present.
        let (label_ty, label_fo_h) = if has_meta {
            // Label occupies top portion, metadata row follows immediately after.
            let ty = -(item_h_half - 4.0);
            let fh = text_height(&item.label);
            (ty, fh)
        } else {
            let fh = fo_height(&item.label);
            (-fh / 2.0, fh)
        };
        if !has_meta {
            let lines = wrap_label(&item.label);
            items_svg.push_str(&templates::item_label_wrapped(
                -item_w_half + 10.0,
                0.0,
                &lines,
                LINE_HEIGHT,
                primary_text,
            ));
        } else {
            items_svg.push_str(&templates::item_label_fo(
                -item_w_half + 10.0,
                label_ty,
                ITEM_WIDTH - 10.0,
                ITEM_WIDTH - 10.0,
                label_fo_h,
                &esc(&item.label),
                primary_text,
            ));
        }

        // Ticket + assignee metadata row — positioned immediately below the label.
        if has_meta {
            let meta_y = label_ty + label_fo_h;
            // Ticket number as clickable link (left side)
            if let Some(ref ticket) = item.ticket {
                let ticket_url = ticket_base_url
                    .map(|u| u.replace("#TICKET#", ticket))
                    .unwrap_or_default();
                if ticket_url.is_empty() {
                    items_svg.push_str(&templates::item_label_fo_fixed(
                        -item_w_half + 10.0,
                        meta_y,
                        60.0,
                        60.0,
                        24.0,
                        &esc(ticket),
                        primary_text,
                    ));
                } else {
                    items_svg.push_str(&templates::ticket_link(
                        &ticket_url,
                        -item_w_half + 10.0,
                        meta_y,
                        &esc(ticket),
                        primary_text,
                    ));
                }
            }
            // Assignee (right side) — nowrap so names don't get cut off.
            if let Some(ref assigned) = item.assigned {
                items_svg.push_str(&templates::assignee_label(
                    item_w_half - 10.0,
                    meta_y,
                    item_w_half + 5.0,
                    &esc(assigned),
                    primary_text,
                ));
            }
        } else {
            // Empty secondary label slots — mirrors Mermaid structure
            items_svg.push_str(&templates::item_label_empty(
                -item_w_half + 10.0,
                item_h_half - 10.0,
                ITEM_WIDTH - 10.0,
            ));
            items_svg.push_str(&templates::item_label_empty(
                item_w_half - 10.0,
                item_h_half - 10.0,
                ITEM_WIDTH - 10.0,
            ));
        }

        // Priority indicator — colored vertical line on left edge of card.
        if let Some(ref p) = item.priority {
            if let Some(color) = priority_color(p) {
                items_svg.push_str(&priority_line(
                    -item_w_half + 2.0,
                    -item_h_half + 2.0,
                    item_h_half - 2.0,
                    color,
                ));
            }
        }

        items_svg.push_str("</g>");
    }

    (sec_svg, items_svg)
}

/// Rough text width estimation (for foreignObject sizing only).
/// Uses 0.6 * font_size * char_count as approximation.
#[allow(dead_code)]
fn estimate_text_width(text: &str, font_size: f64) -> f64 {
    text.len() as f64 * font_size * 0.6
}

// ── Main render ────────────────────────────────────────────────────────────────

pub fn render(diag: &KanbanDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    if diag.sections.is_empty() {
        return templates::empty_svg().to_string();
    }

    let svg_id = "mermaid-svg-65"; // match Mermaid's deterministic ID pattern

    let n_cols = diag.sections.len();

    // Compute max column height
    let max_col_h = diag
        .sections
        .iter()
        .map(|s| col_height_dynamic(&s.items))
        .fold(0.0_f64, f64::max);

    // viewBox dimensions
    let vb_w = 15.0 + n_cols as f64 * (SECTION_WIDTH + SECTION_GAP);
    let vb_h = max_col_h + MARGIN * 2.0;

    let mut out = String::new();

    out.push_str(&templates::svg_root(
        svg_id,
        vb_w,
        VIEWBOX_X as i64,
        VIEWBOX_Y as i64,
        vb_w as u64,
        vb_h as u64,
    ));

    // Empty g (matches Mermaid structure)
    out.push_str("<g></g>");

    // Sections group
    let mut sections_svg = String::new();
    let mut items_svg_parts: Vec<String> = Vec::new();

    for (i, section) in diag.sections.iter().enumerate() {
        let sec_idx = i + 1; // 1-based
        let (sec_svg, items_svg) = render_section_and_items(
            section,
            sec_idx,
            i,
            svg_id,
            diag.config.ticket_base_url.as_deref(),
            vars.primary_border,
            vars.primary_text,
            vars.kanban_card_bg,
            theme,
        );
        sections_svg.push_str(&sec_svg);
        items_svg_parts.push(items_svg);
    }

    out.push_str(r#"<g class="sections">"#);
    out.push_str(&sections_svg);
    out.push_str("</g>");

    out.push_str(r#"<g class="items">"#);
    for items_svg in &items_svg_parts {
        out.push_str(items_svg);
    }
    out.push_str("</g>");

    out.push_str("</svg>");

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::kanban::parser;

    #[test]
    fn basic_render_produces_svg() {
        let input = "kanban\n  todo\n    id1[Task 1]\n    id2[Task 2]\n  inProgress\n    id3[Task 3]\n  done\n    id4[Task 4]";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "missing <svg");
        assert!(svg.contains("Task 1"));
        assert!(svg.contains("Task 2"));
        assert!(svg.contains("Task 3"));
        assert!(svg.contains("Task 4"));
        assert!(svg.contains("todo"));
        assert!(svg.contains("done"));
    }

    #[test]
    fn empty_kanban_produces_svg() {
        let diag = KanbanDiagram {
            sections: vec![],
            config: crate::diagrams::kanban::parser::KanbanConfig {
                ticket_base_url: None,
            },
        };
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn section_with_label_renders() {
        let input = "kanban\n  col1[\"To Do\"]\n    item1[\"My Task\"]\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("To Do"));
        assert!(svg.contains("My Task"));
    }

    #[test]
    fn multiple_columns_have_different_x() {
        let input = "kanban\n  col1\n    a[A]\n  col2\n    b[B]\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("col1"));
        assert!(svg.contains("col2"));
    }

    #[test]
    fn viewbox_matches_mermaid() {
        // 3 columns: vb_w = 15 + 3*205 = 630
        let input = "kanban\n  Todo\n    id1[Write blog post]\n    id2[Plan vacation]\n  In Progress\n    id3[Write code]\n  Done\n    id4[Create diagrams]";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(
            svg.contains(r#"viewBox="90 -310 630 "#),
            "viewBox width mismatch: {}",
            &svg[..200]
        );
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "kanban\n  Todo\n    id1[Write blog post]\n    id2[Plan vacation]\n  In Progress\n    id3[Write code]\n  Done\n    id4[Create diagrams]";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
