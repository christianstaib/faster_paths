use std::usize;

use crate::{
    ch::{
        preprocessor::ContractedGraph,
        shortcut_replacer::{slow_shortcut_replacer::SlowShortcutReplacer, ShortcutReplacer},
    },
    dijkstra_data::DijkstraData,
    graphs::{
        fast_graph::FastGraph,
        path::{Path, PathFinding, ShortestPathRequest},
        types::{VertexId, Weight},
    },
    queue::heap_queue::State,
};

use super::bidirectional_helpers::path_from_bidirectional_search;

#[derive(Clone)]
pub struct ChDijkstra {
    graph: FastGraph,
    shortcut_replacer: SlowShortcutReplacer,
}

impl PathFinding for ChDijkstra {
    fn get_shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let (meeting_vertex, _, forward, backward) = self.get_data(&route_request)?;
        let path = path_from_bidirectional_search(meeting_vertex, &forward, &backward)?;
        let path = self.shortcut_replacer.get_path(&path);
        Some(path)
    }

    fn get_shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let (_, weight, _, _) = self.get_data(path_request)?;
        Some(weight)
    }
}

impl ChDijkstra {
    pub fn new(contracted_graph: &ContractedGraph) -> ChDijkstra {
        let graph = FastGraph::from_graph(&contracted_graph.graph);
        let shortcut_map = contracted_graph.shortcuts_map.iter().cloned().collect();
        let shortcut_replacer = SlowShortcutReplacer::new(&shortcut_map);
        ChDijkstra {
            graph,
            shortcut_replacer,
        }
    }

    pub fn get_data(
        &self,
        request: &ShortestPathRequest,
    ) -> Option<(VertexId, Weight, DijkstraData, DijkstraData)> {
        let number_of_vertices = self.graph.number_of_vertices() as usize;
        let mut forward_data = DijkstraData::new(number_of_vertices, request.source());
        let mut backward_data = DijkstraData::new(number_of_vertices, request.target());

        let mut meeting_weight = u32::MAX;
        let mut meeting_vertex = u32::MAX;

        while !forward_data.is_empty() || !backward_data.is_empty() {
            if let Some(State { vertex, .. }) = forward_data.pop() {
                let forward_weight = forward_data.verticies[vertex as usize].weight.unwrap();

                let mut stall = false;
                for in_edge in self.graph.in_edges(vertex).iter() {
                    if let Some(predecessor_weight) =
                        forward_data.verticies[in_edge.tail as usize].weight
                    {
                        if predecessor_weight + in_edge.weight < forward_weight {
                            stall = true;
                            break;
                        }
                    }
                }

                if !stall {
                    if let Some(backward_weight) = backward_data.verticies[vertex as usize].weight {
                        let weight = forward_weight + backward_weight;
                        if weight < meeting_weight {
                            meeting_weight = weight;
                            meeting_vertex = vertex;
                        }
                    }
                    self.graph
                        .out_edges(vertex)
                        .iter()
                        .for_each(|edge| forward_data.update(vertex, edge.head, edge.weight));
                }
            }

            if let Some(State { vertex, .. }) = backward_data.pop() {
                let backward_weight = backward_data.verticies[vertex as usize].weight.unwrap();

                let mut stall = false;
                for out_edge in self.graph.out_edges(vertex).iter() {
                    if let Some(predecessor_weight) =
                        backward_data.verticies[out_edge.head as usize].weight
                    {
                        if predecessor_weight + out_edge.weight < backward_weight {
                            stall = true;
                            break;
                        }
                    }
                }

                if !stall {
                    if forward_data.verticies[vertex as usize].is_expanded {
                        if let Some(forward_weight) = forward_data.verticies[vertex as usize].weight
                        {
                            let weight = forward_weight + backward_weight;
                            if weight < meeting_weight {
                                meeting_weight = weight;
                                meeting_vertex = vertex;
                            }
                        }
                    }
                    self.graph.in_edges(vertex).iter().for_each(|edge| {
                        backward_data.update(vertex, edge.tail, edge.weight);
                    });
                }
            }
        }

        if meeting_weight == u32::MAX {
            return None;
        }

        Some((meeting_vertex, meeting_weight, forward_data, backward_data))
    }
}
