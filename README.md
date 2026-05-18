# ariel-rs

[![CI](https://github.com/rinfimate/ariel-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/rinfimate/ariel-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ariel-rs.svg)](https://crates.io/crates/ariel-rs)
[![docs.rs](https://docs.rs/ariel-rs/badge.svg)](https://docs.rs/ariel-rs)
[![Coverage](https://codecov.io/gh/rinfimate/ariel-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/rinfimate/ariel-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A faithful Rust port of [Mermaid JS](https://mermaid.js.org/) — renders diagrams to SVG without a browser, Node.js, or any JavaScript runtime.

## Features

- **29 diagram types** — flowchart, sequence, class, state, ER, gantt, git, pie, mindmap, timeline, quadrant, xychart, C4, block, packet, journey, requirement, kanban, sankey, treemap, radar, venn, architecture, eventmodeling, cynefin, ishikawa, wardley, zenuml, railroad
- **Headless** — pure Rust, no DOM, no JS runtime
- **Fast** — renders diagrams in microseconds
- **Accurate** — pixel-tested against Mermaid JS reference output (71-diagram visual regression suite)
- **Themes** — Default, Dark, Forest, Neutral

## Usage

```toml
[dependencies]
ariel-rs = "0.1.0"
```

```rust
use ariel_rs::{render, theme::Theme};

let svg = render("flowchart TD\n    A --> B --> C", Theme::Default);
println!("{}", svg);
```

## Supported diagram types

| Type | Keyword |
|------|---------|
| Flowchart | `flowchart` / `graph` |
| Sequence | `sequenceDiagram` |
| Class | `classDiagram` |
| State | `stateDiagram-v2` |
| Entity Relationship | `erDiagram` |
| Gantt | `gantt` |
| Git | `gitGraph` |
| Pie | `pie` |
| Mindmap | `mindmap` |
| Timeline | `timeline` |
| Quadrant | `quadrantChart` |
| XY Chart | `xychart-beta` |
| C4 | `C4Context` / `C4Container` / … |
| Block | `block-beta` |
| Packet | `packet-beta` |
| Journey | `journey` |
| Requirement | `requirementDiagram` |
| Kanban | `kanban` |
| Sankey | `sankey-beta` |
| Treemap | `treemap` |
| Radar | `radar` |
| Venn | `venn` |
| Architecture | `architecture-beta` |
| Event Modeling | `eventmodeling` |
| Cynefin | `cynefin` |
| Ishikawa | `fishbone` / `ishikawa` |
| Wardley | `wardley` |
| ZenUML | `zenuml` |
| Railroad | `railroad-beta` |

## Error handling

If the input is invalid or the diagram type is unrecognised, `render()` returns a styled error SVG matching Mermaid's browser error format — it never panics.

## Themes

```rust
use ariel_rs::{render, theme::Theme};

let svg = render(source, Theme::Dark);
```

Available themes: `Theme::Default`, `Theme::Dark`, `Theme::Forest`, `Theme::Neutral`.

## Development

### Run tests

```sh
cargo test
```

### Run tests with coverage

```sh
cargo install cargo-tarpaulin
cargo tarpaulin --out Stdout
```

### Visual regression

Requires Node.js.

```sh
cd visual-regression
npm install                                   # first time only
cargo run --bin render_corpus --release        # generate Rust SVGs
node svg_to_png.mjs rust                      # convert to PNG
node compare.mjs                              # compare vs reference
```

## Architecture

ariel-rs uses [dagre-dgl-rs](https://crates.io/crates/dagre-dgl-rs) for directed graph layout (flowcharts, class diagrams, state diagrams, ER diagrams) and implements all other diagram types from scratch.

## Dependencies

| Crate | Purpose |
|-------|---------|
| [`dagre-dgl-rs`](https://crates.io/crates/dagre-dgl-rs) | Directed graph layout |
| [`ab_glyph`](https://crates.io/crates/ab_glyph) | Font metrics for text measurement |
| [`indexmap`](https://crates.io/crates/indexmap) | Deterministic map iteration |
| [`serde_json`](https://crates.io/crates/serde_json) | JSON parsing (corpus loader) |
| [`chrono`](https://crates.io/crates/chrono) | Date handling for gantt diagrams |

## License

MIT © 2026 Rochanglien Infimate
