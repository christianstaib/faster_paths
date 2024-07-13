use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::graphs::{graph_functions::random_paths, path::read_pathfinder};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    pathfinder: PathBuf,
    /// Path of .fmi file
    #[arg(short, long, default_value = "100000")]
    number_of_paths: u32,
    /// Path of .fmi file
    #[arg(short, long)]
    paths: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Loading pathfinder");
    let pathfinder = read_pathfinder(&args.pathfinder).unwrap();

    println!("Generating random pair paths");
    let paths = random_paths(
        &*pathfinder,
        args.number_of_paths,
        pathfinder.number_of_vertices() as u32,
    );

    println!("Writing paths to file");
    let writer = BufWriter::new(File::create(&args.paths).unwrap());
    serde_json::to_writer(writer, &paths).unwrap();
}
