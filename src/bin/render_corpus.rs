use ariel_rs::theme::Theme;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Usage: render_corpus [theme]
    // theme: default | dark | forest | neutral  (default if omitted)
    let theme_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "default".to_string());
    let theme = match theme_name.to_lowercase().as_str() {
        "dark" => Theme::Dark,
        "forest" => Theme::Forest,
        "neutral" => Theme::Neutral,
        _ => Theme::Default,
    };

    let out_subdir = if theme_name == "default" {
        "rust".to_string()
    } else {
        format!("rust_{}", theme_name.to_lowercase())
    };

    let corpus_path = PathBuf::from("visual-regression/corpus/corpus.json");
    let out_dir = PathBuf::from("visual-regression").join(&out_subdir);
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

        let svg = ariel_rs::render(&diagram_text, theme.clone());

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
