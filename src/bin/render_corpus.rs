use ariel_rs::theme::Theme;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Usage: render_corpus [theme] [--corpus <path>] [--out <dir>]
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

    // Parse optional --corpus and --out flags
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

    let out_subdir = if theme_name == "default" {
        "rust".to_string()
    } else {
        format!("rust_{}", theme_name.to_lowercase())
    };

    let corpus_path = corpus_path_override
        .unwrap_or_else(|| PathBuf::from("visual-regression/corpus/corpus.json"));
    let out_dir =
        out_dir_override.unwrap_or_else(|| PathBuf::from("visual-regression").join(&out_subdir));
    fs::create_dir_all(&out_dir).unwrap();

    let corpus_str = fs::read_to_string(&corpus_path).expect("Could not read corpus.json");
    let corpus: HashMap<String, String> =
        serde_json::from_str(&corpus_str).expect("Could not parse corpus.json");

    let mut rendered = 0;
    let mut skipped = 0;

    let mut entries: Vec<(String, String)> = corpus.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, diagram_text) in entries {
        dagre_dgl_rs::util::reset_id_counter();

        let svg = ariel_rs::render(&diagram_text, theme);

        if svg.contains("Syntax error in text") || svg.contains("Unrecognized diagram type") {
            skipped += 1;
            continue;
        }

        let out_path = out_dir.join(format!("{}.svg", name));
        fs::write(&out_path, &svg).expect("Could not write SVG");
        println!("  v  {}", name);
        rendered += 1;
    }

    println!(
        "\n{} rendered, {} skipped (unsupported diagram type)",
        rendered, skipped
    );
}
