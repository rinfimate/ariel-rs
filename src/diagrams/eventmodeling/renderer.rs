// Faithful Rust port of mermaid/src/diagrams/eventmodeling/db.ts + renderer.ts
//
// db.ts implements an event-sourcing layout machine:
//   for each frame: PositionFrame → FramePositioned → evolveFramePositioned
//                   PositionRelation → RelationPositioned → evolveRelationPositioned
//
// renderer.ts draws:
//   state.sortedSwimlanesArray → renderD3Swimlane (em-swimlane group)
//   state.boxes                → renderD3Box       (em-box group)
//   state.relations            → renderD3Relation   (em-relation path)
//   defs                       → arrowhead marker
//   setupGraphViewbox          → SVG root with viewBox fitted to content + padding

use std::collections::HashMap;

use super::constants::*;
use super::parser::{EmFrame, EntityType, EventModelDiagram};
use super::templates::{self, esc, fmt};
use crate::text_browser_metrics::measure_browser;
use crate::theme::Theme;

// ─── Visual properties (calculateEntityVisualProps) ───────────────────────────

struct EntityVisual {
    fill: &'static str,
    stroke: &'static str,
}

fn entity_visual(entity_type: &EntityType, vars: &crate::theme::ThemeVars) -> EntityVisual {
    match entity_type {
        EntityType::Ui => EntityVisual {
            fill: vars.em_ui_fill,
            stroke: vars.em_ui_stroke,
        },
        EntityType::Processor => EntityVisual {
            fill: vars.em_processor_fill,
            stroke: vars.em_processor_stroke,
        },
        EntityType::ReadModel => EntityVisual {
            fill: vars.em_readmodel_fill,
            stroke: vars.em_readmodel_stroke,
        },
        EntityType::Command => EntityVisual {
            fill: vars.em_command_fill,
            stroke: vars.em_command_stroke,
        },
        EntityType::Event => EntityVisual {
            fill: vars.em_event_fill,
            stroke: vars.em_event_stroke,
        },
    }
}

// ─── Swimlane props (calculateSwimlaneProps) ──────────────────────────────────

struct SwimlaneProps {
    index: i64,
    label: String,
}

fn swimlane_props(frame: &EmFrame, swimlanes: &HashMap<i64, Swimlane>) -> SwimlaneProps {
    let namespace = extract_namespace(&frame.entity_id);
    let sw = namespace
        .as_deref()
        .and_then(|ns| find_swimlane_by_namespace(swimlanes, ns));

    match frame.entity_type {
        EntityType::Ui | EntityType::Processor => {
            if let Some(existing) = sw {
                SwimlaneProps {
                    index: existing.index,
                    label: namespace
                        .clone()
                        .unwrap_or_else(|| LABEL_UI_AUTOMATION.to_string()),
                }
            } else if namespace.is_some() {
                let ns = namespace.unwrap();
                SwimlaneProps {
                    index: find_next_available_index(swimlanes, SWIMLANE_UI_IDX, SWIMLANE_UI_MAX),
                    label: format!("{}{}", LABEL_UI_AUTOMATION_PREFIX, ns),
                }
            } else {
                SwimlaneProps {
                    index: SWIMLANE_UI_IDX,
                    label: LABEL_UI_AUTOMATION.to_string(),
                }
            }
        }
        EntityType::ReadModel | EntityType::Command => {
            if let Some(existing) = sw {
                SwimlaneProps {
                    index: existing.index,
                    label: namespace
                        .clone()
                        .unwrap_or_else(|| LABEL_COMMAND_READMODEL.to_string()),
                }
            } else if namespace.is_some() {
                let ns = namespace.unwrap();
                SwimlaneProps {
                    index: find_next_available_index(swimlanes, SWIMLANE_CMD_IDX, SWIMLANE_CMD_MAX),
                    label: format!("{}{}", LABEL_COMMAND_READMODEL_PREFIX, ns),
                }
            } else {
                SwimlaneProps {
                    index: SWIMLANE_CMD_IDX,
                    label: LABEL_COMMAND_READMODEL.to_string(),
                }
            }
        }
        EntityType::Event => {
            if let Some(existing) = sw {
                SwimlaneProps {
                    index: existing.index,
                    label: namespace
                        .clone()
                        .unwrap_or_else(|| LABEL_EVENTS.to_string()),
                }
            } else if namespace.is_some() {
                let ns = namespace.unwrap();
                SwimlaneProps {
                    index: find_next_available_index(swimlanes, SWIMLANE_EVT_IDX, SWIMLANE_EVT_MAX),
                    label: format!("{}{}", LABEL_EVENTS_PREFIX, ns),
                }
            } else {
                SwimlaneProps {
                    index: SWIMLANE_EVT_IDX,
                    label: LABEL_EVENTS.to_string(),
                }
            }
        }
    }
}

fn extract_namespace(entity_id: &str) -> Option<String> {
    let parts: Vec<&str> = entity_id.splitn(2, '.').collect();
    if parts.len() == 2 {
        Some(parts[0].to_string())
    } else {
        None
    }
}

fn extract_name(entity_id: &str) -> &str {
    if let Some(dot) = entity_id.find('.') {
        &entity_id[dot + 1..]
    } else {
        entity_id
    }
}

fn find_swimlane_by_namespace<'a>(
    swimlanes: &'a HashMap<i64, Swimlane>,
    namespace: &str,
) -> Option<&'a Swimlane> {
    swimlanes
        .values()
        .find(|sw| sw.namespace.as_deref() == Some(namespace))
}

fn find_next_available_index(
    swimlanes: &HashMap<i64, Swimlane>,
    boundary_min: i64,
    boundary_max: i64,
) -> i64 {
    let max_existing = swimlanes
        .keys()
        .filter(|&&k| k > boundary_min && k < boundary_max)
        .copied()
        .fold(boundary_min, i64::max);
    max_existing + 1
}

// ─── Layout state (db.ts initial / state types) ───────────────────────────────

#[derive(Clone)]
struct Swimlane {
    index: i64,
    label: String,
    namespace: Option<String>,
    /// x right-edge of the last box placed in this swimlane.
    r: f64,
    /// Top y of this swimlane strip.
    y: f64,
    /// Total height of this swimlane strip.
    height: f64,
    /// Maximum box height seen so far in this swimlane.
    max_height: f64,
}

#[derive(Clone)]
struct LayoutBox {
    x: f64,
    /// y = swimlane.y + swimlanePadding (set at placement time)
    y: f64,
    /// right edge of box + boxPadding gap
    r: f64,
    /// (width, height)
    dim: (f64, f64),
    swimlane_index: i64,
    visual_fill: &'static str,
    visual_stroke: &'static str,
    text: String,
    frame_name: String,
}

struct LayoutRelation {
    source_idx: usize,
    target_idx: usize,
}

struct LayoutState {
    boxes: Vec<LayoutBox>,
    swimlanes: HashMap<i64, Swimlane>,
    relations: Vec<LayoutRelation>,
    max_r: f64,
    prev_swimlane_idx: Option<i64>,
}

// ─── calculateX ───────────────────────────────────────────────────────────────

fn calculate_x(
    swimlane: &Swimlane,
    prev_swimlane: Option<&Swimlane>,
    last_box: Option<&LayoutBox>,
) -> f64 {
    match prev_swimlane {
        None => CONTENT_START_X,
        Some(psw) => {
            if psw.index == swimlane.index && swimlane.r > 0.0 {
                // same swimlane: continue from right edge + padding
                swimlane.r + BOX_PADDING
            } else if let Some(lb) = last_box {
                // different swimlane: overlap with previous box
                lb.r - BOX_OVERLAP + BOX_PADDING
            } else {
                CONTENT_START_X
            }
        }
    }
}

// ─── calculateMaxRight ───────────────────────────────────────────────────────

fn calculate_max_right(swimlanes: &HashMap<i64, Swimlane>, swimlane_r: f64) -> f64 {
    swimlanes.values().map(|sw| sw.r).fold(swimlane_r, f64::max)
}

// ─── Recalculate swimlane y positions ─────────────────────────────────────────
// Mirrors the code at the end of evolveFramePositioned:
//   swimlanes[0].y = 0
//   for i in 1..: sw.y = prevSw.y + prevSw.height + swimlaneGap

fn recalculate_swimlane_ys(swimlanes: &mut HashMap<i64, Swimlane>) {
    // collect sorted by index
    let mut sorted_idxs: Vec<i64> = swimlanes.keys().copied().collect();
    sorted_idxs.sort_unstable();
    for (i, idx) in sorted_idxs.iter().enumerate() {
        let y = if i == 0 {
            0.0
        } else {
            let prev_idx = sorted_idxs[i - 1];
            let prev_y = swimlanes[&prev_idx].y;
            let prev_h = swimlanes[&prev_idx].height;
            prev_y + prev_h + SWIMLANE_GAP
        };
        if let Some(sw) = swimlanes.get_mut(idx) {
            sw.y = y;
        }
    }
}

// ─── Text dimensions (mirrors calculateTextProps / calculateTextDimensions) ──────
//
// Mermaid JS passes `content = "<b>${name}</b>"` to `calculateTextDimensions`, which
// measures the string **including the HTML markup** as plain SVG text via getBBox().
// So the measured width is the width of the literal string `<b>PlaceOrder</b>` (not
// just `PlaceOrder`) at FONT_SIZE px in Arial (bold).  We replicate this by measuring
// the same formatted string with measure_browser(), which uses pre-tabulated Arial
// character widths that closely match browser getBBox() output.

fn text_dimensions(name: &str) -> (f64, f64) {
    // Mermaid measures "<b>Name</b>" as literal SVG text via getBBox().
    // Our measure_browser table runs ~3% low vs browser getBBox for this content,
    // so we apply a calibration factor derived from empirical ref SVG comparison.
    let tagged = format!("<b>{}</b>", name);
    let (w, _h) = measure_browser(&tagged, FONT_SIZE);
    let width = w * 1.03;
    let height = FONT_SIZE * 1.25;
    (width, height)
}

// ─── evolveFramePositioned ────────────────────────────────────────────────────

fn evolve_frame_positioned(
    state: &mut LayoutState,
    frame: &EmFrame,
    _frame_index: usize,
    visual: EntityVisual,
    text_w: f64,
    text_h: f64,
) {
    let sp = swimlane_props(frame, &state.swimlanes);

    // Ensure swimlane exists
    if !state.swimlanes.contains_key(&sp.index) {
        let ns = extract_namespace(&frame.entity_id);
        state.swimlanes.insert(
            sp.index,
            Swimlane {
                index: sp.index,
                label: sp.label.clone(),
                namespace: ns,
                r: 0.0,
                y: sp.index as f64 * SWIMLANE_MIN_HEIGHT + SWIMLANE_GAP,
                height: SWIMLANE_MIN_HEIGHT,
                max_height: SWIMLANE_MIN_HEIGHT,
            },
        );
    } else {
        // Update label if it changed (namespace sub-swimlane)
        state.swimlanes.get_mut(&sp.index).unwrap().label = sp.label.clone();
    }

    let last_box = state.boxes.last().cloned();
    let prev_swimlane = state
        .prev_swimlane_idx
        .and_then(|idx| state.swimlanes.get(&idx).cloned());

    // Box dimension = clamp(min, max, text + 2*textPadding) + 2*boxPadding
    let raw_w = text_w + 2.0 * BOX_TEXT_PADDING;
    let raw_h = text_h + 2.0 * BOX_TEXT_PADDING;
    let dim_w = raw_w.clamp(BOX_MIN_WIDTH, BOX_MAX_WIDTH) + 2.0 * BOX_PADDING;
    let dim_h = raw_h.clamp(BOX_MIN_HEIGHT, BOX_MAX_HEIGHT) + 2.0 * BOX_PADDING;

    let swimlane = state.swimlanes.get(&sp.index).unwrap();
    let x = calculate_x(swimlane, prev_swimlane.as_ref(), last_box.as_ref());
    let r = x + dim_w + BOX_PADDING;

    // Update swimlane
    {
        let sw = state.swimlanes.get_mut(&sp.index).unwrap();
        sw.r = x + dim_w;
        sw.max_height = sw.max_height.max(dim_h);
        sw.height = SWIMLANE_MIN_HEIGHT.max(sw.max_height) + 2.0 * SWIMLANE_PADDING;
    }

    let max_r = calculate_max_right(&state.swimlanes, r);

    let swimlane_y = state.swimlanes[&sp.index].y;
    let box_y = SWIMLANE_PADDING + swimlane_y;

    state.boxes.push(LayoutBox {
        x,
        y: box_y,
        r,
        dim: (dim_w, dim_h),
        swimlane_index: sp.index,
        visual_fill: visual.fill,
        visual_stroke: visual.stroke,
        text: extract_name(&frame.entity_id).to_string(),
        frame_name: frame.name.clone(),
    });

    state.max_r = max_r;
    state.prev_swimlane_idx = Some(sp.index);

    // Recalculate all swimlane y positions
    recalculate_swimlane_ys(&mut state.swimlanes);

    // Re-sync all box y positions to reflect updated swimlane.y values
    for b in state.boxes.iter_mut() {
        let sy = state.swimlanes[&b.swimlane_index].y;
        b.y = SWIMLANE_PADDING + sy;
    }
}

// ─── isFirstFrame ─────────────────────────────────────────────────────────────

fn is_first_frame(index: usize, frame: &EmFrame) -> bool {
    index == 0 && frame.source_refs.is_empty()
}

// ─── findBoxByFrame ───────────────────────────────────────────────────────────

fn find_box_by_name<'a>(boxes: &'a [LayoutBox], name: &str) -> Option<(usize, &'a LayoutBox)> {
    boxes.iter().enumerate().find(|(_, b)| b.frame_name == name)
}

// ─── findBoxByLineIndex ───────────────────────────────────────────────────────
//
// Walk backwards from lineIndex to find a box NOT in targetSwimlane.

fn find_box_by_line_index(
    boxes: &[LayoutBox],
    target_swimlane: i64,
    line_index: usize,
) -> Option<(usize, &LayoutBox)> {
    if boxes.is_empty() {
        return None;
    }
    let start = line_index.min(boxes.len().saturating_sub(1));
    for i in (0..=start).rev() {
        let b = &boxes[i];
        if b.swimlane_index != target_swimlane {
            return Some((i, b));
        }
    }
    None
}

// ─── evolveRelationPositioned ─────────────────────────────────────────────────

fn evolve_relation_positioned(state: &mut LayoutState, source_idx: usize, target_idx: usize) {
    state.relations.push(LayoutRelation {
        source_idx,
        target_idx,
    });
}

// ─── Main render ──────────────────────────────────────────────────────────────

pub fn render(diag: &EventModelDiagram, theme: Theme) -> String {
    let vars = theme.resolve();

    // ── Build layout state ─────────────────────────────────────────────────────
    let mut state = LayoutState {
        boxes: Vec::new(),
        swimlanes: HashMap::new(),
        relations: Vec::new(),
        max_r: 0.0,
        prev_swimlane_idx: None,
    };

    // Process each frame (mirrors getState() in db.ts)
    for (index, frame) in diag.frames.iter().enumerate() {
        let name = extract_name(&frame.entity_id).to_string();
        let (tw, th) = text_dimensions(&name);
        let visual = entity_visual(&frame.entity_type, &vars);
        evolve_frame_positioned(&mut state, frame, index, visual, tw, th);

        // Position relations
        if frame.is_reset || is_first_frame(index, frame) {
            // No relation emitted
        } else if !frame.source_refs.is_empty() {
            // Explicit source refs
            for src_ref in &frame.source_refs {
                let target_box_idx = match find_box_by_name(&state.boxes, &frame.name) {
                    Some((i, _)) => i,
                    None => continue,
                };
                if let Some((src_box_idx, _)) = find_box_by_name(&state.boxes, src_ref) {
                    evolve_relation_positioned(&mut state, src_box_idx, target_box_idx);
                }
            }
        } else {
            // Auto: find box in different swimlane by walking backwards
            let target_box_idx = match find_box_by_name(&state.boxes, &frame.name) {
                Some((i, _)) => i,
                None => continue,
            };
            let target_swimlane = state.boxes[target_box_idx].swimlane_index;
            // index - 1 is the previous frame's position
            let search_start = if index > 0 { index - 1 } else { 0 };
            // findBoxByLineIndex searches state.boxes[0..search_start] for a different swimlane
            let boxes_snapshot: Vec<LayoutBox> = state.boxes.clone();
            if let Some((src_idx, _)) =
                find_box_by_line_index(&boxes_snapshot, target_swimlane, search_start)
            {
                evolve_relation_positioned(&mut state, src_idx, target_box_idx);
            }
        }
    }

    // Sort swimlanes by index
    let mut sorted_swimlanes: Vec<&Swimlane> = state.swimlanes.values().collect();
    sorted_swimlanes.sort_by_key(|sw| sw.index);

    // ── Compute canvas bounds ─────────────────────────────────────────────────
    let max_r = state.max_r;
    let content_h = if sorted_swimlanes.is_empty() {
        0.0
    } else {
        let last = sorted_swimlanes.last().unwrap();
        last.y + last.height
    };
    let content_w = max_r + SWIMLANE_PADDING; // = swimlane rect width (matches Mermaid getBBox())

    // Mermaid JS setupGraphViewbox: viewBox = (-pad -pad (cw+2*pad) (ch+2*pad))
    // with width="100%" and style="max-width: Xpx". Content is drawn at (0,0).
    let vb_w = content_w + 2.0 * DIAGRAM_PADDING;
    let vb_h = content_h + 2.0 * DIAGRAM_PADDING;

    // ── Build SVG ─────────────────────────────────────────────────────────────
    let mut out = String::new();

    out.push_str(&templates::svg_root(
        &fmt(-DIAGRAM_PADDING),
        &fmt(-DIAGRAM_PADDING),
        &fmt(vb_w),
        &fmt(vb_h),
        &fmt(vb_w),
    ));

    // Arrowhead marker (added to defs after drawing in Mermaid JS, order irrelevant for SVG)
    let arrowhead_id = "em-arrowhead";
    out.push_str(&templates::arrowhead_marker(
        arrowhead_id,
        vars.em_arrowhead,
    ));

    // Swimlanes (renderD3Swimlane)
    for sw in &sorted_swimlanes {
        out.push_str(&templates::swimlane(
            sw.y,
            max_r + SWIMLANE_PADDING,
            sw.height,
            vars.em_swimlane_bg,
            vars.em_swimlane_bg_stroke,
            SWIMLANE_LABEL_X,
            sw.y + SWIMLANE_LABEL_Y_OFFSET,
            vars.font_family,
            SWIMLANE_FONT_SIZE,
            vars.text_color,
            &esc(&sw.label),
        ));
    }

    // Boxes (renderD3Box)
    for bx in &state.boxes {
        let (bw, bh) = bx.dim;
        out.push_str(&templates::em_box(
            bx.x,
            bx.y,
            bw,
            bh,
            bx.visual_stroke,
            bx.visual_fill,
            vars.font_family,
            FONT_SIZE,
            vars.text_color,
            &esc(&bx.text),
        ));
    }

    // Relations (renderD3Relation)
    for rel in &state.relations {
        let src = &state.boxes[rel.source_idx];
        let tgt = &state.boxes[rel.target_idx];

        // Source and target swimlane y (box.y = swimlane.y + swimlanePadding,
        // so swimlaneY = box.y - swimlanePadding, but renderer uses box.y directly)
        let source_box_y = src.y;
        let target_box_y = tgt.y;

        let source_x = src.x + src.dim.0 * 2.0 / 3.0;
        let target_x = tgt.x + tgt.dim.0 / 3.0;

        let upwards = source_box_y > target_box_y;
        let (source_y, target_y) = if upwards {
            (source_box_y, target_box_y + tgt.dim.1)
        } else {
            (source_box_y + src.dim.1, target_box_y)
        };

        out.push_str(&templates::relation_path(
            "none",
            vars.em_relation_stroke,
            arrowhead_id,
            source_x,
            source_y,
            target_x,
            target_y,
        ));
    }

    // Title
    if let Some(t) = &diag.title {
        out.push_str(&templates::title_text(
            content_w / 2.0,
            vars.font_family,
            FONT_SIZE,
            vars.title_color,
            &esc(t),
        ));
    }

    out.push_str("</svg>");
    out
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    // Canonical mermaid-syntax test case.
    // title omitted: Mermaid JS 11.15 rejects the title directive in the eventmodeling grammar.
    const EM_BASIC: &str = "eventmodeling\n  tf 01 cmd PlaceOrder\n  tf 02 evt OrderPlaced\n  tf 03 rmo OrderView\n  tf 04 cmd ProcessPayment";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(EM_BASIC).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "missing <svg tag");
        assert!(svg.contains("PlaceOrder"), "missing box text");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(EM_BASIC).diagram;
        let svg = render(&diag, Theme::Dark);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn source_refs_create_relations() {
        let input = "eventmodeling\n  tf 01 cmd PlaceOrder\n  tf 02 evt OrderPlaced ->> 01\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("em-relation"), "missing relation path");
    }

    #[test]
    fn namespace_swimlane() {
        let input =
            "eventmodeling\n  tf 01 cmd Checkout.PlaceOrder\n  tf 02 cmd Inventory.Reserve\n";
        let diag = parser::parse(input).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("C/RM: Checkout"), "missing namespace label");
    }

    #[test]
    fn snapshot_default_theme() {
        let diag = parser::parse(EM_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
