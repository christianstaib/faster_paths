use std::{
    collections::HashSet,
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
    sync::atomic::AtomicU64,
};

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph, Vertex},
    reading_pathfinder,
    search::{ch::contracted_graph::vertex_to_level, PathFinding},
    utility::{
        average_hl_label_size, average_hl_label_size_vertices, get_progressbar,
        write_json_with_spinnner,
    },
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

    /// Number of seartes.
    #[arg(short, long, default_value = "100000")]
    m: usize,

    /// Number of seartes.
    #[arg(short, long, default_value = "1000")]
    every_label: u32,

    /// Path to the output file where the vertex to level mapping will be
    /// stored.
    #[arg(short, long)]
    level_to_vertex: PathBuf,
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
    let mut seen_paths: u64 = 0;

    let vertices = (0..graph.number_of_vertices()).collect_vec();
    let verticesx = vertices
        .choose_multiple(&mut thread_rng(), 1_000)
        .cloned()
        .collect_vec();

    let mut paths = Vec::new();

    let pb = get_progressbar("hitting-set ", args.number_of_searches as u64);
    while !active_vertices.is_empty() && pb.position() < args.number_of_searches as u64 {
        let needed_legal_paths = args.m - paths.len();

        let this_seen_paths = fill_paths(
            &active_vertices,
            &pathfinder,
            &hitting_set_set,
            needed_legal_paths,
            &mut paths,
        );
        seen_paths += this_seen_paths;

        let hits = get_hits(&paths, number_of_vertices);

        let vertex = hits
            .par_iter()
            .enumerate()
            .max_by_key(|&(_vertex, hits)| hits)
            .map(|(vertex, _)| vertex as Vertex)
            .expect("hits cannot be empty if number_of_vertices > 0");

        level_to_vertex.insert(0, vertex);
        hitting_set_set.insert(vertex);
        active_vertices.remove(&vertex);
        pb.inc(1);

        paths = paths
            .into_par_iter()
            .filter(|path| path.vertices.iter().all(|&v| v != vertex))
            .collect::<Vec<_>>();

        if pb.position() % args.every_label as u64 == 0 {
            let mut active_verticesx = active_vertices.iter().cloned().collect_vec();
            active_verticesx.sort_by_cached_key(|vertex| graph.out_graph().edges(*vertex).len());
            let mut level_to_vertex = level_to_vertex.clone();
            level_to_vertex.splice(0..0, active_verticesx);

            let average_hl_label_size = average_hl_label_size_vertices(
                graph.out_graph(),
                &vertex_to_level(&level_to_vertex),
                &verticesx,
            );
            println!(
            "seen {:>9} paths. not hitting {:>2.20}%, hs contains {:>4} vertices, average hl label size {:>3.1}. (averaged over {} out of {} vertices)",
            seen_paths,
            ((paths.len()) as f64 / (seen_paths) as f64) * 100.0,
            // 100.0-(((args.m - paths.len()) as f64 / (seen_paths) as f64) * 100.0),
            hitting_set_set.len(),
            average_hl_label_size,
            verticesx.len(),
            graph.number_of_vertices());

            write_json_with_spinnner(
                "",
                Path::new(
                    format!(
                        "{}_ltv_degree_{}_{}.json",
                        args.graph.file_stem().unwrap().to_str().unwrap(),
                        pb.position(),
                        average_hl_label_size.round() as u32
                    )
                    .as_str(),
                ),
                &level_to_vertex,
            )
        }
    }

    let mut active_vertices = active_vertices.into_iter().collect_vec();
    active_vertices.sort_by_cached_key(|vertex| graph.out_graph().edges(*vertex).len());
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

fn get_hits(
    paths: &Vec<faster_paths::search::collections::dijkstra_data::Path>,
    number_of_vertices: u32,
) -> Vec<i32> {
    let hits = paths
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
    hits
}

fn fill_paths(
    active_vertices: &HashSet<u32>,
    pathfinder: &Box<dyn PathFinding>,
    hitting_set_set: &HashSet<u32>,
    needed_legal_paths: usize,
    paths: &mut Vec<faster_paths::search::collections::dijkstra_data::Path>,
) -> u64 {
    let this_seen_paths = AtomicU64::new(0);
    let vertices = active_vertices.iter().cloned().collect_vec();
    let this_legal = (0..)
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
        .filter(|path| {
            this_seen_paths.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            path.vertices.iter().all(|v| !hitting_set_set.contains(v))
        })
        .take_any(needed_legal_paths)
        .collect::<Vec<_>>();

    let this_seen_paths = this_seen_paths.load(std::sync::atomic::Ordering::Relaxed);
    paths.extend(this_legal);
    this_seen_paths
}
