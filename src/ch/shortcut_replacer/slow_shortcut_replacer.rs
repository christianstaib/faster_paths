use ahash::HashMap;
use serde::{Deserialize, Serialize};

use crate::graphs::{edge::DirectedEdge, path::Path, VertexId};

use super::ShortcutReplacer;

#[derive(Clone, Serialize, Deserialize)]
pub struct SlowShortcutReplacer {
    pub shortcuts: HashMap<DirectedEdge, VertexId>,
}

impl ShortcutReplacer for SlowShortcutReplacer {
    fn replace_shortcuts(&self, path_with_shortcuts: &Path) -> Path {
        let mut path = path_with_shortcuts.clone();
        path.vertices = self.replace_shortcuts_vec(&path.vertices);
        path
    }
}

impl SlowShortcutReplacer {
    pub fn new(shortcuts: &[(DirectedEdge, VertexId)]) -> Self {
        let shortcuts = shortcuts.iter().cloned().collect();

        SlowShortcutReplacer { shortcuts }
    }

    pub fn replace_shortcuts_vec(&self, vertices_with_shortcuts: &[VertexId]) -> Vec<VertexId> {
        let mut vertices_with_shortcuts = vertices_with_shortcuts.to_vec();
        let mut vertices = Vec::new();

        while vertices_with_shortcuts.len() >= 2 {
            let head = vertices_with_shortcuts.pop().unwrap();
            let tail = *vertices_with_shortcuts.last().unwrap();
            let edge = DirectedEdge::new(tail, head).unwrap();

            if let Some(&skiped_vertex) = self.shortcuts.get(&edge) {
                vertices_with_shortcuts.push(skiped_vertex);
                vertices_with_shortcuts.push(edge.head());
            } else {
                vertices.push(edge.head());
            }
        }

        vertices.push(vertices_with_shortcuts[0]);
        vertices.reverse();

        vertices
    }
}
