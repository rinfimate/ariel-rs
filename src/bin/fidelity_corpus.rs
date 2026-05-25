//! fidelity_corpus — render all corpus diagrams with Node.js-backed backends.
//!
//! Requires `--features fidelity`.  Uses:
//! - `BrowserMeasurer`: headless Chrome via `measure_oracle.mjs` for text metrics
//! - `DagreJsLayouter`: dagre-d3-es via `dagre_oracle.mjs` for graph layout
//!
//! Outputs SVGs to `visual-regression/fidelity/`.
//! After this, rasterise with:
//!   node svg_to_png_browser.mjs fidelity
//! Then compare with:
//!   node compare.mjs --dir fidelity

use ariel_rs::backends::fidelity::{BrowserMeasurer, DagreJsLayouter};
use ariel_rs::backends::{set_layouter, set_measurer};
use ariel_rs::theme::Theme;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn oracle_path(filename: &str) -> String {
    // Resolve relative to the visual-regression/ directory.
    // When running from repo root (cargo run), prefix accordingly.
    let candidates = [
        format!("visual-regression/{filename}"),
        format!("../{filename}"),
        filename.to_string(),
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return path.clone();
        }
    }
    panic!("Cannot find oracle script '{filename}'. Run from the repo root.");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let theme_name = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "default".to_string());
    let theme = match theme_name.to_lowercase().as_str() {
        "dark" => Theme::Dark,
        "forest" => Theme::Forest,
        "neutral" => Theme::Neutral,
        _ => Theme::Default,
    };

    // Parse optional --corpus and --out flags (matching render_corpus).
    let mut corpus_path_override: Option<PathBuf> = None;
    let mut out_dir_override: Option<PathBuf> = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--corpus" if i + 1 < args.len() => {
                corpus_path_override = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--out" if i + 1 < args.len() => {
                out_dir_override = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    let corpus_path = corpus_path_override
        .unwrap_or_else(|| PathBuf::from("visual-regression/corpus/corpus.json"));
    let out_dir = out_dir_override.unwrap_or_else(|| PathBuf::from("visual-regression/fidelity"));
    fs::create_dir_all(&out_dir).unwrap();

    let corpus_str = fs::read_to_string(&corpus_path).expect("Could not read corpus.json");
    let corpus: HashMap<String, String> =
        serde_json::from_str(&corpus_str).expect("Could not parse corpus.json");

    // Spawn oracle processes once, reuse across all diagrams.
    println!("Starting browser measurer (measure_oracle.mjs)...");
    let measurer = BrowserMeasurer::spawn(&oracle_path("measure_oracle.mjs"));

    println!("Starting dagre-js layouter (dagre_oracle.mjs)...");
    let layouter = DagreJsLayouter::spawn(&oracle_path("dagre_oracle.mjs"));

    // Install fidelity backends.
    set_measurer(Box::new(measurer));
    set_layouter(Box::new(layouter));

    let mut entries: Vec<(String, String)> = corpus.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut rendered = 0;
    let mut skipped = 0;

    for (name, diagram_text) in &entries {
        dagre_dgl_rs::util::reset_id_counter();

        let svg = ariel_rs::render(diagram_text, theme);

        if svg.contains("Syntax error in text") || svg.contains("Unrecognized diagram type") {
            skipped += 1;
            continue;
        }

        let out_path = out_dir.join(format!("{name}.svg"));
        fs::write(&out_path, &svg).expect("Could not write SVG");
        println!("  v  {name}");
        rendered += 1;
    }

    println!(
        "\n{rendered} rendered, {skipped} skipped — fidelity SVGs in visual-regression/fidelity/"
    );
}
