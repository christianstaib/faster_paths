use std::{collections::VecDeque, usize};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use rand::{thread_rng, Rng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::Heuristic;
use crate::{
    classical_search::dijkstra::{single_source, single_target},
    dijkstra_data::dijkstra_data_vec::DijkstraDataVec,
    graphs::{
        edge::{DirectedEdge, DirectedWeightedEdge},
        graph_functions::{all_edges, shortests_path_tree},
        path::ShortestPathRequest,
        Graph, VertexId, Weight,
    },
};

#[derive(Clone)]
pub struct Landmark {
    pub to_weight: Vec<Option<Weight>>,
    pub from_weight: Vec<Option<Weight>>,
}

impl Landmark {
    fn generate_landmark(graph: &dyn Graph, source: VertexId) -> Landmark {
        let data_source = single_source(graph, source);
        let data_target = single_target(graph, source);
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
    }
}

impl Heuristic for Landmark {
    fn lower_bound(&self, request: &ShortestPathRequest) -> Option<u32> {
        let to_target = (*self.to_weight.get(request.target() as usize)?)?;
        let to_source = (*self.to_weight.get(request.source() as usize)?)?;
        let from_target = (*self.from_weight.get(request.target() as usize)?)?;
        let from_source = (*self.from_weight.get(request.source() as usize)?)?;

        Some(std::cmp::max(
            to_target.checked_sub(to_source).unwrap_or(0),
            from_source.checked_sub(from_target).unwrap_or(0),
        ))
    }

    fn upper_bound(&self, request: &ShortestPathRequest) -> Option<u32> {
        let from_source = (*self.from_weight.get(request.source() as usize)?)?;
        let to_target = (*self.to_weight.get(request.target() as usize)?)?;
        Some(from_source + to_target)
    }
}

pub struct Landmarks {
    pub landmarks: Vec<Landmark>,
    pub directed_edges: HashMap<DirectedEdge, Weight>,
}

impl Heuristic for Landmarks {
    fn lower_bound(&self, request: &ShortestPathRequest) -> Option<u32> {
        self.landmarks
            .iter()
            .flat_map(|landmark| landmark.lower_bound(request))
            .max()
    }

    fn upper_bound(&self, request: &ShortestPathRequest) -> Option<u32> {
        let edge = DirectedEdge::new(request.source(), request.target()).unwrap();
        if let Some(&weight) = self.directed_edges.get(&edge) {
            return Some(weight);
        }

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
        let mut landmarks = Self::for_vertices(&vertices, graph);

        for edge in all_edges(graph) {
            landmarks
                .directed_edges
                .insert(edge.unweighted(), edge.weight());
        }

        landmarks
    }

    pub fn avoid(num_landmarks: u32, graph: &dyn Graph) -> Landmarks {
        let mut landmarks_heuristic = Landmarks {
            landmarks: Vec::new(),
            directed_edges: HashMap::with_capacity(graph.number_of_edges() as usize),
        };

        let mut landmarks_vertices: HashSet<VertexId> = HashSet::new();

        for _ in (0..num_landmarks).progress() {
            let source = thread_rng().gen_range(0..graph.number_of_vertices());
            let data = single_source(graph, source);
            let tree = shortests_path_tree(&data);

            let mut size = get_size_avoid(graph, source, &landmarks_heuristic, &data);

            for &vertex in get_level_order(source, &tree).iter().rev() {
                let mut children = Vec::from([vertex]);
                children.extend(tree[vertex as usize].iter());

                if landmarks_vertices.contains(&vertex) {
                    size[vertex as usize] = None;
                }

                size[vertex as usize] = children
                    .into_iter()
                    .map(|child| size[child as usize])
                    .fold(Some(0), |acc, opt| {
                        acc.and_then(|sum| opt.map(|value| sum + value))
                    });
            }

            let landmark_vertex = select_landmark_avoid(&size, &tree);

            landmarks_vertices.insert(landmark_vertex);
            let landmark = Landmark::generate_landmark(graph, landmark_vertex);
            landmarks_heuristic.landmarks.push(landmark);
        }

        for edge in all_edges(graph) {
            landmarks_heuristic
                .directed_edges
                .insert(edge.unweighted(), edge.weight());
        }

        landmarks_heuristic
    }

    pub fn for_vertices(vertices: &[VertexId], graph: &dyn Graph) -> Landmarks {
        let landmarks = vertices
            .into_par_iter()
            .progress()
            .map_init(rand::thread_rng, |rng, _| {
                let source = rng.gen_range(0..graph.number_of_vertices());
                Landmark::generate_landmark(graph, source)
            })
            .collect();

        Landmarks {
            landmarks,
            directed_edges: HashMap::new(),
        }
    }
}

fn get_size_avoid(
    graph: &dyn Graph,
    source: u32,
    landmarks_heuristic: &Landmarks,
    data: &DijkstraDataVec,
) -> Vec<Option<u64>> {
    let weight = (0..graph.number_of_vertices())
        .map(|target| {
            let lower_bound = ShortestPathRequest::new(source, target)
                .map(|request| landmarks_heuristic.lower_bound(&request).unwrap_or(0))
                .unwrap_or(0);
            (data.vertices[target as usize].weight.unwrap_or(0) - lower_bound) as u64
        })
        .collect_vec();

    (0..graph.number_of_vertices())
        .map(|target| Some(weight[target as usize]))
        .collect_vec()
}

fn get_level_order(source: u32, tree: &Vec<Vec<u32>>) -> Vec<u32> {
    let mut level_order = Vec::new();

    let mut stack = VecDeque::from([source]);
    while let Some(vertex) = stack.pop_front() {
        stack.extend(tree[vertex as usize].iter());
        level_order.push(vertex);
    }

    level_order
}

fn select_landmark_avoid(size: &[Option<u64>], tree: &[Vec<VertexId>]) -> VertexId {
    let mut max_vertex = size
        .iter()
        .position_max_by_key(|size| size.unwrap_or(0))
        .unwrap() as VertexId;

    loop {
        let possible_children = tree[max_vertex as usize]
            .iter()
            .filter(|&&child| size[child as usize].is_some())
            .cloned()
            .collect_vec();

        if possible_children.is_empty() {
            break;
        }

        max_vertex = possible_children
            .into_iter()
            .max_by_key(|&child| size[child as usize].unwrap_or(0))
            .unwrap();
    }
    max_vertex
}
