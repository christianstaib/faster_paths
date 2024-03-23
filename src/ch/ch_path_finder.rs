use std::usize;

use crate::{
    ch::shortcut_replacer::ShortcutReplacer,
    dijkstra_data_hash::DijkstraDataHash,
    graphs::{
        fast_graph::FastGraph,
        path::{Path, PathFinding, ShortestPathRequest},
        VertexId, Weight,
    },
    hl::label::Label,
    queue::DijkstraQueueElement,
    simple_algorithms::bidirectional_helpers::path_from_bidirectional_search,
};

pub struct ChPathFinder {
    ch_graph: FastGraph,
    shortcut_replacer: Box<dyn ShortcutReplacer>,
}

impl PathFinding for ChPathFinder {
    fn get_shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let (meeting_vertex, _, forward, backward) = self.get_data(route_request);
        let path = path_from_bidirectional_search(meeting_vertex, &forward, &backward)?;
        let path = self.shortcut_replacer.replace_shortcuts(&path);
        Some(path)
    }

    fn get_shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let (_, weight, _, _) = self.get_data(path_request);
        Some(weight)
    }
}

impl ChPathFinder {
    pub fn new(ch_graph: FastGraph, shortcut_replacer: Box<dyn ShortcutReplacer>) -> ChPathFinder {
        ChPathFinder {
            ch_graph,
            shortcut_replacer,
        }
    }

    // pub fn forward_search(&self, source: VertexId) -> DijkstraDataHash {
    //     let number_of_vertices = self.ch_graph.number_of_vertices() as usize;
    //     let mut data = DijkstraDataHash::new(number_of_vertices, source);

    //     while !data.is_empty() {
    //         if let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
    //             let forward_weight = data.vertices[vertex as usize].weight.unwrap();

    //             let mut stall = false;
    //             for in_edge in self.ch_graph.in_edges(vertex).iter() {
    //                 if let Some(predecessor_weight) = data.vertices[in_edge.tail as usize].weight {
    //                     if predecessor_weight + in_edge.weight < forward_weight {
    //                         stall = true;
    //                         break;
    //                     }
    //                 }
    //             }

    //             if !stall {
    //                 self.ch_graph
    //                     .out_edges(vertex)
    //                     .iter()
    //                     .for_each(|edge| data.update(vertex, edge.head, edge.weight));
    //             }
    //         }
    //     }

    //     data
    // }

    // pub fn backward_search(&self, source: VertexId) -> DijkstraDataHash {
    //     let number_of_vertices = self.ch_graph.number_of_vertices() as usize;
    //     let mut data = DijkstraDataHash::new(number_of_vertices, source);

    //     if let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
    //         let backward_weight = data.vertices[vertex as usize].weight.unwrap();

    //         let mut stall = false;
    //         for out_edge in self.ch_graph.out_edges(vertex).iter() {
    //             if let Some(predecessor_weight) = data.vertices[out_edge.head as usize].weight {
    //                 if predecessor_weight + out_edge.weight < backward_weight {
    //                     stall = true;
    //                     break;
    //                 }
    //             }
    //         }

    //         if !stall {
    //             self.ch_graph.in_edges(vertex).iter().for_each(|edge| {
    //                 data.update(vertex, edge.tail, edge.weight);
    //             });
    //         }
    //     }

    //     data
    // }

    pub fn get_data(
        &self,
        request: &ShortestPathRequest,
    ) -> (VertexId, Weight, DijkstraDataHash, DijkstraDataHash) {
        let number_of_vertices = self.ch_graph.number_of_vertices() as usize;
        let mut forward_data = DijkstraDataHash::new(number_of_vertices, request.source());
        let mut backward_data = DijkstraDataHash::new(number_of_vertices, request.target());

        let mut meeting_weight = u32::MAX;
        let mut meeting_vertex = u32::MAX;

        let mut f = 0;
        let mut b = 0;

        while (!forward_data.is_empty() && (f < meeting_weight))
            || (!backward_data.is_empty() && (b < meeting_weight))
        {
            if f < meeting_weight {
                if let Some(DijkstraQueueElement { vertex, .. }) = forward_data.pop() {
                    let forward_weight = forward_data.vertices[&vertex].weight.unwrap();
                    f = std::cmp::max(f, forward_weight);

                    let mut stall = false;
                    for in_edge in self.ch_graph.in_edges(vertex).iter() {
                        if let Some(predecessor_entry) = forward_data.vertices.get(&in_edge.tail) {
                            if let Some(predecessor_weight) = predecessor_entry.weight {
                                if predecessor_weight + in_edge.weight < forward_weight {
                                    stall = true;
                                    break;
                                }
                            }
                        }
                    }

                    if !stall {
                        if let Some(backward_entry) = backward_data.vertices.get(&vertex) {
                            if let Some(backward_weight) = backward_entry.weight {
                                let weight = forward_weight + backward_weight;
                                if weight < meeting_weight {
                                    meeting_weight = weight;
                                    meeting_vertex = vertex;
                                }
                            }
                        }
                        self.ch_graph
                            .out_edges(vertex)
                            .iter()
                            .for_each(|edge| forward_data.update(vertex, edge.head, edge.weight));
                    }
                }
            }

            if b < meeting_weight {
                if let Some(DijkstraQueueElement { vertex, .. }) = backward_data.pop() {
                    let backward_weight = backward_data.vertices[&vertex].weight.unwrap();
                    b = std::cmp::max(b, backward_weight);

                    let mut stall = false;
                    for out_edge in self.ch_graph.out_edges(vertex).iter() {
                        if let Some(predecessor_entry) = backward_data.vertices.get(&out_edge.head)
                        {
                            if let Some(predecessor_weight) = predecessor_entry.weight {
                                if predecessor_weight + out_edge.weight < backward_weight {
                                    stall = true;
                                    break;
                                }
                            }
                        }
                    }

                    if !stall {
                        if let Some(forward_entry) = forward_data.vertices.get(&vertex) {
                            if let Some(forward_weight) = forward_entry.weight {
                                let weight = forward_weight + backward_weight;
                                if weight < meeting_weight {
                                    meeting_weight = weight;
                                    meeting_vertex = vertex;
                                }
                            }
                        }
                        self.ch_graph.in_edges(vertex).iter().for_each(|edge| {
                            backward_data.update(vertex, edge.tail, edge.weight);
                        });
                    }
                }
            }

            if f >= meeting_weight && b >= meeting_weight {
                break;
            }
        }

        (meeting_vertex, meeting_weight, forward_data, backward_data)
    }
}
