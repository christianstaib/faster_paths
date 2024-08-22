use faster_paths::{
    graphs::large_test_graph,
    search::{
        ch::{contracted_graph::ContractedGraph, contraction::contraction_with_witness_search},
        dijkstra::dijkstra_one_to_one_wrapped,
        path::{ShortestPathRequest, ShortestPathTestCase},
    },
};

#[test]
fn dijkstra() {
    let (graph, test_cases) = large_test_graph();

    test_cases.iter().for_each(
        |ShortestPathTestCase {
             request: ShortestPathRequest { source, target },
             distance,
         }| {
            let dijkstra_distance =
                dijkstra_one_to_one_wrapped(graph.out_graph(), *source, *target)
                    .map(|path| path.distance);

            assert_eq!(distance, &dijkstra_distance);
        },
    );
}

#[test]
fn ch_wittness() {
    let (graph, test_cases) = large_test_graph();

    let (level_to_vertex, edges) = contraction_with_witness_search(graph);
    let contracted_graph = ContractedGraph::new(edges, &level_to_vertex);

    test_cases.iter().for_each(
        |ShortestPathTestCase {
             request: ShortestPathRequest { source, target },
             distance,
         }| {
            let ch_distance = contracted_graph.shortest_path_distance(*source, *target);

            assert_eq!(distance, &ch_distance);
        },
    );
}
