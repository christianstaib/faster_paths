use ch::{
    fmi::{read_fmi_ch, read_tests},
    hub_labeling::{HubLabeling, entry::min_distance_intersection, merge},
    path::{PathDistance, PathQuery},
    types::Distance,
    validation::validate_distance,
};
use clap::Parser;
use indicatif::ProgressIterator;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Contraction hierarchy in .fmi format
    #[arg(short, long)]
    contraction_hierarchy: PathBuf,

    /// Test queries with expected distances
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

    let num_failures = validate_tests(&hub_labeling, &tests);

    if num_failures > 0 {
        eprintln!("{} of {} distances failed.", num_failures, tests.len());
        std::process::exit(1);
    }

    println!("All {} distances correct.", tests.len());
}

fn validate_tests<D: Distance>(hub_labeling: &HubLabeling<D>, tests: &[PathDistance<D>]) -> usize {
    tests
        .iter()
        .progress()
        .filter_map(|test| {
            let distance = distance(hub_labeling, test.query());
            validate_distance(test, &distance).err()
        })
        .inspect(|message| eprintln!("{message}"))
        .count()
}

fn distance<D: Distance>(hub_labeling: &HubLabeling<D>, query: &PathQuery) -> Option<D> {
    let source_label = hub_labeling.up_hub_labeling.nested(query.source.as_usize());
    let target_label = hub_labeling
        .down_hub_labeling
        .nested(query.target.as_usize());

    min_distance_intersection(source_label, target_label).map(|(distance, _, _)| distance)
}
