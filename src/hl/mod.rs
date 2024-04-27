use crate::graphs::VertexId;

use self::label::Label;

pub mod hl_from_ch;
pub mod hl_path_finding;
pub mod hub_graph;
pub mod hub_graph_investigator;
pub mod label;

pub trait HubGraphTrait: Send + Sync {
    fn forward_label<'a>(&'a self, vertex: VertexId) -> Option<&'a Label>;

    fn reverse_label<'a>(&'a self, vertex: VertexId) -> Option<&'a Label>;
}
