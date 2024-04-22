use crate::graphs::{Graph, VertexId};

use self::{
    cost_of_queries::CostOfQueries, deleted_neighbors::DeletedNeighbors,
    edge_difference::EdgeDifference,
};

use super::Shortcut;

pub mod cost_of_queries;
pub mod deleted_neighbors;
pub mod edge_difference;
pub mod hitting_set;

pub trait PriorityFunction {
    fn initialize(&mut self, graph: &Box<dyn Graph>);

    /// Gets the priority of node v in the graph
    fn priority(&self, vertex: VertexId, graph: &Box<dyn Graph>, shortcuts: &Vec<Shortcut>) -> i32;

    /// Gets called just ERFORE a vertex is contracted. Gives priority terms the oppernunity to updated
    /// neighboring nodes priorities.
    fn update(&mut self, vertex: VertexId, graph: &Box<dyn Graph>);
}

pub fn decode_function(
    priority_functions_letters: &str,
) -> Vec<(i32, Box<dyn PriorityFunction + Sync>)> {
    let mut terms = Vec::new();

    for letter in priority_functions_letters.split('_') {
        let letter = letter.split(':').collect::<Vec<_>>();
        let priority_function = *letter.first().unwrap();
        let coefficent = letter.get(1).unwrap().parse::<i32>().unwrap();
        match priority_function {
            "E" => register(&mut terms, coefficent, EdgeDifference::new()),
            "D" => register(&mut terms, coefficent, DeletedNeighbors::new()),
            "C" => register(&mut terms, coefficent, CostOfQueries::new()),
            _ => panic!("letter not recognized"),
        }
    }

    terms
}

fn register(
    priority_terms: &mut Vec<(i32, Box<dyn PriorityFunction + Sync>)>,
    coefficent: i32,
    priority_function: impl PriorityFunction + 'static + Sync,
) {
    priority_terms.push((coefficent, Box::new(priority_function)));
}
