// Faithful Rust port of mermaid/src/diagrams/cynefin/cynefinRenderer.ts
//
// Renders the Cynefin framework decision-making model as SVG:
//
//   1. Four quadrant domain background rectangles (complex, complicated, chaotic, clear)
//   2. Wavy boundary paths separating the quadrants
//   3. A cliff (S-curve) boundary between Clear and Chaotic
//   4. A central Confusion ellipse overlay
//   5. Domain name labels
//   6. Optional domain description subtitles (model + practice)
//   7. Item badges within each domain (capped at 3 for Confusion)
//   8. Transition arrows with optional labels
//   9. Diagram title
//
// Output CSS class structure mirrors cynefinRenderer.ts:
//   .cynefin-backgrounds, .cynefin-boundaries, .cynefinBoundary, .cynefinCliff,
//   .cynefinConfusion, .cynefin-labels, .cynefinDomainLabel,
//   .cynefin-subtitles, .cynefinSubtitle,
//   .cynefin-items, .cynefinItem, .cynefinItemText, .cynefinItemOverflow,
//   .cynefin-arrows, .cynefinArrowLine, .cynefinArrowLabel, .cynefinArrowHead,
//   .cynefinTitle

use super::constants::*;
use super::parser::{CynefinDiagram, CynefinDomain, DomainName};
#[allow(unused_imports)]
use super::templates;
use crate::text::measure;
use crate::theme::Theme;
use std::collections::HashMap;

// ─── Domain meta (mirrors DOMAIN_META in cynefinRenderer.ts) ─────────────────
struct DomainMeta {
    model: &'static str,
    practice: &'static str,
}

fn domain_meta(name: &DomainName) -> DomainMeta {
    match name {
        DomainName::Complex => DomainMeta {
            model: "Probe → Sense → Respond",
            practice: "Emergent Practices",
        },
        DomainName::Complicated => DomainMeta {
            model: "Sense → Analyse → Respond",
            practice: "Good Practices",
        },
        DomainName::Clear => DomainMeta {
            model: "Sense → Categorise → Respond",
            practice: "Best Practices",
        },
        DomainName::Chaotic => DomainMeta {
            model: "Act → Sense → Respond",
            practice: "Novel Practices",
        },
        DomainName::Confusion => DomainMeta {
            model: "",
            practice: "Disorder",
        },
    }
}

fn domain_bg(name: &DomainName) -> &'static str {
    match name {
        DomainName::Complex => COMPLEX_BG,
        DomainName::Complicated => COMPLICATED_BG,
        DomainName::Chaotic => CHAOTIC_BG,
        DomainName::Clear => CLEAR_BG,
        DomainName::Confusion => CONFUSION_BG,
    }
}

// ─── Layout (mirrors getDomainLayouts in cynefinRenderer.ts) ──────────────────
struct DomainLayout {
    cx: f64,
    cy: f64,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn get_domain_layouts(width: f64, height: f64) -> HashMap<String, DomainLayout> {
    let hw = width / 2.0;
    let hh = height / 2.0;
    let mut m = HashMap::new();
    m.insert(
        "complex".to_string(),
        DomainLayout {
            cx: hw / 2.0,
            cy: hh / 2.0,
            x: 0.0,
            y: 0.0,
            w: hw,
            h: hh,
        },
    );
    m.insert(
        "complicated".to_string(),
        DomainLayout {
            cx: hw + hw / 2.0,
            cy: hh / 2.0,
            x: hw,
            y: 0.0,
            w: hw,
            h: hh,
        },
    );
    m.insert(
        "chaotic".to_string(),
        DomainLayout {
            cx: hw / 2.0,
            cy: hh + hh / 2.0,
            x: 0.0,
            y: hh,
            w: hw,
            h: hh,
        },
    );
    m.insert(
        "clear".to_string(),
        DomainLayout {
            cx: hw + hw / 2.0,
            cy: hh + hh / 2.0,
            x: hw,
            y: hh,
            w: hw,
            h: hh,
        },
    );
    m.insert(
        "confusion".to_string(),
        DomainLayout {
            cx: hw,
            cy: hh,
            x: hw * 0.7,
            y: hh * 0.7,
            w: hw * 0.6,
            h: hh * 0.6,
        },
    );
    m
}

// ─── Boundary path generators ─────────────────────────────────────────────────

/// Vertical wavy boundary (fold path) through the horizontal centre.
/// Mirrors generateFoldPath in cynefinBoundaries.ts.
fn generate_fold_path(width: f64, height: f64, seed: u32, amplitude: f64) -> String {
    let amp = if amplitude == 0.0 {
        width * 0.015
    } else {
        amplitude
    };
    let cx = width / 2.0;

    // Build a series of cubic bezier segments going top to bottom
    let segments = 6usize;
    let seg_h = height / segments as f64;
    let mut rng = Mulberry32::new(seed);

    let mut d = format!("M{},{}", fmt(cx), fmt(0.0));
    for i in 0..segments {
        let y0 = i as f64 * seg_h;
        let y1 = (i + 1) as f64 * seg_h;
        let mx = cx + (rng.next_f64() * 2.0 - 1.0) * amp;
        let cp1x = cx + (rng.next_f64() * 2.0 - 1.0) * amp;
        let cp2x = cx + (rng.next_f64() * 2.0 - 1.0) * amp;
        d.push_str(&format!(
            " C{},{} {},{} {},{}",
            fmt(cp1x),
            fmt(y0 + seg_h * 0.33),
            fmt(cp2x),
            fmt(y0 + seg_h * 0.66),
            fmt(mx),
            fmt(y1),
        ));
    }
    d
}

/// Horizontal wavy boundary through the vertical centre.
/// Mirrors generateHorizontalBoundary in cynefinBoundaries.ts.
fn generate_horizontal_boundary(width: f64, height: f64, seed: u32, amplitude: f64) -> String {
    let amp = if amplitude == 0.0 {
        height * 0.015
    } else {
        amplitude
    };
    let cy = height / 2.0;

    let segments = 6usize;
    let seg_w = width / segments as f64;
    let mut rng = Mulberry32::new(seed);

    let mut d = format!("M{},{}", fmt(0.0), fmt(cy));
    for i in 0..segments {
        let x0 = i as f64 * seg_w;
        let x1 = (i + 1) as f64 * seg_w;
        let my = cy + (rng.next_f64() * 2.0 - 1.0) * amp;
        let cp1y = cy + (rng.next_f64() * 2.0 - 1.0) * amp;
        let cp2y = cy + (rng.next_f64() * 2.0 - 1.0) * amp;
        d.push_str(&format!(
            " C{},{} {},{} {},{}",
            fmt(x0 + seg_w * 0.33),
            fmt(cp1y),
            fmt(x0 + seg_w * 0.66),
            fmt(cp2y),
            fmt(x1),
            fmt(my),
        ));
    }
    d
}

/// S-curve cliff boundary between Clear (bottom-right) and Chaotic (bottom-left).
/// Mirrors generateCliffPath in cynefinBoundaries.ts.
fn generate_cliff_path(width: f64, height: f64) -> String {
    let cx = width / 2.0;
    let cy = height / 2.0;

    // S-curve from mid-bottom going up to mid-centre
    let x1 = cx;
    let y1 = height;
    let x4 = cx;
    let y4 = cy;
    let cp1x = cx - width * 0.08;
    let cp1y = cy + height * 0.25;
    let cp2x = cx + width * 0.08;
    let cp2y = cy + height * 0.1;

    format!(
        "M{},{} C{},{} {},{} {},{}",
        fmt(x1),
        fmt(y1),
        fmt(cp1x),
        fmt(cp1y),
        fmt(cp2x),
        fmt(cp2y),
        fmt(x4),
        fmt(y4)
    )
}

/// Elliptical confusion region (closed path using arcs).
/// Mirrors generateConfusionPath in cynefinBoundaries.ts.
fn generate_confusion_path(cx: f64, cy: f64, rx: f64, ry: f64) -> String {
    // Two arc commands to draw a full ellipse
    format!(
        "M{},{} A{},{} 0 1,0 {},{} A{},{} 0 1,0 {},{}",
        fmt(cx - rx),
        fmt(cy),
        fmt(rx),
        fmt(ry),
        fmt(cx + rx),
        fmt(cy),
        fmt(rx),
        fmt(ry),
        fmt(cx - rx),
        fmt(cy),
    )
}

// ─── Seeded PRNG (mulberry32, mirrors seededRandom in cynefinBoundaries.ts) ───

struct Mulberry32 {
    state: u32,
}

impl Mulberry32 {
    fn new(seed: u32) -> Self {
        Mulberry32 {
            state: seed.wrapping_add(1),
        }
    }

    fn next_u32(&mut self) -> u32 {
        let mut z = self.state.wrapping_add(0x6D2B79F5);
        self.state = z;
        z = (z ^ (z >> 15)).wrapping_mul(z | 1);
        z ^= z.wrapping_add((z ^ (z >> 7)).wrapping_mul(z | 61));
        z ^ (z >> 14)
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u32() as f64) / (u32::MAX as f64)
    }
}

/// Simple string hash for seed derivation (mirrors hashString in cynefinBoundaries.ts).
fn hash_string(s: &str) -> u32 {
    let mut hash: u32 = 0;
    for b in s.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as u32);
    }
    hash
}

// ─── Renderer ─────────────────────────────────────────────────────────────────

pub fn render(diag: &CynefinDiagram, theme: Theme) -> String {
    let vars = theme.resolve();

    let width = DIAGRAM_WIDTH;
    let height = DIAGRAM_HEIGHT;
    let padding = PADDING;
    let total_width = width + padding * 2.0;
    let total_height = height + padding * 2.0;

    // Seed from diagram title or fixed default
    let seed = hash_string(diag.title.as_deref().unwrap_or("cynefin"));
    let boundary_amplitude = BOUNDARY_AMPLITUDE_DEFAULT;
    let show_descriptions = SHOW_DOMAIN_DESCRIPTIONS;

    let layouts = get_domain_layouts(width, height);

    // Build domain lookup by name key
    let domain_by_key: HashMap<String, &CynefinDomain> = diag
        .domains
        .iter()
        .map(|d| (d.name.label().to_lowercase(), d))
        .collect();

    let mut out = String::new();

    out.push_str(&format!(
        r#"<svg id="mermaid-cynefin" xmlns="http://www.w3.org/2000/svg" width="{tw}" height="{th}" viewBox="0 0 {tw} {th}" role="graphics-document">"#,
        tw = fmt(total_width), th = fmt(total_height),
    ));

    // Accessibility
    if let Some(t) = &diag.acc_title {
        out.push_str(&format!("<title>{}</title>", esc(t)));
    }
    if let Some(d) = &diag.acc_description {
        out.push_str(&format!("<desc>{}</desc>", esc(d)));
    }

    // CSS styles (mirror CSS classes from cynefinRenderer.ts)
    out.push_str(&format!(
        r#"<style>
#mermaid-cynefin {{ font-family: {ff}; font-size: {fs}px; }}
.cynefinBoundary {{ stroke: {tc}; stroke-width: 2; opacity: 0.5; }}
.cynefinCliff {{ stroke: {tc}; stroke-width: 4; opacity: 0.7; }}
.cynefinConfusion {{ stroke: {tc}; stroke-width: 2; }}
.cynefinDomainLabel {{ font-weight: bold; font-size: {fsl}px; }}
.cynefinSubtitle {{ font-size: 11px; fill: {tc}; opacity: 0.75; }}
.cynefinItem {{ stroke: none; }}
.cynefinItemText {{ font-size: 12px; fill: {tc}; }}
.cynefinItemOverflow {{ stroke: none; opacity: 0.6; }}
.cynefinArrowLine {{ stroke: {tc}; stroke-width: 1.5; }}
.cynefinArrowLabel {{ font-size: 11px; fill: {tc}; }}
.cynefinArrowHead {{ fill: {tc}; }}
.cynefinTitle {{ font-size: 18px; font-weight: bold; fill: {title_c}; }}
</style>"#,
        ff = vars.font_family,
        fs = FONT_SIZE,
        fsl = FONT_SIZE + 2.0,
        tc = vars.text_color,
        title_c = vars.title_color,
    ));

    // Defs for arrowhead
    let marker_id = "cynefin-arrow";
    out.push_str(&format!(
        r#"<defs><marker id="{mid}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" class="cynefinArrowHead"/></marker></defs>"#,
        mid = marker_id,
    ));

    // Root group shifted by padding
    out.push_str(&format!(
        r#"<g transform="translate({p},{p})">"#,
        p = fmt(padding)
    ));

    // 1. Domain background rectangles
    out.push_str(r#"<g class="cynefin-backgrounds">"#);
    for name_str in &["complex", "complicated", "chaotic", "clear"] {
        if let Some(layout) = layouts.get(*name_str) {
            let dn = domain_name_from_key(name_str);
            let bg = domain_bg(&dn);
            out.push_str(&format!(
                r#"<rect class="cynefinDomain" x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}" fill-opacity="0.4" stroke="none"/>"#,
                x = fmt(layout.x), y = fmt(layout.y), w = fmt(layout.w), h = fmt(layout.h),
                fill = bg,
            ));
        }
    }
    out.push_str("</g>");

    // 2. Wavy boundaries
    out.push_str(r#"<g class="cynefin-boundaries">"#);
    out.push_str(&format!(
        r#"<path class="cynefinBoundary" d="{d}" fill="none"/>"#,
        d = generate_fold_path(width, height, seed, boundary_amplitude),
    ));
    out.push_str(&format!(
        r#"<path class="cynefinBoundary" d="{d}" fill="none"/>"#,
        d = generate_horizontal_boundary(width, height, seed.wrapping_add(100), boundary_amplitude),
    ));

    // 3. Cliff
    out.push_str(&format!(
        r#"<path class="cynefinCliff" d="{d}" fill="none"/>"#,
        d = generate_cliff_path(width, height),
    ));
    out.push_str("</g>");

    // 4. Confusion ellipse
    let confusion_rx = width * 0.15;
    let confusion_ry = height * 0.15;
    let cx = width / 2.0;
    let cy = height / 2.0;
    out.push_str(&format!(
        r#"<path class="cynefinConfusion" d="{d}" fill="{fill}" fill-opacity="0.5"/>"#,
        d = generate_confusion_path(cx, cy, confusion_rx, confusion_ry),
        fill = CONFUSION_BG,
    ));

    // 5. Domain labels
    out.push_str(r#"<g class="cynefin-labels">"#);
    for name_str in &["complex", "complicated", "chaotic", "clear"] {
        if let Some(layout) = layouts.get(*name_str) {
            let dn = domain_name_from_key(name_str);
            let label = dn.label();
            let label_y = if show_descriptions {
                layout.cy - 30.0
            } else {
                layout.cy
            };
            out.push_str(&format!(
                r#"<text class="cynefinDomainLabel" x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{ff}" fill="{tc}">{text}</text>"#,
                x = fmt(layout.cx), y = fmt(label_y),
                ff = vars.font_family, tc = vars.text_color,
                text = esc(label),
            ));
        }
    }
    // Confusion label
    let confusion_label_y = if show_descriptions { cy - 10.0 } else { cy };
    out.push_str(&format!(
        r#"<text class="cynefinDomainLabel" x="{cx}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{ff}" fill="{tc}">Confusion</text>"#,
        cx = fmt(cx), y = fmt(confusion_label_y),
        ff = vars.font_family, tc = vars.text_color,
    ));
    out.push_str("</g>");

    // 6. Domain description subtitles
    if show_descriptions {
        out.push_str(r#"<g class="cynefin-subtitles">"#);
        for name_str in &["complex", "complicated", "chaotic", "clear"] {
            if let Some(layout) = layouts.get(*name_str) {
                let dn = domain_name_from_key(name_str);
                let meta = domain_meta(&dn);
                if !meta.model.is_empty() {
                    out.push_str(&format!(
                        r#"<text class="cynefinSubtitle" x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{ff}">{text}</text>"#,
                        x = fmt(layout.cx), y = fmt(layout.cy - 10.0),
                        ff = vars.font_family, text = esc(meta.model),
                    ));
                }
                out.push_str(&format!(
                    r#"<text class="cynefinSubtitle" x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{ff}">{text}</text>"#,
                    x = fmt(layout.cx), y = fmt(layout.cy + 5.0),
                    ff = vars.font_family, text = esc(meta.practice),
                ));
            }
        }
        // Confusion subtitle
        out.push_str(&format!(
            r#"<text class="cynefinSubtitle" x="{cx}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{ff}">Disorder</text>"#,
            cx = fmt(cx), y = fmt(cy + 8.0),
            ff = vars.font_family,
        ));
        out.push_str("</g>");
    }

    // 7. Item badges
    let item_height = 26.0;
    let item_padding_x = 10.0;
    out.push_str(r#"<g class="cynefin-items">"#);

    let all_domain_keys = ["complex", "complicated", "chaotic", "clear", "confusion"];
    for &name_str in &all_domain_keys {
        let domain = match domain_by_key.get(name_str) {
            Some(d) if !d.items.is_empty() => *d,
            _ => continue,
        };
        let layout = match layouts.get(name_str) {
            Some(l) => l,
            None => continue,
        };
        let is_confusion = name_str == "confusion";

        let items_to_render;
        let overflow_count;
        if is_confusion && domain.items.len() > MAX_CONFUSION_ITEMS {
            overflow_count = domain.items.len() - MAX_CONFUSION_ITEMS;
            items_to_render = &domain.items[..MAX_CONFUSION_ITEMS];
        } else {
            overflow_count = 0;
            items_to_render = &domain.items[..];
        }

        let start_y = if is_confusion {
            let label_offset = if show_descriptions { 22.0 } else { 14.0 };
            layout.cy + label_offset
        } else {
            layout.cy + if show_descriptions { 25.0 } else { 15.0 }
        };

        let bg = domain_bg(&domain.name);

        for (idx, item) in items_to_render.iter().enumerate() {
            let item_y = start_y + idx as f64 * (item_height + 4.0);
            let (tw, _) = measure(&item.label, 12.0);
            let measured_width = (tw).max(item.label.len() as f64 * 7.0);
            let badge_w = measured_width + item_padding_x * 2.0;
            let item_x = layout.cx - badge_w / 2.0;
            let text_x = item_x + badge_w / 2.0;
            let text_y = item_y + item_height / 2.0;

            out.push_str(&format!(
                r#"<g><rect class="cynefinItem" x="{x}" y="{y}" width="{w}" height="{h}" rx="4" ry="4" fill="{fill}" fill-opacity="0.95"/><text class="cynefinItemText" x="{tx}" y="{ty}" text-anchor="middle" dominant-baseline="central" font-family="{ff}">{text}</text></g>"#,
                x = fmt(item_x), y = fmt(item_y), w = fmt(badge_w), h = fmt(item_height),
                fill = bg, tx = fmt(text_x), ty = fmt(text_y),
                ff = vars.font_family, text = esc(&item.label),
            ));
        }

        // Overflow badge
        if overflow_count > 0 {
            let overflow_y = start_y + items_to_render.len() as f64 * (item_height + 4.0);
            let overflow_label = format!("+{} more", overflow_count);
            let (tw, _) = measure(&overflow_label, 12.0);
            let measured_width = tw.max(overflow_label.len() as f64 * 7.0);
            let badge_w = measured_width + item_padding_x * 2.0;
            let item_x = layout.cx - badge_w / 2.0;
            let text_x = item_x + badge_w / 2.0;
            let text_y = overflow_y + item_height / 2.0;

            out.push_str(&format!(
                r#"<g><rect class="cynefinItemOverflow" x="{x}" y="{y}" width="{w}" height="{h}" rx="4" ry="4" fill="{fill}" fill-opacity="0.6"/><text class="cynefinItemText" x="{tx}" y="{ty}" text-anchor="middle" dominant-baseline="central" font-family="{ff}">{text}</text></g>"#,
                x = fmt(item_x), y = fmt(overflow_y), w = fmt(badge_w), h = fmt(item_height),
                fill = bg, tx = fmt(text_x), ty = fmt(text_y),
                ff = vars.font_family, text = esc(&overflow_label),
            ));
        }
    }
    out.push_str("</g>");

    // 8. Transition arrows
    if !diag.transitions.is_empty() {
        out.push_str(r#"<g class="cynefin-arrows">"#);
        for trans in &diag.transitions {
            let from_key = trans.from.label().to_lowercase();
            let to_key = trans.to.label().to_lowercase();
            let from_layout = match layouts.get(&from_key) {
                Some(l) => l,
                None => continue,
            };
            let to_layout = match layouts.get(&to_key) {
                Some(l) => l,
                None => continue,
            };

            let x1 = from_layout.cx;
            let y1 = from_layout.cy;
            let x2 = to_layout.cx;
            let y2 = to_layout.cy;

            // Quadratic bezier with perpendicular offset
            let mx = (x1 + x2) / 2.0;
            let my = (y1 + y2) / 2.0;
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            let offset = len * 0.15;
            let nx = -dy / len;
            let ny = dx / len;
            let cpx = mx + nx * offset;
            let cpy = my + ny * offset;

            out.push_str(&format!(
                r#"<path class="cynefinArrowLine" d="M{x1},{y1} Q{cpx},{cpy} {x2},{y2}" fill="none" marker-end="url(#{mid})"/>"#,
                x1 = fmt(x1), y1 = fmt(y1),
                cpx = fmt(cpx), cpy = fmt(cpy),
                x2 = fmt(x2), y2 = fmt(y2),
                mid = marker_id,
            ));

            if let Some(label) = &trans.label {
                out.push_str(&format!(
                    r#"<text class="cynefinArrowLabel" x="{x}" y="{y}" text-anchor="middle" dominant-baseline="auto" font-family="{ff}">{text}</text>"#,
                    x = fmt(cpx), y = fmt(cpy - 6.0),
                    ff = vars.font_family, text = esc(label),
                ));
            }
        }
        out.push_str("</g>");
    }

    // 9. Title
    if let Some(t) = &diag.title {
        out.push_str(&format!(
            r#"<text class="cynefinTitle" x="{cx}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{ff}">{text}</text>"#,
            cx = fmt(width / 2.0),
            y = fmt(-padding / 2.0),
            ff = vars.font_family,
            text = esc(t),
        ));
    }

    out.push_str("</g>"); // root translate group
    out.push_str("</svg>");
    out
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn domain_name_from_key(s: &str) -> DomainName {
    DomainName::from_str(s).unwrap_or(DomainName::Confusion)
}

fn fmt(v: f64) -> String {
    let s = format!("{:.3}", v);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    const CYNEFIN_BASIC: &str = "cynefin\n    title My Cynefin\n    complex\n        item \"Explore options\"\n    complicated\n        item \"Analyze\"\n    clear\n        item \"Follow process\"";

    #[test]
    fn basic_render_produces_svg() {
        let diag = parser::parse(CYNEFIN_BASIC).diagram;
        let svg = render(&diag, Theme::Default);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn dark_theme() {
        let diag = parser::parse(CYNEFIN_BASIC).diagram;
        let svg = render(&diag, Theme::Dark);
        assert!(svg.contains("<svg"), "missing <svg tag");
    }

    #[test]
    fn snapshot_default_theme() {
        let diag = parser::parse(CYNEFIN_BASIC).diagram;
        let svg = render(&diag, crate::theme::Theme::Default);
        insta::assert_snapshot!(crate::svg::normalize_floats(&svg));
    }
}
