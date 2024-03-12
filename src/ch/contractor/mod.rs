use crate::graphs::types::VertexId;

use super::shortcut::Shortcut;

pub mod parallel_contractor;
pub mod serial_contractor;

pub trait Contractor {
    fn contract(self) -> (Vec<Shortcut>, Vec<Vec<VertexId>>);
}
