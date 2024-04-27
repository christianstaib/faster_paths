use crate::graphs::{
    path::{Path, PathFinding, ShortestPathRequest},
    Weight,
};

use super::{hub_graph::overlap, HubGraphTrait};

pub struct HLPathFinder<'a> {
    pub hub_graph: &'a dyn HubGraphTrait,
}

impl<'a> PathFinding for HLPathFinder<'a> {
    fn shortest_path(&self, path_request: &ShortestPathRequest) -> Option<Path> {
        // wanted: source -> target
        let forward_label = self.hub_graph.forward_label(path_request.source())?;
        let backward_label = self.hub_graph.reverse_label(path_request.target())?;
        let (_, forward_index, reverse_index) = overlap(forward_label, backward_label)?;

        let mut forward_path = forward_label.get_path(forward_index)?;
        let reverse_path = backward_label.get_path(reverse_index)?;

        // now got: forward(meeting -> source) and reverse (meeting -> target)
        forward_path.vertices.reverse();
        forward_path.vertices.pop();

        forward_path.vertices.extend(reverse_path.vertices);
        forward_path.weight += reverse_path.weight;

        Some(forward_path)
    }

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let forward_label = self.hub_graph.forward_label(path_request.source())?;
        let backward_label = self.hub_graph.reverse_label(path_request.target())?;
        let (weight, _, _) = overlap(forward_label, backward_label)?;

        Some(weight)
    }
}
