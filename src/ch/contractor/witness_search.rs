use ahash::{HashMap, HashMapExt, HashSet};

use crate::{
    graphs::{Graph, VertexId, Weight},
    queue::{radix_queue::RadixQueue, DijkstaQueue, DijkstraQueueElement},
};

pub fn witness_search(
    graph: &dyn Graph,
    source: VertexId,
    without: VertexId,
    max_weight: Weight,
    max_hops: u32,
    targets: &HashSet<VertexId>,
) -> HashMap<VertexId, Weight> {
    let mut queue = RadixQueue::new();
    let mut weight = HashMap::new();
    let mut hops = HashMap::new();

    let mut targets = targets.clone();

    queue.push(DijkstraQueueElement::new(0, source));
    weight.insert(source, 0);
    hops.insert(source, 0);

    while let Some(DijkstraQueueElement { vertex, .. }) = queue.pop() {
        if targets.remove(&vertex) && targets.is_empty() {
            break;
        }

        for edge in graph.out_edges(vertex) {
            let alternative_weight = weight[&vertex] + edge.weight();
            let alternative_hops = hops[&vertex] + 1;
            if (edge.head() != without)
                && (alternative_weight <= max_weight)
                && (alternative_hops <= max_hops)
            {
                let current_cost = *weight.get(&edge.head()).unwrap_or(&u32::MAX);
                if alternative_weight < current_cost {
                    queue.push(DijkstraQueueElement::new(alternative_weight, edge.head()));
                    weight.insert(edge.head(), alternative_weight);
                    hops.insert(edge.head(), alternative_hops);
                }
            }
        }
    }

    weight
}
