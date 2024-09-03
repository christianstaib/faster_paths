use super::generic::contraction;
use crate::{
    graphs::{reversible_graph::ReversibleGraph, Graph, Level},
    search::{
        ch::{
            bottom_up::heuristic::par_simulate_contraction_heuristic,
            contracted_graph::ContractedGraph,
        },
        DistanceHeuristic,
    },
};

impl ContractedGraph {
    pub fn by_contraction_top_down_with_heuristic<G: Graph + Default + Clone>(
        graph: &ReversibleGraph<G>,
        level_to_vertex: &Vec<Level>,
        heuristic: &dyn DistanceHeuristic,
    ) -> ContractedGraph {
        let graph = graph.clone();
        let (level_to_vertex, edges, shortcuts) =
            contraction(graph, level_to_vertex, |graph, vertex| {
                par_simulate_contraction_heuristic(graph, heuristic, vertex)
            });

        ContractedGraph::new(level_to_vertex, edges, shortcuts)
    }
}
