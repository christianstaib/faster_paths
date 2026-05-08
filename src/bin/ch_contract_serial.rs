use ch::{
    contraction_hierachy::contract_graph_sequential,
    fmi::{read_fmi_graph, write_fmi_ch},
};
use clap::Parser;
use std::{fs::File, io::BufWriter, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input graph in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Output CH graph in .fmi format
    #[arg(short = 'o', long)]
    contraction_hierarchy: PathBuf,
}

type DistanceType = u32;

fn main() {
    let args = Args::parse();
    let graph = read_fmi_graph::<DistanceType>(&args.graph).unwrap();

    let contraction_hierarchy = contract_graph_sequential(&graph);
    let output = File::create(args.contraction_hierarchy).unwrap();

    write_fmi_ch(BufWriter::new(output), &contraction_hierarchy).unwrap();
}
