use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use clap::Parser;
use faster_paths::{
    graphs::{Distance, Vertex},
    utility::get_progressbar,
};
use itertools::Itertools;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph_in: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    simple_graph_in: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    graph_out: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    simple_graph_out: PathBuf,
}

struct RawGraphData {
    pub vertices: HashMap<Vertex, (f64, f64)>,
    pub edges: Vec<(Vertex, Vertex, Distance)>,
}

fn main() {
    let args = Args::parse();

    let raw_graph = read_graph(args.graph_in.as_path(), true);
    let raw_simple_graph = read_graph(&args.simple_graph_in.as_path(), false);

    let graph_vetices = raw_graph
        .edges
        .iter()
        .map(|&(tail, head, _weight)| vec![tail, head])
        .flatten()
        .collect::<HashSet<_>>();

    let simple_graph_vetices = raw_simple_graph
        .edges
        .iter()
        .map(|&(tail, head, _weight)| vec![tail, head])
        .flatten()
        .collect::<HashSet<_>>();

    let mut simple_graph_only_vertices = simple_graph_vetices
        .difference(&graph_vetices)
        .cloned()
        .collect_vec();
    simple_graph_only_vertices.sort();

    let mut graph_vetices = graph_vetices.iter().cloned().collect_vec();
    graph_vetices.sort();

    let mut old_to_new = HashMap::new();
    let mut new_to_old = HashMap::new();

    for &old in graph_vetices
        .iter()
        .chain(simple_graph_only_vertices.iter())
    {
        let new = old_to_new.len() as Vertex;
        old_to_new.insert(old, new);
        new_to_old.insert(new, old);
    }

    write_graph(
        &args.graph_out,
        &raw_graph,
        graph_vetices.len(),
        &old_to_new,
        &new_to_old,
    );

    write_graph(
        &args.simple_graph_out,
        &raw_simple_graph,
        simple_graph_vetices.len(),
        &old_to_new,
        &new_to_old,
    );
}

fn write_graph(
    path: &Path,
    data: &RawGraphData,
    num_vertices: usize,
    old_to_new: &HashMap<Vertex, Vertex>,
    new_to_old: &HashMap<Vertex, Vertex>,
) {
    let mut writer = BufWriter::new(File::create(path).unwrap());
    for _ in 0..4 {
        writeln!(writer, "#").unwrap();
    }
    writeln!(writer, "").unwrap();

    writeln!(writer, "{}", num_vertices).unwrap();
    writeln!(writer, "{}", data.edges.len()).unwrap();

    let pb = get_progressbar("Writing graph", (num_vertices + data.edges.len()) as u64);

    for new in 0..num_vertices as Vertex {
        pb.inc(1);

        let old = new_to_old[&new];
        writeln!(
            writer,
            "{} {} {} {} 0",
            new, old, data.vertices[&old].0, data.vertices[&old].1
        )
        .unwrap();
    }

    for (old_tail, old_head, weight) in data.edges.iter() {
        pb.inc(1);

        let new_tail = old_to_new[old_tail];
        let new_head = old_to_new[old_head];
        writeln!(writer, "{} {} {} 0 0", new_tail, new_head, weight).unwrap();
    }

    pb.finish_and_clear();
}

fn read_graph(path: &Path, is_visibility: bool) -> RawGraphData {
    let graph_reader = BufReader::new(File::open(path).unwrap());
    let mut graph_lines = graph_reader.lines();
    while graph_lines.next().unwrap().unwrap().starts_with("#") {}

    let number_of_vertices: usize = graph_lines.next().unwrap().unwrap().parse().unwrap();
    let number_of_edges: usize = graph_lines.next().unwrap().unwrap().parse().unwrap();

    let pb = get_progressbar(
        "Reading graph",
        (number_of_vertices + number_of_edges) as u64,
    );

    let mut vertices = HashMap::new();
    graph_lines
        .by_ref()
        .take(number_of_vertices)
        .flatten()
        .for_each(|vertex_string| {
            pb.inc(1);

            let mut vertex_data = vertex_string.split_whitespace();
            let id1 = vertex_data.next().unwrap().parse::<Vertex>().unwrap();
            let _id2 = vertex_data.next().unwrap().parse::<Vertex>().unwrap();
            let lat = vertex_data.next().unwrap().parse::<f64>().unwrap();
            let lon = vertex_data.next().unwrap().parse::<f64>().unwrap();
            vertices.insert(id1, (lat, lon));
        });

    let mut edges = HashMap::new();
    graph_lines
        .by_ref()
        .take(number_of_edges)
        .flatten()
        .for_each(|edge_string| {
            pb.inc(1);

            let mut edge_data = edge_string.split_whitespace();
            let tail = edge_data.next().unwrap().parse::<Vertex>().unwrap();
            let head = edge_data.next().unwrap().parse::<Vertex>().unwrap();
            let weight_string = edge_data.next().unwrap();
            let weight = {
                if is_visibility {
                    (weight_string.parse::<f64>().unwrap() * 100.0).floor() as Distance
                } else {
                    weight_string.parse::<Distance>().unwrap()
                }
            };

            let current_weight = *edges.get(&(tail, head)).unwrap_or(&Distance::MAX);
            if weight < current_weight {
                edges.insert((tail, head), weight);
                edges.insert((head, tail), weight);
            }
        });
    let mut edges = edges
        .into_iter()
        .map(|((tail, head), weight)| (tail, head, weight))
        .collect_vec();
    edges.sort();

    pb.finish_and_clear();
    RawGraphData { vertices, edges }
}
