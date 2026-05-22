//! Faster Paths provides different pathfinding algorithms.
//!
//! Minimal Contraction Hierarchy usage:
//!
//! ```
//! use ch::{
//!     contraction_hierarchy::{ContractionHierarchyPathfinder, contract_graph_parallel},
//!     graph::WeightedEdge,
//!     path::PathQuery,
//!     pathfinder::ShortestPathFinder,
//!     types::VertexId,
//! };
//! use ordered_float::OrderedFloat;
//!
//! let edges = vec![
//!     WeightedEdge {
//!         tail: VertexId::new(0),
//!         head: VertexId::new(1),
//!         weight: OrderedFloat(2.0),
//!     },
//!     WeightedEdge {
//!         tail: VertexId::new(0),
//!         head: VertexId::new(2),
//!         weight: OrderedFloat(10.0),
//!     },
//!     WeightedEdge {
//!         tail: VertexId::new(1),
//!         head: VertexId::new(2),
//!         weight: OrderedFloat(3.0),
//!     },
//! ];
//!
//! let contraction_hierarchy = contract_graph_parallel(edges);
//! let mut pathfinder = ContractionHierarchyPathfinder::new(&contraction_hierarchy);
//!
//! let query = PathQuery {
//!     source: VertexId::new(0),
//!     target: VertexId::new(2),
//! };
//!
//! assert_eq!(pathfinder.distance(&query), Some(OrderedFloat(5.0)));
//! assert_eq!(
//!     pathfinder.path(&query).unwrap().vertices,
//!     vec![VertexId::new(0), VertexId::new(1), VertexId::new(2)]
//! );
//! ```

pub mod classical_search;
pub mod contraction_hierarchy;
pub mod data_structures;
pub mod graph;
pub mod hub_labeling;
pub mod path;
pub mod pathfinder;
pub mod types;
pub mod validation;
