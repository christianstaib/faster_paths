use ahash::HashMap;
use serde_derive::{Deserialize, Serialize};

use crate::graphs::{edge::DirectedEdge, path::Path, types::VertexId};

use super::slow_shortcut_replacer::SlowShortcutReplacer;

#[derive(Serialize, Deserialize)]
pub struct FastShortcutReplacer {
    shortcuts: HashMap<DirectedEdge, Vec<VertexId>>,
}

impl FastShortcutReplacer {
    pub fn new(shortcuts: &HashMap<DirectedEdge, VertexId>) -> Self {
        let slow_shortcut_replacer = SlowShortcutReplacer::new(&shortcuts);
        let shortcuts = shortcuts
            .iter()
            .map(|(edge, skiped_vertex)| {
                let vertices_with_shortcuts = vec![edge.tail, *skiped_vertex, edge.head];
                let mut vertices =
                    slow_shortcut_replacer.replace_shortcuts(&vertices_with_shortcuts);
                vertices.remove(0);
                vertices.pop();
                (edge.clone(), vertices)
            })
            .collect();
        FastShortcutReplacer { shortcuts }
    }

    fn replace_shortcuts(&self, vertices_with_shortcuts: &[VertexId]) -> Vec<VertexId> {
        let mut vertices = Vec::new();

        vertices.push(vertices_with_shortcuts[0]);

        for windows in vertices_with_shortcuts.windows(2) {
            let edge = DirectedEdge {
                tail: windows[0],
                head: windows[1],
            };
            if let Some(x) = self.shortcuts.get(&edge) {
                vertices.extend(x);
            }
            vertices.push(edge.head);
        }

        vertices
    }

    pub fn get_path(&self, path_with_shortcuts: &Path) -> Path {
        let mut path = path_with_shortcuts.clone();
        path.vertices = self.replace_shortcuts(&path.vertices);
        path
    }
}
