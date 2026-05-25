# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-05-25

### Added
- **Fidelity mode** (`--features fidelity`): new backends abstraction with
  `BrowserMeasurer` (Puppeteer + Chrome `getBBox()`) and `DagreJsLayouter`
  (Node oracle wrapping `dagre-d3-es`) for pixel-accurate parity with Mermaid JS
- Visual regression pipeline expanded with fidelity audit scripts
- `dagre_oracle.mjs`: post-process intersection clipping for diamond / ellipse / circle nodes

### Changed
- **Theme externalization**: every renderer now threads `vars.font_family`
  through to its templates — `label_tspan` / `label_tspan_raw` and all SVG-root
  / label functions across 26 modules take a `font_family` parameter instead of
  hardcoding `"Arial, sans-serif"`
- **Templates refactor**: all inline SVG `format!()` strings moved out of
  `renderer.rs` files into `templates.rs` (architecture, block, sequence,
  ishikawa, class_diagram, state)
- **Constants extraction**: Mermaid-canonical hardcoded colours pulled into
  `constants.rs` (class_diagram `NAMESPACE_FILL/STROKE`, `NOTE_FILL/STROKE`,
  `SHADOW_FLOOD_COLOR`; state `DIVIDER_FILL`)
- Text rendering switched to Mermaid's `tspan`-with-`dy` pattern across all
  renderers (fidelity + production)
- Edge label height calibrated to `FONT_SIZE * 1.3125` (= 21 for 16 px) to
  match Mermaid's browser line-height
- All renderer height constants use `lineHeight = 1.1` to match Mermaid
- Removed all `ab_glyph` scale factors — text widths now come from
  pre-tabulated Arial metrics (`text_browser_metrics.rs`)
- Mindmap layout: replaced slot-based `count_leaves × NODE_SLOT` with a
  recursive subtree-height algorithm that generalises to circle nodes

### Fixed
- Class diagram: pre-compute namespace inner layout (matches Mermaid's
  `adjustClustersAndEdges`); correct empty-class height (79 → 77); header text
  positioned at `top + 20` (not `HEADER_H / 2`); cluster top-alignment + visual
  padding; route edge labels through backends; edge label / arrow alignment
- Flowchart: hexagon padding 8 → 15 (Mermaid default); preserve `fa:fa-xxx`
  literal in text; +2 px edge-label background padding; bidirectional `minlen`
  match
- Requirement: header layout (`HEADER = 66`, first row at `top + 8.5`); edge
  label font size
- Sequence: `box_bottom_extra` 20 → 10 to match Mermaid's viewBox formula
- Kanban: section-2 header text white on purple background (default theme);
  single-line item card height exactly equals `ITEM_HEIGHT` (37)
- State diagram: edge labels, fork/join bar, divider alignment; concurrent
  cluster height
- ER: relationship label positioning; self-loop edge label rendering
- `measure_oracle` uses SVG `<text>` `getBBox()` (not HTML span) for accurate
  measurement parity
- Removed unused `PAD_Y` constant that broke CI under `-D warnings`

### Internal
- All snapshots regenerated; visual regression stable at **PASS 80 / WARN 17 /
  FAIL 3** (mindmap force-directed cases + 2 near-threshold WARNs)
- 261 lib tests pass; `cargo clippy --release --lib --all-targets` clean

## [0.3.0] - 2026-05-22

### Added
- Visual regression suite expanded to 100 diagrams across all 4 themes
- `render_rsvg.mjs` script: resvg-based rasterization for all themes → `rsvg_output/{theme}/`

### Fixed
- ER self-loop arc shape now matches Mermaid JS exactly — root cause was wrong virtual node size (10×10 → 0.1×0.1) and missing edge label width on the mid segment
- Treemap: dark theme leaf labels use `lightgrey` (cScaleLabel), neutral section labels use `#F4F4F4` on dark fills, forest labels use `black`
- Block diagram: dark theme text color `#F9FFFE`, vertical centering via `dominant-baseline=middle`
- All SVG output is pure SVG — no `<foreignObject>`, rasterizable by any tool (resvg, librsvg, Inkscape)
- `cargo clippy -- -D warnings` compliance: removed dead code, fixed boolean expressions, manual strip, unwrap-after-is_some

### Changed
- ER diagram: `EDGESEP` corrected to Mermaid's actual default (20), cyclic-special virtual nodes set to 0.1×0.1

## [0.2.0] - 2026-05-20

### Added
- Full 4-theme support (Default, Dark, Forest, Neutral) with pixel-accurate colors for all 28+ diagram types
- Theme-aware color fixes across sequence, flowchart, class, state, ER, gantt, git, block, treemap, kanban, packet, sankey, radar, timeline, C4, architecture, ishikawa, wardley, mindmap, pie, venn
- `ThemeVars` struct with 180+ resolved color fields used throughout all renderers
- Visual regression pipeline: `run_regression_all.mjs` runs all 4 themes end-to-end
- Per-theme HTML comparison reports (`compare_{theme}.html`)
- TreeView and treeview-beta diagram support

### Fixed
- Sequence: arrow/arrowhead color, actor-man head/text, autonumber circle fill
- State: edge label background, fork/join line color, composite cluster border
- ER: relationship label background, entity box colors
- Block: nested group layout, container fill/stroke, group borders, style directives
- Gantt: tick lines, active/activeCrit text color, section fills, edge label background
- Flowchart: cylinder/asymmetric label positions
- Class: terminal label positions, cardinality labels, annotation text

## [0.1.0] - 2026-05-18

### Added
- Initial release — faithful Rust port of [Mermaid JS](https://mermaid.js.org/) producing SVG without a browser or Node.js
- 23 diagram types: flowchart, sequence, class, state, ER, gantt, git, pie, mindmap, timeline, quadrant, xychart, C4, block, packet, journey, requirement, kanban, sankey, treemap, radar, venn, architecture, eventmodeling, cynefin, ishikawa, wardley, zenuml, railroad
- Headless SVG rendering — no DOM, no JavaScript runtime required
- Dagre graph layout via [dagre-dgl-rs](https://crates.io/crates/dagre-dgl-rs)
- Theme support: Default, Dark, Forest, Neutral
- Visual regression test suite: 71 diagrams, pixel-accurate comparison against Mermaid JS reference output
- Error SVG output matching Mermaid's browser error format when given invalid input
- Public API: `render(input, theme) -> String` and `render_svg(input, theme) -> String`
- MIT licence

[Unreleased]: https://github.com/rinfimate/ariel-rs/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/rinfimate/ariel-rs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/rinfimate/ariel-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/rinfimate/ariel-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/rinfimate/ariel-rs/releases/tag/v0.1.0
