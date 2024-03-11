use crate::{dijkstra_data::DijkstraData, graphs::graph::Graph};

#[derive(Clone)]
pub struct SlowDijkstra<'a> {
    graph: &'a Graph,
}

impl<'a> SlowDijkstra<'a> {
    pub fn new(graph: &'a Graph) -> SlowDijkstra {
        SlowDijkstra { graph }
    }

    pub fn get_data(&self, source: u32) -> DijkstraData {
        let mut data = DijkstraData::new(self.graph.number_of_vertices() as usize, source);

        while let Some(state) = data.pop() {
            self.graph
                .out_edges(state.vertex)
                .iter()
                .for_each(|edge| data.update(state.vertex, edge.head, edge.weight));
        }

        data
    }
}
