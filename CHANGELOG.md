# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/rinfimate/ariel-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/rinfimate/ariel-rs/releases/tag/v0.1.0
