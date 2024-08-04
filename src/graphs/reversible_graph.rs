use super::{Distance, Edge, Graph};

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
}
