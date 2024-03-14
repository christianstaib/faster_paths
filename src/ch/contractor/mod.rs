use crate::graphs::{graph::Graph, VertexId};

use super::Shortcut;

pub mod contraction_helper;
pub mod parallel_contractor;
pub mod serial_contractor;

pub trait Contractor {
    fn contract(&self, graph: &Graph) -> (Vec<Shortcut>, Vec<Vec<VertexId>>);
}
