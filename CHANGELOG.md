# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/rinfimate/ariel-rs/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/rinfimate/ariel-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/rinfimate/ariel-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/rinfimate/ariel-rs/releases/tag/v0.1.0
