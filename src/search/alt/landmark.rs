use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    graphs::{reversible_graph::ReversibleGraph, Distance, Graph, Vertex},
    search::{
        collections::dijkstra_data::DijkstraData, dijkstra::dijkstra_one_to_all_wraped,
        DistanceHeuristic,
    },
};

pub struct Landmarks {
    landmarks: Vec<Landmark>,
}

impl Landmarks {
    pub fn new<G: Graph + Default>(
        graph: &ReversibleGraph<G>,
        vertices: &Vec<Vertex>,
    ) -> Landmarks {
        let landmarks = vertices
            .par_iter()
            .map(|&vertex| Landmark::new(graph, vertex))
            .collect();

        Landmarks { landmarks }
    }
}

impl DistanceHeuristic for Landmarks {
    fn lower_bound(&self, source: Vertex, target: Vertex) -> Option<Distance> {
        self.landmarks
            .iter()
            .filter_map(|landmark| landmark.lower_bound(source, target))
            .max()
    }

    fn upper_bound(&self, source: Vertex, target: Vertex) -> Option<Distance> {
        self.landmarks
            .iter()
            .filter_map(|landmark| landmark.upper_bound(source, target))
            .min()
    }

    fn is_less_or_equal_upper_bound(
        &self,
        source: Vertex,
        target: Vertex,
        distance: Distance,
    ) -> bool {
        self.landmarks
            .iter()
            .filter_map(|landmark| landmark.upper_bound(source, target))
            .all(|upper_bound_distance| distance <= upper_bound_distance)
    }
}

pub struct Landmark {
    distance_to: Vec<Option<Distance>>,
    distance_from: Vec<Option<Distance>>,
}

impl Landmark {
    pub fn new<G: Graph + Default>(graph: &ReversibleGraph<G>, vertex: Vertex) -> Self {
        let out_graph_data = dijkstra_one_to_all_wraped(graph.out_graph(), vertex);
        let distance_to = (0..graph.out_graph().number_of_vertices())
            .map(|vertex| out_graph_data.get_distance(vertex))
            .collect_vec();

        let in_graph_data = dijkstra_one_to_all_wraped(graph.in_graph(), vertex);
        let distance_from = (0..graph.in_graph().number_of_vertices())
            .map(|vertex| in_graph_data.get_distance(vertex))
            .collect_vec();

        Landmark {
            distance_to,
            distance_from,
        }
    }
}

impl DistanceHeuristic for Landmark {
    fn lower_bound(&self, source: Vertex, target: Vertex) -> Option<Distance> {
        let distance_to_source = self.distance_to[source as usize];
        let distance_to_target = self.distance_to[target as usize];
        let potential_forward = match (distance_to_target, distance_to_source) {
            (Some(distance_to_target), Some(distance_to_source)) => {
                distance_to_target.checked_sub(distance_to_source)
            }
            _ => None,
        };

        let distance_from_source = self.distance_from[source as usize];
        let distance_from_target = self.distance_from[target as usize];
        let potential_backward = match (distance_from_source, distance_from_target) {
            (Some(distance_from_source), Some(distance_from_target)) => {
                distance_from_source.checked_sub(distance_from_target)
            }
            _ => None,
        };

        std::cmp::max(potential_forward, potential_backward)
    }

    fn upper_bound(&self, source: Vertex, target: Vertex) -> Option<Distance> {
        let distance_from_source = self.distance_from[source as usize];
        let distance_to_target = self.distance_to[target as usize];

        match (distance_from_source, distance_to_target) {
            (Some(distance_from_source), Some(distance_to_target)) => {
                distance_from_source.checked_add(distance_to_target)
            }
            _ => None,
        }
    }

    fn is_less_or_equal_upper_bound(
        &self,
        source: Vertex,
        target: Vertex,
        distance: Distance,
    ) -> bool {
        distance <= self.upper_bound(source, target).unwrap_or(Distance::MAX)
    }
}
