use std::usize;

use indicatif::ProgressIterator;
use rand::Rng;
use rayon::iter::{ParallelBridge, ParallelIterator};

use super::Heuristic;
use crate::{
    classical_search::dijkstra::Dijkstra,
    graphs::{path::ShortestPathRequest, Graph, Weight},
};

pub struct Landmark {
    pub to_weight: Vec<Option<Weight>>,
    pub from_weight: Vec<Option<Weight>>,
}

impl Heuristic for Landmark {
    fn lower_bound(&self, request: &ShortestPathRequest) -> Option<u32> {
        let to_target = (*self.to_weight.get(request.target() as usize)?)? as i32;
        let to_source = (*self.to_weight.get(request.source() as usize)?)? as i32;
        let from_target = (*self.from_weight.get(request.target() as usize)?)? as i32;
        let from_source = (*self.from_weight.get(request.source() as usize)?)? as i32;
        Some(std::cmp::max(to_target - to_source, from_source - from_target) as u32)
    }

    fn upper_bound(&self, request: &ShortestPathRequest) -> Option<u32> {
        let to_target = (*self.to_weight.get(request.target() as usize)?)?;
        let from_source = (*self.from_weight.get(request.source() as usize)?)?;
        Some(from_source + to_target)
    }
}

pub struct Landmarks {
    pub landmarks: Vec<Landmark>,
}

impl Heuristic for Landmarks {
    fn lower_bound(&self, request: &ShortestPathRequest) -> Option<u32> {
        self.landmarks
            .iter()
            .flat_map(|landmark| landmark.lower_bound(request))
            .max()
    }

    fn upper_bound(&self, request: &ShortestPathRequest) -> Option<u32> {
        self.landmarks
            .iter()
            .flat_map(|landmark| landmark.upper_bound(request))
            .min()
    }
}

impl Landmarks {
    pub fn new(num_landmarks: u32, graph: &dyn Graph) -> Landmarks {
        let dijkstra = Dijkstra::new(graph);
        let landmarks = (0..num_landmarks)
            .progress()
            .par_bridge()
            .map_init(rand::thread_rng, |rng, _| {
                let source = rng.gen_range(0..graph.number_of_vertices());
                let data_source = dijkstra.single_source(source);
                let data_target = dijkstra.single_target(source);
                Landmark {
                    to_weight: data_source
                        .vertices
                        .iter()
                        .map(|entry| entry.weight)
                        .collect(),
                    from_weight: data_target
                        .vertices
                        .iter()
                        .map(|entry| entry.weight)
                        .collect(),
                }
            })
            .collect();

        Landmarks { landmarks }
    }
}
