use dagre_dgl_rs::graph::{EdgeLabel, Graph, GraphLabel, NodeLabel};
use dagre_dgl_rs::layout::layout;

fn main() {
    let mut g = Graph::with_options(true, true, true);
    g.set_graph(GraphLabel {
        rankdir: Some("lr".to_string()),
        nodesep: Some(50.0),
        ranksep: Some(50.0),
        marginx: Some(8.0),
        marginy: Some(8.0),
        ..Default::default()
    });

    g.set_node(
        "A",
        NodeLabel {
            width: 93.8,
            height: 54.0,
            ..Default::default()
        },
    );
    g.set_node(
        "B",
        NodeLabel {
            width: 113.6,
            height: 113.6,
            ..Default::default()
        },
    );
    g.set_node(
        "C",
        NodeLabel {
            width: 117.8,
            height: 54.0,
            ..Default::default()
        },
    );
    g.set_node(
        "D",
        NodeLabel {
            width: 88.5,
            height: 54.0,
            ..Default::default()
        },
    );

    g.set_edge(
        "A",
        "B",
        EdgeLabel {
            minlen: Some(1),
            weight: Some(1.0),
            width: Some(0.0),
            height: Some(0.0),
            labelpos: Some("r".to_string()),
            labeloffset: Some(10.0),
            ..Default::default()
        },
        None,
    );
    g.set_edge(
        "B",
        "C",
        EdgeLabel {
            minlen: Some(1),
            weight: Some(1.0),
            width: Some(26.1),
            height: Some(24.0),
            labelpos: Some("r".to_string()),
            labeloffset: Some(10.0),
            ..Default::default()
        },
        Some("yes"),
    );
    g.set_edge(
        "B",
        "D",
        EdgeLabel {
            minlen: Some(1),
            weight: Some(1.0),
            width: Some(20.5),
            height: Some(24.0),
            labelpos: Some("r".to_string()),
            labeloffset: Some(10.0),
            ..Default::default()
        },
        Some("no"),
    );
    g.set_edge(
        "C",
        "D",
        EdgeLabel {
            minlen: Some(1),
            weight: Some(1.0),
            width: Some(0.0),
            height: Some(0.0),
            labelpos: Some("r".to_string()),
            labeloffset: Some(10.0),
            ..Default::default()
        },
        None,
    );

    layout(&mut g);

    println!("=== RUST DAGRE: Node positions ===");
    for v in &["A", "B", "C", "D"] {
        let n = g.node(v);
        println!(
            "  {} : x={:.4} y={:.4} rank={:?} order={:?}",
            v,
            n.x.unwrap_or(0.0),
            n.y.unwrap_or(0.0),
            n.rank,
            n.order
        );
    }
    let gw = g.graph().width.unwrap_or(0.0);
    let gh = g.graph().height.unwrap_or(0.0);
    println!("\n  Graph: {:.4} x {:.4}", gw, gh);
}
