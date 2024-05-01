use std::{
    fs::File,
    io::{BufWriter, Write},
    time::Instant,
};

use ahash::{HashMap, HashMapExt, HashSet};
use indicatif::ProgressBar;
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};

use crate::{
    ch::{
        contracted_graph::DirectedContractedGraph,
        contraction_adaptive_simulated::partition_by_levels, Shortcut,
    },
    graphs::{
        edge::{DirectedEdge, DirectedWeightedEdge},
        hash_graph::HashGraph,
        path::ShortestPathRequest,
        Graph, VertexId,
    },
    heuristics::{landmarks::Landmarks, Heuristic},
};

pub fn contract_adaptive_non_simulated_all_in(
    mut graph: Box<dyn Graph>,
) -> (DirectedContractedGraph, HashMap<DirectedEdge, VertexId>) {
    println!("copying graph");
    let mut base_graph = HashGraph::from_graph(&*graph);

    let mut shortcuts: HashMap<DirectedEdge, Shortcut> = HashMap::new();
    let mut levels = Vec::new();

    let mut remaining_vertices: HashSet<VertexId> = (0..graph.number_of_vertices()).collect();
    remaining_vertices
        .retain(|&vertex| graph.out_edges(vertex).len() + graph.in_edges(vertex).len() > 0);

    let mut writer = BufWriter::new(File::create("reasons_slow.csv").unwrap());
    writeln!(
            writer,
            "duration_create_shortcuts,duration_add_edges,duration_add_shortcuts,duration_remove_vertex,possible_vertex_shortcuts,vertex_shortcuts,number_of_edges,number_of_shortcuts,number_of_vertices"
        )
        .unwrap();

    println!("starting actual contraction");
    let bar = ProgressBar::new(remaining_vertices.len() as u64);

    let landmarks = Landmarks::new(25, &*graph);

    let mut vertex_buf = Vec::new();
    while let Some(vertex) = get_next_vertex(&graph, &mut remaining_vertices) {
        let start = Instant::now();
        graph.in_edges(vertex).for_each(|in_edge| {
            graph
                .out_edges(vertex)
                .filter_map(|out_edge| {
                    // println!("{:?}", in_edge);
                    // println!("{:?}", out_edge);
                    // println!("");
                    let edge = DirectedWeightedEdge::new(
                        in_edge.tail(),
                        out_edge.head(),
                        in_edge.weight() + out_edge.weight(),
                    )?;
                    let shortcut = Shortcut { edge, vertex };
                    Some(shortcut)
                })
                .for_each(|edge| {
                    vertex_buf.push(edge);
                })
        });

        let vertex_shortcuts: Vec<_> = vertex_buf
            .par_iter()
            // only add edges that are less expensive than currently
            .filter(|shortcut| {
                let current_weight = graph
                    .get_edge_weight(&shortcut.edge.unweighted())
                    .unwrap_or(u32::MAX);
                shortcut.edge.weight() < current_weight
            })
            .filter(|shortcut| {
                let request =
                    ShortestPathRequest::new(shortcut.edge.tail(), shortcut.edge.head()).unwrap();
                let current_weight = landmarks.upper_bound(&request).unwrap_or(u32::MAX);
                shortcut.edge.weight() < current_weight
            })
            .cloned()
            .collect();
        vertex_buf.clear();
        let duration_create_shortcuts = start.elapsed();

        let start = Instant::now();
        vertex_shortcuts.iter().for_each(|shortcut| {
            graph.set_edge(&shortcut.edge);
        });
        let duration_add_edges = start.elapsed();

        let possible_shortcuts = graph.in_edges(vertex).len() * graph.out_edges(vertex).len();
        let vertex_shortcuts_len = shortcuts.len();

        let start = Instant::now();
        // insert serial
        for shortcut in vertex_shortcuts {
            shortcuts.insert(shortcut.edge.unweighted(), shortcut);
        }
        let duration_add_shortcuts = start.elapsed();

        let start = Instant::now();
        graph.remove_vertex(vertex);
        let duration_remove_vertex = start.elapsed();

        levels.push(vec![vertex]);
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{}",
            duration_create_shortcuts.as_nanos(),
            duration_add_edges.as_nanos(),
            duration_add_shortcuts.as_nanos(),
            duration_remove_vertex.as_nanos(),
            possible_shortcuts,
            vertex_shortcuts_len,
            graph.number_of_edges(),
            shortcuts.len(),
            graph.number_of_vertices() - levels.len() as u32
        )
        .unwrap();
        writer.flush().unwrap();

        bar.inc(1);
    }
    bar.finish();

    println!("writing base_grap and shortcuts to file");
    let writer = BufWriter::new(File::create("all_in.bincode").unwrap());
    bincode::serialize_into(writer, &(&base_graph, &shortcuts)).unwrap();

    println!("Adding shortcuts to base graph");
    let edges = shortcuts
        .values()
        .map(|shortcut| shortcut.edge.clone())
        .collect_vec();
    base_graph.set_edges(&edges);

    println!("creating upward and downward_graph");
    let (upward_graph, downward_graph) = partition_by_levels(&base_graph, &levels);

    println!("generatin shortcut lookup map");
    let shortcuts = shortcuts
        .values()
        .map(|shortcut| (shortcut.edge.unweighted(), shortcut.vertex))
        .collect();

    let directed_contracted_graph = DirectedContractedGraph {
        upward_graph,
        downward_graph,
        levels,
    };

    (directed_contracted_graph, shortcuts)
}

fn get_next_vertex(
    graph: &Box<dyn Graph>,
    remaining_vertices: &mut HashSet<VertexId>,
) -> Option<VertexId> {
    let min_vertex = *remaining_vertices.par_iter().min_by_key(|&&vertex| {
        (graph.in_edges(vertex).len() as i32 * graph.out_edges(vertex).len() as i32)
            - (graph.in_edges(vertex).len() as i32)
            - graph.out_edges(vertex).len() as i32
    })?;
    remaining_vertices.remove(&min_vertex);
    Some(min_vertex)
}
