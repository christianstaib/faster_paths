use crate::{
    Edge as ChEdge,
    types::{Distance, VertexId},
};

pub(super) struct WorkingGraph {
    outgoing: Vec<Vec<ChEdge>>,
    incoming: Vec<Vec<(VertexId, Distance)>>,
    contracted_edges: Vec<ChEdge>,
}

impl WorkingGraph {
    pub(super) fn new(n: usize) -> Self {
        Self {
            outgoing: vec![Vec::new(); n],
            incoming: vec![Vec::new(); n],
            contracted_edges: Vec::new(),
        }
    }

    pub(super) fn num_vertices(&self) -> usize {
        self.outgoing.len()
    }

    pub(super) fn get_out(&self, vertex: VertexId) -> &[ChEdge] {
        return &self.outgoing[vertex.as_usize()];
    }

    pub(super) fn get_in(&self, vertex: VertexId) -> &[(VertexId, Distance)] {
        return &self.incoming[vertex.as_usize()];
    }

    pub(super) fn add_edge(&mut self, edge: ChEdge) {
        let tail = edge.tail();
        let head = edge.head();

        if tail == head {
            return;
        }

        if let Some(old_edge) = self.outgoing[tail.as_usize()]
            .iter_mut()
            .find(|old| old.head() == head)
        {
            if old_edge.weight() <= edge.weight() {
                return;
            }

            *old_edge = edge;
            self.incoming[head.as_usize()]
                .iter_mut()
                .find(|(incoming_tail, _)| *incoming_tail == tail)
                .unwrap()
                .1 = edge.weight();
            return;
        }

        self.outgoing[tail.as_usize()].push(edge);
        self.incoming[head.as_usize()].push((tail, edge.weight()));
    }

    pub(super) fn contract_vertex(&mut self, vertex: VertexId) {
        let vertex_index = vertex.as_usize();

        for (tail, _) in std::mem::take(&mut self.incoming[vertex_index]) {
            let outgoing = &mut self.outgoing[tail.as_usize()];

            if let Some(index) = outgoing.iter().position(|edge| edge.head() == vertex) {
                self.contracted_edges.push(outgoing.swap_remove(index));
            }
        }

        for edge in std::mem::take(&mut self.outgoing[vertex_index]) {
            self.incoming[edge.head().as_usize()].retain(|(tail, _)| *tail != vertex);
            self.contracted_edges.push(edge);
        }
    }

    pub(super) fn contracted_edges(&self) -> &[ChEdge] {
        &self.contracted_edges
    }
}
