use super::{hub_graph::overlap, label::Label, HubGraphTrait};
use crate::graphs::{
    path::{Path, PathFinding, ShortestPathRequest},
    Weight,
};

pub struct HLPathFinder<'a> {
    hub_graph: &'a dyn HubGraphTrait,
}

impl<'a> HLPathFinder<'a> {
    pub fn new(hub_graph: &'a dyn HubGraphTrait) -> Self {
        HLPathFinder { hub_graph }
    }
}

impl<'a> PathFinding for HLPathFinder<'a> {
    fn shortest_path(&self, path_request: &ShortestPathRequest) -> Option<Path> {
        // wanted: source -> target
        let forward_label = self.hub_graph.forward_label(path_request.source())?;
        let backward_label = self.hub_graph.reverse_label(path_request.target())?;

        shortest_path(forward_label, backward_label)
    }

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let forward_label = self.hub_graph.forward_label(path_request.source())?;
        let backward_label = self.hub_graph.reverse_label(path_request.target())?;

        shortest_path_weight(forward_label, backward_label)
    }
}

pub fn shortest_path(forward_label: &Label, backward_label: &Label) -> Option<Path> {
    // wanted: source -> target
    let (_, forward_index, reverse_index) = overlap(forward_label, backward_label)?;

    // unwrap can be called as be found overlapp and therefore path exists
    let mut forward_path = forward_label.get_path(forward_index).unwrap();
    let reverse_path = backward_label.get_path(reverse_index).unwrap();

    // now got: forward_path (meeting -> source) and reverse_reverse (meeting ->
    // target)
    forward_path.vertices.reverse();
    forward_path.vertices.pop();

    forward_path.vertices.extend(reverse_path.vertices);
    forward_path.weight += reverse_path.weight;

    Some(forward_path)
}

pub fn shortest_path_weight(forward_label: &Label, backward_label: &Label) -> Option<Weight> {
    let (weight, _, _) = overlap(forward_label, backward_label)?;

    Some(weight)
}
