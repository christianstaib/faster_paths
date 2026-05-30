//! Faster Paths provides different pathfinding algorithms.
//!
//! Edge weights must be non-negative. The shortest-path algorithms in this
//! crate are not designed for graphs with negative edge weights.
//!
//! Minimal Contraction Hierarchy usage:
//!
//! ```
//! use faster_paths::{
//!     contraction_hierarchy::{ContractionHierarchyPathfinder, contract_graph_parallel},
//!     graph::WeightedEdge,
//!     path::Query,
//!     pathfinder::ShortestPathFinder,
//!     types::Vertex,
//! };
//! use ordered_float::OrderedFloat;
//!
//! let edges = vec![
//!     WeightedEdge {
//!         tail: 0,
//!         head: 1,
//!         weight: OrderedFloat(2.0),
//!     },
//!     WeightedEdge {
//!         tail: 0,
//!         head: 2,
//!         weight: OrderedFloat(10.0),
//!     },
//!     WeightedEdge {
//!         tail: 1,
//!         head: 2,
//!         weight: OrderedFloat(3.0),
//!     },
//! ];
//!
//! let contraction_hierarchy = contract_graph_parallel(&edges);
//! let mut pathfinder = ContractionHierarchyPathfinder::new(&contraction_hierarchy);
//!
//! let query = Query {
//!     source: 0,
//!     target: 2,
//! };
//!
//! assert_eq!(pathfinder.distance(&query), Some(OrderedFloat(5.0)));
//! assert_eq!(
//!     pathfinder.path(&query).unwrap().vertices,
//!     vec![0, 1, 2]
//! );
//! ```

pub mod classical_search;
pub mod contraction_hierarchy;
pub mod data_structures;
pub mod graph;
pub mod hub_labeling;
pub mod path;
pub mod pathfinder;
pub mod progress_bar;
pub mod types;
pub mod validation;
