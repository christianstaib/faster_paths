use ch::{
    fmi::{read_fmi_ch, read_tests},
    hub_labeling::{HubLabelingPathfinder, merge},
    path::PathDistance,
    pathfinder::ShortestPathFinder,
    types::Distance,
    validation::validate_distance,
};
use clap::Parser;
use indicatif::ProgressIterator;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Contraction hierarchy file
    #[arg(short, long)]
    contraction_hierarchy: PathBuf,

    /// Test file
    #[arg(short, long)]
    tests: PathBuf,
}

type DistanceType = u32;

fn main() {
    let args = Args::parse();

    let contraction_hierarchy = read_fmi_ch::<DistanceType>(&args.contraction_hierarchy).unwrap();
    let tests = read_tests::<DistanceType>(&args.tests).unwrap();

    let hub_labeling = merge(&contraction_hierarchy);

    let avg_label_size = hub_labeling.up_hub_labeling.num_flat() as f32
        / hub_labeling.up_hub_labeling.num_nested() as f32;
    println!("Average label size is {}", avg_label_size);

    let mut hub_labeling_pathfinder = HubLabelingPathfinder {
        contraction_hierarchy: &contraction_hierarchy,
        hub_labeling: &hub_labeling,
    };

    let num_failures = validate_tests(&mut hub_labeling_pathfinder, &tests);

    if num_failures > 0 {
        eprintln!("{} of {} distances failed.", num_failures, tests.len());
        std::process::exit(1);
    }

    println!("All {} distances correct.", tests.len());
}

fn validate_tests<D: Distance>(
    hub_labeling: &mut HubLabelingPathfinder<D>,
    tests: &[PathDistance<D>],
) -> usize {
    tests
        .iter()
        .progress()
        .filter_map(|test| {
            let distance = hub_labeling.path(test.query()).map(|path| path.distance);
            validate_distance(test, &distance).err()
        })
        .inspect(|message| eprintln!("{message}"))
        .count()
}
