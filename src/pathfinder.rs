use crate::{
    path::{Path, PathQuery},
    types::Distance,
};

pub trait ShortestPathFinder {
    fn path(&mut self, query: &PathQuery) -> Option<Path>;

    fn distance(&mut self, query: &PathQuery) -> Option<Distance>;
}
