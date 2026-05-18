use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn main() {
    let corpus_path = PathBuf::from("visual-regression/corpus/corpus.json");
    let out_dir = PathBuf::from("visual-regression/rust");
    fs::create_dir_all(&out_dir).unwrap();

    let corpus_str = fs::read_to_string(&corpus_path).expect("Could not read corpus.json");
    let corpus: HashMap<String, String> =
        serde_json::from_str(&corpus_str).expect("Could not parse corpus.json");

    let mut rendered = 0;
    let mut skipped = 0;

    // Sort for deterministic output
    let mut entries: Vec<(String, String)> = corpus.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, diagram_text) in entries {
        // Reset the dagre ID counter before each diagram to ensure deterministic
        // dummy-node IDs regardless of the order or number of diagrams processed.
        dagre_dgl_rs::util::reset_id_counter();

        let svg = ariel_rs::render(&diagram_text, ariel_rs::theme::Theme::Default);

        // Skip diagrams that returned an error SVG (unrecognized type)
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
