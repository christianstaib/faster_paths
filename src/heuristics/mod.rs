use crate::graphs::{edge::DirectedWeightedEdge, path::ShortestPathRequest};
pub mod landmarks;
pub mod none_heuristic;

pub trait Heuristic: Send + Sync {
    fn lower_bound(&self, request: &ShortestPathRequest) -> Option<u32>;

    fn upper_bound(&self, request: &ShortestPathRequest) -> Option<u32>;

    fn respects_upper_bound(&self, edge: &DirectedWeightedEdge) -> bool {
        let request = ShortestPathRequest::new(edge.tail(), edge.head()).unwrap();
        if let Some(upper_bound) = self.upper_bound(&request) {
            return edge.weight() <= upper_bound;
        }
        true
    }
}
