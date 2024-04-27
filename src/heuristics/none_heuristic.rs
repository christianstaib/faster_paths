use super::Heuristic;
use crate::graphs::path::ShortestPathRequest;

pub struct NoneHeuristic {}

impl Heuristic for NoneHeuristic {
    fn lower_bound(&self, _request: &ShortestPathRequest) -> Option<u32> {
        None
    }

    fn upper_bound(&self, _request: &ShortestPathRequest) -> Option<u32> {
        None
    }
}
