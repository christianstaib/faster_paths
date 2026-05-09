use ch::contraction_hierachy::ContractionHierarchyPathfinder;
use ch::fmi::read_fmi_ch;
use ch::fmi::read_tests;
use ch::pathfinder::ShortestPathFinder;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum BenchmarkMode {
    Distance,
    Path,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Contraction hierarchy file
    #[arg(short, long)]
    contraction_hierarchy: PathBuf,

    /// Test file
    #[arg(short, long)]
    tests: PathBuf,

    /// Benchmark mode
    #[arg(short, long, value_enum, default_value = "distance")]
    mode: BenchmarkMode,
}

type DistanceType = u32; //OrderedFloat<f32>;

fn main() {
    let args = Args::parse();

    // Parse the ChGraph from the file
    let contraction_hierarchy = read_fmi_ch::<DistanceType>(&args.contraction_hierarchy).unwrap();
    let tests = read_tests::<DistanceType>(&args.tests).unwrap();

    let mut pathfinder = ContractionHierarchyPathfinder::new(&contraction_hierarchy);

    let start = Instant::now();
    let correct = match args.mode {
        BenchmarkMode::Distance => tests
            .iter()
            .all(|test| pathfinder.distance(test.query()) == test.distance()),

        BenchmarkMode::Path => tests
            .iter()
            .all(|test| pathfinder.path(test.query()).map(|path| path.distance) == test.distance()),
    };
    let whole_duration = start.elapsed();

    println!(
        "{:?}: took {:?} on average. All correct? {:?}",
        args.mode,
        whole_duration / tests.len() as u32,
        correct
    );
}
