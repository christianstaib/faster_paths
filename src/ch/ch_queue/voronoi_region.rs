use crate::graphs::graph::Graph;

use super::queue::PriorityTerm;

pub struct VoronoiRegion {}

impl PriorityTerm for VoronoiRegion {
    fn priority(&self, v: u32, graph: &Graph) -> i32 {
        let neighbors = graph.open_neighborhood(v, 1);

        let mut region_size = 0;
        neighbors.iter().for_each(|&neighbor| {
            if let Some(nearest) = graph
                .in_edges
                .get(neighbor as usize)
                .unwrap()
                .iter()
                .map(|edge| (edge.tail, edge.cost))
                .min_by_key(|(_, cost)| *cost)
            {
                if nearest.0 == v {
                    region_size += 1;
                }
            }
        });
        (region_size as f32).sqrt() as i32
    }

    #[allow(unused_variables)]
    fn update_before_contraction(&mut self, v: u32, graph: &Graph) {}
}

impl Default for VoronoiRegion {
    fn default() -> Self {
        Self::new()
    }
}

impl VoronoiRegion {
    pub fn new() -> Self {
        Self {}
    }
}
