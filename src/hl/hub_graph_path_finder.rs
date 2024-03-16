use crate::{
    ch::shortcut_replacer::ShortcutReplacer,
    graphs::{
        path::{Path, PathFinding, ShortestPathRequest},
        Weight,
    },
};

use super::hub_graph::HubGraph;

pub struct HubGraphPathFinder<'a> {
    hub_graph: &'a HubGraph,
    shortcut_replacer: &'a Box<dyn ShortcutReplacer + Sync + Send>,
}

impl<'a> HubGraphPathFinder<'a> {
    pub fn new(
        hub_graph: &'a HubGraph,
        shortcut_replacer: &'a Box<dyn ShortcutReplacer + Sync + Send>,
    ) -> HubGraphPathFinder<'a> {
        HubGraphPathFinder {
            hub_graph,
            shortcut_replacer,
        }
    }
}

impl<'a> PathFinding for HubGraphPathFinder<'a> {
    fn get_shortest_path(&self, path_request: &ShortestPathRequest) -> Option<Path> {
        // wanted: source -> target
        let forward_label = self
            .hub_graph
            .forward_labels
            .get(path_request.source() as usize)?;
        let backward_label = self
            .hub_graph
            .reverse_labels
            .get(path_request.target() as usize)?;
        let (_, forward_index, reverse_index) = HubGraph::overlap(forward_label, backward_label)?;

        let mut forward_path = forward_label.get_path(forward_index)?;
        let reverse_path = backward_label.get_path(reverse_index)?;

        // now got: forward(meeting -> source) and reverse (meeting -> target)
        forward_path.vertices.reverse();
        forward_path.vertices.pop();

        forward_path.vertices.extend(reverse_path.vertices);
        forward_path.weight += reverse_path.weight;

        let path = self.shortcut_replacer.replace_shortcuts(&forward_path);

        Some(path)
    }

    fn get_shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let forward_label = self
            .hub_graph
            .forward_labels
            .get(path_request.source() as usize)?;
        let backward_label = self
            .hub_graph
            .reverse_labels
            .get(path_request.target() as usize)?;
        let (weight, _, _) = HubGraph::overlap(forward_label, backward_label)?;

        Some(weight)
    }
}
