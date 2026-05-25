//! Swappable backends for text measurement and graph layout.
//!
//! **Production** (always active): `AbGlyphMeasurer` + `DagreDglRsLayouter`.
//!
//! **Fidelity mode** (`--features fidelity`, never in release builds):
//! swap in `BrowserMeasurer` and `DagreJsLayouter` to route through live
//! Node.js oracle processes.  This lets us isolate how much of the PDIFF gap
//! versus Mermaid JS is caused by font/layout differences vs SVG-structure.
//!
//! All renderers call [`measure`] and [`layout`] through this module rather
//! than importing from `crate::text` or `dagre_dgl_rs` directly.

use dagre_dgl_rs::graph::Graph;
use std::cell::RefCell;

// ---------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------

/// Text measurement backend.
pub trait Measurer {
    /// Returns `(width_px, height_px)` for `text` rendered at `font_size` px.
    /// `bold` is honoured by the browser backend; ignored by ab_glyph.
    fn measure(&self, text: &str, font_size: f64, bold: bool) -> (f64, f64);
}

/// Dagre graph layout backend.
pub trait Layouter {
    /// Run the layout algorithm on `graph`, writing positions back in-place.
    fn layout(&self, graph: &mut Graph);
}

// ---------------------------------------------------------------------------
// Production backend: ab_glyph
// ---------------------------------------------------------------------------

struct AbGlyphMeasurer;

impl Measurer for AbGlyphMeasurer {
    fn measure(&self, text: &str, font_size: f64, bold: bool) -> (f64, f64) {
        // Use pre-tabulated Arial metrics (same font Mermaid JS renders with).
        // Bold text is ~8.2% wider than regular (ratio of NAME_SCALE/CONTENT_SCALE).
        let (w, h) = crate::text_browser_metrics::measure_browser(text, font_size);
        if bold {
            (w * 1.082, h)
        } else {
            (w, h)
        }
    }
}

// ---------------------------------------------------------------------------
// Production backend: dagre-dgl-rs
// ---------------------------------------------------------------------------

struct DagreDglRsLayouter;

impl Layouter for DagreDglRsLayouter {
    fn layout(&self, graph: &mut Graph) {
        dagre_dgl_rs::layout::layout(graph);
    }
}

// ---------------------------------------------------------------------------
// Thread-local active backends
// ---------------------------------------------------------------------------

thread_local! {
    static MEASURER: RefCell<Box<dyn Measurer>> =
        RefCell::new(Box::new(AbGlyphMeasurer));
    static LAYOUTER: RefCell<Box<dyn Layouter>> =
        RefCell::new(Box::new(DagreDglRsLayouter));
}

/// Measure text using the active backend.
/// Drop-in replacement for `crate::text::measure`.
pub fn measure(text: &str, font_size: f64) -> (f64, f64) {
    MEASURER.with(|m| m.borrow().measure(text, font_size, false))
}

/// Measure bold text using the active backend.
#[allow(dead_code)]
pub fn measure_bold(text: &str, font_size: f64) -> (f64, f64) {
    MEASURER.with(|m| m.borrow().measure(text, font_size, true))
}

/// Run dagre layout using the active backend.
/// Drop-in replacement for `dagre_dgl_rs::layout::layout`.
pub fn layout(graph: &mut Graph) {
    LAYOUTER.with(|l| l.borrow().layout(graph));
}

// ---------------------------------------------------------------------------
// Fidelity backend management (only with `--features fidelity`)
// ---------------------------------------------------------------------------

/// Replace the active measurer. Call once before rendering in fidelity mode.
#[cfg(feature = "fidelity")]
#[allow(dead_code)]
pub fn set_measurer(m: Box<dyn Measurer>) {
    MEASURER.with(|s| *s.borrow_mut() = m);
}

/// Replace the active layouter. Call once before rendering in fidelity mode.
#[cfg(feature = "fidelity")]
#[allow(dead_code)]
pub fn set_layouter(l: Box<dyn Layouter>) {
    LAYOUTER.with(|s| *s.borrow_mut() = l);
}

// ---------------------------------------------------------------------------
// Fidelity backends — Node.js oracle processes
// ---------------------------------------------------------------------------

/// Fidelity backends backed by Node.js oracle processes.
/// Only compiled with `--features fidelity`; never present in release builds.
#[cfg(feature = "fidelity")]
#[allow(dead_code)]
pub mod fidelity {
    use super::{Layouter, Measurer};
    use dagre_dgl_rs::graph::{Edge, Graph, Point};
    use std::cell::RefCell;
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Child, ChildStdin, Stdio};

    // ── Node.js subprocess wrapper ─────────────────────────────────────────

    struct Oracle {
        _child: Child,
        stdin: ChildStdin,
        stdout: BufReader<std::process::ChildStdout>,
    }

    impl Oracle {
        fn spawn(script_path: &str) -> Self {
            let mut child = std::process::Command::new("node")
                .arg(script_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap_or_else(|e| panic!("failed to spawn node {script_path}: {e}"));
            let stdin = child.stdin.take().unwrap();
            let stdout = BufReader::new(child.stdout.take().unwrap());
            Oracle {
                _child: child,
                stdin,
                stdout,
            }
        }

        /// Send one JSON line, read one JSON line back.
        fn call(&mut self, request: &str) -> String {
            writeln!(self.stdin, "{request}").expect("oracle stdin write");
            let mut line = String::new();
            self.stdout
                .read_line(&mut line)
                .expect("oracle stdout read");
            line.trim_end().to_owned()
        }
    }

    // ── BrowserMeasurer ────────────────────────────────────────────────────

    /// Measures text via a headless Chrome session running `measure_oracle.mjs`.
    /// Matches Mermaid JS's `getBoundingClientRect()` on Arial HTML `<span>`.
    pub struct BrowserMeasurer {
        oracle: RefCell<Oracle>,
    }

    impl BrowserMeasurer {
        /// Spawn the `measure_oracle.mjs` Node.js process and return a handle.
        pub fn spawn(script_path: &str) -> Self {
            BrowserMeasurer {
                oracle: RefCell::new(Oracle::spawn(script_path)),
            }
        }
    }

    impl Measurer for BrowserMeasurer {
        fn measure(&self, text: &str, font_size: f64, bold: bool) -> (f64, f64) {
            let req = format!(
                r#"{{"text":{},"fontSize":{},"bold":{}}}"#,
                serde_json::to_string(text).unwrap(),
                font_size,
                bold,
            );
            let resp = self.oracle.borrow_mut().call(&req);
            let v: serde_json::Value = serde_json::from_str(&resp)
                .unwrap_or_else(|e| panic!("measure oracle bad JSON: {e}\nresp={resp}"));
            (
                v["width"].as_f64().unwrap_or(0.0),
                v["height"].as_f64().unwrap_or(0.0),
            )
        }
    }

    // ── DagreJsLayouter ───────────────────────────────────────────────────

    /// Runs dagre layout via dagre-d3-es in Node.js (`dagre_oracle.mjs`).
    /// Given identical node dimensions, produces positions equivalent to
    /// dagre-dgl-rs.  Useful for confirming there is no algorithmic gap.
    pub struct DagreJsLayouter {
        oracle: RefCell<Oracle>,
    }

    impl DagreJsLayouter {
        /// Spawn the `dagre_oracle.mjs` Node.js process and return a handle.
        pub fn spawn(script_path: &str) -> Self {
            DagreJsLayouter {
                oracle: RefCell::new(Oracle::spawn(script_path)),
            }
        }
    }

    impl Layouter for DagreJsLayouter {
        fn layout(&self, graph: &mut Graph) {
            let req = graph_to_json(graph);
            let resp = self.oracle.borrow_mut().call(&req);
            let v: serde_json::Value = serde_json::from_str(&resp)
                .unwrap_or_else(|e| panic!("dagre oracle bad JSON: {e}\nresp={resp}"));
            apply_layout_result(graph, &v);
        }
    }

    // ── Graph → JSON (request) ────────────────────────────────────────────

    fn graph_to_json(g: &Graph) -> String {
        use serde_json::{json, Value};

        let gl = g.graph();
        let graph_label = json!({
            "rankdir":  gl.rankdir.as_deref().unwrap_or("LR"),
            "nodesep":  gl.nodesep.unwrap_or(50.0),
            "ranksep":  gl.ranksep.unwrap_or(50.0),
            "marginx":  gl.marginx.unwrap_or(8.0),
            "marginy":  gl.marginy.unwrap_or(8.0),
        });

        let nodes: Vec<Value> = g
            .nodes()
            .iter()
            .map(|id| {
                let n = g.node(id);
                let mut obj = json!({
                    "id":     id,
                    "width":  n.width,
                    "height": n.height,
                });
                if let Some(p) = g.parent(id) {
                    obj["parent"] = Value::String(p.to_owned());
                }
                if let Some(it) = n.intersect_type {
                    obj["intersect_type"] = Value::String(it.to_string());
                }
                obj
            })
            .collect();

        let edges: Vec<Value> = g
            .edges()
            .iter()
            .map(|e| {
                let mut obj = json!({"v": e.v, "w": e.w});
                if let Some(name) = &e.name {
                    obj["name"] = Value::String(name.clone());
                }
                let lbl_opt = if let Some(name) = &e.name {
                    g.edge(&Edge::named(&e.v, &e.w, name))
                } else {
                    g.edge(&Edge::new(&e.v, &e.w))
                };
                if let Some(lbl) = lbl_opt {
                    obj["width"] = json!(lbl.width.unwrap_or(0.0));
                    obj["height"] = json!(lbl.height.unwrap_or(0.0));
                    obj["minlen"] = json!(lbl.minlen.unwrap_or(1));
                    obj["weight"] = json!(lbl.weight.unwrap_or(1.0));
                    if let Some(lp) = &lbl.labelpos {
                        obj["labelpos"] = Value::String(lp.clone());
                    }
                    if let Some(lo) = lbl.labeloffset {
                        obj["labeloffset"] = json!(lo);
                    }
                }
                obj
            })
            .collect();

        serde_json::to_string(&json!({
            "graph": graph_label,
            "nodes": nodes,
            "edges": edges,
        }))
        .unwrap()
    }

    // ── JSON (response) → Graph ───────────────────────────────────────────

    fn apply_layout_result(g: &mut Graph, v: &serde_json::Value) {
        if let Some(nodes) = v["nodes"].as_array() {
            for node in nodes {
                let id = node["id"].as_str().unwrap_or_default();
                if g.has_node(id) {
                    let n = g.node_mut(id);
                    n.x = node["x"].as_f64();
                    n.y = node["y"].as_f64();
                    if let Some(w) = node["width"].as_f64() {
                        n.width = w;
                    }
                    if let Some(h) = node["height"].as_f64() {
                        n.height = h;
                    }
                }
            }
        }

        if let Some(edges) = v["edges"].as_array() {
            for edge in edges {
                let v_id = edge["v"].as_str().unwrap_or_default();
                let w_id = edge["w"].as_str().unwrap_or_default();
                let e = match edge["name"].as_str() {
                    Some(name) => Edge::named(v_id, w_id, name),
                    None => Edge::new(v_id, w_id),
                };
                if let Some(lbl) = g.edge_mut(&e) {
                    lbl.x = edge["x"].as_f64();
                    lbl.y = edge["y"].as_f64();
                    if let Some(pts) = edge["points"].as_array() {
                        lbl.points = Some(
                            pts.iter()
                                .map(|p| Point {
                                    x: p["x"].as_f64().unwrap_or(0.0),
                                    y: p["y"].as_f64().unwrap_or(0.0),
                                })
                                .collect(),
                        );
                    }
                }
            }
        }

        if let (Some(gw), Some(gh)) = (v["graph"]["width"].as_f64(), v["graph"]["height"].as_f64())
        {
            g.graph_mut().width = Some(gw);
            g.graph_mut().height = Some(gh);
        }
    }
}
