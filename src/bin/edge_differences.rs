use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use faster_paths::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph, Vertex, WeightedEdge,
    },
    search::{
        ch::bottom_up::heuristic::par_new_edges, hl::hub_graph::HubGraph, shortcuts, PathFinding,
        PathfinderHeuristic,
    },
};
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};

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

    let heuristic = PathfinderHeuristic {
        pathfinder: &hub_graph,
    };

    let mut edge_differences = vec![0; graph.number_of_vertices() as usize];

    let mut vertices = graph.out_graph().vertices().collect_vec();
    vertices.shuffle(&mut thread_rng());

    let start = Instant::now();
    for (i, &vertex) in vertices.iter().enumerate() {
        let prop_edge_diff = fun_name(&graph, vertex, &hub_graph, 0.05);

        let new_edges = par_new_edges(&graph, &heuristic, vertex);
        let current_in_edges = graph.in_graph().edges(vertex).len();
        let current_out_edges = graph.out_graph().edges(vertex).len();

        let edge_difference = new_edges - current_in_edges as i32 - current_out_edges as i32;
        println!(
            "v {:>9} edge diff {:>7} prop1 {:>7} prop2 {:>7} (in {:>5}, out {:>5}). remaining {:?}",
            vertex,
            edge_difference,
            prop_edge_diff,
            simpler(
                graph.in_graph().edges(vertex).collect(),
                graph.out_graph().edges(vertex).collect(),
                &hub_graph,
                0.05
            ),
            current_in_edges,
            current_out_edges,
            start.elapsed() / (i as u32 + 1) * (graph.number_of_vertices() - (i as u32 + 1))
        );

        edge_differences.push(edge_difference);
    }

    {
        let writer = BufWriter::new(File::create(&args.edge_differences).unwrap());
        serde_json::to_writer(writer, &edge_differences).unwrap();
    }
}

fn simpler(
    in_edges: Vec<WeightedEdge>,
    out_edges: Vec<WeightedEdge>,
    pathfinder: &dyn PathFinding,
    factor: f32,
) -> i32 {
    let searches = ((in_edges.len() * out_edges.len()) as f32 * factor).round() as u64;

    if searches == 0 {
        return 0;
    }

    let shortcuts = (0..searches)
        .par_bridge()
        .map_init(
            || thread_rng(),
            |rng, _| {
                let in_edge = in_edges.choose(rng).unwrap();
                let out_edge = out_edges.choose(rng).unwrap();

                let shortcut_distance = in_edge.weight + out_edge.weight;
                let true_weight = pathfinder
                    .shortest_path_distance(in_edge.head, out_edge.head)
                    .unwrap();

                shortcut_distance == true_weight
            },
        )
        .filter(|&x| x)
        .count();

    ((shortcuts as f64 / (in_edges.len() * out_edges.len()) as f64)
        * in_edges.len() as f64
        * out_edges.len() as f64) as i32
        - in_edges.len() as i32
        - out_edges.len() as i32
}

fn fun_name(
    graph: &ReversibleGraph<VecVecGraph>,
    vertex: u32,
    hub_graph: &HubGraph,
    factor: f32,
) -> i32 {
    let in_edges = graph.in_graph().edges(vertex).collect_vec();
    let out_edges = graph.out_graph().edges(vertex).collect_vec();

    let mut prop_edge_diff = 0;
    if in_edges.len() + out_edges.len() > 0 {
        let pairs =
            probabilistic_edge_difference(in_edges.len() as u32, out_edges.len() as u32, factor);

        let shortcuts = pairs
            .par_iter()
            .filter(|&&(in_index, out_index)| {
                let shortcut_distance =
                    in_edges[in_index as usize].weight + out_edges[out_index as usize].weight;
                let true_distance = hub_graph
                    .shortest_path_distance(
                        in_edges[in_index as usize].head,
                        out_edges[out_index as usize].head,
                    )
                    .unwrap();
                shortcut_distance == true_distance
            })
            .count();

        prop_edge_diff = ((shortcuts as f64 / pairs.len() as f64)
            * in_edges.len() as f64
            * out_edges.len() as f64) as i32
            - in_edges.len() as i32
            - out_edges.len() as i32;
    }
    prop_edge_diff
}

fn probabilistic_edge_difference(
    num_in_edges: u32,
    num_out_edges: u32,
    factor: f32,
) -> Vec<(u32, u32)> {
    let in_edges: Vec<u32> = (0..num_in_edges).collect();
    let out_edges: Vec<u32> = (0..num_out_edges).collect();

    let num_selections = (num_in_edges as f32 * num_out_edges as f32 * factor).round() as u64;
    let num_selections = num_selections.clamp(
        num_in_edges as u64,
        num_in_edges as u64 * num_out_edges as u64,
    );

    let base_value = num_selections / num_in_edges as u64;
    let remainder = num_selections % num_in_edges as u64;

    let mut out_edges_to_choose: Vec<u64> = vec![base_value; num_in_edges as usize];
    for i in 0..remainder as usize {
        out_edges_to_choose[i] += 1;
    }

    let mut rng = thread_rng();

    in_edges
        .iter()
        .flat_map(|&in_index| {
            out_edges
                .choose_multiple(&mut rng, out_edges_to_choose[in_index as usize] as usize)
                .map(move |&out_index| (in_index, out_index))
        })
        .collect()
}
