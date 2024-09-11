use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph, WeightedEdge},
    search::{hl::hub_graph::HubGraph, PathFinding},
};
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{ParallelBridge, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    hub_graph: PathBuf,
    #[arg(short, long)]
    edge_differences: PathBuf,
}

fn main() {
    let args = Args::parse();

    let graph: ReversibleGraph<VecVecGraph> =
        ReversibleGraph::<VecVecGraph>::from_fmi_file(&args.graph);

    let hub_graph: HubGraph = {
        let reader = BufReader::new(File::open(&args.hub_graph).unwrap());
        bincode::deserialize_from(reader).unwrap()
    };

    let mut edge_differences = vec![false; graph.number_of_vertices() as usize];

    let vertices = graph.out_graph().vertices().collect_vec();
    // vertices.shuffle(&mut thread_rng());

    let _factors = vec![0.01, 0.025, 0.05, 0.075, 0.1, 0.15, 0.2, 0.25, 0.5];

    for (_i, &vertex) in vertices.iter().enumerate() {
        let in_edges = graph.in_graph().edges(vertex).collect_vec();
        let out_edges = graph.out_graph().edges(vertex).collect_vec();

        //  let new_edges = par_new_edges(&graph, &heuristic, vertex);
        //  let current_in_edges = graph.in_graph().edges(vertex).len();
        //  let current_out_edges = graph.out_graph().edges(vertex).len();
        //  let true_edge_diff = new_edges - current_in_edges as i32 -
        // current_out_edges as i32;

        //  print!("{:>9} {:>7}", vertex, true_edge_diff);

        //  let edge_diffs = factors
        //      .iter()
        //      .map(|&factor| simpler(&in_edges, &out_edges, &hub_graph,
        // factor))      .collect_vec();

        //  for diff in edge_diffs {
        //      print!(" {:>7}", diff);
        //  }
        //  println!("");

        edge_differences[_i] = test_if_on_shortest_path(&in_edges, &out_edges, &hub_graph, 1000);
    }

    {
        let writer = BufWriter::new(File::create(&args.edge_differences).unwrap());
        serde_json::to_writer(writer, &edge_differences).unwrap();
    }
}

fn test_if_on_shortest_path(
    in_edges: &Vec<WeightedEdge>,
    out_edges: &Vec<WeightedEdge>,
    pathfinder: &dyn PathFinding,
    searches: u32,
) -> bool {
    if in_edges.is_empty() || out_edges.is_empty() {
        return false;
    }

    (0..searches)
        .par_bridge()
        .map_init(
            || thread_rng(),
            |mut rng, _| {
                let in_edge = in_edges.choose(&mut rng).unwrap();
                let out_edge = out_edges.choose(&mut rng).unwrap();
                let shortcut_distance = in_edge.weight + out_edge.weight;
                let true_weight = pathfinder
                    .shortest_path_distance(in_edge.head, out_edge.head)
                    .unwrap();

                shortcut_distance == true_weight
            },
        )
        .any(|x| x)
}

fn simpler(
    in_edges: &Vec<WeightedEdge>,
    out_edges: &Vec<WeightedEdge>,
    pathfinder: &dyn PathFinding,
    factor: f32,
) -> i32 {
    let searches = (out_edges.len() as f32 * factor).round() as u32;
    let searches = searches.clamp(1, u32::MAX);

    if searches == 0 {
        return 0;
    }

    let shortcuts = in_edges
        .iter()
        .par_bridge()
        .map_init(
            || thread_rng(),
            |rng, in_edge| {
                out_edges
                    .choose_multiple(rng, searches as usize)
                    .map(|out_edge| {
                        let shortcut_distance = in_edge.weight + out_edge.weight;
                        let true_weight = pathfinder
                            .shortest_path_distance(in_edge.head, out_edge.head)
                            .unwrap();

                        shortcut_distance == true_weight
                    })
                    .collect_vec()
            },
        )
        .flatten()
        .filter(|&x| x)
        .count();

    ((shortcuts as f64 / (in_edges.len() * out_edges.len()) as f64)
        * in_edges.len() as f64
        * out_edges.len() as f64) as i32
        - in_edges.len() as i32
        - out_edges.len() as i32
}
