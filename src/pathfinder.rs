use crate::{
    path::{Path, PathQuery},
    types::Distance,
};

pub trait ShortestPathFinder {
    type Distance: Distance;

    fn path(&mut self, query: &PathQuery) -> Option<Path<Self::Distance>>;

    fn distance(&mut self, query: &PathQuery) -> Option<Self::Distance>;
}
