use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    graphs::Vertex,
    search::{hl::hub_graph::HubGraph, shortcuts::replace_shortcuts_slowly, PathFinding},
    utility::read_bincode_with_spinnner,
};
use indicatif::ProgressIterator;
use itertools::Itertools;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    h1: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    h2: PathBuf,
}

fn main() {
    let args = Args::parse();

    let h1: HubGraph = read_bincode_with_spinnner("h1", args.h1.as_path());
    let h2: HubGraph = read_bincode_with_spinnner("h2", args.h2.as_path());

    assert_eq!(h1.number_of_vertices(), h2.number_of_vertices());

    // assert_pruned(&h1);
    // assert_pruned(&h2);

    let mut min_len = usize::MAX;
    let mut min_vertex = Vertex::MAX;
    for vertex in (0..h1.number_of_vertices()).progress() {
        if h1.forward.get_label(vertex).len() != h2.forward.get_label(vertex).len() {
            let len_sum = h1.forward.get_label(vertex).len() + h2.forward.get_label(vertex).len();
            if len_sum < min_len {
                min_len = len_sum;
                min_vertex = vertex;
            }

            if h1.forward.get_label(vertex).len() > h2.forward.get_label(vertex).len() {
                println!("err {}", vertex);
            }
        }
    }

    println!(
        "{:?} (level {}) {} {}",
        min_vertex,
        h1.vertex_to_level[min_vertex as usize],
        h1.forward.get_label(min_vertex).len(),
        h2.forward.get_label(min_vertex).len()
    );

    let h1_set = h1
        .forward
        .get_label(min_vertex)
        .iter()
        .map(|entry| entry.vertex)
        .collect::<HashSet<_>>();

    let h2_set = h2
        .forward
        .get_label(min_vertex)
        .iter()
        .map(|entry| entry.vertex)
        .collect::<HashSet<_>>();

    println!("{:?}", h2_set.difference(&h1_set));

    // for &target in h2_set.difference(&h1_set) {
    //     println!(
    //         "{:?}",
    //         h2.shortest_path(min_vertex, target).unwrap().vertices
    //     );

    //     let sp = h2.shortest_path(min_vertex, target).unwrap().vertices;
    //     assert!(sp
    //         .iter()
    //         .skip(1)
    //         .take(sp.len() - 2)
    //         .all(|vertex| !h2_set.contains(vertex)),);
    // }

    fun_name(h1, min_vertex);
    fun_name(h2, min_vertex);
}

fn fun_name(h1: HubGraph, min_vertex: u32) {
    let h1_label = h1.forward.get_label(min_vertex);
    let h1_tree: Vec<Vec<Vertex>> = h1_label
        .iter()
        .filter(|entry| entry.vertex != min_vertex)
        .map(|entry| {
            let mut path = vec![
                h1_label[entry.predecessor_index.unwrap() as usize].vertex,
                entry.vertex,
            ];

            // replace_shortcuts_slowly(&mut path, &h1.shortcuts);

            path
        })
        .collect();

    for entry_idx in 0..h1_label.len() {
        for j in 0..h1_label.len() {
            if entry_idx == j {
                continue;
            }

            if h1_label[entry_idx].vertex == h1_label[j].vertex {
                println!("err");
            }
        }
    }

    let level_map = h1_tree
        .iter()
        .flatten()
        .map(|&vertex| (vertex, h1.vertex_to_level[vertex as usize]))
        .collect::<HashMap<_, _>>();
    println!("{:?}", level_map);

    // let mut children = HashMap::new();
    // for st in h1_tree.iter() {
    //     let s = st[0];
    //     let t = st[1];
    //     children.entry(s).or_insert(Vec::new()).push(t);
    // }

    // let mut mop = HashMap::new();
    // let mut stack = Vec::new();
    // stack.push(min_vertex);
    // mop.insert(min_vertex, h1.vertex_to_level[min_vertex as usize]);
    // while let Some(v) = stack.pop() {
    //     if let Some(children) = children.get(&v) {
    //         for &child in children.iter() {
    //             stack.push(child);
    //             mop.insert(
    //                 child,
    //                 std::cmp::max(
    //                     h1.vertex_to_level[v as usize],
    //                     h1.vertex_to_level[child as usize],
    //                 ),
    //             );
    //         }
    //     }
    // }

    // println!("{:?}", mop);
    println!("{:?}", h1_tree);
}

fn assert_pruned(h1: &HubGraph) {
    for source in (0..h1.number_of_vertices()).progress() {
        for entry in h1.forward.get_label(source) {
            let target = entry.vertex;
            assert_eq!(
                h1.shortest_path_distance(source, target).unwrap(),
                entry.distance
            );
        }
    }
}
