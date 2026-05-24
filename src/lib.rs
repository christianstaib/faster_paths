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
//!         tail: Vertex::new(0),
//!         head: Vertex::new(1),
//!         weight: OrderedFloat(2.0),
//!     },
//!     WeightedEdge {
//!         tail: Vertex::new(0),
//!         head: Vertex::new(2),
//!         weight: OrderedFloat(10.0),
//!     },
//!     WeightedEdge {
//!         tail: Vertex::new(1),
//!         head: Vertex::new(2),
//!         weight: OrderedFloat(3.0),
//!     },
//! ];
//!
//! let contraction_hierarchy = contract_graph_parallel(&edges);
//! let mut pathfinder = ContractionHierarchyPathfinder::new(&contraction_hierarchy);
//!
//! let query = Query {
//!     source: Vertex::new(0),
//!     target: Vertex::new(2),
//! };
//!
//! assert_eq!(pathfinder.distance(&query), Some(OrderedFloat(5.0)));
//! assert_eq!(
//!     pathfinder.path(&query).unwrap().vertices,
//!     vec![Vertex::new(0), Vertex::new(1), Vertex::new(2)]
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
