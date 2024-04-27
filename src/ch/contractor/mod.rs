use super::Shortcut;
use crate::graphs::{Graph, VertexId};

pub mod contraction_helper;
pub mod serial_witness_search_contractor;

pub trait Contractor {
    fn contract(&mut self, graph: Box<dyn Graph>) -> (Vec<Shortcut>, Vec<Vec<VertexId>>);
}
