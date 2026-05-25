// ER diagram constants — port of erRenderer-unified.ts + erBox.ts (Mermaid v11)
#![allow(dead_code)]

// ── Typography ────────────────────────────────────────────────────────────────
/// Font size used for ER entity and attribute text measurement (px).
/// defaultConfig.er.fontSize = 12, but Mermaid renders at the global CSS
/// font-size of 16px — erBox.ts uses foreignObject whose text inherits 16px.
/// We measure at 16 to match actual browser-rendered box widths and row heights.
pub const FONT_SIZE: f64 = 16.0;
/// SVG text height for one line. Calibrated from ref/er_attributes.svg:
/// row spacing = 35.75 = FO_H + TEXT_PADDING(18.75) → FO_H = 17.
/// This is the empirical text bbox.height for Arial 16px in Chromium.
pub const FO_H: f64 = 17.0;
/// Font size for relationship labels — .edgeLabel .label{font-size:14px} in Mermaid CSS.
pub const REL_FONT_SIZE: f64 = 14.0;
// ── Entity geometry ───────────────────────────────────────────────────────────
/// Diagram padding (conf.diagramPadding = 20, px).
pub const DIAGRAM_PADDING: f64 = 20.0;
/// Entity padding (conf.entityPadding = 15, px).
pub const ENTITY_PADDING: f64 = 15.0;
/// Minimum entity width (conf.minEntityWidth = 100, px).
pub const MIN_ENTITY_W: f64 = 100.0;
/// Minimum entity height (conf.minEntityHeight = 75, px).
pub const MIN_ENTITY_H: f64 = 75.0;

// erBox.ts: !htmlLabels branch: TEXT_PADDING = entityPadding * 1.25
pub const TEXT_PADDING: f64 = ENTITY_PADDING * 1.25; // = 18.75
                                                     // erBox.ts: !htmlLabels branch: PADDING = diagramPadding * 1.25
pub const ATTR_PADDING: f64 = DIAGRAM_PADDING * 1.25; // = 25.0

// Row heights:
pub const ROW_H: f64 = FO_H + TEXT_PADDING; // = 42.75

// ── Dagre layout ──────────────────────────────────────────────────────────────
// Dagre params: marginx/marginy=8 (from setupViewPortForSVG padding=8)
pub const MARGINX: f64 = 8.0;
pub const MARGINY: f64 = 8.0;
/// Node separation. Matches erRenderer-unified.ts nodeSpacing = 140.
pub const NODESEP: f64 = 140.0;
/// Rank separation. erRenderer-unified.ts rankSpacing=80; make_space_for_edge_labels
/// halves it to 40, but the label proxy adds ~21px giving ~101px effective gap.
pub const RANKSEP: f64 = 101.0;
/// Edge separation. Mermaid default edgeSep=20.
pub const EDGESEP: f64 = 20.0;

// ── ViewBox ───────────────────────────────────────────────────────────────────
// ViewBox padding (from setupViewPortForSVG call: padding=8)
pub const VB_PAD: f64 = 8.0;
