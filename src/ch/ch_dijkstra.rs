use super::ContractedGraphTrait;
use crate::{
    classical_search::bidirectional_helpers::path_from_bidirectional_search,
    dijkstra_data::{dijkstra_data_map::DijkstraDataHashMap, DijkstraData},
    graphs::{
        path::{Path, PathFinding, ShortestPathRequest},
        VertexId, Weight,
    },
    queue::DijkstraQueueElement,
};

pub struct ChDijkstra<'a> {
    ch: &'a dyn ContractedGraphTrait,
}

impl<'a> PathFinding for ChDijkstra<'a> {
    fn shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let (meeting_vertex, weight, forward, backward) = self.get_data(route_request);
        if weight == u32::MAX {
            return None;
        }
        let path = path_from_bidirectional_search(meeting_vertex, &forward, &backward)?;
        Some(path)
    }

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let (_, weight, _, _) = self.get_data(path_request);
        if weight == u32::MAX {
            return None;
        }
        Some(weight)
    }
}

impl<'a> ChDijkstra<'a> {
    pub fn new(ch: &'a dyn ContractedGraphTrait) -> ChDijkstra<'_> {
        ChDijkstra { ch }
    }

    pub fn get_data(
        &self,
        request: &ShortestPathRequest,
    ) -> (
        VertexId,
        Weight,
        Box<dyn DijkstraData>,
        Box<dyn DijkstraData>,
    ) {
        let number_of_vertices = self.ch.number_of_vertices() as usize;
        let forward_data = DijkstraDataHashMap::new(number_of_vertices, request.source());
        let backward_data = DijkstraDataHashMap::new(number_of_vertices, request.target());

        let mut forward_data: Box<dyn DijkstraData> = Box::new(forward_data);
        let mut backward_data: Box<dyn DijkstraData> = Box::new(backward_data);

        let mut meeting_weight = u32::MAX;
        let mut meeting_vertex = u32::MAX;

        let mut f = 0;
        let mut b = 0;

        while (!forward_data.is_empty() && (f < meeting_weight))
            || (!backward_data.is_empty() && (b < meeting_weight))
        {
            if f < meeting_weight {
                if let Some(DijkstraQueueElement { vertex, .. }) = forward_data.pop() {
                    let forward_weight = forward_data.get_vertex_entry(vertex).weight.unwrap();
                    f = std::cmp::max(f, forward_weight);

                    let mut stall = false;
                    for in_edge in self.ch.downard_edges(vertex) {
                        if let Some(predecessor_weight) =
                            forward_data.get_vertex_entry(in_edge.head()).weight
                        {
                            if predecessor_weight + in_edge.weight() < forward_weight {
                                stall = true;
                                break;
                            }
                        }
                    }

                    if !stall {
                        if let Some(backward_weight) = backward_data.get_vertex_entry(vertex).weight
                        {
                            let weight = forward_weight + backward_weight;
                            if weight < meeting_weight {
                                meeting_weight = weight;
                                meeting_vertex = vertex;
                            }
                        }
                        self.ch.upward_edges(vertex).for_each(|edge| {
                            forward_data.update(vertex, edge.head(), edge.weight())
                        });
                    }
                }
            }

            if b < meeting_weight {
                if let Some(DijkstraQueueElement { vertex, .. }) = backward_data.pop() {
                    let backward_weight = backward_data.get_vertex_entry(vertex).weight.unwrap();
                    b = std::cmp::max(b, backward_weight);

                    let mut stall = false;
                    for out_edge in self.ch.upward_edges(vertex) {
                        if let Some(predecessor_weight) =
                            backward_data.get_vertex_entry(out_edge.head()).weight
                        {
                            if predecessor_weight + out_edge.weight() < backward_weight {
                                stall = true;
                                break;
                            }
                        }
                    }

                    if !stall {
                        if let Some(forward_weight) = forward_data.get_vertex_entry(vertex).weight {
                            let weight = forward_weight + backward_weight;
                            if weight < meeting_weight {
                                meeting_weight = weight;
                                meeting_vertex = vertex;
                            }
                        }
                        self.ch.downard_edges(vertex).for_each(|edge| {
                            backward_data.update(vertex, edge.head(), edge.weight());
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
