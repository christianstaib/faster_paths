use ch::ch::contraction_hierarchy::ContractionHierarchy;
use ch::ch::edge::Edge;
use ch::ch::pathfinder::Pathfinder;
use ch::flattened_nested::FlattenedNested;
use ch::fmi_helper::{read_fmi_ch, read_tests};
use ch::path::PathDistance;
use ch::search_state::hash_search_state::HashSearchState;
use ch::types::{Distance, VertexId};
use clap::Parser;
use std::collections::BinaryHeap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// CH graph in .fmi format
    #[arg(short, long)]
    graph_in: PathBuf,

    /// Test queries in .txt format
    #[arg(short, long)]
    test_in: PathBuf,
}

fn main() {
    let args = Args::parse();

    let ch = read_fmi_ch(&args.graph_in).unwrap();
    let tests = read_tests(&args.test_in).unwrap();
    let mut pathfinder = Pathfinder::new(
        &ch,
        BinaryHeap::new(),
        HashSearchState::new(),
        HashSearchState::new(),
    );

    let mut failures = 0;
    for test in &tests {
        if let Err(message) = validate_path(&ch, &mut pathfinder, test) {
            failures += 1;
            eprintln!("{message}");
        }
    }

    if failures == 0 {
        println!("All {} paths correct.", tests.len());
    } else {
        println!("{failures} of {} paths failed.", tests.len());
        std::process::exit(1);
    }
}

fn validate_path(
    ch: &ContractionHierarchy,
    pathfinder: &mut Pathfinder<'_>,
    test: &PathDistance,
) -> Result<(), String> {
    let path = pathfinder.path(test.query());

    match (path, test.distance()) {
        (None, None) => Ok(()),
        (None, Some(expected)) => Err(format!(
            "No path found for {:?} -> {:?}; expected {:?}",
            test.query().source(),
            test.query().target(),
            expected
        )),
        (Some(path), None) => Err(format!(
            "Path found for unreachable query {:?} -> {:?}: {:?}",
            test.query().source(),
            test.query().target(),
            path.vertices()
        )),
        (Some(path), Some(expected)) => {
            if path.distance() != expected {
                return Err(format!(
                    "Wrong distance for {:?} -> {:?}: got {:?}, expected {:?}",
                    test.query().source(),
                    test.query().target(),
                    path.distance(),
                    expected
                ));
            }

            if path.vertices().first() != Some(&test.query().source()) {
                return Err(format!(
                    "Path for {:?} -> {:?} starts at {:?}",
                    test.query().source(),
                    test.query().target(),
                    path.vertices().first()
                ));
            }

            if path.vertices().last() != Some(&test.query().target()) {
                return Err(format!(
                    "Path for {:?} -> {:?} ends at {:?}",
                    test.query().source(),
                    test.query().target(),
                    path.vertices().last()
                ));
            }

            let Some(sum) = sum_leaf_path(path.vertices(), ch) else {
                return Err(format!(
                    "Path for {:?} -> {:?} contains a non-leaf or missing edge: {:?}",
                    test.query().source(),
                    test.query().target(),
                    path.vertices()
                ));
            };

            if sum != expected {
                return Err(format!(
                    "Expanded path sum wrong for {:?} -> {:?}: got {:?}, expected {:?}; path {:?}",
                    test.query().source(),
                    test.query().target(),
                    sum,
                    expected,
                    path.vertices()
                ));
            }

            Ok(())
        }
    }
}

fn sum_leaf_path(path: &[VertexId], ch: &ContractionHierarchy) -> Option<Distance> {
    let mut sum = Distance::ZERO;

    for window in path.windows(2) {
        sum = sum + leaf_edge_weight(window[0], window[1], ch)?;
    }

    Some(sum)
}

fn leaf_edge_weight(tail: VertexId, head: VertexId, ch: &ContractionHierarchy) -> Option<Distance> {
    find_leaf_edge_weight(ch.up_graph(), tail, head)
        .or_else(|| find_leaf_edge_weight(ch.down_graph(), head, tail))
}

fn find_leaf_edge_weight(
    graph: &FlattenedNested<Edge>,
    tail: VertexId,
    head: VertexId,
) -> Option<Distance> {
    if tail.as_usize() >= graph.num_nested() {
        return None;
    }

    graph
        .nested(tail.as_usize())
        .iter()
        .find(|edge| edge.head() == head && edge.skipped().is_none())
        .map(Edge::weight)
}
