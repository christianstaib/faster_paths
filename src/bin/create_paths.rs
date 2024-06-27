use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::graphs::{graph_functions::random_paths, path::read_pathfinder};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    graph: PathBuf,
    /// Path of .fmi file
    #[arg(short, long, default_value = "1000")]
    number_of_paths: u32,
    /// Path of .fmi file
    #[arg(short, long, default_value = "3600")]
    max_seconds: u64,
    /// Path of .fmi file
    #[arg(short, long)]
    paths: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Loading hub graph");
    let pathfinder = read_pathfinder(&args.graph).unwrap();

    println!("Generating random pair paths");
    let paths = random_paths(
        &*pathfinder,
        args.number_of_paths,
        pathfinder.number_of_vertices() as u32,
        args.max_seconds,
    );

    println!("Writing paths to file");
    let writer = BufWriter::new(File::create(&args.paths).unwrap());
    serde_json::to_writer(writer, &paths).unwrap();
}
