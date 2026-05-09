use crate::{
    contraction_hierachy::{ContractionHierarchy, unpack_and_concat_shortcut_paths},
    hub_labeling::{
        HubLabeling,
        entry::{min_distance_intersection, path_to_root},
    },
    path::{Path, PathQuery},
    pathfinder::ShortestPathFinder,
    types::Distance,
};

pub struct HubLabelingPathfinder<'a, D: Distance> {
    pub contraction_hierarchy: &'a ContractionHierarchy<D>,
    pub hub_labeling: &'a HubLabeling<D>,
}

impl<'a, D: Distance> ShortestPathFinder for HubLabelingPathfinder<'a, D> {
    type Distance = D;

    fn path(&mut self, query: &PathQuery) -> Option<Path<Self::Distance>> {
        let up_label = self
            .hub_labeling
            .up_hub_labeling
            .nested(query.source.as_usize());
        let down_label = self
            .hub_labeling
            .down_hub_labeling
            .nested(query.target.as_usize());

        let (distance, up_index, down_index) = min_distance_intersection(up_label, down_label)?;

        let up_reversed_shortcut_path = path_to_root(up_label, up_index)?;
        let down_reversed_shortcut_path = path_to_root(down_label, down_index)?;

        let vertices = unpack_and_concat_shortcut_paths(
            self.contraction_hierarchy,
            &up_reversed_shortcut_path,
            &down_reversed_shortcut_path,
        )?;

        Some(Path { vertices, distance })
    }

    fn distance(&mut self, query: &PathQuery) -> Option<Self::Distance> {
        let up_label = self
            .hub_labeling
            .up_hub_labeling
            .nested(query.source.as_usize());
        let down_label = self
            .hub_labeling
            .down_hub_labeling
            .nested(query.target.as_usize());

        let (distance, _up_index, _down_index) = min_distance_intersection(up_label, down_label)?;

        Some(distance)
    }
}
