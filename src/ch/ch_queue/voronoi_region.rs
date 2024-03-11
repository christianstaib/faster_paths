use crate::{
    ch::contraction_helper::ShortcutSearchResult,
    graphs::{graph::Graph, types::VertexId},
};

use super::priority_function::PriorityFunction;

pub struct VoronoiRegion {
    contracted: Vec<bool>,
}

impl PriorityFunction for VoronoiRegion {
    #[allow(unused_variables)]
    fn priority(
        &self,
        vertex: VertexId,
        graph: &Graph,
        shortcuts_results: &ShortcutSearchResult,
    ) -> i32 {
        let mut in_neighborhood = graph.in_neighborhood(vertex);
        in_neighborhood.retain(|&neighbor| self.contracted[neighbor as usize]);

        let mut region_size = 0;
        in_neighborhood.iter().for_each(|&neighbor| {
            if let Some(nearest) = graph
                .out_edges(neighbor)
                .iter()
                .filter(|edge| !self.contracted[edge.head as usize])
                .min_by_key(|edge| edge.weight)
            {
                if nearest.head == vertex {
                    region_size += 1;
                }
            }
        });
        (region_size as f32).sqrt() as i32
    }

    #[allow(unused_variables)]
    fn update(&mut self, vertex: VertexId, graph: &Graph) {
        self.contracted[vertex as usize] = true;
    }
}

impl VoronoiRegion {
    pub fn new(graph: &Graph) -> VoronoiRegion {
        VoronoiRegion {
            contracted: vec![false; graph.number_of_vertices() as usize],
        }
    }
}
