use ahash::HashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::graphs::{edge::DirectedEdge, path::Path, VertexId};

use super::{slow_shortcut_replacer::SlowShortcutReplacer, ShortcutReplacer};

#[derive(Clone, Serialize, Deserialize)]
pub struct FastShortcutReplacer {
    pub shortcuts: HashMap<DirectedEdge, Vec<VertexId>>,
}

impl ShortcutReplacer for FastShortcutReplacer {
    fn replace_shortcuts(&self, path_with_shortcuts: &Path) -> Path {
        let mut path = path_with_shortcuts.clone();
        path.vertices = self.replace_shortcuts(&path.vertices);
        path
    }
}

impl FastShortcutReplacer {
    pub fn new(slow_shortcut_replacer: &SlowShortcutReplacer) -> Self {
        let shortcuts = slow_shortcut_replacer
            .shortcuts
            .par_iter()
            .map(|(edge, skiped_vertex)| {
                let vertices_with_shortcuts = vec![edge.tail(), *skiped_vertex, edge.head()];
                let mut vertices =
                    slow_shortcut_replacer.replace_shortcuts_vec(&vertices_with_shortcuts);

                // tail and head of the shortcut need to be removed
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
            let edge = DirectedEdge::new(windows[0], windows[1]).unwrap();
            if let Some(x) = self.shortcuts.get(&edge) {
                vertices.extend(x);
            }
            vertices.push(edge.head());
        }

        vertices
    }
}
