mod general;
mod parallel;
mod queue;
mod sequential;
mod working_graph;

pub use parallel::contract_graph_parallel;
pub use sequential::contract_graph_sequential;
