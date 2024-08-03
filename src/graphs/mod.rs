pub mod vec_vec_graph;

pub type VertexId = u32;
pub type EdgeId = u32;
pub type Weight = u32;

// struct Edge {
//     pub tail: VertexId,
//     pub head: VertexId,
// }

#[derive(Clone)]
pub struct Edge {
    pub tail: VertexId,
    pub head: VertexId,
}

#[derive(Clone)]
pub struct WeightedEdge {
    pub tail: VertexId,
    pub head: VertexId,
    pub weight: Weight,
}

impl WeightedEdge {
    pub fn remove_tail(&self) -> TaillessEdge {
        TaillessEdge {
            head: self.head,
            weight: self.weight,
        }
    }
}

#[derive(Clone)]
pub struct TaillessEdge {
    pub head: VertexId,
    pub weight: Weight,
}

impl TaillessEdge {
    pub fn set_tail(&self, tail: VertexId) -> WeightedEdge {
        WeightedEdge {
            tail,
            head: self.head,
            weight: self.weight,
        }
    }
}

pub trait Graph: Send + Sync {
    fn number_of_vertices(&self) -> u32;

    fn edges(&self, source: VertexId) -> impl Iterator<Item = WeightedEdge> + '_;

    fn get_weight(&self, edge: &Edge) -> Option<Weight>;

    fn set_weight(&mut self, edge: &Edge, weight: Option<Weight>);
}
