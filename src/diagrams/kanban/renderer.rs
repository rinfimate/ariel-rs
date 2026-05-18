use super::constants::*;
use super::parser::{KanbanDiagram, KanbanSection, NodeShape};
use super::templates;
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
use crate::theme::Theme;

// ── SVG helpers ────────────────────────────────────────────────────────────────

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── Layout math ────────────────────────────────────────────────────────────────

/// Column left-edge x position (0-based index).
fn col_left_x(idx: usize) -> f64 {
    COL_LEFT_BASE + idx as f64 * (SECTION_WIDTH + SECTION_GAP)
}

/// Column center x position (0-based index).
fn col_center_x(idx: usize) -> f64 {
    col_left_x(idx) + SECTION_WIDTH / 2.0
}

/// Column height for a given number of items.
/// Mirrors Mermaid: 79 for 1 item, +49 per additional item, min 50 for 0 items.
fn col_height(n_items: usize) -> f64 {
    if n_items == 0 {
        50.0
    } else {
        79.0 + (n_items as f64 - 1.0) * (ITEM_HEIGHT + ITEM_GAP)
    }
}

/// Y center of item at position `item_idx` (0-based) within the column.
/// First item center = COL_TOP + LABEL_HEIGHT + ITEM_HEIGHT/2
///                   = -300 + 25 + 22 = -253
/// Each subsequent item is (ITEM_HEIGHT + ITEM_GAP) = 49 below.
fn item_center_y(item_idx: usize) -> f64 {
    COL_TOP + LABEL_HEIGHT + ITEM_HEIGHT / 2.0 + item_idx as f64 * (ITEM_HEIGHT + ITEM_GAP)
}

// ── HSL section color palette (mirrors Mermaid's kanban CSS generation) ────────
//
// Mermaid generates per-section CSS rules with HSL colors.
// Hue sequence for section-1 onward: 80, 270, 300, 330, 0, 30, 90, 150, 180, 210.
// Lightness 86.27% (= 88/102 * 100% approximately, exact = 86.2745098039%).
// Section-1 text: black; section-2 text: #ffffff; section-3+ text: black (varies).
//
// We embed the same CSS classes as Mermaid so the PNG renderer applies them.

// section-N text color: white for section where hue makes it dark enough
fn section_text_color(section_idx: usize) -> &'static str {
    match section_idx {
        2 => "#ffffff",
        _ => "black",
    }
}

// ── CSS generation ─────────────────────────────────────────────────────────────

fn build_css(svg_id: &str, ff: &str) -> String {
    let mut s = format!(
        concat!(
            "#{id}{{font-family:{ff};font-size:16px;fill:#333;}}",
            "#{id} p{{margin:0;}}",
            "#{id} .edge{{stroke-width:3;}}",
        ),
        id = svg_id,
        ff = ff,
    );

    // section--1 (default/fallback)
    s.push_str(&format!(
        concat!(
            "#{id} .section--1 rect,#{id} .section--1 path,#{id} .section--1 circle,",
            "#{id} .section--1 polygon,#{id} .section--1 path{{",
            "fill:hsl(240, 100%, {l});stroke:hsl(240, 100%, {l});}}",
            "#{id} .section--1 text{{fill:#ffffff;}}",
        ),
        id = svg_id,
        l = SECTION_L,
    ));

    // section-0
    s.push_str(&format!(
        concat!(
            "#{id} .section-0 rect,#{id} .section-0 path,#{id} .section-0 circle,",
            "#{id} .section-0 polygon,#{id} .section-0 path{{",
            "fill:hsl({h}, 100%, {l});stroke:hsl({h}, 100%, {l});}}",
            "#{id} .section-0 text{{fill:black;}}",
        ),
        id = svg_id,
        h = SECTION_HUES[0],
        l = SECTION_L_0,
    ));

    // section-1 through section-10
    for (i, &hue) in SECTION_HUES.iter().enumerate().skip(1) {
        let text_color = section_text_color(i);
        let (l, _ld) = if i == 0 {
            (SECTION_L_0, SECTION_L_0_DARK)
        } else {
            (SECTION_L, SECTION_L_DARK)
        };
        s.push_str(&format!(
            concat!(
                "#{id} .section-{idx} rect,#{id} .section-{idx} path,#{id} .section-{idx} circle,",
                "#{id} .section-{idx} polygon,#{id} .section-{idx} path{{",
                "fill:hsl({h}, 100%, {l});stroke:hsl({h}, 100%, {l});}}",
                "#{id} .section-{idx} text{{fill:{tc};}}",
            ),
            id = svg_id,
            idx = i,
            h = hue,
            l = l,
            tc = text_color,
        ));
    }

    // Node (card) styles
    s.push_str(&format!(
        concat!(
            "#{id} .node rect,#{id} .node circle,#{id} .node ellipse,",
            "#{id} .node polygon,#{id} .node path{{",
            "fill:white;stroke:#9370DB;stroke-width:1px;}}",
            "#{id} .kanban-ticket-link{{fill:white;stroke:#9370DB;text-decoration:underline;}}",
        ),
        id = svg_id,
    ));

    // kanban-label
    s.push_str(&format!(
        concat!(
            "#{id} .kanban-label{{",
            "dy:1em;alignment-baseline:middle;text-anchor:middle;",
            "dominant-baseline:middle;text-align:center;}}",
        ),
        id = svg_id,
    ));

    // cluster-label / label
    s.push_str(&format!(
        "#{id} .cluster-label,#{id} .label{{color:#333;fill:#333;}}",
        id = svg_id,
    ));

    // section-root
    s.push_str(&format!(
        concat!(
            "#{id} .section-root rect,#{id} .section-root path,",
            "#{id} .section-root circle,#{id} .section-root polygon{{",
            "fill:hsl(240, 100%, 46.2745098039%);}}",
            "#{id} .section-root text{{fill:#ffffff;}}",
        ),
        id = svg_id,
    ));

    s
}

// ── Section rendering ──────────────────────────────────────────────────────────

/// Render a single column section.
/// Returns: (section_svg, items_svg) — sections go in <g class="sections">,
/// items go in <g class="items">.
fn render_section_and_items(
    section: &KanbanSection,
    sec_idx: usize, // 1-based
    col_idx: usize, // 0-based
    svg_id: &str,
) -> (String, String) {
    let cx = col_center_x(col_idx);
    let lx = col_left_x(col_idx);
    let col_top = COL_TOP;
    let n = section.items.len();
    let col_h = col_height(n);

    // ── Section column rect ──────────────────────────────────────────────────
    let mut sec_svg = templates::section_group_open(sec_idx, svg_id, &escape(&section.id));

    sec_svg.push_str(&templates::section_rect(lx, col_top, SECTION_WIDTH, col_h));

    // Column header label
    let label_x = cx;
    sec_svg.push_str(&templates::section_label_fo(
        label_x - 80.0,
        col_top,
        &escape(&section.label),
    ));

    sec_svg.push_str("</g>");

    // ── Items ────────────────────────────────────────────────────────────────
    let mut items_svg = String::new();
    let item_w_half = ITEM_WIDTH / 2.0;
    let item_h_half = ITEM_HEIGHT / 2.0;

    for (item_idx, item) in section.items.iter().enumerate() {
        let icy = item_center_y(item_idx);

        items_svg.push_str(&templates::item_group_open(
            svg_id,
            &escape(&item.id),
            cx,
            icy,
        ));

        // Card rect (shape determines style)
        let (rx_val, _shape_extra) = match item.shape {
            NodeShape::Circle => {
                let r = item_h_half.min(item_w_half);
                items_svg.push_str(&templates::item_circle(r));
                items_svg.push_str(&templates::item_label_fo_fixed(
                    -item_w_half + 10.0,
                    -item_h_half + 4.0,
                    ITEM_WIDTH - 20.0,
                    ITEM_WIDTH - 10.0,
                    &escape(&item.label),
                ));
                items_svg.push_str("</g>");
                continue;
            }
            NodeShape::RoundedRect => (item_h_half / 2.0, None::<String>),
            NodeShape::Hexagon => {
                // Polygon
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
                items_svg.push_str(&templates::item_hexagon(&pts));
                items_svg.push_str(&templates::item_label_fo_fixed(
                    -item_w_half + 10.0,
                    -item_h_half / 2.0,
                    ITEM_WIDTH - 20.0,
                    ITEM_WIDTH - 10.0,
                    &escape(&item.label),
                ));
                items_svg.push_str("</g>");
                continue;
            }
            NodeShape::Cloud => (item_h_half, None),
            NodeShape::Bang => (4.0, None),
            NodeShape::Default => {
                // No border
                items_svg.push_str(&templates::item_default_rect(
                    -item_w_half,
                    -item_h_half,
                    ITEM_WIDTH,
                    ITEM_HEIGHT,
                ));
                items_svg.push_str(&templates::item_label_fo_fixed(
                    -item_w_half + 10.0,
                    -item_h_half / 2.0 - 6.0,
                    ITEM_WIDTH - 20.0,
                    ITEM_WIDTH - 10.0,
                    &escape(&item.label),
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
            ITEM_HEIGHT,
        ));

        // Primary label: translate(-82.5, -12) — mirrors Mermaid's label placement
        items_svg.push_str(&templates::item_label_fo(
            -item_w_half + 10.0,
            -item_h_half + 10.0,
            estimate_text_width(&item.label, 16.0),
            ITEM_WIDTH - 10.0,
            &escape(&item.label),
        ));

        // Secondary label slots (empty) — mirrors Mermaid structure
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

        items_svg.push_str("</g>");
    }

    (sec_svg, items_svg)
}

/// Rough text width estimation (for foreignObject sizing only).
/// Uses 0.6 * font_size * char_count as approximation.
fn estimate_text_width(text: &str, font_size: f64) -> f64 {
    text.len() as f64 * font_size * 0.6
}

// ── Main render ────────────────────────────────────────────────────────────────

pub fn render(diag: &KanbanDiagram, theme: Theme) -> String {
    let vars = theme.resolve();
    let ff = vars.font_family;
    if diag.sections.is_empty() {
        return templates::empty_svg().to_string();
    }

    let svg_id = "mermaid-svg-65"; // match Mermaid's deterministic ID pattern

    let n_cols = diag.sections.len();

    // Compute max column height
    let max_col_h = diag
        .sections
        .iter()
        .map(|s| col_height(s.items.len()))
        .fold(0.0_f64, f64::max);

    // viewBox dimensions
    let vb_w = 15.0 + n_cols as f64 * (SECTION_WIDTH + SECTION_GAP);
    let vb_h = max_col_h + MARGIN * 2.0;

    let css = build_css(svg_id, ff);

    let mut out = String::new();

    out.push_str(&templates::svg_root(
        svg_id,
        vb_w,
        VIEWBOX_X as i64,
        VIEWBOX_Y as i64,
        vb_w as u64,
        vb_h as u64,
    ));

    out.push_str(&format!("<style>{css}</style>"));

    // Empty g (matches Mermaid structure)
    out.push_str("<g></g>");

    // Sections group
    let mut sections_svg = String::new();
    let mut items_svg_parts: Vec<String> = Vec::new();

    for (i, section) in diag.sections.iter().enumerate() {
        let sec_idx = i + 1; // 1-based
        let (sec_svg, items_svg) = render_section_and_items(section, sec_idx, i, svg_id);
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
        let diag = KanbanDiagram { sections: vec![] };
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
        // 3 columns: vb_w = 15 + 3*205 = 630, vb_h = 148 (max col h=128 + 20)
        let input = "kanban\n  Todo\n    id1[Write blog post]\n    id2[Plan vacation]\n  In Progress\n    id3[Write code]\n  Done\n    id4[Create diagrams]";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(
            svg.contains(r#"viewBox="90 -310 630 148""#),
            "viewBox mismatch: {}",
            &svg[..500]
        );
    }

    #[test]
    fn snapshot_default_theme() {
        let input = "kanban\n  Todo\n    id1[Write blog post]\n    id2[Plan vacation]\n  In Progress\n    id3[Write code]\n  Done\n    id4[Create diagrams]";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(svg);
    }
}
