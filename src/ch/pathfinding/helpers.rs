use crate::{
    ch::directed_contracted_graph::DirectedContractedGraph,
    dijkstra_data::DijkstraData,
    graphs::{VertexId, Weight},
    queue::DijkstraQueueElement,
};

pub fn get_data(
    graph: &DirectedContractedGraph,
    forward_data: &mut dyn DijkstraData,
    backward_data: &mut dyn DijkstraData,
) -> (VertexId, Weight) {
    let mut meeting_weight = u32::MAX;
    let mut meeting_vertex = u32::MAX;

    let mut f = 0;
    let mut b = 0;

    while (!forward_data.is_empty() && (f < meeting_weight))
        || (!backward_data.is_empty() && (b < meeting_weight))
    {
        if f < meeting_weight {
            if let Some(DijkstraQueueElement { vertex, .. }) = forward_data.pop() {
                let forward_weight = forward_data.get_vertex_entry(vertex).weight.unwrap();
                f = std::cmp::max(f, forward_weight);

                let mut stall = false;
                for in_edge in graph.downard_edges(vertex) {
                    if let Some(predecessor_weight) =
                        forward_data.get_vertex_entry(in_edge.head()).weight
                    {
                        if predecessor_weight + in_edge.weight() < forward_weight {
                            stall = true;
                            break;
                        }
                    }
                }

                if !stall {
                    if let Some(backward_weight) = backward_data.get_vertex_entry(vertex).weight {
                        let weight = forward_weight + backward_weight;
                        if weight < meeting_weight {
                            meeting_weight = weight;
                            meeting_vertex = vertex;
                        }
                    }
                    graph
                        .upward_edges(vertex)
                        .for_each(|edge| forward_data.update(vertex, edge.head(), edge.weight()));
                }
            }
        }

        if b < meeting_weight {
            if let Some(DijkstraQueueElement { vertex, .. }) = backward_data.pop() {
                let backward_weight = backward_data.get_vertex_entry(vertex).weight.unwrap();
                b = std::cmp::max(b, backward_weight);

                let mut stall = false;
                for out_edge in graph.upward_edges(vertex) {
                    if let Some(predecessor_weight) =
                        backward_data.get_vertex_entry(out_edge.head()).weight
                    {
                        if predecessor_weight + out_edge.weight() < backward_weight {
                            stall = true;
                            break;
                        }
                    }
                }

                if !stall {
                    if let Some(forward_weight) = forward_data.get_vertex_entry(vertex).weight {
                        let weight = forward_weight + backward_weight;
                        if weight < meeting_weight {
                            meeting_weight = weight;
                            meeting_vertex = vertex;
                        }
                    }
                    graph.downard_edges(vertex).for_each(|edge| {
                        backward_data.update(vertex, edge.head(), edge.weight());
                    });
                }
            }
        }

        if f >= meeting_weight && b >= meeting_weight {
            break;
        }
    }

    (meeting_vertex, meeting_weight)
}
