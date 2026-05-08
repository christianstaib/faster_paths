use ch::contraction_hierachy::ContractionHierarchyPathfinder;
use ch::fmi::read_fmi_ch;
use ch::fmi::read_fmi_graph;
use ch::fmi::read_tests;
use ch::pathfinder::ShortestPathFinder;
use ch::validation::validate_path;
use clap::Parser;
use indicatif::ProgressIterator;
use ordered_float::OrderedFloat;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    contraction_hierarchy: PathBuf,

    /// CH graph in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    tests: PathBuf,
}

type DistanceType = OrderedFloat<f32>;

fn main() {
    let args = Args::parse();

    // Parse the ChGraph from the file
    let contraction_hierarchy = read_fmi_ch::<DistanceType>(&args.contraction_hierarchy).unwrap();
    let graph = read_fmi_graph(&args.graph).unwrap();
    let tests = read_tests(&args.tests).unwrap();

    let mut pathfinder = ContractionHierarchyPathfinder::new(&contraction_hierarchy);

    let num_failures = tests
        .iter()
        .progress()
        .filter_map(|test| {
            let path = pathfinder.path(test.query());
            validate_path(&graph, test, &path).err()
        })
        .inspect(|message| eprintln!("{message}"))
        .count();

    if num_failures > 0 {
        eprintln!("{} of {} paths failed.", num_failures, tests.len());
        std::process::exit(1);
    }

    println!("All {} paths correct.", tests.len());
}
