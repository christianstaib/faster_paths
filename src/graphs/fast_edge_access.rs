use std::usize;

use serde::{Deserialize, Serialize};

use super::{
    edge::{
        DirectedEdge, DirectedHeadlessWeightedEdge, DirectedTaillessWeightedEdge,
        DirectedWeightedEdge,
    },
    types::{VertexId, Weight},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct FastOutEdgeAccess {
    edges: Vec<DirectedTaillessWeightedEdge>,
    tail_start_index: Vec<u32>,
}

impl FastOutEdgeAccess {
    pub fn new(edges: &[Vec<DirectedTaillessWeightedEdge>]) -> FastOutEdgeAccess {
        let mut tail_start_index = vec![0];

        for edges in edges.iter() {
            tail_start_index.push(tail_start_index.last().unwrap() + edges.len() as u32);
        }

        let mut edges = edges.to_vec();
        edges.iter_mut().for_each(|this_edges| {
            this_edges.sort_unstable_by_key(|edge| edge.head);
        });

        let edges = edges.iter().flatten().cloned().collect();

        FastOutEdgeAccess {
            edges,
            tail_start_index,
        }
    }

    pub fn edges(&self, tail: VertexId) -> &[DirectedTaillessWeightedEdge] {
        let start = self.tail_start_index[tail as usize] as usize;
        let end = self.tail_start_index[tail as usize + 1] as usize;

        &self.edges[start..end]
    }

    pub fn remove_edge(&mut self, edge: &DirectedWeightedEdge) {
        let start = self.tail_start_index[edge.tail as usize] as usize;
        let end = self.tail_start_index[edge.tail as usize + 1] as usize;
        let tailless_edge = edge.tailless();
        if let Ok(idx) = self
            .edges(edge.tail)
            .binary_search_by_key(&edge.head, |other_edge| other_edge.head)
        {
            self.edges.remove(idx);
            for end_index in (edge.tail as usize + 1)..self.tail_start_index.len() {
                self.tail_start_index[end_index] -= 1;
            }
        }
    }

    pub fn add_edge(&mut self, edge: &DirectedWeightedEdge) {
        let start = self.tail_start_index[edge.tail as usize] as usize;
        let tailless_edge = edge.tailless();
        match self
            .edges(edge.tail)
            .binary_search_by_key(&edge.head, |other_edge| other_edge.head)
        {
            Ok(idx) => {
                if self.edges[start + idx].weight > edge.weight {
                    self.edges[start + idx].weight = edge.weight;
                }
            }
            Err(idx) => {
                self.edges.insert(start + idx, tailless_edge);
                for end_index in (edge.tail as usize + 1)..self.tail_start_index.len() {
                    self.tail_start_index[end_index] += 1;
                }
            }
        }
    }

    pub fn max_edge_weight(&self) -> Option<Weight> {
        self.edges.iter().map(|edge| edge.weight).max()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FastInEdgeAccess {
    edges: Vec<DirectedHeadlessWeightedEdge>,
    head_start_index: Vec<u32>,
}

impl FastInEdgeAccess {
    pub fn new(edges: &[Vec<DirectedHeadlessWeightedEdge>]) -> FastInEdgeAccess {
        let mut head_start_index = vec![0];

        for edges in edges.iter() {
            head_start_index.push(head_start_index.last().unwrap() + edges.len() as u32);
        }

        let mut edges = edges.to_vec();
        edges.iter_mut().for_each(|this_edges| {
            this_edges.sort_unstable_by_key(|edge| edge.tail);
        });

        let edges = edges.iter().flatten().cloned().collect();

        FastInEdgeAccess {
            edges,
            head_start_index,
        }
    }

    pub fn edges(&self, head: VertexId) -> &[DirectedHeadlessWeightedEdge] {
        let start = self.head_start_index[head as usize] as usize;
        let end = self.head_start_index[head as usize + 1] as usize;

        &self.edges[start..end]
    }

    pub fn remove_edge(&mut self, edge: &DirectedWeightedEdge) {
        let start = self.head_start_index[edge.head as usize] as usize;
        let end = self.head_start_index[edge.head as usize + 1] as usize;
        if let Ok(idx) = self
            .edges(edge.head)
            .binary_search_by_key(&edge.tail, |other_edge| other_edge.tail)
        {
            self.edges.remove(idx);
            for end_index in (edge.head as usize + 1)..self.head_start_index.len() {
                self.head_start_index[end_index] -= 1;
            }
        }
    }

    pub fn add_edge(&mut self, edge: &DirectedWeightedEdge) {
        let start = self.head_start_index[edge.tail as usize] as usize;
        match self
            .edges(edge.head)
            .binary_search_by_key(&edge.tail, |other_edge| other_edge.tail)
        {
            Ok(idx) => {
                if self.edges[start + idx].weight > edge.weight {
                    self.edges[start + idx].weight = edge.weight;
                }
            }
            Err(idx) => {
                self.edges.insert(start + idx, edge.headless());
                for end_index in (edge.head as usize + 1)..self.head_start_index.len() {
                    self.head_start_index[end_index] += 1;
                }
            }
        }
    }

    pub fn max_edge_weight(&self) -> Option<Weight> {
        self.edges.iter().map(|edge| edge.weight).max()
    }
}
