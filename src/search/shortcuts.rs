use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::graphs::Vertex;

pub fn get_fast_shortcuts(
    single_step_shortcuts: &HashMap<(Vertex, Vertex), Vertex>,
) -> HashMap<(Vertex, Vertex), Vec<Vertex>> {
    let mut shortcuts = HashMap::new();

    for (&(tail, head), &vertex) in single_step_shortcuts.iter() {
        let mut path = vec![tail, vertex, head];
        path.remove(0); // remove tail
        path.pop(); // remove head
        replace_shortcuts_slowly(&mut path, &single_step_shortcuts);
        shortcuts.insert((tail, head), path);
    }

    shortcuts
}

pub fn replace_shortcuts_slowly(
    path_with_shortcuts: &mut Vec<Vertex>,
    shortcuts: &HashMap<(Vertex, Vertex), Vertex>,
) {
    let mut path_without_shortcuts = Vec::new();

    let mut already_seen = HashSet::new();

    while path_with_shortcuts.len() >= 2 {
        let head = path_with_shortcuts.pop().unwrap();
        let tail = *path_with_shortcuts.last().unwrap();

        if let Some(vertex) = shortcuts.get(&(tail, head)) {
            // println!("{} -> {} skipped {}", tail, head, vertex);
            path_with_shortcuts.push(*vertex);
            path_with_shortcuts.push(head);
        } else {
            // println!("{} -> {} is a normal edge", tail, head);
            path_without_shortcuts.push(head);
        }

        if !already_seen.insert((tail, head)) {
            // panic!("illegal loop {} -> {}", tail, head);
        }
    }
    path_without_shortcuts.push(path_with_shortcuts.pop().unwrap());
    path_without_shortcuts.reverse();

    *path_with_shortcuts = path_without_shortcuts;
}

pub fn replace_shortcuts_fast(
    path_with_shortcuts: &mut Vec<Vertex>,
    shortcuts: &HashMap<(Vertex, Vertex), Vec<Vertex>>,
) {
    let mut path_without_shortcuts = vec![*path_with_shortcuts.first().unwrap()];

    for (&tail, &head) in path_with_shortcuts.iter().tuple_windows() {
        if let Some(skiped_vertices) = shortcuts.get(&(tail, head)) {
            path_without_shortcuts.extend(skiped_vertices);
        }
        path_without_shortcuts.push(head)
    }

    *path_with_shortcuts = path_without_shortcuts;
}
