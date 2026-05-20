// ER diagram constants — port of erRenderer-unified.ts + erBox.ts (Mermaid v11)

// ── Typography ────────────────────────────────────────────────────────────────
pub const FONT_SIZE: f64 = 16.0; // global CSS font-size
pub const FO_H: f64 = 24.0; // font_size * line-height(1.5)
pub const REL_FONT_SIZE: f64 = 14.0;
pub const TEXT_SCALE: f64 = 1.117;

// ── Entity geometry ───────────────────────────────────────────────────────────
pub const DIAGRAM_PADDING: f64 = 20.0; // conf.diagramPadding (used for no-attr padding)
pub const ENTITY_PADDING: f64 = 15.0; // conf.entityPadding
pub const MIN_ENTITY_W: f64 = 100.0; // conf.minEntityWidth
pub const MIN_ENTITY_H: f64 = 75.0; // conf.minEntityHeight

// erBox.ts: !htmlLabels branch: TEXT_PADDING = entityPadding * 1.25
pub const TEXT_PADDING: f64 = ENTITY_PADDING * 1.25; // = 18.75, per row (both sides)
                                                     // erBox.ts: !htmlLabels branch: PADDING = diagramPadding * 1.25
pub const ATTR_PADDING: f64 = DIAGRAM_PADDING * 1.25; // = 25.0, per column margin

// Row heights:
pub const ROW_H: f64 = FO_H + TEXT_PADDING; // = 42.75 (for both name row and attr rows)

// ── Dagre layout ──────────────────────────────────────────────────────────────
// Dagre params: marginx/marginy=8 (from setupViewPortForSVG padding=8)
pub const MARGINX: f64 = 8.0;
pub const MARGINY: f64 = 8.0;
// erRenderer-unified: nodeSpacing=140, rankSpacing=80+edge_label_height(21)=101
pub const NODESEP: f64 = 140.0;
pub const RANKSEP: f64 = 101.0;
pub const EDGESEP: f64 = 100.0;

// ── ViewBox ───────────────────────────────────────────────────────────────────
// ViewBox padding (from setupViewPortForSVG call: padding=8)
pub const VB_PAD: f64 = 8.0;
