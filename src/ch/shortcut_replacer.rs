use ahash::HashMap;
use indicatif::ProgressIterator;
use serde_derive::{Deserialize, Serialize};

use crate::{edge::DirectedEdge, path::Path, types::VertexId};

#[derive(Serialize, Deserialize)]
pub struct ShortcutReplacer {
    pub org_shortcuts: HashMap<DirectedEdge, VertexId>,
    pub shortcuts: HashMap<DirectedEdge, Vec<VertexId>>,
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
        for _ in 0..25 {
            shortcut_replacer.extend_shortcuts();
        }
        shortcut_replacer
    }

    pub fn extend_shortcuts(&mut self) {
        let shortcuts = self.shortcuts.clone();
        shortcuts
            .iter()
            .progress()
            .for_each(|(shortcut, skiped_verticies)| {
                if let Some(skiped_verticies) = self.extend_one_level(shortcut, skiped_verticies) {
                    self.shortcuts.insert(shortcut.clone(), skiped_verticies);
                }
            });
    }

    fn extend_one_level(
        &self,
        shortcut: &DirectedEdge,
        skiped_verticies: &Vec<u32>,
    ) -> Option<Vec<u32>> {
        assert!(!skiped_verticies.is_empty());

        let mut vec = vec![shortcut.tail];
        vec.extend(skiped_verticies);
        vec.push(shortcut.head);

        let mut next_level = Vec::new();
        for window in vec.windows(2) {
            let edge = DirectedEdge {
                tail: window[0],
                head: window[1],
            };
            if let Some(vertex) = self.org_shortcuts.get(&edge) {
                next_level.push(*vertex);
            } else {
                next_level.push(u32::MAX);
            }
        }

        let mut result = Vec::new();
        for i in 0..next_level.len() {
            result.push(next_level[i]);
            result.push(vec[i + 1]);
        }
        result.pop();
        result.dedup();
        result.retain(|&vertex| vertex != u32::MAX);

        if skiped_verticies.is_empty() {
            return None;
        }

        Some(result)
    }

    pub fn get_route(&self, path_with_shortcuts: &Path) -> Path {
        let mut path_with_shortcuts = path_with_shortcuts.clone();
        let mut path = Path {
            vertices: Vec::new(),
            weight: path_with_shortcuts.weight,
        };

        while path_with_shortcuts.vertices.len() >= 2 {
            let head = path_with_shortcuts.vertices.pop().unwrap();
            let tail = *path_with_shortcuts.vertices.last().unwrap();
            let edge = DirectedEdge { tail, head };

            if let Some(skiped_verticies) = self.shortcuts.get(&edge) {
                path_with_shortcuts.vertices.extend(skiped_verticies);
                path_with_shortcuts.vertices.push(edge.head);
            } else {
                path.vertices.push(edge.head);
            }
        }

        path.vertices.push(path_with_shortcuts.vertices[0]);
        path.vertices.reverse();

        path
    }
}
