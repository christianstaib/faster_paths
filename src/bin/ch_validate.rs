use ch::contraction_hierachy::{ContractionHierarchy, ContractionHierarchyPathfinder};
use ch::fmi::read_fmi_graph;
use ch::fmi::read_tests;
use ch::validation::validate;
use clap::Parser;
use std::{fs::File, io::BufReader, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input graph file
    #[arg(short, long)]
    graph: Option<PathBuf>,

    /// Test file
    #[arg(short, long)]
    tests: PathBuf,

    /// Contraction hierarchy file
    #[arg(short, long)]
    contraction_hierarchy: PathBuf,
}

type DistanceType = u32;

fn main() {
    let args = Args::parse();

    let (contraction_hierarchy, _): (ContractionHierarchy<DistanceType>, _) = postcard::from_io((
        BufReader::new(File::open(&args.contraction_hierarchy).unwrap()),
        &mut [0; 1024],
    ))
    .unwrap();
    let graph = args
        .graph
        .as_ref()
        .map(|graph| read_fmi_graph::<DistanceType>(graph).unwrap());
    let tests = read_tests::<DistanceType>(&args.tests).unwrap();

    let mut pathfinder = ContractionHierarchyPathfinder::new(&contraction_hierarchy);
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
