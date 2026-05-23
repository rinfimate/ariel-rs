// dagre/index.js render(): nodesep=50, ranksep=50, marginx=8, marginy=8
pub const NODE_SEP: f64 = 50.0;
pub const RANK_SEP: f64 = 50.0;
pub const MARGIN: f64 = 8.0;
pub const FONT_SIZE: f64 = 16.0;
pub const LABEL_SCALE: f64 = 1.117;
pub const NODE_PADDING: f64 = 8.0;
pub const START_RADIUS: f64 = 7.0;
pub const END_OUTER_RADIUS: f64 = 7.0;
pub const END_INNER_RADIUS: f64 = 2.5;
pub const FORK_JOIN_WIDTH: f64 = 70.0;
pub const FORK_JOIN_HEIGHT: f64 = 10.0;
pub const CHOICE_SIZE: f64 = 15.0;
// Note padding: Mermaid uses flowchart.padding=15 for note boxes (ref 181×54px)
pub const NOTE_PADDING: f64 = 15.0;
// NOTE_HEIGHT = text_line_height(24) + 2*NOTE_PADDING = 54
pub const NOTE_HEIGHT: f64 = 54.0;
pub const NOTE_MIN_WIDTH: f64 = 100.0;
pub const CLUSTER_LABEL_H: f64 = 34.0; // used for apply_title_offset on main-graph compounds
pub const CLUSTER_TITLE_AREA: f64 = 26.0; // ref: label_height(24) + 2 = inner_rect offset from outer rect
#[allow(dead_code)]
pub const CLUSTER_PAD: f64 = 8.0;
// dagre/index.js recursiveRender: node.graph.ranksep = parent_ranksep + 25
// Main=50 → LR concurrent sub-graph=75 → TB divider sub-sub-graph=100
// SUB_RANK_SEP is ranksep inside divider content sub-graphs
pub const SUB_RANK_SEP: f64 = 100.0;
pub const SUB_NODE_SEP: f64 = 50.0;
// Concurrent sub-graph margins (from reference: translate(35,37.5) within Concurrent cluster)
// marginx=35 gives correct left/right padding; marginy=19.5 gives 11.5px top padding
pub const CONCURRENT_MARGINY: f64 = 19.5;
pub const CONCURRENT_MARGINX: f64 = 35.0;
// dagre-dgl-rs computes concurrent compound height 6px larger than dagre-d3-es
// Subtract to match reference outer height exactly
pub const CONCURRENT_HEIGHT_ADJUST: f64 = -6.0;
// Composite state sub-graph margins (from reference state_composite: outer w=137.59, h=293)
// marginx=35 matches outer_w=node_w+2*marginx=67.59+70=137.59
// marginy=11.5 matches reference top padding (inner_top to first_node_top = 11.5px)
// COMPOSITE_BOTTOM_EXT=22 adds extra bottom space so inner_h=content+23+22=263 matches reference
pub const COMPOSITE_MARGINX: f64 = 35.0;
pub const COMPOSITE_MARGINY: f64 = 11.5;
pub const COMPOSITE_BOTTOM_EXT: f64 = 22.0;
// Rect-edge gap between concurrent divider boxes (reference = 50px)
pub const CONCURRENT_DIV_GAP: f64 = 50.0;
