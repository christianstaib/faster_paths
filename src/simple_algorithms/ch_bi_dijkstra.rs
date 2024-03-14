use std::usize;

use crate::{
    ch::{shortcut_replacer::ShortcutReplacer, ContractedGraphInformation},
    dijkstra_data::DijkstraData,
    graphs::{
        fast_graph::FastGraph,
        path::{Path, PathFinding, ShortestPathRequest},
        types::{VertexId, Weight},
    },
    queue::DijkstraQueueElement,
};

use super::bidirectional_helpers::path_from_bidirectional_search;

pub struct ChDijkstra<'a> {
    ch_graph: &'a FastGraph,
    shortuct_replacer: &'a Box<dyn ShortcutReplacer>,
}

impl<'a> PathFinding for ChDijkstra<'a> {
    fn get_shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let (meeting_vertex, _, forward, backward) = self.get_data(&route_request)?;
        let path = path_from_bidirectional_search(meeting_vertex, &forward, &backward)?;
        let path = self.shortuct_replacer.replace_shortcuts(&path);
        Some(path)
    }

    fn get_shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let (_, weight, _, _) = self.get_data(path_request)?;
        Some(weight)
    }
}

impl<'a> ChDijkstra<'a> {
    pub fn new(
        ch_graph: &'a FastGraph,
        shortuct_replacer: &'a Box<dyn ShortcutReplacer>,
    ) -> ChDijkstra<'a> {
        ChDijkstra {
            ch_graph,
            shortuct_replacer,
        }
    }

    pub fn get_data(
        &self,
        request: &ShortestPathRequest,
    ) -> Option<(VertexId, Weight, DijkstraData, DijkstraData)> {
        let number_of_vertices = self.ch_graph.number_of_vertices() as usize;
        let mut forward_data = DijkstraData::new(number_of_vertices, request.source());
        let mut backward_data = DijkstraData::new(number_of_vertices, request.target());

        let mut meeting_weight = u32::MAX;
        let mut meeting_vertex = u32::MAX;

        while !forward_data.is_empty() || !backward_data.is_empty() {
            if let Some(DijkstraQueueElement { vertex, .. }) = forward_data.pop() {
                let forward_weight = forward_data.verticies[vertex as usize].weight.unwrap();

                let mut stall = false;
                for in_edge in self.ch_graph.in_edges(vertex).iter() {
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
                    self.ch_graph
                        .out_edges(vertex)
                        .iter()
                        .for_each(|edge| forward_data.update(vertex, edge.head, edge.weight));
                }
            }

            if let Some(DijkstraQueueElement { vertex, .. }) = backward_data.pop() {
                let backward_weight = backward_data.verticies[vertex as usize].weight.unwrap();

                let mut stall = false;
                for out_edge in self.ch_graph.out_edges(vertex).iter() {
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
                    self.ch_graph.in_edges(vertex).iter().for_each(|edge| {
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
