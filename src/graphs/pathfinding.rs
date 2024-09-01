use super::{Distance, Graph, Vertex};
use crate::search::{
    collections::dijkstra_data::Path,
    dijkstra::{dijkstra_one_to_one_distance_wrapped, dijkstra_one_to_one_path_wrapped},
    PathFinding,
};

impl<T: Graph> PathFinding for T {
    fn shortest_path(&self, source: Vertex, target: Vertex) -> Option<Path> {
        dijkstra_one_to_one_path_wrapped(self, source, target)
    }

    fn shortest_path_distance(&self, source: Vertex, target: Vertex) -> Option<Distance> {
        dijkstra_one_to_one_distance_wrapped(self, source, target)
    }

    fn number_of_vertices(&self) -> u32 {
        self.number_of_vertices()
    }
}

impl PathFind
ing 
f
