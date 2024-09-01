use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::Vertex,
    reading_pathfinder,
    utility::{get_paths, level_to_vertex},
    FileType,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[arg(short, long)]
    file: PathBuf,

    /// Type of the input file
    #[arg(short = 't', long, value_enum, default_value = "fmi")]
    file_type: FileType,

    /// Number of seartes.
    #[arg(short, long)]
    number_of_searches: u32,

    /// Path to the output file where the vertex to level mapping will be
    /// stored.
    #[arg(short, long)]
    level_to_vertex: PathBuf,
}

fn main() {
    let args = Args::parse();

    let pathfinder = reading_pathfinder(&args.file.as_path(), &args.file_type);

    // Get paths and level_to_vertex
    let paths = get_paths(&*pathfinder, args.number_of_searches);
    let level_to_vertex: Vec<Vertex> = level_to_vertex(&paths, pathfinder.number_of_vertices());

    // Write level_to_vertex to file
    let writer = BufWriter::new(File::create(args.level_to_vertex).unwrap());
    serde_json::to_writer(writer, &level_to_vertex).unwrap();
}
