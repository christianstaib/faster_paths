use std::{fs::File, io::BufReader};

use osm_test::{
    ch::contractor::Contractor,
    fast_graph::FastGraph,
    graph::Graph,
    naive_graph::NaiveGraph,
    path::{RouteValidationRequest, Routing},
    simple_algorithms::{bi_dijkstra::BiDijkstra, ch_bi_dijkstra::ChDijkstra, dijkstra::Dijkstra},
};

#[test]
fn dijkstra() {
    let naive_graph = NaiveGraph::from_gr_file("tests/data/fmi/USA-road-d.NY.gr");
    let graph = Graph::from_edges(&naive_graph.edges);

    let reader = BufReader::new(File::open("tests/data/fmi/USA-road-d.NY.gr.tests.json").unwrap());
    let tests: Vec<RouteValidationRequest> = serde_json::from_reader(reader).unwrap();

    let ch_graph = Contractor::get_contracted_graph(&graph);
    let ch_bi_dijkstra = ChDijkstra::new(&ch_graph);

    let fast_graph = FastGraph::from_graph(&graph);

    let dijkstra = Dijkstra::new(&fast_graph);

    let bi_dijkstra = BiDijkstra::new(&fast_graph);

    let hl_graph = ch_bi_dijkstra.get_hl();

    for test in tests.iter() {
        let request = &test.request;

        // test dijkstra
        let response = dijkstra.get_route(&request);
        let mut cost = None;
        if let Some(route) = response {
            cost = Some(route.weight);
        }
        assert_eq!(test.cost, cost, "dijkstra wrong");

        // test bi dijkstra
        let response = bi_dijkstra.get_route(&request);
        let mut cost = None;
        if let Some(route) = response {
            cost = Some(route.weight);
        }
        assert_eq!(test.cost, cost, "bi dijkstra wrong");

        // test ch dijkstra
        let response = ch_bi_dijkstra.get_route(&request);
        let mut cost = None;
        if let Some(route) = response {
            cost = Some(route.weight);
        }
        assert_eq!(test.cost, cost, "ch dijkstra wrong");

        // test hl
        let response = hl_graph.get_path(&request);
        let mut cost = None;
        if let Some(this_cost) = response {
            cost = Some(this_cost.weight);
        }
        assert_eq!(test.cost, cost, "hl wrong");
    }
}
