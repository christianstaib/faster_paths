use super::contracted_graph::DirectedContractedGraph;
use crate::{
    classical_search::bidirectional_helpers::path_from_bidirectional_search,
    dijkstra_data::{dijkstra_data_map::DijkstraDataHashMap, DijkstraData},
    graphs::{
        path::{Path, PathFinding, PathFindingWithInternalState, ShortestPathRequest},
        VertexId, Weight,
    },
    queue::DijkstraQueueElement,
};

pub struct ChDijkstraState<'a> {
    ch: &'a DirectedContractedGraph,
    forward_data: DijkstraDataHashMap,
    backward_data: DijkstraDataHashMap,
    meeting_vertex: VertexId,
    meeting_weight: Weight,
    forward_search_limit: Weight,
    backward_search_limit: Weight,
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

    fn number_of_vertices(&self) -> u32 {
        self.ch.number_of_vertices()
    }
}

impl<'a> ChDijkstraState<'a> {
    pub fn new(ch: &'a DirectedContractedGraph) -> ChDijkstraState<'_> {
        let number_of_vertices = ch.number_of_vertices() as usize;
        ChDijkstraState {
            ch,
            forward_data: DijkstraDataHashMap::new(number_of_vertices, 0),
            backward_data: DijkstraDataHashMap::new(number_of_vertices, 0),
            meeting_vertex: u32::MAX,
            meeting_weight: u32::MAX,
            forward_search_limit: 0,
            backward_search_limit: 0,
        }
    }

    fn clear(&mut self, request: &ShortestPathRequest) {
        // Clear the forward and backward data for the source and target vertices
        self.forward_data.clear(request.source());
        self.backward_data.clear(request.target());

        // Initialize the meeting weight and vertex to the maximum possible value
        self.meeting_weight = u32::MAX;
        self.meeting_vertex = u32::MAX;

        // Initialize the forward and backward search limits
        self.forward_search_limit = 0;
        self.backward_search_limit = 0;
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
        self.clear(request);

        // Run the bidirectional search
        while (!self.forward_data.is_empty() && (self.forward_search_limit < self.meeting_weight))
            || (!self.backward_data.is_empty()
                && (self.backward_search_limit < self.meeting_weight))
        {
            // Perform the forward search step
            if self.forward_search_limit < self.meeting_weight {
                self.process_forward_step();
            }

            // Perform the backward search step
            if self.backward_search_limit < self.meeting_weight {
                self.process_backward_step();
            }

            // Break if both search limits have reached or exceeded the meeting weight
            if self.forward_search_limit >= self.meeting_weight
                && self.backward_search_limit >= self.meeting_weight
            {
                break;
            }
        }

        // Return the meeting vertex, weight, and the forward and backward data
        (
            self.meeting_vertex,
            self.meeting_weight,
            &self.forward_data,
            &self.backward_data,
        )
    }

    fn process_forward_step(&mut self) {
        if let Some(DijkstraQueueElement { vertex, .. }) = self.forward_data.pop() {
            let forward_weight = self.forward_data.get_vertex_entry(vertex).weight.unwrap();
            self.forward_search_limit = std::cmp::max(self.forward_search_limit, forward_weight);

            if self.is_forward_stalled(vertex, forward_weight) {
                return;
            }

            if let Some(backward_weight) = self.backward_data.get_vertex_entry(vertex).weight {
                let total_weight = forward_weight + backward_weight;
                if total_weight < self.meeting_weight {
                    self.meeting_weight = total_weight;
                    self.meeting_vertex = vertex;
                }
            }
            self.ch
                .upward_edges(vertex)
                .for_each(|edge| self.forward_data.update(vertex, edge.head(), edge.weight()));
        }
    }

    fn process_backward_step(&mut self) {
        if let Some(DijkstraQueueElement { vertex, .. }) = self.backward_data.pop() {
            let backward_weight = self.backward_data.get_vertex_entry(vertex).weight.unwrap();
            self.backward_search_limit = std::cmp::max(self.backward_search_limit, backward_weight);

            if self.is_backward_stalled(vertex, backward_weight) {
                return;
            }

            if let Some(forward_weight) = self.forward_data.get_vertex_entry(vertex).weight {
                let total_weight = forward_weight + backward_weight;
                if total_weight < self.meeting_weight {
                    self.meeting_weight = total_weight;
                    self.meeting_vertex = vertex;
                }
            }
            self.ch.downard_edges(vertex).for_each(|edge| {
                self.backward_data
                    .update(vertex, edge.head(), edge.weight());
            });
        }
    }

    fn is_forward_stalled(&mut self, vertex: VertexId, forward_weight: u32) -> bool {
        for in_edge in self.ch.downard_edges(vertex) {
            if let Some(predecessor_weight) =
                self.forward_data.get_vertex_entry(in_edge.head()).weight
            {
                if predecessor_weight + in_edge.weight() < forward_weight {
                    return true;
                }
            }
        }
        false
    }

    fn is_backward_stalled(&mut self, vertex: VertexId, backward_weight: u32) -> bool {
        for out_edge in self.ch.upward_edges(vertex) {
            if let Some(predecessor_weight) =
                self.backward_data.get_vertex_entry(out_edge.head()).weight
            {
                if predecessor_weight + out_edge.weight() < backward_weight {
                    return true;
                }
            }
        }
        false
    }
}
