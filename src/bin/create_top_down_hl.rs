use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::Instant,
};

use ahash::{HashMap, HashMapExt};
use clap::Parser;
use faster_paths::{
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::{generate_vertex_to_level_map, validate_path},
        path::{Path, ShortestPathRequest, ShortestPathTestCase},
        Graph,
    },
    hl::{
        hl_from_top_down::{
            generate_directed_hub_graph, generate_forward_label, generate_reverse_label,
        },
        pathfinding::{shortest_path, shortest_path_weight},
    },
    shortcut_replacer::slow_shortcut_replacer::replace_shortcuts_slow,
};
use itertools::Itertools;

/// Creates a hub graph top down.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .gr or .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Path of .fmi file
    #[arg(short, long)]
    paths: PathBuf,
    /// Outfile in .bincode format
    #[arg(short, long)]
    hub_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("loading paths");
    let reader = BufReader::new(File::open(&args.paths).unwrap());
    let paths: Vec<Path> = serde_json::from_reader(reader).unwrap();

    println!("loading graph");
    let graph = GraphFactory::from_file(&args.graph);

    let vertex_to_level_map = (0..graph.number_of_vertices()).collect_vec(); //generate_vertex_to_level_map(paths, graph.number_of_vertices);

    let request = ShortestPathRequest::new(1234, 46360).unwrap();
    let (f_label, f_shortcuts) =
        generate_forward_label(request.source(), &graph, &vertex_to_level_map);
    let (b_label, b_shortcuts) =
        generate_reverse_label(request.target(), &graph, &vertex_to_level_map);

    let mut shortcuts = HashMap::new();
    shortcuts.extend(f_shortcuts);
    shortcuts.extend(b_shortcuts);

    let n = 254516;
    for (edge, &vertex) in shortcuts.iter() {
        if edge.tail() == n || edge.head() == n || vertex == n {
            println!("{} (-> {}) -> {}", edge.tail(), vertex, edge.head());
        }
    }
    let mut path = shortest_path(&f_label, &b_label).unwrap();
    println!("path has len {}", path.vertices.len());
    replace_shortcuts_slow(&mut path.vertices, &shortcuts);
    println!("path has len {}", path.vertices.len());
    let validation = ShortestPathTestCase {
        request,
        weight: Some(path.weight),
    };
    validate_path(&graph, &validation, &Some(path)).unwrap();

    println!("Generating hub graph");
    let start = Instant::now();
    let hub_graph = generate_directed_hub_graph(&graph, &vertex_to_level_map);
    println!("Generating all labels took {:?}", start.elapsed());

    println!("Saving hub graph as json");
    let writer = BufWriter::new(File::create(&args.hub_graph).unwrap());
    bincode::serialize_into(writer, &hub_graph).unwrap();
}
