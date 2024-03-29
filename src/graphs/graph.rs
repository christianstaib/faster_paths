use std::{
    collections::{BTreeSet, HashSet},
    usize,
};

use serde_derive::{Deserialize, Serialize};

use super::{
    edge::{DirectedHeadlessWeightedEdge, DirectedTaillessWeightedEdge, DirectedWeightedEdge},
    path::{Path, ShortestPathValidation},
    VertexId,
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
            .flat_map(|(tail, out_edges)| {
                out_edges
                    .iter()
                    .map(|out_edge| out_edge.set_tail(tail as VertexId))
                    .collect::<Vec<_>>()
            })
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

    pub fn independent_set(&self, vertices: &BTreeSet<VertexId>, degree: u32) -> Vec<VertexId> {
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
        if edge.tail != edge.head {
            self.add_out_edge(edge);
            self.add_in_edge(edge);
        }
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
    pub fn validate_path(
        &self,
        validation: &ShortestPathValidation,
        path: &Option<Path>,
    ) -> Result<(), String> {
        if let Some(path) = path {
            if let Some(weight) = validation.weight {
                if path.weight != weight {
                    return Err("wrong path weight".to_string());
                }

                // Ensure that path is not empty when it should not be.
                if path.vertices.is_empty() && validation.request.source() != validation.request.target() {
                    return Err("path is empty".to_string());
                }

                // Ensure fist and last vertex of path are source and target of request.
                if let Some(first_vertex) = path.vertices.first() {
                    if first_vertex != &validation.request.source() {
                        return Err("first vertex of path is not source of request".to_string());
                    }
                }
                if let Some(last_vertex) = path.vertices.last() {
                    if last_vertex != &validation.request.target() {
                        return Err("last vertex of path is not target of request".to_string());
                    }
                }

                // check if there is an edge between consecutive path vertices.
                let mut edges = Vec::new();
                for index in 0..(path.vertices.len() - 1) {
                    let tail = path.vertices[index];
                    let head = path.vertices[index + 1];
                    if let Some(min_edge) = self.out_edges[tail as usize]
                        .iter().find(|edge| edge.head == head)
                    {
                        edges.push(min_edge);
                    } else {
                        return Err(format!("no edge between {} and {} found", tail, head));
                    }
                }

                // check if total weight of path is correct.
                let true_cost = edges.iter().map(|edge| edge.weight).sum::<u32>();
                if path.weight != true_cost || path.weight != weight {
                    return Err("wrong path weight".to_string());
                }
            } else {
                return Err("a path was found where there should be none".to_string());
            }
        } else if validation.weight.is_some() {
            return Err("no path is found but there should be one".to_string());
        }

        Ok(())
    }
}
