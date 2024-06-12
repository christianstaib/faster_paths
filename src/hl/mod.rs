use self::label::Label;
use crate::graphs::VertexId;

pub mod directed_hub_graph;
pub mod hl_from_ch;
pub mod hl_from_top_down;
pub mod hub_graph_investigator;
pub mod label;
pub mod pathfinding;

pub trait HubGraphTrait: Send + Sync {
    fn forward_label(&self, vertex: VertexId) -> Option<&Label>;

    fn reverse_label(&self, vertex: VertexId) -> Option<&Label>;
}
