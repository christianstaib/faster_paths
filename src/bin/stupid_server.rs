use std::path::PathBuf;

use actix_web::{
    web::{self, JsonConfig},
    App, HttpResponse, HttpServer, Responder,
};
use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
    },
    utility::average_hl_label_size,
};
use serde::{Deserialize, Serialize};

// Predict average label size by brute forcing a number of labels.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Number of labels to calculate
    #[arg(short, long)]
    num_labels: u32,
}

// Define a structure to hold shared application state
struct AppState {
    graph: ReversibleGraph<VecVecGraph>,
    num_labels: u32,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    // Initialize shared state
    let state = web::Data::new(AppState {
        graph,
        num_labels: args.num_labels,
    });

    //
    // Configure JSON payload size (e.g., 10 MB)
    let json_config = JsonConfig::default()
        .limit(50 * 1024 * 1024) // 10 MB
        .error_handler(|err, _req| {
            actix_web::error::InternalError::from_response(
                err,
                actix_web::HttpResponse::BadRequest().json("Invalid JSON payload"),
            )
            .into()
        });

    // Start the HTTP server with shared state
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone()) // Clone the Arc for each worker
            .app_data(json_config.clone()) // Apply JSON configuration
            .route("/process", web::post().to(process_numbers))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// Define the structure of the incoming JSON data
#[derive(Deserialize)]
struct InputData {
    numbers: Vec<u32>,
}

// Define the structure of the outgoing JSON data
#[derive(Serialize)]
struct OutputData {
    result: f32,
}

// Handler for processing the input and returning the result
async fn process_numbers(state: web::Data<AppState>, data: web::Json<InputData>) -> impl Responder {
    let graph = &state.graph;
    let num_labels = state.num_labels;
    let numbers = &data.numbers;

    // // Example processing: calculate the average
    // let sum: u32 = numbers.iter().sum();
    // let count = numbers.len() as f32;

    // // Handle division by zero
    // let average = if count > 0.0 { sum as f32 / count } else { 0.0 };

    let average = average_hl_label_size(graph.out_graph(), &numbers, num_labels);

    let response = OutputData { result: average };

    HttpResponse::Ok().json(response)
}
