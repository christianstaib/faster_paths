// use std::{fs::File, io::BufReader};
//
// use faster_paths::{
//     ch::contractor::Contractor,
//     graphs::fast_graph::FastGraph,
//     graphs::graph_factory::GraphFactory,
//     graphs::path::{Routing, ShortestPathValidation},
//     simple_algorithms::{bi_dijkstra::BiDijkstra, ch_bi_dijkstra::ChDijkstra, dijkstra::Dijkstra},
// };
//
// #[test]
// fn dijkstra() {
//     let graph = GraphFactory::from_gr_file("tests/data/USA-road-d.NY.gr");
//
//     let reader = BufReader::new(File::open("tests/data/USA-road-d.NY.gr.tests.json").unwrap());
//     let tests: Vec<ShortestPathValidation> = serde_json::from_reader(reader).unwrap();
//
//     let ch_graph = Contractor::get_contracted_graph(&graph);
//     let ch_bi_dijkstra = ChDijkstra::new(&ch_graph);
//
//     let fast_graph = FastGraph::from_graph(&graph);
//
//     let dijkstra = Dijkstra::new(&fast_graph);
//
//     let bi_dijkstra = BiDijkstra::new(&fast_graph);
//
//     let hl_graph = ch_bi_dijkstra.get_hl();
//
//     for test in tests.iter() {
//         let request = &test.request;
//
//         // test dijkstra
//         let path = dijkstra.get_shortest_path(&request);
//         let mut cost = None;
//         if let Some(path) = path {
//             cost = Some(path.weight);
//             graph.validate_route(request, &path);
//         }
//         assert_eq!(test.weight, cost, "dijkstra wrong");
//
//         // test bi dijkstra
//         let path = bi_dijkstra.get_shortest_path(&request);
//         let mut cost = None;
//         if let Some(path) = path {
//             cost = Some(path.weight);
//         }
//         assert_eq!(test.weight, cost, "bi dijkstra wrong");
//
//         // test ch dijkstra
//         let path = ch_bi_dijkstra.get_route(&request);
//         let mut cost = None;
//         if let Some(path) = path {
//             cost = Some(path.weight);
//         }
//         assert_eq!(test.weight, cost, "ch dijkstra wrong");
//
//         // test hl
//         let path = hl_graph.get_path(&request);
//         let mut cost = None;
//         if let Some(path) = path {
//             cost = Some(path.weight);
//             graph.validate_route(request, &path);
//         }
//         assert_eq!(test.weight, cost, "hl wrong");
//     }
// }
