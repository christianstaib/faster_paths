use crate::{
    dijkstra_data::DijkstraData,
    graphs::{path::Path, VertexId},
};

pub fn path_from_bidirectional_search(
    contact_node: VertexId,
    forward_data: &dyn DijkstraData,
    backward_data: &dyn DijkstraData,
) -> Option<Path> {
    let mut forward_route = forward_data.get_path(contact_node)?;
    let mut backward_route = backward_data.get_path(contact_node)?;
    backward_route.vertices.pop();
    backward_route.vertices.reverse();
    forward_route.vertices.extend(backward_route.vertices);
    forward_route.weight += backward_route.weight;

    Some(forward_route)
}
