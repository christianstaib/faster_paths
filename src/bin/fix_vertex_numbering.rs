use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::graphs::{
    reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph, Vertex, WeightedEdge,
};

/// Reading a .bincode file is way faster than a .fmi file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph_in: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    vis_in: PathBuf,
    // /// Infile in .fmi format
    // #[arg(short, long)]
    // graph_out: PathBuf,
    // /// Infile in .fmi format
    // #[arg(short, long)]
    // vis_out: PathBuf,
}

fn main() {
    let args = Args::parse();

    let graph = ReversibleGraph::<VecVecGraph>::from_fmi_file(&args.graph_in);
    let vis = ReversibleGraph::<VecVecGraph>::from_fmi_file(&args.vis_in);

    println!(
        "graph has {:>9} vertices of which are {:>9} non trivial. {:>9} edges",
        graph.out_graph().number_of_vertices(),
        graph.out_graph().non_trivial_vertices().len(),
        graph.out_graph().number_of_edges()
    );

    println!(
        "vis has {:>9} vertices of which are {:>9} non trivial. {:>9} edges",
        vis.out_graph().number_of_vertices(),
        vis.out_graph().non_trivial_vertices().len(),
        vis.out_graph().number_of_edges()
    );

    let non_trivial_vertices = graph.out_graph().non_trivial_vertices();
    // maps old_vertex to new_vertex
    let mut vertex_map: HashMap<Vertex, Vertex> = non_trivial_vertices
        .into_iter()
        .enumerate()
        .map(|(new_vertex, old_vertex)| (old_vertex, new_vertex as Vertex))
        .collect();

    let old_vertices: HashSet<_> = vertex_map.keys().cloned().collect();
    let new_vertices: HashSet<_> = vertex_map.values().cloned().collect();
    assert_eq!(old_vertices.len(), new_vertices.len());

    for vertex in vis.out_graph().non_trivial_vertices() {
        let new_vertex = vertex_map.len() as Vertex;
        vertex_map.entry(vertex).or_insert(new_vertex);
    }

    let graph = VecVecGraph::from_edges(
        &graph
            .out_graph()
            .all_edges()
            .into_iter()
            .map(|edge| {
                WeightedEdge::new(vertex_map[&edge.tail], vertex_map[&edge.head], edge.weight)
            })
            .collect(),
    );

    let vis = VecVecGraph::from_edges(
        &vis.out_graph()
            .all_edges()
            .into_iter()
            .map(|edge| {
                WeightedEdge::new(vertex_map[&edge.tail], vertex_map[&edge.head], edge.weight)
            })
            .collect(),
    );
    println!(
        "graph has {:>9} vertices of which are {:>9} non trivial. {:>9} edges",
        graph.number_of_vertices(),
        graph.non_trivial_vertices().len(),
        graph.number_of_edges()
    );

    println!(
        "vis has {:>9} vertices of which are {:>9} non trivial. {:>9} edges",
        vis.number_of_vertices(),
        vis.non_trivial_vertices().len(),
        vis.number_of_edges()
    );
}
