use ch::{
    ch::sequential::contract_graph_sequential,
    fmi::{read_fmi_graph, write_fmi_ch},
};
use clap::Parser;
use std::{fs::File, io::BufWriter, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input graph in .fmi format
    #[arg(short, long)]
    graph_in: PathBuf,

    /// Output CH graph in .fmi format
    #[arg(short = 'o', long)]
    graph_out: PathBuf,
}

fn main() {
    let args = Args::parse();
    let graph = read_fmi_graph(&args.graph_in).unwrap();

    let ch = contract_graph_sequential(&graph);
    let output = File::create(args.graph_out).unwrap();

    write_fmi_ch(BufWriter::new(output), &ch).unwrap();
}
