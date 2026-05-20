mod general;
mod parallel;
mod queue;
mod sequential;
mod terms;

pub use parallel::contract_graph_parallel;
pub use sequential::contract_graph_sequential;
