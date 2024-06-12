use super::helpers::get_data;
use crate::{
    ch::directed_contracted_graph::DirectedContractedGraph,
    classical_search::bidirectional_helpers::path_from_bidirectional_search,
    dijkstra_data::dijkstra_data_map::DijkstraDataHashMap,
    graphs::{
        path::{Path, PathFinding, ShortestPathRequest},
        Graph, Weight,
    },
    shortcut_replacer::slow_shortcut_replacer::replace_shortcuts_slow,
};

impl PathFinding for DirectedContractedGraph {
    fn shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let number_of_vertices = self.number_of_vertices() as usize;
        let mut forward_data = DijkstraDataHashMap::new(number_of_vertices, route_request.source());
        let mut backward_data =
            DijkstraDataHashMap::new(number_of_vertices, route_request.target());

        let (meeting_vertex, weight) = get_data(self, &mut forward_data, &mut backward_data);
        if weight == u32::MAX {
            return None;
        }
        let mut path =
            path_from_bidirectional_search(meeting_vertex, &forward_data, &backward_data)?;

        replace_shortcuts_slow(&mut path.vertices, &self.shortcuts);

        Some(path)
    }

    fn shortest_path_weight(&self, route_request: &ShortestPathRequest) -> Option<Weight> {
        let number_of_vertices = self.number_of_vertices() as usize;
        let mut forward_data = DijkstraDataHashMap::new(number_of_vertices, route_request.source());
        let mut backward_data =
            DijkstraDataHashMap::new(number_of_vertices, route_request.target());

        let (_, weight) = get_data(self, &mut forward_data, &mut backward_data);
        if weight == u32::MAX {
            return None;
        }
        Some(weight)
    }

    fn number_of_vertices(&self) -> u32 {
        self.upward_graph.number_of_vertices()
    }
}
