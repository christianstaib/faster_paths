use std::{collections::VecDeque, usize};

use ahash::{HashSet, HashSetExt};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use rand::{thread_rng, Rng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::Heuristic;
use crate::{
    classical_search::dijkstra::Dijkstra,
    graphs::{
        edge::DirectedWeightedEdge, graph_functions::shortests_path_tree,
        path::ShortestPathRequest, Graph, VertexId, Weight,
    },
};

pub struct Landmark {
    pub to_weight: Vec<Option<Weight>>,
    pub from_weight: Vec<Option<Weight>>,
}

impl Heuristic for Landmark {
    fn lower_bound(&self, request: &ShortestPathRequest) -> Option<u32> {
        let to_target = (*self.to_weight.get(request.target() as usize)?)? as i32;
        let to_source = (*self.to_weight.get(request.source() as usize)?)? as i32;
        let from_target = (*self.from_weight.get(request.target() as usize)?)? as i32;
        let from_source = (*self.from_weight.get(request.source() as usize)?)? as i32;
        Some(std::cmp::max(to_target - to_source, from_source - from_target) as u32)
    }

    fn upper_bound(&self, request: &ShortestPathRequest) -> Option<u32> {
        let from_source = (*self.from_weight.get(request.source() as usize)?)?;
        let to_target = (*self.to_weight.get(request.target() as usize)?)?;
        Some(from_source + to_target)
    }
}

pub struct Landmarks {
    pub landmarks: Vec<Landmark>,
}

impl Heuristic for Landmarks {
    fn lower_bound(&self, request: &ShortestPathRequest) -> Option<u32> {
        self.landmarks
            .iter()
            .flat_map(|landmark| landmark.lower_bound(request))
            .max()
    }

    fn upper_bound(&self, request: &ShortestPathRequest) -> Option<u32> {
        self.landmarks
            .iter()
            .flat_map(|landmark| landmark.upper_bound(request))
            .min()
    }

    fn respects_upper_bound(&self, edge: &DirectedWeightedEdge) -> bool {
        self.landmarks
            .iter()
            .all(|landmark| landmark.respects_upper_bound(edge))
    }
}

impl Landmarks {
    pub fn new(num_landmarks: u32, graph: &dyn Graph) -> Landmarks {
        let vertices = (0..num_landmarks).collect_vec();
        Self::for_vertices(&vertices, graph)
    }

    pub fn avoid(num_landmarks: u32, graph: &dyn Graph) -> Landmarks {
        let landmarks_heuristic = Landmarks {
            landmarks: Vec::new(),
        };

        let landmarks_vertices: HashSet<VertexId> = HashSet::new();

        let dijkstra = Dijkstra::new(graph);
        for _ in (0..num_landmarks).progress() {
            let source = thread_rng().gen_range(0..graph.number_of_vertices());
            let data = dijkstra.single_source(source);
            let tree = shortests_path_tree(&data);

            let mut level_order = Vec::new();
            {
                let mut stack = VecDeque::from([source]);
                while let Some(vertex) = stack.pop_front() {
                    stack.extend(tree[vertex as usize].iter());
                    level_order.push(vertex);
                }
            }

            let weight = (0..graph.number_of_vertices())
                .map(|target| {
                    let mut lower_bound = 0;
                    if let Some(request) = ShortestPathRequest::new(source, target) {
                        lower_bound = landmarks_heuristic.lower_bound(&request).unwrap_or(0);
                    }
                    data.vertices[target as usize].weight.unwrap_or(u32::MAX) - lower_bound
                })
                .collect_vec();

            let mut size = (0..graph.number_of_vertices())
                .map(|target| Some(weight[target as usize]))
                .collect_vec();

            for &vertex in level_order.iter().rev() {
                let mut children = Vec::from([vertex]);
                children.extend(tree[vertex as usize].iter());
                if landmarks_vertices.contains(&vertex)
                    || children.iter().any(|&child| size[child as usize].is_none())
                {
                    size[vertex as usize] = None;
                } else {
                    size[vertex as usize] =
                        children.into_iter().map(|child| size[child as usize]).sum();
                }
            }
        }

        todo!()
    }

    pub fn for_vertices(vertices: &[VertexId], graph: &dyn Graph) -> Landmarks {
        let dijkstra = Dijkstra::new(graph);
        let landmarks = vertices
            .into_par_iter()
            .progress()
            .map_init(rand::thread_rng, |rng, _| {
                let source = rng.gen_range(0..graph.number_of_vertices());
                let data_source = dijkstra.single_source(source);
                let data_target = dijkstra.single_target(source);
                Landmark {
                    to_weight: data_source
                        .vertices
                        .iter()
                        .map(|entry| entry.weight)
                        .collect(),
                    from_weight: data_target
                        .vertices
                        .iter()
                        .map(|entry| entry.weight)
                        .collect(),
                }
            })
            .collect();

        Landmarks { landmarks }
    }
}
