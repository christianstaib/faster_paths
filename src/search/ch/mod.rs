use std::{fs::File, io::BufReader};

use contracted_graph::ContractedGraph;

pub mod brute_force;
pub mod contracted_graph;
pub mod contraction;
pub mod contraction_generic;
pub mod probabilistic_contraction;

pub fn large_test_contracted_graph() -> ContractedGraph {
    let reader = BufReader::new(File::open("tests/data/stgtregbz_contracted_graph.json").unwrap());
    serde_json::from_reader(reader).unwrap()
}
