use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rand::{seq::IteratorRandom, thread_rng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    graphs::{reversible_graph::ReversibleGraph, Distance, Graph, Vertex},
    search::{
        collections::dijkstra_data::DijkstraData, dijkstra::dijkstra_one_to_all_wraped,
        DistanceHeuristic, PathFinding,
    },
    utility::{get_paths, get_progressbar, level_to_vertex},
};

pub struct Landmarks {
    pub landmarks: Vec<Landmark>,
}

impl Landmarks {
    pub fn new<G: Graph + Default>(graph: &ReversibleGraph<G>, vertices: &[Vertex]) -> Landmarks {
        let landmarks = vertices
            .par_iter()
            .progress_with(get_progressbar(
                "Generating landmarks",
                vertices.len() as u64,
            ))
            .map(|&vertex| Landmark::new(graph, vertex))
            .collect();

        Landmarks { landmarks }
    }

    pub fn hitting_set<G: Graph + Default>(
        graph: &ReversibleGraph<G>,
        number_of_paths: u32,
        number_of_landmarks: u32,
    ) -> Landmarks {
        let paths = get_paths(
            graph.out_graph(),
            &graph.out_graph().vertices().collect_vec(),
            number_of_paths,
            0,
            usize::MAX,
        );
        let level_to_vertex: Vec<Vertex> = level_to_vertex(&paths, graph.number_of_vertices());
        Landmarks::new(
            graph,
            &level_to_vertex
                .iter()
                .rev()
                .take(number_of_landmarks as usize)
                .cloned()
                .collect_vec(),
        )
    }

    pub fn random<G: Graph + Default>(
        graph: &ReversibleGraph<G>,
        number_of_landmarks: u32,
    ) -> Landmarks {
        let vertices = (0..graph.out_graph().number_of_vertices())
            .choose_multiple(&mut thread_rng(), number_of_landmarks as usize);
        Landmarks::new(graph, &vertices)
    }
}

impl DistanceHeuristic for Landmarks {
    fn lower_bound(&self, source: Vertex, target: Vertex) -> Distance {
        self.landmarks
            .iter()
            .map(|landmark| landmark.lower_bound(source, target))
            .max()
            .unwrap_or(0)
    }

    fn upper_bound(&self, source: Vertex, target: Vertex) -> Distance {
        self.landmarks
            .iter()
            .map(|landmark| landmark.upper_bound(source, target))
            .min()
            .unwrap_or(Distance::MAX)
    }

    fn is_less_or_equal_upper_bound(
        &self,
        source: Vertex,
        target: Vertex,
        distance: Distance,
    ) -> bool {
        self.landmarks
            .iter()
            .map(|landmark| landmark.upper_bound(source, target))
            .all(|upper_bound_distance| distance <= upper_bound_distance)
    }
}

pub struct Landmark {
    pub vertex: Vertex,
    pub distance_to: Vec<Distance>,
    pub distance_from: Vec<Distance>,
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
            vertex,
            distance_to,
            distance_from,
        }
    }
}

impl DistanceHeuristic for Landmark {
    fn lower_bound(&self, source: Vertex, target: Vertex) -> Distance {
        let distance_to_source = self.distance_to[source as usize];
        let distance_to_target = self.distance_to[target as usize];
        let potential_forward = distance_to_target
            .checked_sub(distance_to_source)
            .unwrap_or(0);

        let distance_from_source = self.distance_from[source as usize];
        let distance_from_target = self.distance_from[target as usize];
        let potential_backward = distance_from_source
            .checked_sub(distance_from_target)
            .unwrap_or(0);

        std::cmp::max(potential_forward, potential_backward)
    }

    fn upper_bound(&self, source: Vertex, target: Vertex) -> Distance {
        let distance_from_source = self.distance_from[source as usize];
        let distance_to_target = self.distance_to[target as usize];

        distance_from_source
            .checked_add(distance_to_target)
            .unwrap_or(Distance::MAX)
    }
}
