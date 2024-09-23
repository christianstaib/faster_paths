use std::path::PathBuf;

use clap::Parser;
use faster_paths::{
    graphs::{Distance, Vertex},
    search::{
        ch::contracted_graph::{self, ContractedGraph},
        dijkstra::dijkstra_one_to_all_wraped,
        shortcuts::replace_shortcuts_slowly,
        PathFinding,
    },
    utility::{benchmark_distances, benchmark_path, read_bincode_with_spinnner},
};
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{ParallelBridge, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    contracted_graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    num_runs: u32,
}

fn main() {
    let args = Args::parse();

    // Read contracted_graph
    let contracted_graph: ContractedGraph =
        read_bincode_with_spinnner("contrated graph", &args.contracted_graph.as_path());

    let source = 1234;

    let x = dijkstra_one_to_all_wraped(contracted_graph.upward_graph(), source);
    let max_dist = *x
        .distances
        .iter()
        .filter(|&&dist| dist != Distance::MAX)
        .max()
        .unwrap() as f32;

    let mut paths = Vec::new();
    let mut dists = Vec::new();
    for v in contracted_graph.upward_graph().vertices() {
        let pre = x.predecessors[v as usize];
        if pre != Vertex::MAX {
            let mut path = vec![pre, v];
            replace_shortcuts_slowly(&mut path, contracted_graph.shortcuts());
            dists.push(
                path.iter()
                    .map(|&v| x.distances[v as usize] as f32 / max_dist)
                    .collect_vec(),
            );
            paths.push(path);
        }
    }
    println!("{:?}", paths);
    println!("\n\n\n");
    println!("{:?}", dists);

    println!(
        "Contracted graph has {} shortcuts",
        contracted_graph.shortcuts().len()
    );

    let vertices = (0..contracted_graph.number_of_vertices()).collect_vec();
    let mut rng = thread_rng();
    let pairs: Vec<(Vertex, Vertex)> = (0..args.num_runs)
        .map(|_| {
            vertices
                .choose_multiple(&mut rng, 2)
                .cloned()
                .collect_tuple()
                .unwrap()
        })
        .collect_vec();

    println!(
        "getting random paths distances takes {:?} on average",
        benchmark_distances(&contracted_graph, &pairs)
    );

    let vertices = (0..contracted_graph.number_of_vertices()).collect_vec();
    let mut rng = thread_rng();
    let pairs: Vec<(Vertex, Vertex)> = (0..args.num_runs)
        .map(|_| {
            vertices
                .choose_multiple(&mut rng, 2)
                .cloned()
                .collect_tuple()
                .unwrap()
        })
        .collect_vec();

    println!(
        "getting random paths takes {:?} on average",
        benchmark_path(&contracted_graph, &pairs)
    );
}
