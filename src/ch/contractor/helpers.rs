use itertools::Itertools;

use crate::graphs::{adjacency_vec_graph::AdjacencyVecGraph, edge::WeightedEdge, Graph};

pub fn partition_by_levels(
    graph: &dyn Graph,
    levels: &[Vec<u32>],
) -> (AdjacencyVecGraph, AdjacencyVecGraph) {
    let mut vertex_to_level = vec![0; graph.number_of_vertices() as usize];
    for (level, level_list) in levels.iter().enumerate() {
        for &vertex in level_list.iter() {
            vertex_to_level[vertex as usize] = level;
        }
    }

    let edges: Vec<_> = (0..graph.number_of_vertices())
        .flat_map(|vertex| graph.out_edges(vertex))
        .collect();

    let order = levels.iter().flatten().cloned().collect_vec();

    println!("creating upward graph");
    let upward_edges: Vec<_> = edges
        .iter()
        .filter(|edge| {
            vertex_to_level[edge.tail() as usize] <= vertex_to_level[edge.head() as usize]
        })
        .cloned()
        .collect();
    let upward_graph = AdjacencyVecGraph::new(&upward_edges, &order);

    println!("creating downward graph");
    let downward_edges: Vec<_> = edges
        .iter()
        .map(WeightedEdge::reversed)
        .filter(|edge| {
            vertex_to_level[edge.tail() as usize] <= vertex_to_level[edge.head() as usize]
        })
        .collect();
    let downard_graph = AdjacencyVecGraph::new(&downward_edges, &order);

    (upward_graph, downard_graph)
}
