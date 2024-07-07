use super::Graph;

pub struct ReversibleGraph<T: Graph + Default> {
    _out_graph: T,
    _in_graph: T,
}

impl<T: Graph + Default> Default for ReversibleGraph<T> {
    fn default() -> Self {
        Self {
            _out_graph: T::default(),
            _in_graph: T::default(),
        }
    }
}
