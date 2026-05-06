use ch::ch::ContractionHierarchyPathfinder;
use ch::fmi::read_fmi_ch;
use ch::fmi::read_tests;
use clap::Parser;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph_in: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    test_in: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Parse the ChGraph from the file
    let graph = read_fmi_ch(&args.graph_in).unwrap();

    let mut pathfiner = ContractionHierarchyPathfinder::new(&graph);

    let tests = read_tests(&args.test_in).unwrap();

    let start = Instant::now();
    let correct = tests.iter().all(|test| {
        pathfiner
            .search(test.query())
            .map(|(distance, _vertex)| distance)
            == test.distance()
    });
    let whole_duration = start.elapsed();

    println!(
        "Took {:?} on average. All correct? {:?}",
        whole_duration / tests.len() as u32,
        correct
    );
}
