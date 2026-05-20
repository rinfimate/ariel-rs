// State diagram constants — from Mermaid defaultConfig.state + stateRenderer-v3-unified.ts

// ── Typography ────────────────────────────────────────────────────────────────
pub const FONT_SIZE: f64 = 16.0;
pub const LABEL_SCALE: f64 = 1.117; // Liberation Sans → browser Arial

// ── Node geometry ─────────────────────────────────────────────────────────────
pub const NODE_PADDING: f64 = 8.0; // state.padding
#[allow(dead_code)]
pub const NOTE_PADDING: f64 = 16.0; // groupData.padding in dataFetcher.ts
#[allow(dead_code)]
pub const NOTE_LABEL_PADDING: f64 = 15.0; // note.ts node.padding
pub const NODE_H: f64 = 40.0;
pub const NOTE_H: f64 = 54.0; // note.ts totalHeight for single-line
pub const START_R: f64 = 7.0; // sizeUnit
pub const END_OUTER_R: f64 = 7.0; // stateEnd.ts: r=7
pub const END_INNER_R: f64 = 2.5;
pub const FORK_W: f64 = 70.0;
pub const FORK_H: f64 = 10.0; // forkJoin.ts: Math.max(10, node.height)
pub const CHOICE_R: f64 = 14.0;

// ── Dagre layout ──────────────────────────────────────────────────────────────
pub const RANKSEP: f64 = 50.0;
pub const NODESEP: f64 = 50.0;
pub const MARGIN: f64 = 8.0;

// Inner (composite state) dagre layout
pub const INNER_RANKSEP: f64 = 75.0;
pub const INNER_NODESEP: f64 = 50.0;
pub const INNER_MARGINX: f64 = 43.0; // measured from reference SVG
pub const INNER_MARGINY: f64 = 45.5;

// ── Composite state rendering ─────────────────────────────────────────────────
pub const CLUSTER_PADDING: f64 = 8.0; // state.padding applied around cluster rects
pub const CLUSTER_TITLE_H: f64 = 24.0;

// ── Colours (note box — fixed, not theme-derived) ─────────────────────────────
pub const NOTE_FILL: &str = "#fff5ad";
pub const NOTE_STROKE: &str = "#aaaa33";

// ── SVG id ────────────────────────────────────────────────────────────────────
pub const SVG_ID: &str = "mermaid-svg";
