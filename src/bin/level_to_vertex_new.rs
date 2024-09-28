use std::{collections::HashSet, fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph, Vertex},
    reading_pathfinder,
    search::{ch::contracted_graph::vertex_to_level, PathFinding},
    utility::{average_hl_label_size, get_progressbar},
    FileType,
};
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Input file
    #[arg(short, long)]
    file: PathBuf,

    /// Type of the input file
    #[arg(short = 't', long, value_enum, default_value = "fmi")]
    file_type: FileType,

    /// Number of seartes.
    #[arg(short, long, default_value = "100000")]
    number_of_searches: u32,

    /// Path to the output file where the vertex to level mapping will be
    /// stored.
    #[arg(short, long)]
    level_to_vertex: PathBuf,

    #[arg(short, long)]
    hit_percentage: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let graph = ReversibleGraph::<VecVecGraph>::from_fmi_file(&args.graph);

    let pathfinder = reading_pathfinder(&args.file.as_path(), &args.file_type);

    let number_of_vertices = graph.number_of_vertices();

    let mut active_vertices: HashSet<Vertex> = HashSet::from_iter(0..number_of_vertices);
    let mut hitting_set_set = HashSet::new();
    let mut level_to_vertex = Vec::new();
    let mut all_hits = vec![0; number_of_vertices as usize];
    let vertices = graph.out_graph().vertices().collect_vec();

    let n = 10_000;

    let pb = get_progressbar("hitting-set ", graph.number_of_vertices() as u64);
    while !active_vertices.is_empty() && pb.position() < args.number_of_searches as u64 {
        let paths = (0..)
            .par_bridge()
            .map_init(
                || thread_rng(),
                |rng, _| {
                    let (source, target) = vertices
                        .choose_multiple(rng, 2)
                        .into_iter()
                        .cloned()
                        .collect_tuple()
                        .unwrap();
                    pathfinder.shortest_path(source, target)
                },
            )
            .flatten()
            .filter(|path| path.vertices.iter().all(|v| !hitting_set_set.contains(v)))
            .take_any(10_000)
            .collect::<Vec<_>>();

        let mut hits = paths
            // Split the active_paths into chunks for parallel processing.
            .par_chunks(paths.len().div_ceil(rayon::current_num_threads()))
            // For each chunk, calculate how frequently each vertex appears across the active paths.
            .map(|paths| {
                let mut partial_hits = vec![0; number_of_vertices as usize];
                for path in paths.iter() {
                    for &vertex in path.vertices.iter() {
                        partial_hits[vertex as usize] += 1;
                    }
                }
                partial_hits
            })
            // Sum the results from all threads to get the total hit count for each vertex.
            .reduce(
                || vec![0; number_of_vertices as usize],
                |mut hits, partial_hits| {
                    for index in 0..number_of_vertices as usize {
                        hits[index] += partial_hits[index]
                    }
                    hits
                },
            );

        all_hits
            .par_iter_mut()
            .zip(hits.par_iter())
            .for_each(|(all, this)| *all += this);

        let vertex = hits
            .par_iter()
            .enumerate()
            .max_by_key(|&(_vertex, hits)| hits)
            .map(|(vertex, _)| vertex as Vertex)
            .expect("hits cannot be empty if number_of_vertices > 0");

        // hits.sort_unstable();
        // println!(
        //     "seen {} paths. {:?}",
        //     n * hitting_set_set.len() as u32,
        //     hits.iter().rev().take(10).collect_vec()
        // );

        level_to_vertex.insert(0, vertex);
        hitting_set_set.insert(vertex);
        active_vertices.remove(&vertex);
        pb.inc(1);
    }

    let mut active_vertices = active_vertices.into_iter().collect_vec();

    active_vertices.sort_by_cached_key(|vertex| all_hits[*vertex as usize]);

    // Insert the remaining vertices at the front, e.g. assign them the lowest
    // levels.
    level_to_vertex.splice(0..0, active_vertices);

    // Write level_to_vertex to file
    let writer = BufWriter::new(File::create(args.level_to_vertex).unwrap());
    serde_json::to_writer(writer, &level_to_vertex).unwrap();

    let num_samples = 1_000;
    let average_hl_label_size = average_hl_label_size(
        graph.out_graph(),
        &vertex_to_level(&level_to_vertex),
        num_samples,
    );
    println!(
        "average hl label size will be approximately {:.1}. (averaged over {} out of {} vertices)",
        average_hl_label_size,
        num_samples,
        graph.number_of_vertices()
    );
}
