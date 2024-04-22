use crate::graphs::path::ShortestPathRequest;
pub mod landmarks;
pub mod none_heuristic;

pub trait Heuristic: Send + Sync {
    fn lower_bound(&self, request: &ShortestPathRequest) -> Option<u32>;

    fn upper_bound(&self, request: &ShortestPathRequest) -> Option<u32>;
}
