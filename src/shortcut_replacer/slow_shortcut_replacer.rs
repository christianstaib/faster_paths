use ahash::HashMap;

use crate::graphs::{
    edge::DirectedEdge,
    path::{Path, PathFinding, ShortestPathRequest},
    VertexId, Weight,
};

pub struct SlowShortcutReplacer<'a> {
    shortcuts: &'a HashMap<DirectedEdge, VertexId>,
    path_finder: &'a dyn PathFinding,
}

impl<'a> PathFinding for SlowShortcutReplacer<'a> {
    fn shortest_path(&self, path_request: &ShortestPathRequest) -> Option<Path> {
        let mut path = self.path_finder.shortest_path(path_request)?;
        replace_shortcuts_slow(&mut path.vertices, self.shortcuts);

        Some(path)
    }

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        self.path_finder.shortest_path_weight(path_request)
    }

    fn number_of_vertices(&self) -> u32 {
        self.path_finder.number_of_vertices()
    }
}

impl<'a> SlowShortcutReplacer<'a> {
    pub fn new(
        shortcuts: &'a HashMap<DirectedEdge, VertexId>,
        path_finder: &'a dyn PathFinding,
    ) -> Self {
        SlowShortcutReplacer {
            shortcuts,
            path_finder,
        }
    }
}

pub fn replace_shortcuts_slow(
    path: &mut Vec<VertexId>,
    shortcuts: &HashMap<DirectedEdge, VertexId>,
) {
    let mut path_with_shortcuts = std::mem::take(path);

    while path_with_shortcuts.len() >= 2 {
        let head = path_with_shortcuts.pop().unwrap();
        let tail = *path_with_shortcuts.last().unwrap();
        let edge = DirectedEdge::new(tail, head).unwrap();

        if let Some(&skiped_vertex) = shortcuts.get(&edge) {
            if skiped_vertex == head || skiped_vertex == tail {
                panic!("{} {} {}", tail, skiped_vertex, head);
            }
            path_with_shortcuts.push(skiped_vertex);
            path_with_shortcuts.push(edge.head());
        } else {
            path.push(edge.head());
        }
    }

    path.push(path_with_shortcuts[0]);
    path.reverse();
}
