use self::label::Label;
use crate::graphs::VertexId;

pub mod hl_from_ch;
pub mod hl_path_finding;
pub mod hub_graph;
pub mod hub_graph_investigator;
pub mod label;
pub mod top_down_hl;

pub trait HubGraphTrait: Send + Sync {
    fn forward_label(&self, vertex: VertexId) -> Option<&Label>;

    fn reverse_label(&self, vertex: VertexId) -> Option<&Label>;
}
