use std::collections::HashMap;

use indicatif::ProgressIterator;

use super::{Distance, Edge, Graph, TaillessEdge, Vertex, WeightedEdge};
use crate::utility::get_progressbar_long_jobs;

pub trait FromEdges {
    fn from_edges(edges: &Vec<WeightedEdge>) -> Self;
}

#[derive(Clone)]
pub struct ReversibleGraph<G: Graph> {
    out_graph: Box<G>,
    in_graph: Box<G>,
}

impl<G: Graph + Default> ReversibleGraph<G> {
    pub fn new() -> Self {
        ReversibleGraph {
            out_graph: Box::new(G::default()),
            in_graph: Box::new(G::default()),
        }
    }

    pub fn from_edges(edges: &Vec<WeightedEdge>) -> Self {
        let mut graph = Self::new();

        edges
            .iter()
            .progress_with(get_progressbar_long_jobs(
                "Building graph from edges",
                edges.len() as u64,
            ))
            .for_each(|edge| {
                if edge.weight
                    < graph
                        .get_weight(&edge.remove_weight())
                        .unwrap_or(Distance::MAX)
                {
                    graph.set_weight(&edge.remove_weight(), Some(edge.weight));
                }
            });

        graph
    }

    pub fn out_graph(&self) -> &G {
        &self.out_graph
    }

    pub fn in_graph(&self) -> &G {
        &self.in_graph
    }

    pub fn set_weight(&mut self, edge: &Edge, weight: Option<Distance>) {
        self.out_graph.set_weight(edge, weight);
        self.in_graph.set_weight(&edge.reversed(), weight);
    }

    pub fn get_weight(&self, edge: &Edge) -> Option<Distance> {
        self.out_graph.get_weight(edge)
    }

    pub fn disconnect(&mut self, vertex: Vertex) {
        for edge in self.in_graph.edges(vertex) {
            self.out_graph
                .set_weight(&edge.reversed().remove_weight(), None);
        }

        for edge in self.out_graph.edges(vertex) {
            self.in_graph
                .set_weight(&edge.reversed().remove_weight(), None)
        }

        self.out_graph.disconnect(vertex);
        self.in_graph.disconnect(vertex);
    }

    pub fn insert_and_update(
        &mut self,
        new_and_updated_edges: &HashMap<Vertex, (Vec<TaillessEdge>, Vec<TaillessEdge>)>,
    ) {
        for (&vertex, (new_edges, updated_edges)) in new_and_updated_edges {
            for tailless_edge in new_edges.iter().chain(updated_edges.iter()) {
                let edge = tailless_edge.set_tail(vertex).remove_weight();
                self.set_weight(&edge, Some(tailless_edge.weight));
            }
        }
    }
}
