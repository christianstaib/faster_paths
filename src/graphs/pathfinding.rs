use super::{reversible_graph::ReversibleGraph, Distance, Graph, Vertex};
use crate::search::{
    collections::dijkstra_data::Path,
    dijkstra::{dijkstra_one_to_one_distance_wrapped, dijkstra_one_to_one_path_wrapped},
    PathFinding,
};

impl<G: Graph> PathFinding for G {
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

impl<G: Graph> PathFinding for ReversibleGraph<G> {
    fn shortest_path(&self, source: Vertex, target: Vertex) -> Option<Path> {
        self.out_graph().shortest_path(source, target)
    }

    fn shortest_path_distance(&self, source: Vertex, target: Vertex) -> Option<Distance> {
        self.out_graph().shortest_path_distance(source, target)
    }

    fn number_of_vertices(&self) -> u32 {
        self.out_graph().number_of_vertices()
    }
}
