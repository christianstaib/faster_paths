use std::{
    collections::{BTreeSet, BinaryHeap, HashSet},
    usize,
};

use ahash::{HashMap, HashMapExt};
use serde_derive::{Deserialize, Serialize};

use crate::ch::binary_heap::MinimumItem;

use super::{
    edge::{DirectedHeadlessWeightedEdge, DirectedTaillessWeightedEdge, DirectedWeightedEdge},
    path::{Path, ShortestPathRequest},
    types::VertexId,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Graph {
    out_edges: Vec<Vec<DirectedTaillessWeightedEdge>>,
    in_edges: Vec<Vec<DirectedHeadlessWeightedEdge>>,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    fn new() -> Self {
        Graph {
            out_edges: Vec::new(),
            in_edges: Vec::new(),
        }
    }

    pub fn from_out_in_edges(
        out_edges: Vec<Vec<DirectedTaillessWeightedEdge>>,
        in_edges: Vec<Vec<DirectedHeadlessWeightedEdge>>,
    ) -> Self {
        Graph {
            out_edges,
            in_edges,
        }
    }

    pub fn all_edges(&self) -> Vec<DirectedWeightedEdge> {
        self.out_edges
            .iter()
            .enumerate()
            .map(|(tail, out_edges)| {
                out_edges
                    .iter()
                    .map(|out_edge| out_edge.set_tail(tail as VertexId))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }

    pub fn out_edges(&self, vertex: VertexId) -> &Vec<DirectedTaillessWeightedEdge> {
        &self.out_edges[vertex as usize]
    }

    pub fn in_edges(&self, vertex: VertexId) -> &Vec<DirectedHeadlessWeightedEdge> {
        &self.in_edges[vertex as usize]
    }

    pub fn from_edges(edges: &[DirectedWeightedEdge]) -> Graph {
        let mut graph = Graph::new();
        edges.iter().for_each(|edge| {
            graph.add_edge(edge);
        });
        graph
    }

    pub fn number_of_vertices(&self) -> u32 {
        self.out_edges.len() as u32
    }

    pub fn out_neighborhood(&self, vertex: VertexId) -> HashSet<VertexId> {
        self.out_edges[vertex as usize]
            .iter()
            .map(|edge| edge.head)
            .collect()
    }

    pub fn in_neighborhood(&self, vertex: VertexId) -> HashSet<VertexId> {
        self.in_edges[vertex as usize]
            .iter()
            .map(|edge| edge.tail)
            .collect()
    }

    /// Does include vertex
    pub fn closed_neighborhood(&self, vertex: VertexId, hops: u32) -> HashSet<VertexId> {
        let mut neighbors = HashSet::new();
        neighbors.insert(vertex);

        for _ in 0..hops {
            let mut new_neighbors = HashSet::new();
            for &neighbor in neighbors.iter() {
                new_neighbors.extend(self.out_neighborhood(neighbor));
                new_neighbors.extend(self.in_neighborhood(neighbor));
            }
            neighbors.extend(new_neighbors);
        }

        neighbors
    }

    /// Does not include vertex
    pub fn open_neighborhood_dijkstra(&self, source: VertexId, max_hops: u32) -> HashSet<VertexId> {
        let mut queue = BinaryHeap::new();
        let mut hops = HashMap::new();

        queue.push(MinimumItem::new(0, source));
        hops.insert(source, 0);

        while let Some(MinimumItem { vertex, .. }) = queue.pop() {
            let mut neighbors = self.out_neighborhood(vertex);
            neighbors.extend(self.in_neighborhood(vertex));
            for &neighbor in neighbors.iter() {
                let alternative_hops = hops[&vertex] + 1;
                if alternative_hops <= max_hops {
                    let current_cost = *hops.get(&neighbor).unwrap_or(&u32::MAX);
                    if alternative_hops < current_cost {
                        queue.push(MinimumItem::new(alternative_hops, neighbor));
                        hops.insert(neighbor, alternative_hops);
                    }
                }
            }
        }

        hops.remove(&source);

        hops.into_keys().collect()
    }

    /// Does not include vertex
    pub fn open_neighborhood(&self, vertex: VertexId, hops: u32) -> HashSet<VertexId> {
        let mut neighbors = self.closed_neighborhood(vertex, hops);
        neighbors.remove(&vertex);

        neighbors
    }

    fn add_out_edge(&mut self, edge: &DirectedWeightedEdge) {
        if (self.out_edges.len() as u32) <= edge.tail {
            self.out_edges.resize((edge.tail + 1) as usize, Vec::new());
        }

        match self.out_edges[edge.tail as usize]
            .binary_search_by_key(&edge.head, |out_edge| out_edge.head)
        {
            Ok(idx) => {
                if edge.weight < self.out_edges[edge.tail as usize][idx].weight {
                    self.out_edges[edge.tail as usize][idx].weight = edge.weight;
                }
            }
            Err(idx) => self.out_edges[edge.tail as usize].insert(idx, edge.tailless()),
        }
    }

    fn add_in_edge(&mut self, edge: &DirectedWeightedEdge) {
        if (self.in_edges.len() as u32) <= edge.head {
            self.in_edges.resize((edge.head + 1) as usize, Vec::new());
        }

        match self.in_edges[edge.head as usize]
            .binary_search_by_key(&edge.tail, |out_edge| out_edge.tail)
        {
            Ok(idx) => {
                if edge.weight < self.in_edges[edge.head as usize][idx].weight {
                    self.in_edges[edge.head as usize][idx].weight = edge.weight;
                }
            }
            Err(idx) => self.in_edges[edge.head as usize].insert(idx, edge.headless()),
        }
    }

    pub fn independent_size(&self, vertices: &BTreeSet<VertexId>, degree: u32) -> Vec<VertexId> {
        let mut ids = Vec::new();

        let mut remaining: BTreeSet<_> = vertices.clone();

        while let Some(node) = remaining.pop_first() {
            ids.push(node);
            for neighbor in self.open_neighborhood(node, degree) {
                remaining.remove(&neighbor);
            }
        }

        ids
    }

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, edge: &DirectedWeightedEdge) {
        self.add_out_edge(edge);
        self.add_in_edge(edge);
    }

    /// Removes the node from the graph.
    pub fn remove_vertex(&mut self, vertex: VertexId) {
        let out_edges = std::mem::take(&mut self.out_edges[vertex as usize]);
        out_edges.iter().for_each(|out_edge| {
            if let Ok(idx) = self.in_edges[out_edge.head as usize]
                .binary_search_by_key(&vertex, |in_edge| in_edge.tail)
            {
                self.in_edges[out_edge.head as usize].remove(idx);
            }
        });

        let in_edges = std::mem::take(&mut self.in_edges[vertex as usize]);
        in_edges.iter().for_each(|in_edge| {
            if let Ok(idx) = self.out_edges[in_edge.tail as usize]
                .binary_search_by_key(&vertex, |in_edge| in_edge.head)
            {
                self.out_edges[in_edge.tail as usize].remove(idx);
            }
        });
    }

    /// Check if a route is correct for a given request. Panics if not.
    pub fn validate_path(&self, request: &ShortestPathRequest, path: &Path) -> Result<(), String> {
        // Ensure that path is not empty when it should not be.
        if path.vertices.is_empty() {
            if request.source() != request.target() {
                return Err("path is empty".to_string());
            }
        }

        // Ensure fist and last vertex of path are source and target of request.
        if let Some(first_vertex) = path.vertices.first() {
            if first_vertex != &request.source() {
                return Err("first vertex of path is not source of request".to_string());
            }
        }
        if let Some(last_vertex) = path.vertices.last() {
            if last_vertex != &request.target() {
                return Err("last vertex of path is not target of request".to_string());
            }
        }

        // check if there is an edge between consecutive path vertices.
        let mut edges = Vec::new();
        for index in 0..(path.vertices.len() - 1) {
            let tail = path.vertices[index];
            let head = path.vertices[index + 1];
            if let Some(min_edge) = self.out_edges[tail as usize]
                .iter()
                .filter(|edge| edge.head == head)
                .next()
            {
                edges.push(min_edge);
            } else {
                return Err(format!("no edge between {} and {} found", tail, head));
            }
        }

        // check if total weight of path is correct.
        let true_cost = edges.iter().map(|edge| edge.weight).sum::<u32>();
        if path.weight != true_cost {
            return Err(format!(
                "path weight should be {}, but was {}",
                true_cost, path.weight
            ));
        }

        Ok(())
    }
}
