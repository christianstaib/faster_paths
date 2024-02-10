use ahash::HashMap;
use indicatif::ProgressIterator;
use serde_derive::{Deserialize, Serialize};

use crate::{edge::DirectedEdge, path::Path, types::VertexId};

#[derive(Serialize, Deserialize)]
pub struct ShortcutReplacer {
    org_shortcuts: HashMap<DirectedEdge, VertexId>,
    shortcuts: HashMap<DirectedEdge, Vec<VertexId>>,
}

impl ShortcutReplacer {
    pub fn new(shortcuts: &HashMap<DirectedEdge, VertexId>) -> Self {
        let org_shortcuts = shortcuts.clone();
        let shortcuts = shortcuts
            .iter()
            .map(|(shortcut, vertex)| (shortcut.clone(), vec![*vertex]))
            .collect();
        let mut shortcut_replacer = ShortcutReplacer {
            org_shortcuts,
            shortcuts,
        };
        shortcut_replacer.extend_shortcuts();
        shortcut_replacer
    }

    fn extend_shortcuts(&mut self) {
        let shortcuts = self.shortcuts.clone();
        shortcuts
            .iter()
            .progress()
            .for_each(|(shortcut, skiped_verticies)| {
                let skiped_verticies = self.extend_one_level(shortcut, skiped_verticies);
                self.shortcuts.insert(shortcut.clone(), skiped_verticies);
            });
    }

    fn extend_one_level(&self, shortcut: &DirectedEdge, skiped_verticies: &Vec<u32>) -> Vec<u32> {
        assert!(!skiped_verticies.is_empty());

        let mut vertices_with_shortcuts = vec![shortcut.tail];
        vertices_with_shortcuts.extend(skiped_verticies);
        vertices_with_shortcuts.push(shortcut.head);

        let mut vertices = self.replace_shortcuts_slow(&vertices_with_shortcuts);
        vertices.remove(0);
        vertices.pop();

        vertices
    }

    fn replace_shortcuts_slow(&self, vertices_with_shortcuts: &[VertexId]) -> Vec<VertexId> {
        let mut vertices_with_shortcuts = vertices_with_shortcuts.to_vec();
        let mut vertices = Vec::new();

        while vertices_with_shortcuts.len() >= 2 {
            let head = vertices_with_shortcuts.pop().unwrap();
            let tail = *vertices_with_shortcuts.last().unwrap();
            let edge = DirectedEdge { tail, head };

            if let Some(&skiped_vertex) = self.org_shortcuts.get(&edge) {
                vertices_with_shortcuts.push(skiped_vertex);
                vertices_with_shortcuts.push(edge.head);
            } else {
                vertices.push(edge.head);
            }
        }

        vertices.push(vertices_with_shortcuts[0]);
        vertices.reverse();

        vertices
    }

    pub fn get_route(&self, path_with_shortcuts: &Path) -> Path {
        let mut path = Path {
            vertices: Vec::new(),
            weight: path_with_shortcuts.weight,
        };

        path.vertices.push(path_with_shortcuts.vertices[0]);

        for windows in path_with_shortcuts.vertices.windows(2) {
            let edge = DirectedEdge {
                tail: windows[0],
                head: windows[1],
            };
            if let Some(x) = self.shortcuts.get(&edge) {
                path.vertices.extend(x);
            }
            path.vertices.push(edge.head);
        }

        path
    }
}
