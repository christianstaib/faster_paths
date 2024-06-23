use super::{adjacency_vec_graph::AdjacencyVecGraph, vec_graph::VecGraph, Graph};

pub struct ReversibleGraph<T: Graph + Default> {
    out_graph: T,
    in_graph: T,
}

impl<T: Graph + Default> Default for ReversibleGraph<T> {
    fn default() -> Self {
        Self {
            out_graph: T::default(),
            in_graph: T::default(),
        }
    }
}

fn test() {
    let graph: ReversibleGraph<VecGraph> = ReversibleGraph::default();
}
