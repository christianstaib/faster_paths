use faster_paths::graphs::{
    edge::WeightedEdge, graph_functions::add_edge_bidrectional,
    reversible_vec_graph::ReversibleVecGraph,
};

fn get_small_graph() -> ReversibleVecGraph {
    // https://jlazarsfeld.github.io/ch.150.project/img/contraction/contract-full-1.png
    let mut graph = ReversibleVecGraph::new();
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(0, 1, 3).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(0, 2, 5).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(0, 10, 3).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(1, 2, 3).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(1, 3, 5).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(2, 3, 2).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(2, 9, 2).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(3, 4, 7).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(3, 9, 4).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(4, 5, 6).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(4, 9, 3).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(5, 6, 4).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(5, 7, 2).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(6, 7, 3).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(6, 8, 5).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(7, 8, 3).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(7, 9, 2).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(8, 9, 4).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(8, 10, 6).unwrap());
    add_edge_bidrectional(&mut graph, &WeightedEdge::new(9, 10, 3).unwrap());
    graph
}

#[test]
fn dijkstra() {}
