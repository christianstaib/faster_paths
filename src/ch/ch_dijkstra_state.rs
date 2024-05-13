use super::ContractedGraphTrait;
use crate::{
    classical_search::bidirectional_helpers::path_from_bidirectional_search,
    dijkstra_data::{dijkstra_data_map::DijkstraDataHashMap, DijkstraData},
    graphs::{
        path::{Path, PathFindingWithInternalState, ShortestPathRequest},
        VertexId, Weight,
    },
    queue::DijkstraQueueElement,
};

pub struct ChDijkstraState<'a> {
    ch: &'a dyn ContractedGraphTrait,
    forward_data: DijkstraDataHashMap,
    backward_data: DijkstraDataHashMap,
}

impl<'a> PathFindingWithInternalState for ChDijkstraState<'a> {
    fn shortest_path(&mut self, route_request: &ShortestPathRequest) -> Option<Path> {
        let (meeting_vertex, weight, forward, backward) = self.get_data(route_request);
        if weight == u32::MAX {
            return None;
        }
        let path = path_from_bidirectional_search(meeting_vertex, forward, backward)?;
        Some(path)
    }

    fn shortest_path_weight(&mut self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let (_, weight, _, _) = self.get_data(path_request);
        if weight == u32::MAX {
            return None;
        }
        Some(weight)
    }
}

impl<'a> ChDijkstraState<'a> {
    pub fn new(ch: &'a dyn ContractedGraphTrait) -> ChDijkstraState<'_> {
        let number_of_vertices = ch.number_of_vertices() as usize;
        ChDijkstraState {
            ch,
            forward_data: DijkstraDataHashMap::new(number_of_vertices, 0),
            backward_data: DijkstraDataHashMap::new(number_of_vertices, 0),
        }
    }

    pub fn get_data(
        &mut self,
        request: &ShortestPathRequest,
    ) -> (
        VertexId,
        Weight,
        &'_ DijkstraDataHashMap,
        &'_ DijkstraDataHashMap,
    ) {
        self.forward_data.clear(request.source());
        self.backward_data.clear(request.target());

        let mut meeting_weight = u32::MAX;
        let mut meeting_vertex = u32::MAX;

        let mut f = 0;
        let mut b = 0;

        while (!self.forward_data.is_empty() && (f < meeting_weight))
            || (!self.backward_data.is_empty() && (b < meeting_weight))
        {
            if f < meeting_weight {
                if let Some(DijkstraQueueElement { vertex, .. }) = self.forward_data.pop() {
                    let forward_weight = self.forward_data.get_vertex_entry(vertex).weight.unwrap();
                    f = std::cmp::max(f, forward_weight);

                    let mut stall = false;
                    for in_edge in self.ch.downard_edges(vertex) {
                        if let Some(predecessor_weight) =
                            self.forward_data.get_vertex_entry(in_edge.head()).weight
                        {
                            if predecessor_weight + in_edge.weight() < forward_weight {
                                stall = true;
                                break;
                            }
                        }
                    }

                    if !stall {
                        if let Some(backward_weight) =
                            self.backward_data.get_vertex_entry(vertex).weight
                        {
                            let weight = forward_weight + backward_weight;
                            if weight < meeting_weight {
                                meeting_weight = weight;
                                meeting_vertex = vertex;
                            }
                        }
                        self.ch.upward_edges(vertex).for_each(|edge| {
                            self.forward_data.update(vertex, edge.head(), edge.weight())
                        });
                    }
                }
            }

            if b < meeting_weight {
                if let Some(DijkstraQueueElement { vertex, .. }) = self.backward_data.pop() {
                    let backward_weight =
                        self.backward_data.get_vertex_entry(vertex).weight.unwrap();
                    b = std::cmp::max(b, backward_weight);

                    let mut stall = false;
                    for out_edge in self.ch.upward_edges(vertex) {
                        if let Some(predecessor_weight) =
                            self.backward_data.get_vertex_entry(out_edge.head()).weight
                        {
                            if predecessor_weight + out_edge.weight() < backward_weight {
                                stall = true;
                                break;
                            }
                        }
                    }

                    if !stall {
                        if let Some(forward_weight) =
                            self.forward_data.get_vertex_entry(vertex).weight
                        {
                            let weight = forward_weight + backward_weight;
                            if weight < meeting_weight {
                                meeting_weight = weight;
                                meeting_vertex = vertex;
                            }
                        }
                        self.ch.downard_edges(vertex).for_each(|edge| {
                            self.backward_data
                                .update(vertex, edge.head(), edge.weight());
                        });
                    }
                }
            }

            if f >= meeting_weight && b >= meeting_weight {
                break;
            }
        }

        (
            meeting_vertex,
            meeting_weight,
            &self.forward_data,
            &self.backward_data,
        )
    }
}
