use crate::{
    dijkstra_data::DijkstraData,
    graphs::{path::Path, types::VertexId},
};

pub fn construct_route(
    contact_node: VertexId,
    forward_data: &DijkstraData,
    backward_data: &DijkstraData,
) -> Option<Path> {
    let mut forward_route = forward_data.get_route(contact_node)?;
    let mut backward_route = backward_data.get_route(contact_node)?;
    backward_route.vertices.pop();
    backward_route.vertices.reverse();
    forward_route.vertices.extend(backward_route.vertices);
    forward_route.weight += backward_route.weight;

    Some(forward_route)
}
