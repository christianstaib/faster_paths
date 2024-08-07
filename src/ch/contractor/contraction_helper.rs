use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

use super::Shortcut;
use crate::{
    ch::contractor::witness_search::optimal_witness_search,
    graphs::{edge::WeightedEdge, path::ShortestPathRequest, Graph, VertexId},
    heuristics::Heuristic,
};

pub trait ShortcutGenerator: Send + Sync {
    fn get_edge_difference_predicited(&self, graph: &dyn Graph, vertex: VertexId) -> i32 {
        self.get_shortcuts(graph, vertex).len() as i32
            - graph.out_edges(vertex).len() as i32
            - graph.in_edges(vertex).len() as i32
    }
    fn get_shortcuts(&self, graph: &dyn Graph, vertex: VertexId) -> Vec<Shortcut>;
}

pub struct ShortcutGeneratorWithWittnessSearch {
    pub max_hops: u32,
}

impl ShortcutGenerator for ShortcutGeneratorWithWittnessSearch {
    fn get_shortcuts(&self, graph: &dyn Graph, vertex: VertexId) -> Vec<Shortcut> {
        let max_out_edge_weight = graph
            .in_edges(vertex)
            .map(|edge| edge.weight())
            .max()
            .unwrap_or(0);

        let heads = graph.out_edges(vertex).map(|edge| edge.head()).collect();

        graph
            .in_edges(vertex)
            .par_bridge()
            .flat_map(|in_edge| {
                let tail = in_edge.tail();
                let max_search_weight = in_edge.weight() + max_out_edge_weight;
                let witness_cost = optimal_witness_search(
                    graph,
                    tail,
                    vertex,
                    max_search_weight,
                    self.max_hops,
                    &heads,
                );

                graph
                    .out_edges(vertex)
                    .filter_map(|out_ede| {
                        let head = out_ede.head();
                        let weight = in_edge.weight() + out_ede.weight();

                        if &weight >= witness_cost.get(&head).unwrap_or(&u32::MAX) {
                            // (tail -> vertex -> head) is not THE shortest path from tail to head
                            return None;
                        }

                        let edge = WeightedEdge::new(tail, head, weight).unwrap();
                        Some(Shortcut { edge, vertex })
                    })
                    .collect_vec()
            })
            .collect()
    }
}

pub struct ShortcutGeneratorWithHeuristic {
    pub heuristic: Box<dyn Heuristic>,
}

fn sample_pairs_1(
    vec1: &Vec<WeightedEdge>,
    vec2: &Vec<WeightedEdge>,
    n: usize,
) -> Vec<(WeightedEdge, WeightedEdge)> {
    if vec1.is_empty() || vec2.is_empty() {
        return Vec::new();
    }

    let mut rng = thread_rng();

    let mut pairs = Vec::with_capacity(n);

    for _ in 0..n {
        let a = vec1.choose(&mut rng).expect("Vector 1 is empty").clone();
        let b = vec2.choose(&mut rng).expect("Vector 2 is empty").clone();
        pairs.push((a, b));
    }

    pairs
}

fn sample_pairs_2(
    vec1: &Vec<WeightedEdge>,
    vec2: &Vec<WeightedEdge>,
    n: usize,
) -> Vec<(WeightedEdge, WeightedEdge)> {
    if vec1.is_empty() || vec2.is_empty() {
        return Vec::new();
    }

    let mut rng = thread_rng();

    let mut pairs = Vec::with_capacity(n);

    let mut shuffled_vec2 = vec2.clone();

    let pairs_per_edge = n.div_ceil(vec1.len());
    for edge1 in vec1.iter() {
        shuffled_vec2.shuffle(&mut rng);
        for edge2 in shuffled_vec2.iter().take(pairs_per_edge) {
            pairs.push((edge1.clone(), edge2.clone()));
        }
    }

    pairs
}

impl ShortcutGenerator for ShortcutGeneratorWithHeuristic {
    fn get_edge_difference_predicited(&self, graph: &dyn Graph, vertex: VertexId) -> i32 {
        let n = 1_000;

        let in_vertices = graph.in_edges(vertex).collect_vec();
        let out_vertices = graph.out_edges(vertex).collect_vec();

        let pairs = sample_pairs_1(&in_vertices, &out_vertices, n);

        let num_shortcuts_from_pairs = pairs
            .into_par_iter()
            .flat_map(|(in_edge, out_ede)| {
                let tail = in_edge.tail();
                let head = out_ede.head();

                let weight = in_edge.weight() + out_ede.weight();

                let request = ShortestPathRequest::new(in_edge.tail(), out_ede.head())?;
                let upper_bound_uw_weight =
                    self.heuristic.upper_bound(&request).unwrap_or(u32::MAX);

                if weight > upper_bound_uw_weight {
                    // (tail -> vertex -> head) is not A shortest path from tail to head
                    return None;
                }

                let edge = WeightedEdge::new(tail, head, weight).unwrap();
                Some(Shortcut { edge, vertex })
            })
            .collect::<Vec<_>>()
            .len() as u32;

        let num_shortcuts = ((num_shortcuts_from_pairs as f32 / n as f32)
            * (in_vertices.len() as f32 * out_vertices.len() as f32))
            as i32;

        num_shortcuts - in_vertices.len() as i32 - out_vertices.len() as i32
    }

    fn get_shortcuts(&self, graph: &dyn Graph, vertex: VertexId) -> Vec<Shortcut> {
        graph
            .in_edges(vertex)
            .par_bridge()
            .flat_map(|in_edge| {
                let tail = in_edge.tail();
                graph
                    .out_edges(vertex)
                    .filter_map(|out_ede| {
                        let head = out_ede.head();
                        let weight = in_edge.weight() + out_ede.weight();

                        let request = ShortestPathRequest::new(in_edge.tail(), out_ede.head())?;
                        let upper_bound_uw_weight =
                            self.heuristic.upper_bound(&request).unwrap_or(u32::MAX);

                        if weight > upper_bound_uw_weight {
                            // (tail -> vertex -> head) is not A shortest path from tail to head
                            return None;
                        }

                        let edge = WeightedEdge::new(tail, head, weight).unwrap();
                        Some(Shortcut { edge, vertex })
                    })
                    .collect_vec()
            })
            .collect()
    }
}
