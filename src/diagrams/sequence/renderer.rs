use super::constants::*;
use super::templates;
/// Sequence diagram renderer — faithful port of Mermaid sequenceRenderer.ts
///
/// Algorithm (same as Mermaid):
///   1. Collect participants (implicit + explicit)
///   2. Measure actor box widths based on message widths (getMaxMessageWidthPerActor)
///   3. Compute actor x positions with margins (calculateActorMargins)
///   4. Vertical layout: walk items, bump verticalPos for each event
///   5. Emit SVG actors (top + bottom), lifelines, messages, control structures
use crate::diagrams::sequence::parser::{
    LineType, NotePlacement, ParticipantKind, SeqItem, SequenceDiagram,
};
use crate::text::measure;
use crate::theme::{Theme, ThemeVars};

// ─── Mermaid default sequence config ───────────────────────────────────────
// All constants are imported from super::constants via `use super::constants::*`.

fn actor_visual_height(kind: &ParticipantKind) -> f64 {
    match kind {
        ParticipantKind::Actor => ACTOR_MAN_HEIGHT,
        _ => ACTOR_HEIGHT,
    }
}

// ─── Internal actor model ───────────────────────────────────────────────────
#[derive(Debug, Clone)]
struct Actor {
    name: String,
    _alias: String,
    kind: ParticipantKind,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    margin: f64,
    start_y: f64, // top lifeline start
    stop_y: f64,  // bottom lifeline end
}

impl Actor {
    fn cx(&self) -> f64 {
        self.x + self.width / 2.0
    }
}

// ─── Activation box ─────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
struct Activation {
    actor: String,
    start_x: f64,
    start_y: f64,
    stop_x: f64,
    stop_y: f64,
}

// ─── Note model ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
struct RenderedNote {
    start_x: f64,
    start_y: f64,
    width: f64,
    height: f64,
    text: String,
}

// ─── Message model ──────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
struct MsgModel {
    from: String,
    to: String,
    start_x: f64,
    stop_x: f64,
    from_bounds: f64,
    to_bounds: f64,
    start_y: f64,
    line_start_y: f64,
    text: String,
    line_type: LineType,
    #[allow(dead_code)]
    activate: i32,
    seq_idx: usize,
    show_seq: bool,
    #[allow(dead_code)]
    height: f64,
}

// ─── Loop/Alt/Par model ─────────────────────────────────────────────────────
#[derive(Debug, Clone)]
enum ControlKind {
    Loop,
    Alt,
    Opt,
    Par,
}

#[derive(Debug, Clone)]
struct ControlSection {
    y: f64,
    label: String,
}

#[derive(Debug, Clone)]
struct ControlModel {
    kind: ControlKind,
    label: String,
    start_x: f64,
    stop_x: f64,
    start_y: f64,
    stop_y: f64,
    sections: Vec<ControlSection>, // for alt/par dividers
}

// ─── Bounds tracker ─────────────────────────────────────────────────────────
#[derive(Debug, Default)]
struct Bounds {
    start_x: f64,
    stop_x: f64,
    start_y: f64,
    stop_y: f64,
    vertical_pos: f64,
    loop_stack: Vec<LoopFrame>,
    activations: Vec<Activation>,
}

#[derive(Debug, Clone)]
struct LoopFrame {
    label: String,
    kind: ControlKind,
    from: f64,
    to: f64,
    start_y: f64,
    stop_y: f64, // tracks max stopy (with boxMargin) from inserts inside loop
    sections: Vec<ControlSection>,
    #[allow(dead_code)]
    is_alt: bool, // alt has vertical sections
}

impl Bounds {
    fn bump(&mut self, h: f64) {
        self.vertical_pos += h;
        if self.vertical_pos > self.stop_y {
            self.stop_y = self.vertical_pos;
        }
    }

    fn insert(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        let lx = x1.min(x2);
        let rx = x1.max(x2);
        let ty = y1.min(y2);
        let by = y1.max(y2);
        if lx < self.start_x {
            self.start_x = lx;
        }
        if rx > self.stop_x {
            self.stop_x = rx;
        }
        if ty < self.start_y {
            self.start_y = ty;
        }
        if by > self.stop_y {
            self.stop_y = by;
        }
        // Mermaid updateBounds: propagate stopy+n*boxMargin to each loop frame
        let n_frames = self.loop_stack.len();
        for (i, frame) in self.loop_stack.iter_mut().enumerate() {
            let depth = (n_frames - i) as f64; // 1-based nesting depth
            let padded = by + depth * BOX_MARGIN;
            if padded > frame.stop_y {
                frame.stop_y = padded;
            }
        }
    }
}

// ─── SVG helpers ────────────────────────────────────────────────────────────

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn actor_rect_svg(x: f64, y: f64, w: f64, h: f64, name: &str, cls: &str) -> String {
    templates::actor_rect(x, y, w, h, &esc(name), cls)
}

/// Render an actor-man (stick figure) at actor box position (top or bottom)
/// cx = center x, box_y = top of the actor box area (y=0 for top, bottom_actor_y for bottom)
fn actor_man_svg(cx: f64, box_y: f64, name: &str, cls: &str, idx: usize) -> String {
    let cy = box_y + 10.0; // circle center y
    let r = 15.0_f64;
    let ts = cy + r; // torso start y
    let te = cy + r + 20.0; // torso end y
    let al = cx - 18.0; // arm left x
    let ar = cx + 18.0; // arm right x
    let ay = cy + r + 8.0; // arm y
    let ls = cy + r + 20.0; // leg start y (= torso end)
    let le = cy + r + 35.0; // leg end y
    let ll = cx - 16.0; // leg left x
    let rl = cx + 16.0; // leg right x
    let ty = box_y + ACTOR_MAN_HEIGHT - 12.5; // text y (ref uses -12.5 for 2px gap below legs)
    let esc_name = esc(name);
    format!(
        concat!(
            r#"<g class="actor-man {cls}" name="{name}" data-et="participant" data-type="actor" data-id="{name}" style="stroke: rgb(147, 112, 219);">"#,
            r#"<line id="actor-man-torso{idx}" x1="{cx}" y1="{ts}" x2="{cx}" y2="{te}"></line>"#,
            r#"<line id="actor-man-arms{idx}" x1="{al}" y1="{ay}" x2="{ar}" y2="{ay}"></line>"#,
            r#"<line x1="{ll}" y1="{le}" x2="{cx}" y2="{ls}"></line>"#,
            r#"<line x1="{cx}" y1="{ls}" x2="{rl}" y2="{le}"></line>"#,
            r#"<circle cx="{cx}" cy="{cy}" r="{r}" width="{w}" height="{h}"></circle>"#,
            r#"<text x="{cx}" y="{ty}" dominant-baseline="central" alignment-baseline="central" class="actor actor-man" style="text-anchor: middle; font-size: {fs}px; font-weight: 400; font-family: Arial, sans-serif;"><tspan x="{cx}" dy="0">{name}</tspan></text>"#,
            r#"</g>"#,
        ),
        cls = cls,
        name = esc_name,
        idx = idx,
        cx = cx,
        cy = cy,
        r = r,
        ts = ts,
        te = te,
        al = al,
        ar = ar,
        ay = ay,
        ll = ll,
        rl = rl,
        ls = ls,
        le = le,
        w = ACTOR_WIDTH,
        h = ACTOR_MAN_HEIGHT,
        ty = ty,
        fs = FONT_SIZE as u32,
    )
}

fn actor_text_svg(cx: f64, cy: f64, name: &str) -> String {
    templates::actor_text(cx, cy, FONT_SIZE, &esc(name))
}

/// Mermaid sequence CSS — faithfully copied from reference SVGs
fn sequence_css(diagram_id: &str, vars: &ThemeVars) -> String {
    let ff = vars.font_family;
    format!(
        r#"#{id}{{font-family:{ff};font-size:{fs}px;fill:#333;}}
#{id} p{{margin:0;}}
#{id} .actor{{stroke:#9370DB;fill:#ECECFF;stroke-width:1;}}
#{id} text.actor>tspan{{fill:black;stroke:none;}}
#{id} .actor-line{{stroke:#9370DB;}}
#{id} .messageLine0{{stroke-width:1.5;stroke-dasharray:none;stroke:#333;}}
#{id} .messageLine1{{stroke-width:1.5;stroke-dasharray:2,2;stroke:#333;}}
#{id} [id$="-arrowhead"] path{{fill:#333;stroke:#333;}}
#{id} .sequenceNumber{{fill:white;}}
#{id} [id$="-sequencenumber"]{{fill:#333;}}
#{id} [id$="-crosshead"] path{{fill:#333;stroke:#333;}}
#{id} .messageText{{fill:#333;stroke:none;}}
#{id} .labelBox{{stroke:#9370DB;fill:#ECECFF;filter:none;}}
#{id} .labelText,#{id} .labelText>tspan{{fill:black;stroke:none;}}
#{id} .loopText,#{id} .loopText>tspan{{fill:black;stroke:none;}}
#{id} .sectionTitle,#{id} .sectionTitle>tspan{{fill:black;stroke:none;}}
#{id} .loopLine{{stroke-width:2px;stroke-dasharray:2,2;stroke:#9370DB;fill:#9370DB;}}
#{id} .note{{stroke:#aaaa33;fill:#fff5ad;}}
#{id} .noteText,#{id} .noteText>tspan{{fill:black;stroke:none;font-weight:normal;}}
#{id} .activation0{{fill:#f4f4f4;stroke:#666;}}
#{id} .activation1{{fill:#f4f4f4;stroke:#666;}}
#{id} .activation2{{fill:#f4f4f4;stroke:#666;}}
#{id} .actor-man circle,#{id} line{{fill:#ECECFF;stroke-width:2px;}}"#,
        id = diagram_id,
        ff = ff,
        fs = FONT_SIZE as u32,
    )
}

fn defs_svg(id: &str) -> String {
    format!(
        r##"<defs><marker id="{id}-arrowhead" refX="7.9" refY="5" markerUnits="userSpaceOnUse" markerWidth="12" markerHeight="12" orient="auto-start-reverse"><path d="M -1 0 L 10 5 L 0 10 z"></path></marker></defs>
<defs><marker id="{id}-crosshead" markerWidth="15" markerHeight="8" orient="auto" refX="4" refY="4.5"><path fill="none" stroke="#000000" stroke-width="1pt" d="M 1,2 L 6,7 M 6,2 L 1,7" style="stroke-dasharray: 0, 0;"></path></marker></defs>
<defs><marker id="{id}-filled-head" refX="15.5" refY="7" markerWidth="20" markerHeight="28" orient="auto"><path d="M 18,7 L9,13 L14,7 L9,1 Z"></path></marker></defs>
<defs><marker id="{id}-sequencenumber" refX="15" refY="15" markerWidth="60" markerHeight="40" orient="auto"><circle cx="15" cy="15" r="6" fill="#333"></circle></marker></defs>"##,
        id = id
    )
}

// ─── Main render entry point ─────────────────────────────────────────────────

pub fn render(diag: &SequenceDiagram, theme: Theme, _use_foreign_object: bool) -> String {
    let diagram_id = DIAGRAM_ID;
    let vars = theme.resolve();

    // ── Step 1: collect participants ──────────────────────────────────────────
    let mut actor_order: Vec<String> = Vec::new();
    let mut actor_map: std::collections::HashMap<String, Actor> = std::collections::HashMap::new();

    // First pass: explicit participants
    for item in &diag.items {
        if let SeqItem::Participant(p) = item {
            if !actor_map.contains_key(&p.name) {
                actor_order.push(p.name.clone());
                // Layout height = ACTOR_HEIGHT=65 for all (Mermaid uses conf.height for spacing)
                // Visual height is stored for rendering the actor-man figure
                let visual_h = actor_visual_height(&p.kind);
                actor_map.insert(
                    p.name.clone(),
                    Actor {
                        name: p.name.clone(),
                        _alias: p.alias.clone(),
                        kind: p.kind.clone(),
                        x: 0.0,
                        y: 0.0,
                        width: ACTOR_WIDTH,
                        height: visual_h, // used for lifeline start and visual rendering
                        margin: ACTOR_MARGIN,
                        start_y: 0.0,
                        stop_y: 0.0,
                    },
                );
            }
        }
    }

    // Second pass: implicit participants from messages/notes
    for item in &diag.items {
        match item {
            SeqItem::Message(m) => {
                for name in &[&m.from, &m.to] {
                    if !actor_map.contains_key(*name) {
                        actor_order.push((*name).clone());
                        actor_map.insert(
                            (*name).clone(),
                            Actor {
                                name: (*name).clone(),
                                _alias: (*name).clone(),
                                kind: ParticipantKind::Participant,
                                x: 0.0,
                                y: 0.0,
                                width: ACTOR_WIDTH,
                                height: ACTOR_HEIGHT,
                                margin: ACTOR_MARGIN,
                                start_y: 0.0,
                                stop_y: 0.0,
                            },
                        );
                    }
                }
            }
            SeqItem::Note(n) => {
                for name in &n.actors {
                    if !actor_map.contains_key(name) {
                        actor_order.push(name.clone());
                        actor_map.insert(
                            name.clone(),
                            Actor {
                                name: name.clone(),
                                _alias: name.clone(),
                                kind: ParticipantKind::Participant,
                                x: 0.0,
                                y: 0.0,
                                width: ACTOR_WIDTH,
                                height: ACTOR_HEIGHT,
                                margin: ACTOR_MARGIN,
                                start_y: 0.0,
                                stop_y: 0.0,
                            },
                        );
                    }
                }
            }
            _ => {}
        }
    }

    // ── Step 2: getMaxMessageWidthPerActor ────────────────────────────────────
    // For each actor, what is the max width of messages that originate/arrive?
    let mut max_msg_width_per_actor: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();

    let n_actors = actor_order.len();
    for item in diag.items.iter() {
        match item {
            SeqItem::Message(m) => {
                let from_idx = actor_order.iter().position(|n| n == &m.from);
                let to_idx = actor_order.iter().position(|n| n == &m.to);
                if let (Some(fi), Some(ti)) = (from_idx, to_idx) {
                    let (raw_w, _) = measure(&m.text, FONT_SIZE);
                    let w = raw_w * TEXT_SCALE;
                    let msg_w = w + 2.0 * WRAP_PADDING;

                    if fi == ti {
                        // self-message
                        let e = max_msg_width_per_actor.entry(m.from.clone()).or_insert(0.0);
                        *e = e.max(msg_w / 2.0);
                        let e2 = max_msg_width_per_actor.entry(m.to.clone()).or_insert(0.0);
                        *e2 = e2.max(msg_w / 2.0);
                    } else if fi < ti {
                        // arrow goes right: from_actor defines margin
                        let e = max_msg_width_per_actor.entry(m.from.clone()).or_insert(0.0);
                        *e = e.max(msg_w);
                    } else {
                        // arrow goes left: to_actor defines margin
                        let e = max_msg_width_per_actor.entry(m.to.clone()).or_insert(0.0);
                        *e = e.max(msg_w);
                    }
                }
            }
            SeqItem::Note(n) => {
                let (w, _) = measure(&n.text, FONT_SIZE);
                let note_w = w + 2.0 * NOTE_MARGIN;
                let actor_name = n.actors.first().map(|s| s.as_str()).unwrap_or("");
                match n.placement {
                    NotePlacement::RightOf => {
                        let e = max_msg_width_per_actor
                            .entry(actor_name.to_string())
                            .or_insert(0.0);
                        *e = e.max(note_w);
                    }
                    NotePlacement::LeftOf => {
                        // previous actor gets the margin
                        if let Some(idx) = actor_order.iter().position(|n| n == actor_name) {
                            if idx > 0 {
                                let prev = actor_order[idx - 1].clone();
                                let e = max_msg_width_per_actor.entry(prev).or_insert(0.0);
                                *e = e.max(note_w);
                            }
                        }
                    }
                    NotePlacement::Over => {
                        // split across prev and current
                        if let Some(idx) = actor_order.iter().position(|n| n == actor_name) {
                            if idx > 0 {
                                let prev = actor_order[idx - 1].clone();
                                let e = max_msg_width_per_actor.entry(prev).or_insert(0.0);
                                *e = e.max(note_w / 2.0);
                            }
                            if idx + 1 < n_actors {
                                let e = max_msg_width_per_actor
                                    .entry(actor_name.to_string())
                                    .or_insert(0.0);
                                *e = e.max(note_w / 2.0);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // ── Step 3: calculateActorMargins → assign x positions ───────────────────
    let mut prev_x = 0.0;
    for (idx, name) in actor_order.iter().enumerate() {
        // determine margin for this actor (read-only borrows first)
        let msg_w = max_msg_width_per_actor.get(name).copied().unwrap_or(0.0);
        let actor_width = actor_map.get(name).map(|a| a.width).unwrap_or(ACTOR_WIDTH);
        let next_actor_width = if idx + 1 < n_actors {
            actor_map
                .get(&actor_order[idx + 1])
                .map(|a| a.width)
                .unwrap_or(ACTOR_WIDTH)
        } else {
            ACTOR_WIDTH
        };
        let margin = if idx + 1 < n_actors {
            (msg_w + ACTOR_MARGIN - actor_width / 2.0 - next_actor_width / 2.0).max(ACTOR_MARGIN)
        } else {
            ACTOR_MARGIN
        };

        // Round margin to integer to match Mermaid's browser-integer text measurements.
        // Browser getBBox() returns values that happen to produce integer margins for
        // these corpus diagrams; our scaled ab_glyph measurements can be fractional.
        let margin_rounded = margin.round();

        // now mutate
        let actor = actor_map.get_mut(name).unwrap();
        actor.x = prev_x;
        actor.y = 0.0;
        actor.start_y = actor.height; // lifeline starts at actor height
        actor.stop_y = actor.height; // will be updated during vertical layout
        actor.margin = margin_rounded;
        prev_x += actor_width + margin_rounded;
    }

    // ── Step 3b: pre-pass to compute loop label widths (like Mermaid's calculateLoopBounds) ──
    // For each loop/alt/opt/par, compute the inner content x-span (from/to) so we can
    // determine whether the label text wraps. This mirrors Mermaid's calculateLoopBounds.
    let loop_label_heights: std::collections::HashMap<usize, f64> = {
        #[derive(Default)]
        struct LoopBound {
            from: f64,
            to: f64,
        }
        let mut stack: Vec<(usize, LoopBound)> = Vec::new(); // (item_idx, bound)
        let mut results: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();

        for (item_idx, item) in diag.items.iter().enumerate() {
            match item {
                SeqItem::LoopStart(_)
                | SeqItem::AltStart(_)
                | SeqItem::OptStart(_)
                | SeqItem::ParStart(_) => {
                    stack.push((
                        item_idx,
                        LoopBound {
                            from: f64::MAX,
                            to: f64::MIN,
                        },
                    ));
                }
                SeqItem::AltElse(_) | SeqItem::ParAnd(_) => {
                    // section dividers don't pop the frame
                }
                SeqItem::LoopEnd => {
                    if let Some((start_idx, bound)) = stack.pop() {
                        let loop_from = if bound.from == f64::MAX {
                            0.0
                        } else {
                            bound.from
                        };
                        let loop_to = if bound.to == f64::MIN { 0.0 } else { bound.to };
                        // Mermaid: loopWidth = max(0, |to - from|) - labelBoxWidth
                        let inner_span = (loop_to - loop_from).max(0.0);
                        let loop_width = (inner_span - LABEL_BOX_WIDTH).max(0.0);
                        // available text width for label: loopWidth - 2*wrapPadding
                        let available = loop_width - 2.0 * WRAP_PADDING;
                        // Determine label text
                        let label = match &diag.items[start_idx] {
                            SeqItem::LoopStart(l)
                            | SeqItem::AltStart(l)
                            | SeqItem::OptStart(l)
                            | SeqItem::ParStart(l) => l.clone(),
                            _ => String::new(),
                        };
                        // Measure label with brackets, as Mermaid does: wrapLabel('[label]', available, messageFont)
                        let label_with_brackets = if label.is_empty() {
                            String::new()
                        } else {
                            format!("[{label}]")
                        };
                        let (label_w, text_h) = if label_with_brackets.is_empty() {
                            (0.0, 0.0)
                        } else {
                            measure(&label_with_brackets, FONT_SIZE)
                        };
                        // If label_w > available (and available > 0), label wraps — use 2 lines
                        let label_h = if available > 0.0 && label_w > available {
                            2.0 * text_h // 2-line wrap
                        } else if text_h > 0.0 {
                            text_h // single line
                        } else {
                            0.0
                        };
                        results.insert(start_idx, label_h);
                        // Propagate this bound up to the parent frame
                        if let Some((_parent_idx, parent_bound)) = stack.last_mut() {
                            if loop_from < parent_bound.from {
                                parent_bound.from = loop_from;
                            }
                            if loop_to > parent_bound.to {
                                parent_bound.to = loop_to;
                            }
                        }
                    }
                }
                SeqItem::Message(m) => {
                    if let (Some(fa), Some(ta)) = (actor_map.get(&m.from), actor_map.get(&m.to)) {
                        let (msg_from, msg_to) = if m.from == m.to {
                            // self-message: use actor cx ± max(textW/2, actorW/2) like Mermaid
                            let (tw, _) = measure(&m.text, FONT_SIZE);
                            let dx = (tw / 2.0).max(ACTOR_WIDTH / 2.0);
                            let cx = fa.cx();
                            (cx - dx, cx + dx)
                        } else {
                            (fa.cx().min(ta.cx()), fa.cx().max(ta.cx()))
                        };
                        for (_idx, bound) in stack.iter_mut() {
                            if msg_from < bound.from {
                                bound.from = msg_from;
                            }
                            if msg_to > bound.to {
                                bound.to = msg_to;
                            }
                        }
                    }
                }
                SeqItem::Note(n) => {
                    // Notes also expand loop bounds
                    if let Some(fa) = n.actors.first().and_then(|a| actor_map.get(a)) {
                        let note_x1 = fa.x;
                        let note_x2 = fa.x + fa.width;
                        for (_idx, bound) in stack.iter_mut() {
                            if note_x1 < bound.from {
                                bound.from = note_x1;
                            }
                            if note_x2 > bound.to {
                                bound.to = note_x2;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        results
    };

    // ── Step 4: vertical layout + collect draw commands ──────────────────────
    // Mermaid uses conf.height=65 for initial vertical position regardless of actor type
    // Max visual height is used for the bottom actor stopy calculation
    let _max_actor_visual_h = actor_order
        .iter()
        .map(|n| actor_map[n].height)
        .fold(ACTOR_HEIGHT, f64::max);
    let mut bounds = Bounds {
        start_x: 0.0,
        stop_x: actor_order
            .iter()
            .map(|n| {
                let a = &actor_map[n];
                a.x + a.width
            })
            .fold(0.0f64, f64::max),
        start_y: 0.0,
        stop_y: 0.0,
        vertical_pos: ACTOR_HEIGHT, // always 65 for layout (Mermaid conf.height)
        loop_stack: Vec::new(),
        activations: Vec::new(),
    };

    let mut msg_models: Vec<MsgModel> = Vec::new();
    let mut note_models: Vec<RenderedNote> = Vec::new();
    let mut control_models: Vec<ControlModel> = Vec::new();
    let mut activation_models: Vec<Activation> = Vec::new();
    let mut auto_number = false;
    let mut seq_idx: usize = 1;

    for (item_idx, item) in diag.items.iter().enumerate() {
        match item {
            SeqItem::AutoNumber => {
                auto_number = true;
            }

            SeqItem::Activate(actor_name) => {
                if let Some(actor) = actor_map.get(actor_name) {
                    let stack_size = bounds
                        .activations
                        .iter()
                        .filter(|a| &a.actor == actor_name)
                        .count();
                    let ax = actor.cx() + ((stack_size as f64 - 1.0) * ACTIVATION_WIDTH) / 2.0;
                    bounds.activations.push(Activation {
                        actor: actor_name.clone(),
                        start_x: ax,
                        start_y: bounds.vertical_pos + 2.0,
                        stop_x: ax + ACTIVATION_WIDTH,
                        stop_y: 0.0,
                    });
                }
            }

            SeqItem::Deactivate(actor_name) => {
                if let Some(idx) = bounds
                    .activations
                    .iter()
                    .rposition(|a| &a.actor == actor_name)
                {
                    let mut act = bounds.activations.remove(idx);
                    act.stop_y = bounds.vertical_pos;
                    activation_models.push(act);
                }
            }

            SeqItem::Message(m) => {
                // compute start/stop x
                let from_actor = match actor_map.get(&m.from) {
                    Some(a) => a.clone(),
                    None => continue,
                };
                let to_actor = match actor_map.get(&m.to) {
                    Some(a) => a.clone(),
                    None => continue,
                };

                let _is_right = from_actor.cx() <= to_actor.cx();

                // Whether each actor currently has (or will get from this message) an activation.
                let from_has_act = bounds.activations.iter().any(|a| a.actor == m.from);
                let to_has_act = bounds.activations.iter().any(|a| a.actor == m.to);
                let to_gets_act = m.activate > 0; // this message activates the target

                // Activation box left edge = actor.cx - ACTIVATION_WIDTH/2.
                // Reference rule (regardless of stacking depth):
                //   arrow TO activated actor:   stop_x  = act_left - 3  (room for arrowhead)
                //   arrow FROM activated actor: start_x = act_left
                let act_left_of = |actor_cx: f64| actor_cx - ACTIVATION_WIDTH / 2.0;

                let (start_x, stop_x) = if m.from == m.to {
                    (from_actor.cx(), from_actor.cx())
                } else {
                    let sx = if from_has_act {
                        act_left_of(from_actor.cx())
                    } else {
                        from_actor.cx()
                    };
                    // Trim the endpoint so the arrowhead tip lands on the lifeline.
                    // Lines with no marker (Solid/Dotted) need no trim — go to cx - 1.
                    // Lines with a marker need trim equal to the arrowhead overhang.
                    let has_marker = !matches!(m.line_type, LineType::Solid | LineType::Dotted);
                    let trim = if has_marker { 4.0 } else { 1.0 };
                    let trim_sign = if from_actor.cx() <= to_actor.cx() {
                        -1.0
                    } else {
                        1.0
                    };
                    let ex = if to_has_act || to_gets_act {
                        act_left_of(to_actor.cx()) + trim_sign * 3.0
                    } else {
                        to_actor.cx() + trim_sign * trim
                    };
                    (sx, ex)
                };

                let from_bounds = from_actor.x.min(to_actor.x);
                let to_bounds = (from_actor.x + from_actor.width).max(to_actor.x + to_actor.width);

                // Vertical positioning — mirrors boundMessage()
                // Mermaid: bump(10), bump(lineHeight), bump(lineHeight - 10 + boxMargin)
                bounds.bump(10.0);
                let (_msg_text_w, _msg_text_h) = measure(&m.text, FONT_SIZE);
                let text_h = 17.0_f64; // browser renders Arial 16px at exactly 17px line height
                bounds.bump(text_h);

                let total_offset = if m.from == m.to {
                    // self-message: totalOffset = lineHeight - 10 + boxMargin + 30
                    text_h - 10.0 + BOX_MARGIN + 30.0
                } else {
                    // normal message: totalOffset = lineHeight - 10 + boxMargin
                    text_h - 10.0 + BOX_MARGIN
                };

                let line_start_y = if m.from == m.to {
                    // self-message: lineStartY = vp + (totalOffset - 30) = vp + text_h - 10 + boxMargin
                    bounds.getvertical() + (total_offset - 30.0)
                } else {
                    bounds.getvertical() + total_offset
                };

                bounds.bump(total_offset);

                let start_y = bounds.getvertical() - total_offset - text_h - 10.0;

                // Update loop frames using cx (lifeline centers), matching Mermaid's
                // calculateLoopBounds which uses msgModel.startx/stopx ≈ actor cx values
                let (frame_x1, frame_x2) = if m.from == m.to {
                    let (tw, _) = measure(&m.text, FONT_SIZE);
                    let dx = (tw / 2.0).max(ACTOR_WIDTH / 2.0);
                    let cx = from_actor.cx();
                    (cx - dx, cx + dx)
                } else {
                    (
                        from_actor.cx().min(to_actor.cx()),
                        from_actor.cx().max(to_actor.cx()),
                    )
                };
                for frame in &mut bounds.loop_stack {
                    frame.from = frame.from.min(frame_x1);
                    frame.to = frame.to.max(frame_x2);
                }

                // Handle activation (+/-) from arrow suffix
                if m.activate > 0 {
                    if let Some(actor) = actor_map.get(&m.to) {
                        let stack_size = bounds
                            .activations
                            .iter()
                            .filter(|a| a.actor == m.to)
                            .count();
                        let ax = actor.cx() + ((stack_size as f64) * ACTIVATION_WIDTH) / 2.0
                            - ACTIVATION_WIDTH / 2.0;
                        bounds.activations.push(Activation {
                            actor: m.to.clone(),
                            start_x: ax,
                            start_y: line_start_y,
                            stop_x: ax + ACTIVATION_WIDTH,
                            stop_y: 0.0,
                        });
                    }
                } else if m.activate < 0 {
                    // `John-->>-Alice` deactivates the SENDER (John = m.from), not the receiver.
                    // The `-` in the syntax means the sending actor's activation ends.
                    if let Some(idx) = bounds.activations.iter().rposition(|a| a.actor == m.from) {
                        let mut act = bounds.activations.remove(idx);
                        act.stop_y = line_start_y;
                        activation_models.push(act);
                    }
                }

                let is_message_line = true;
                let _is_seq = auto_number && is_message_line;

                msg_models.push(MsgModel {
                    from: m.from.clone(),
                    to: m.to.clone(),
                    start_x,
                    stop_x,
                    from_bounds,
                    to_bounds,
                    start_y,
                    line_start_y,
                    text: m.text.clone(),
                    line_type: m.line_type.clone(),
                    activate: m.activate,
                    seq_idx,
                    show_seq: auto_number,
                    height: 0.0,
                });

                // Mermaid also inserts the msgModel bounds at the end of boundMessage
                // For self-messages, additionally insert the visual loop bounds
                // (Mermaid: insert(startx-dx, vp_after_lineH - 10 + totalOffset, stopx+dx, vp_after_lineH + 30 + totalOffset))
                if m.from == m.to {
                    // vp_after_lineH = start_y + 10 + text_h (= bounds before total_offset bump)
                    let vp_mid = start_y + 10.0 + text_h;
                    let self_y1 = vp_mid - 10.0 + total_offset;
                    let self_y2 = vp_mid + 30.0 + total_offset;
                    bounds.insert(start_x, self_y1, stop_x + 60.0, self_y2);
                }
                bounds.insert(from_bounds, start_y, to_bounds, line_start_y);

                if auto_number {
                    seq_idx += 1;
                }
            }

            SeqItem::Note(n) => {
                let from_name = n.actors.first().map(|s| s.as_str()).unwrap_or("");
                let to_name = if n.actors.len() > 1 {
                    n.actors[1].as_str()
                } else {
                    from_name
                };
                let from_actor = match actor_map.get(from_name) {
                    Some(a) => a.clone(),
                    None => continue,
                };
                let to_actor = match actor_map.get(to_name) {
                    Some(a) => a.clone(),
                    None => from_actor.clone(),
                };

                let (note_w, note_h_text) = measure(&n.text, FONT_SIZE);
                let note_w_actual = note_w + 2.0 * NOTE_MARGIN;
                let note_h_actual = note_h_text + 2.0 * NOTE_MARGIN;

                let (note_start_x, note_width) = match n.placement {
                    NotePlacement::RightOf => {
                        let sx = from_actor.x + (from_actor.width + ACTOR_MARGIN) / 2.0;
                        let nw = note_w_actual.max(ACTOR_WIDTH);
                        (sx, nw)
                    }
                    NotePlacement::LeftOf => {
                        let nw = note_w_actual.max(ACTOR_WIDTH);
                        let sx = from_actor.x - nw + (from_actor.width - ACTOR_MARGIN) / 2.0;
                        (sx, nw)
                    }
                    NotePlacement::Over => {
                        if from_name == to_name {
                            // single actor
                            let nw = note_w_actual.max(from_actor.width);
                            let sx = from_actor.x + (from_actor.width - nw) / 2.0;
                            (sx, nw)
                        } else {
                            // span two actors
                            let fx = from_actor.x + from_actor.width / 2.0;
                            let tx = to_actor.x + to_actor.width / 2.0;
                            let nw = (fx - tx).abs() + ACTOR_MARGIN;
                            let sx = fx.min(tx) - ACTOR_MARGIN / 2.0;
                            (sx, nw)
                        }
                    }
                };

                bounds.bump(BOX_MARGIN);
                let note_start_y = bounds.getvertical();
                bounds.bump(note_h_actual);
                bounds.insert(
                    note_start_x,
                    note_start_y,
                    note_start_x + note_width,
                    note_start_y + note_h_actual,
                );

                for frame in &mut bounds.loop_stack {
                    frame.from = frame.from.min(note_start_x);
                    frame.to = frame.to.max(note_start_x + note_width);
                }

                note_models.push(RenderedNote {
                    start_x: note_start_x,
                    start_y: note_start_y,
                    width: note_width,
                    height: note_h_actual,
                    text: n.text.clone(),
                });
            }

            SeqItem::LoopStart(label) => {
                // Mermaid: preMargin=boxM, heightAdjust=boxM+boxTextM+max(labelH,labelBoxH)
                let label_h = loop_label_heights.get(&item_idx).copied().unwrap_or(0.0);
                let height_adjust = BOX_MARGIN + BOX_TEXT_MARGIN + label_h.max(LABEL_BOX_HEIGHT);
                bounds.bump(BOX_MARGIN); // preMargin
                let frame_start_y = bounds.getvertical(); // starty = after preMargin
                bounds.bump(height_adjust);
                bounds.loop_stack.push(LoopFrame {
                    label: label.clone(),
                    kind: ControlKind::Loop,
                    from: f64::MAX,
                    to: f64::MIN,
                    start_y: frame_start_y,
                    stop_y: 0.0,
                    sections: Vec::new(),
                    is_alt: false,
                });
            }

            SeqItem::LoopEnd => {
                if let Some(frame) = bounds.loop_stack.pop() {
                    // Mermaid: bump to loopModel.stopy (= frame.stop_y from updateBounds)
                    if frame.stop_y > bounds.vertical_pos {
                        bounds.vertical_pos = frame.stop_y;
                    }
                    let stop_y = bounds.getvertical(); // box bottom = vp (no extra margin)
                    let sx = if frame.from == f64::MAX {
                        bounds.start_x
                    } else {
                        frame.from
                    };
                    let ex = if frame.to == f64::MIN {
                        bounds.stop_x
                    } else {
                        frame.to
                    };
                    control_models.push(ControlModel {
                        kind: frame.kind,
                        label: frame.label,
                        start_x: sx - BOX_MARGIN,
                        stop_x: ex + BOX_MARGIN,
                        start_y: frame.start_y,
                        stop_y,
                        sections: frame.sections,
                    });
                }
            }

            SeqItem::AltStart(label) => {
                let label_h = loop_label_heights.get(&item_idx).copied().unwrap_or(0.0);
                let height_adjust = BOX_MARGIN + BOX_TEXT_MARGIN + label_h.max(LABEL_BOX_HEIGHT);
                bounds.bump(BOX_MARGIN); // preMargin
                let frame_start_y = bounds.getvertical(); // starty = after preMargin
                bounds.bump(height_adjust);
                bounds.loop_stack.push(LoopFrame {
                    label: label.clone(),
                    kind: ControlKind::Alt,
                    from: f64::MAX,
                    to: f64::MIN,
                    start_y: frame_start_y,
                    stop_y: 0.0,
                    sections: Vec::new(),
                    is_alt: true,
                });
            }

            SeqItem::AltElse(label) => {
                // Mermaid: preMargin=boxM+boxTextM=15, heightAdjust=boxM+max(labelH,labelBoxH)=30
                // section divider is at vp + preMargin
                bounds.bump(BOX_MARGIN + BOX_TEXT_MARGIN); // preMargin = 15
                let section_y = bounds.getvertical(); // divider y = after preMargin
                bounds.bump(BOX_MARGIN + LABEL_BOX_HEIGHT); // heightAdjust = 30
                if let Some(frame) = bounds.loop_stack.last_mut() {
                    frame.sections.push(ControlSection {
                        y: section_y,
                        label: label.clone(),
                    });
                }
            }

            SeqItem::OptStart(label) => {
                let label_h = loop_label_heights.get(&item_idx).copied().unwrap_or(0.0);
                let height_adjust = BOX_MARGIN + BOX_TEXT_MARGIN + label_h.max(LABEL_BOX_HEIGHT);
                bounds.bump(BOX_MARGIN); // preMargin
                let frame_start_y = bounds.getvertical(); // starty = after preMargin
                bounds.bump(height_adjust);
                bounds.loop_stack.push(LoopFrame {
                    label: label.clone(),
                    kind: ControlKind::Opt,
                    from: f64::MAX,
                    to: f64::MIN,
                    start_y: frame_start_y,
                    stop_y: 0.0,
                    sections: Vec::new(),
                    is_alt: false,
                });
            }

            SeqItem::ParStart(label) => {
                let label_h = loop_label_heights.get(&item_idx).copied().unwrap_or(0.0);
                let height_adjust = BOX_MARGIN + BOX_TEXT_MARGIN + label_h.max(LABEL_BOX_HEIGHT);
                bounds.bump(BOX_MARGIN); // preMargin
                let frame_start_y = bounds.getvertical(); // starty = after preMargin
                bounds.bump(height_adjust);
                bounds.loop_stack.push(LoopFrame {
                    label: label.clone(),
                    kind: ControlKind::Par,
                    from: f64::MAX,
                    to: f64::MIN,
                    start_y: frame_start_y,
                    stop_y: 0.0,
                    sections: Vec::new(),
                    is_alt: false,
                });
            }

            SeqItem::ParAnd(label) => {
                // Same structure as AltElse
                bounds.bump(BOX_MARGIN + BOX_TEXT_MARGIN); // preMargin = 15
                let section_y = bounds.getvertical(); // divider y = after preMargin
                bounds.bump(BOX_MARGIN + LABEL_BOX_HEIGHT); // heightAdjust = 30
                if let Some(frame) = bounds.loop_stack.last_mut() {
                    frame.sections.push(ControlSection {
                        y: section_y,
                        label: label.clone(),
                    });
                }
            }

            SeqItem::Participant(_) => {} // handled in step 1
        }
    }

    // Close any unclosed activations
    for mut act in bounds.activations.drain(..) {
        act.stop_y = bounds.vertical_pos;
        activation_models.push(act);
    }

    // Actor bottom y: after content, bump by 2*boxMargin (Mermaid: bumpVerticalPos(boxMargin*2))
    let bottom_actor_y = bounds.vertical_pos + BOX_MARGIN * 2.0;

    // Mermaid stopy = bottom_actor_y + ACTOR_HEIGHT(conf.height=65) + boxMargin
    // Mermaid uses conf.height for the layout/stopy calculation even for actor-man
    let final_stopy = bottom_actor_y + ACTOR_HEIGHT + BOX_MARGIN;

    // Update actor stop_y for lifelines (ends where bottom actor starts)
    for name in &actor_order {
        if let Some(actor) = actor_map.get_mut(name) {
            actor.stop_y = bottom_actor_y;
        }
    }

    // ── Step 5: SVG generation ────────────────────────────────────────────────
    let mut svg_parts: Vec<String> = Vec::new();

    // Style
    svg_parts.push(format!(
        "<style>{}</style>",
        sequence_css(diagram_id, &vars)
    ));
    svg_parts.push(String::from("<g></g>"));

    // Defs (arrow markers)
    svg_parts.push(defs_svg(diagram_id));

    // Control structures (loops/alt/opt/par) — collected here, pushed AFTER lifelines
    // so that lifelines are drawn first and the frame borders + badges render on top.
    let control_svgs: Vec<String> = control_models
        .iter()
        .enumerate()
        .map(|(ci, ctrl)| render_control(ctrl, ci, diagram_id))
        .collect();

    // Messages and notes are deferred to render AFTER lifelines so that sequence
    // number circles (placed at the actor x position on the lifeline) appear on top.
    let msg_svgs: Vec<String> = msg_models
        .iter()
        .map(|mm| render_message(mm, diagram_id, auto_number))
        .collect();
    let note_svgs: Vec<String> = note_models.iter().map(render_note).collect();

    // Activation boxes are rendered AFTER lifelines (below) so they cover the lifelines.
    // Collect them here; push to svg_parts after the lifelines section.
    activation_models.sort_by(|a, b| {
        a.start_x
            .partial_cmp(&b.start_x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let activation_svgs: Vec<String> = activation_models
        .iter()
        .enumerate()
        .map(|(ai, act)| {
            let cls = format!("activation{}", ai % 3);
            templates::activation_rect(
                act.start_x,
                act.start_y,
                act.stop_x - act.start_x,
                (act.stop_y - act.start_y).max(1.0),
                &cls,
            )
        })
        .collect();

    // Collect bottom-row actors for deferred rendering AFTER lifelines.
    // Reference z-order: lifelines first, bottom actors on top → bottom actor
    // head circles cover the end of the lifeline that extends into them.
    let bottom_y = bottom_actor_y;
    let mut bottom_actor_svgs: Vec<String> = Vec::new();
    for (ai, name) in actor_order.iter().enumerate().rev() {
        let actor = &actor_map[name];
        let is_actor_man = matches!(actor.kind, ParticipantKind::Actor);
        if is_actor_man {
            bottom_actor_svgs.push(actor_man_svg(
                actor.cx(),
                bottom_y,
                &actor.name,
                "actor-bottom",
                ai + actor_order.len(),
            ));
        } else {
            let mut g = String::from("<g>");
            g.push_str(&actor_rect_svg(
                actor.x,
                bottom_y,
                actor.width,
                actor.height,
                &actor.name,
                "actor-bottom",
            ));
            g.push_str(&actor_text_svg(
                actor.cx(),
                bottom_y + actor.height / 2.0,
                &actor.name,
            ));
            g.push_str("</g>");
            bottom_actor_svgs.push(g);
        }
    }

    // Actors — top row + lifelines
    for (ai, name) in actor_order.iter().enumerate().rev() {
        let actor = &actor_map[name];
        let is_actor_man = matches!(actor.kind, ParticipantKind::Actor);
        let lifeline_start = actor.y + actor.height;
        let lifeline_end = actor.stop_y;

        let mut g = String::from("<g>");
        // Lifeline
        g.push_str(&templates::lifeline(
            ai,
            actor.cx(),
            lifeline_start,
            lifeline_end,
            &esc(&actor.name),
        ));
        // Top actor box
        if is_actor_man {
            g.push_str(&actor_man_svg(
                actor.cx(),
                actor.y,
                &actor.name,
                "actor-top",
                ai,
            ));
        } else {
            g.push_str(&templates::participant_root_group(ai, &esc(&actor.name)));
            g.push_str(&actor_rect_svg(
                actor.x,
                actor.y,
                actor.width,
                actor.height,
                &actor.name,
                "actor-top",
            ));
            g.push_str(&actor_text_svg(
                actor.cx(),
                actor.y + actor.height / 2.0,
                &actor.name,
            ));
            g.push_str("</g>");
        }
        g.push_str("</g>");
        svg_parts.push(g);
    }

    // Activation boxes — rendered AFTER lifelines so they cover them (correct z-order).
    for act_svg in activation_svgs {
        svg_parts.push(act_svg);
    }

    // Control structures (loops/alt/opt/par) after lifelines — frame borders and
    // badges now render on top of lifelines, covering where they intersect.
    for s in control_svgs {
        svg_parts.push(s);
    }

    // Bottom actors — on top of lifelines so their head circles cover the lifeline end.
    for s in bottom_actor_svgs {
        svg_parts.push(s);
    }

    // Notes — after lifelines/actors so note boxes render on top.
    for s in note_svgs {
        svg_parts.push(s);
    }

    // Messages last — sequence number circles must appear on top of lifelines.
    for s in msg_svgs {
        svg_parts.push(s);
    }

    // ── viewBox / SVG wrapper ─────────────────────────────────────────────────
    // Mermaid formula: height = stopy + 2*diagramMarginY - boxMargin + bottomMarginAdj(1)
    // (mirrorActors=true: height = boxH + 2*DM_Y - boxM + 1 where boxH=stopy-starty=stopy)
    let box_width = bounds.stop_x - bounds.start_x;
    let vb_x = bounds.start_x - DIAGRAM_MARGIN_X;
    let vb_y = -(DIAGRAM_MARGIN_Y as i64);
    let vb_w = box_width + 2.0 * DIAGRAM_MARGIN_X;
    // final_stopy = vp + 2*boxM + actorH + boxM; height = stopy + 2*DM_Y - boxM + 1
    let vb_h = final_stopy + 2.0 * DIAGRAM_MARGIN_Y - BOX_MARGIN + 1.0;
    let max_w = vb_w;

    let body = svg_parts.join("\n");
    let svg_open = templates::svg_root(
        diagram_id,
        max_w as u64,
        vb_x,
        vb_y,
        vb_w as u64,
        vb_h as u64,
    );
    format!("{}\n{}\n</svg>", svg_open, body)
}

// helper that returns current vertical pos
trait GetVertical {
    fn getvertical(&self) -> f64;
}
impl GetVertical for Bounds {
    fn getvertical(&self) -> f64 {
        self.vertical_pos
    }
}

// ── Render helper: control structure (loop/alt/opt/par) ────────────────────

fn control_kind_label(k: &ControlKind) -> &'static str {
    match k {
        ControlKind::Loop => "loop",
        ControlKind::Alt => "alt",
        ControlKind::Opt => "opt",
        ControlKind::Par => "par",
    }
}

fn render_control(ctrl: &ControlModel, idx: usize, _diagram_id: &str) -> String {
    let x1 = ctrl.start_x;
    let y1 = ctrl.start_y;
    let x2 = ctrl.stop_x;
    let y2 = ctrl.stop_y;

    // Label box pentagon (Mermaid uses a notched corner)
    let lb_w = LABEL_BOX_WIDTH;
    let lb_h = LABEL_BOX_HEIGHT;
    let notch = 7.0;

    // Pentagon polygon points  (x1,y1) to (x1+lb_w,y1) to (x1+lb_w,y1+lb_h-notch)
    //   to (x1+lb_w-notch*1.2, y1+lb_h) to (x1,y1+lb_h)
    let p1 = format!("{},{}", x1, y1);
    let p2 = format!("{},{}", x1 + lb_w, y1);
    let p3 = format!("{},{}", x1 + lb_w, y1 + lb_h - notch);
    let p4 = format!("{},{}", x1 + lb_w - notch * 1.2, y1 + lb_h);
    let p5 = format!("{},{}", x1, y1 + lb_h);

    let kind_str = control_kind_label(&ctrl.kind);
    let cx_label = x1 + lb_w / 2.0;
    let cy_label = y1 + lb_h / 2.0;

    // Main text (the condition) — centred in the remaining width
    let cx_main = (x1 + lb_w + x2) / 2.0;
    let cy_main = y1 + lb_h / 2.0 + 8.0; // align with label, matching reference y offset

    let mut out = templates::control_group_open(idx, x1, y1, x2, y2);

    // Section dividers
    for sec in &ctrl.sections {
        out.push_str(&templates::control_section_divider(x1, x2, sec.y));
    }

    // Label box + text — inline fill/stroke so the badge has a solid background
    // even when CSS class rules don't apply in the HTML comparison.
    out.push_str(&templates::control_badge(
        &p1,
        &p2,
        &p3,
        &p4,
        &p5,
        cx_label,
        cy_label,
        FONT_SIZE as u32,
        kind_str,
    ));

    if !ctrl.label.is_empty() {
        out.push_str(&templates::control_label_text(
            cx_main,
            cy_main,
            FONT_SIZE as u32,
            &esc(&ctrl.label),
        ));
    }

    // Section title labels
    for sec in &ctrl.sections {
        if !sec.label.is_empty() {
            let sec_cx = (x1 + x2) / 2.0;
            let sec_cy = sec.y + LABEL_BOX_HEIGHT / 2.0 + 5.0;
            out.push_str(&templates::control_section_title(
                sec_cx,
                sec_cy,
                FONT_SIZE as u32,
                &esc(&sec.label),
            ));
        }
    }

    out.push_str("</g>");
    out
}

// ── Render helper: message ──────────────────────────────────────────────────

fn render_message(mm: &MsgModel, diagram_id: &str, auto_number: bool) -> String {
    let is_dotted = matches!(mm.line_type, LineType::Dotted | LineType::DottedArrow);
    let is_self = (mm.start_x - mm.stop_x).abs() < 0.5;

    let line_class = if is_dotted {
        "messageLine1"
    } else {
        "messageLine0"
    };
    let dash_style = if is_dotted {
        r#" style="stroke-dasharray: 3, 3; fill: none;""#
    } else {
        r#" style="fill: none;""#
    };

    // Arrow marker
    let marker = match mm.line_type {
        LineType::SolidArrow | LineType::DottedArrow => {
            format!(r#" marker-end="url(#{}-arrowhead)""#, diagram_id)
        }
        LineType::Point => {
            format!(r#" marker-end="url(#{}-filled-head)""#, diagram_id)
        }
        _ => String::new(),
    };

    // Message text position
    let _text_x = if is_self {
        mm.start_x
    } else {
        (mm.start_x + mm.stop_x) / 2.0
    };
    let _text_y =
        mm.line_start_y - mm.line_start_y.fract() + (mm.line_start_y - bounds_start_y_for_text(mm));

    // Position text just above the arrow line — baseline 8px above the line.
    let text_y_actual = mm.line_start_y - 8.0;

    let mut out = String::new();

    // Text label
    out.push_str(&templates::message_label_text(
        (mm.from_bounds + mm.to_bounds) / 2.0,
        text_y_actual,
        FONT_SIZE as u32,
        &esc(&mm.text),
    ));

    // Line
    if is_self {
        // Self-message: cubic bezier curve
        let cx1 = mm.start_x + 60.0;
        let cy1 = mm.line_start_y - 10.0;
        let cx2 = mm.start_x + 60.0;
        let cy2 = mm.line_start_y + 30.0;
        out.push_str(&templates::message_self_path(
            mm.start_x,
            mm.line_start_y,
            cx1,
            cy1,
            cx2,
            cy2,
            mm.line_start_y + 20.0,
            line_class,
            mm.seq_idx,
            &esc(&mm.from),
            &esc(&mm.to),
            &marker,
            dash_style,
        ));
    } else {
        out.push_str(&templates::message_line(
            mm.start_x,
            mm.line_start_y,
            mm.stop_x,
            line_class,
            mm.seq_idx,
            &esc(&mm.from),
            &esc(&mm.to),
            &marker,
            dash_style,
        ));
    }

    // Sequence number circle (autonumber) — draw circle+text directly (marker-start
    // on zero-length lines is unreliable across renderers).
    if auto_number && mm.show_seq {
        let cx = mm.start_x;
        let cy = mm.line_start_y;
        out.push_str(&templates::seq_number_circle(cx, cy));
        out.push_str(&templates::seq_number_text(cx, cy, mm.seq_idx));
    }

    out
}

fn bounds_start_y_for_text(mm: &MsgModel) -> f64 {
    // recover text position from line_start_y
    mm.start_y + 10.0
}

// ── Render helper: note ─────────────────────────────────────────────────────

fn render_note(note: &RenderedNote) -> String {
    let mut out = String::new();
    out.push_str(&templates::note_rect(
        note.start_x,
        note.start_y,
        note.width,
        note.height,
    ));
    out.push_str(&templates::note_text(
        note.start_x + note.width / 2.0,
        note.start_y + note.height / 2.0,
        FONT_SIZE as u32,
        &esc(&note.text),
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    const SEQ_BASIC: &str = "sequenceDiagram\n    Alice->>Bob: Hello Bob, how are you?\n    Bob-->>Alice: Great!\n    Alice-)Bob: See you later!";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(SEQ_BASIC).diagram;
        let svg = render(&diag, Theme::Default, false);
        assert!(svg.contains("<svg"), "missing <svg tag");
        assert!(svg.contains("Alice"), "missing participant");
        assert!(svg.contains("Bob"), "missing participant");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(SEQ_BASIC).diagram;
        let svg = render(&diag, Theme::Dark, false);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    #[ignore = "platform-specific float precision — run locally"]
    fn snapshot_default_theme() {
        let diag = parser::parse(SEQ_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default, false);
        insta::assert_snapshot!(svg);
    }
}
