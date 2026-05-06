use ch::contraction_hierachy::ContractionHierarchyPathfinder;
use ch::fmi::read_fmi_ch;
use ch::fmi::read_fmi_graph;
use ch::fmi::read_tests;
use ch::pathfinder::ShortestPathFinder;
use ch::validation::validate_path;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// CH graph in .fmi format
    #[arg(short, long)]
    ch_in: PathBuf,

    /// CH graph in .fmi format
    #[arg(short, long)]
    graph_in: PathBuf,

    /// Test queries in .txt format
    #[arg(short, long)]
    test_in: PathBuf,
}

fn main() {
    let args = Args::parse();

    let ch = read_fmi_ch(&args.ch_in).unwrap();
    let graph = read_fmi_graph(&args.graph_in).unwrap();
    let tests = read_tests(&args.test_in).unwrap();

    let mut pathfinder = ContractionHierarchyPathfinder::new(&ch);

    let failures = tests
        .iter()
        .filter_map(|test| {
            let path = pathfinder.path(test.query());
            validate_path(&graph, test, &path).err()
        })
        .inspect(|message| eprintln!("{message}"))
        .count();

    if failures > 0 {
        eprintln!("{failures} of {} paths failed.", tests.len());
        std::process::exit(1);
    }

    println!("All {} paths correct.", tests.len());
}
