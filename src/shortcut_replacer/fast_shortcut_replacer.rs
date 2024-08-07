use ahash::HashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::slow_shortcut_replacer::replace_shortcuts_slow;
use crate::graphs::{
    edge::Edge,
    path::{Path, PathFinding, ShortestPathRequest},
    VertexId, Weight,
};

pub struct FastShortcutReplacer<'a> {
    shortcuts: &'a HashMap<Edge, Vec<VertexId>>,
    path_finder: &'a dyn PathFinding,
}

impl<'a> PathFinding for FastShortcutReplacer<'a> {
    fn shortest_path(&self, path_request: &ShortestPathRequest) -> Option<Path> {
        let mut path = self.path_finder.shortest_path(path_request)?;
        replace_shortcuts_fast(&mut path.vertices, self.shortcuts);

        Some(path)
    }

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        self.path_finder.shortest_path_weight(path_request)
    }

    fn number_of_vertices(&self) -> u32 {
        self.path_finder.number_of_vertices()
    }
}

impl<'a> FastShortcutReplacer<'a> {
    pub fn new(
        shortcuts: &'a HashMap<Edge, Vec<VertexId>>,
        path_finder: &'a dyn PathFinding,
    ) -> Self {
        FastShortcutReplacer {
            shortcuts,
            path_finder,
        }
    }
}

pub fn unfold_shortcuts(shortcuts: &HashMap<Edge, VertexId>) -> HashMap<Edge, Vec<VertexId>> {
    shortcuts
        .par_iter()
        .map(|(edge, &skiped_vertex)| {
            let mut path = vec![edge.tail(), skiped_vertex, edge.head()];
            replace_shortcuts_slow(&mut path, shortcuts);

            (edge.clone(), path)
        })
        .collect()
}

pub fn replace_shortcuts_fast(path: &mut Vec<VertexId>, shortcuts: &HashMap<Edge, Vec<VertexId>>) {
    let path_with_shortcuts = std::mem::take(path);

    path.push(path_with_shortcuts[0]);

    for windows in path_with_shortcuts.windows(2) {
        let edge = Edge::new(windows[0], windows[1]).unwrap();
        if let Some(x) = shortcuts.get(&edge) {
            path.extend(x);
        }
        path.push(edge.head());
    }
}
