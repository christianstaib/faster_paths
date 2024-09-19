use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Distance, Vertex,
    },
    search::{
        alt::landmark::{self, Landmarks},
        ch::contracted_graph::ContractedGraph,
        hl::hub_graph::{self, HubGraph},
        DistanceHeuristic, PathFinding,
    },
    utility::{benchmark_and_test_path, generate_test_cases, read_bincode_with_spinnner},
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    hub_graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    contracted_graph: PathBuf,
}

pub struct PathfinderHeuristic<'a> {
    pub hub_graph: &'a HubGraph,
    pub landmarks: &'a Landmarks,
}

impl<'a> DistanceHeuristic for PathfinderHeuristic<'a> {
    fn lower_bound(&self, source: Vertex, target: Vertex) -> Distance {
        std::cmp::max(
            self.hub_graph
                .shortest_path_distance(source, target)
                .unwrap_or(0),
            self.landmarks.lower_bound(source, target),
        )
    }

    fn upper_bound(&self, source: Vertex, target: Vertex) -> Distance {
        std::cmp::max(
            self.hub_graph
                .shortest_path_distance(source, target)
                .unwrap_or(0),
            self.landmarks.upper_bound(source, target),
        )
    }

    fn is_less_or_equal_upper_bound(
        &self,
        source: Vertex,
        target: Vertex,
        distance: Distance,
    ) -> bool {
        if distance <= self.hub_graph.lower_bound(source, target) {
            return self
                .landmarks
                .is_less_or_equal_upper_bound(source, target, distance);
        }
        return false;
    }
}

fn main() {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    let hub_graph: HubGraph = read_bincode_with_spinnner("hub graph", &args.hub_graph.as_path());

    // Create landmakrs
    let landmarks = Landmarks::random(&graph, 1);

    let h = PathfinderHeuristic {
        hub_graph: &hub_graph,
        landmarks: &landmarks,
    };

    // Create contracted_graph
    let contracted_graph = ContractedGraph::by_contraction_with_heuristic(&graph, &h);

    // Write contracted_graph to file
    let writer = BufWriter::new(File::create(&args.contracted_graph).unwrap());
    bincode::serialize_into(writer, &contracted_graph).unwrap();

    // Benchmark and test correctness
    let tests = generate_test_cases(graph.out_graph(), 1_000);
    let average_duration =
        benchmark_and_test_path(graph.out_graph(), &tests, &contracted_graph).unwrap();
    println!("Average duration was {:?}", average_duration);
}
