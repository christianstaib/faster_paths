use super::helpers::get_data;
use crate::{
    ch::contracted_graph::DirectedContractedGraph,
    classical_search::bidirectional_helpers::path_from_bidirectional_search,
    dijkstra_data::dijkstra_data_map::DijkstraDataHashMap,
    graphs::{
        path::{Path, PathFinding, PathFindingWithInternalState, ShortestPathRequest},
        Weight,
    },
};

pub struct ChDijkstraState<'a> {
    ch: &'a DirectedContractedGraph,
    forward_data: DijkstraDataHashMap,
    backward_data: DijkstraDataHashMap,
}

impl<'a> PathFindingWithInternalState for ChDijkstraState<'a> {
    fn shortest_path(&mut self, route_request: &ShortestPathRequest) -> Option<Path> {
        self.clear(route_request);
        let (meeting_vertex, weight) =
            get_data(self.ch, &mut self.forward_data, &mut self.backward_data);
        if weight == u32::MAX {
            return None;
        }
        let path = path_from_bidirectional_search(
            meeting_vertex,
            &self.forward_data,
            &self.backward_data,
        )?;
        Some(path)
    }

    fn shortest_path_weight(&mut self, route_request: &ShortestPathRequest) -> Option<Weight> {
        self.clear(route_request);

        let (_, weight) = get_data(self.ch, &mut self.forward_data, &mut self.backward_data);
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
        }
    }

    fn clear(&mut self, request: &ShortestPathRequest) {
        // Clear the forward and backward data for the source and target vertices
        self.forward_data.clear(request.source());
        self.backward_data.clear(request.target());
    }
}
