use crate::graphs::{graph::Graph, types::VertexId};

use super::queue::PriorityTerm;

pub struct VoronoiRegion {
    contracted: Vec<bool>,
}

impl PriorityTerm for VoronoiRegion {
    fn priority(&self, vertex: VertexId, graph: &Graph) -> i32 {
        let mut in_neighborhood = graph.in_neighborhood(vertex);
        in_neighborhood.retain(|&neighbor| self.contracted[neighbor as usize]);

        let mut region_size = 0;
        in_neighborhood.iter().for_each(|&neighbor| {
            if let Some(nearest) = graph
                .all_out_edges()
                .get(neighbor as usize)
                .unwrap()
                .iter()
                .filter(|edge| !self.contracted[edge.head as usize])
                .min_by_key(|edge| edge.cost)
            {
                if nearest.head == vertex {
                    region_size += 1;
                }
            }
        });
        (region_size as f32).sqrt() as i32
    }

    #[allow(unused_variables)]
    fn update_before_contraction(&mut self, vertex: VertexId, graph: &Graph) {
        self.contracted[vertex as usize] = true;
    }
}

impl VoronoiRegion {
    pub fn new(graph: &Graph) -> VoronoiRegion {
        VoronoiRegion {
            contracted: vec![false; graph.all_out_edges().len() as usize],
        }
    }
}
