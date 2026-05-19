use ch::{
    fmi::{read_fmi_ch, read_fmi_hl, read_tests},
    graph::{FastGraph, WeightedEdge},
    hub_labeling::HubLabelingPathfinder,
    types::VertexId,
    validation::validate,
};
use clap::Parser;
use graph_readers::edges_from_fmi;
use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input graph file
    #[arg(short, long)]
    graph: Option<PathBuf>,

    /// Contraction hierarchy file
    #[arg(short, long)]
    contraction_hierarchy: PathBuf,

    /// Hub labeling File
    #[arg(short = 'l', long)]
    hub_labeling: PathBuf,

    /// Test file
    #[arg(short, long)]
    tests: PathBuf,
}

type DistanceType = u32;

fn main() {
    let args = Args::parse();

    let graph = args.graph.as_ref().map(|graph| {
        let edges = edges_from_fmi(
            BufReader::new(File::open(graph).unwrap()),
            |s| s.parse::<u32>().ok().map(VertexId::new),
            |s| s.parse::<DistanceType>().ok(),
            |tail, head, weight| WeightedEdge { tail, head, weight },
        )
        .unwrap();

        FastGraph::from_flat(edges)
    });
    let contraction_hierarchy = read_fmi_ch::<DistanceType>(&args.contraction_hierarchy).unwrap();
    let hub_labeling = read_fmi_hl::<DistanceType>(&args.hub_labeling).unwrap();
    let tests = read_tests::<DistanceType>(&args.tests).unwrap();

    let mut pathfinder = HubLabelingPathfinder {
        contraction_hierarchy: &contraction_hierarchy,
        hub_labeling: &hub_labeling,
    };

    let validation_target = if graph.is_some() {
        "paths"
    } else {
        "distances"
    };

    match validate(&tests, graph.as_ref(), &mut pathfinder) {
        Ok(average_runtime) => {
            println!(
                "All {} {} correct. Average runtime: {:?}.",
                tests.len(),
                validation_target,
                average_runtime
            );
        }

        Err(failures) => {
            failures.iter().for_each(|message| eprintln!("{message}"));

            eprintln!(
                "{} of {} {} failed.",
                failures.len(),
                tests.len(),
                validation_target
            );
            std::process::exit(1);
        }
    }
}
