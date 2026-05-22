# Faster Paths

[![Build](https://github.com/christianstaib/faster_paths3/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/christianstaib/faster_paths3/actions/workflows/build.yml)
[![Tests](https://github.com/christianstaib/faster_paths3/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/christianstaib/faster_paths3/actions/workflows/test.yml)

> Blazingly fast, parallel creation of Contraction Hierarchies for shortest path queries in microseconds and Hub Labeling for shortest path distance queries in nanoseconds, *almost* independent of graph size. Bring your own weight datatypes!

This crate aims to provide different pathfinding algorithms.
Currently, there are three pathfinding algorithms:
1. Dijkstra (slow, as expected)
2. Contraction Hierarchies (fast & versatile)
3. Hub Labeling (blazingly fast distance queries)

## Data Types

The basic building block of the graph you want to work with is `WeightedEdge`.
A `WeightedEdge` has a `tail` and a `head`, both of type `VertexId`.
`VertexId` is a strong typedef for `u32` and can be instantiated with
`VertexId::new(u32)`.

It also has a generic `weight`. The weight type needs to implement the crate's
`Distance` requirements, including `Ord` and `Add`. Possible weight types are
`u32`, `i32`, `u64`, `i64`, `OrderedFloat<f32>`, `OrderedFloat<f64>`, or your
own custom weight type =).

## Contraction Hierarchies

This requires some preprocessing.
Two preprocessing methods are provided: serial and parallel contraction.
For now, I can only recommend parallel contraction, but serial contraction is
the classic paper version.
A Contraction Hierarchy requires *a bit* more space than the underlying graph.

Minimal usage:

```rust
let edges = vec![
    WeightedEdge {
        tail: VertexId::new(0),
        head: VertexId::new(1),
        weight: OrderedFloat(2.0),
    },
    WeightedEdge {
        tail: VertexId::new(0),
        head: VertexId::new(2),
        weight: OrderedFloat(10.0),
    },
    WeightedEdge {
        tail: VertexId::new(1),
        head: VertexId::new(2),
        weight: OrderedFloat(3.0),
    },
];

let contraction_hierarchy = contract_graph_parallel(edges, 0.5);
let mut pathfinder = ContractionHierarchyPathfinder::new(&contraction_hierarchy);

let query = PathQuery {
    source: VertexId::new(0),
    target: VertexId::new(2),
};

assert_eq!(pathfinder.distance(&query), Some(OrderedFloat(5.0)));
assert_eq!(
    pathfinder.path(&query).unwrap().vertices,
    vec![VertexId::new(0), VertexId::new(1), VertexId::new(2)]
);
```

## Hub Labeling

This requires some preprocessing.
A Hub Labeling can be created from a Contraction Hierarchy and can be seen as
the precomputed search spaces discovered during Contraction Hierarchy queries.
A Hub Labeling requires *a lot* more space than the underlying graph.

For querying, both the Contraction Hierarchy and Hub Labeling are needed.
Distance queries are basically instantaneous, but path queries are, for most
types and sizes of graphs, similar in performance to Contraction Hierarchy
queries (because the use the same mechanism for the path unpacking).

Minimal usage, after building a Contraction Hierarchy as above:
During the creation of the Hub Labeling (during the *pruning* to be exact) the
algorithm needs to determine if two weights are *equal*. As a simple equal
comparison has the possibility to not work with all types, you need to provide
an epsilon that fits your datatype and domain.

```rust
let epsilon = OrderedFloat(1e-6);
let hub_labeling =
    HubLabeling::try_from_contraction_hierarchy(&contraction_hierarchy, epsilon).unwrap();

let mut pathfinder = HubLabelingPathfinder {
    contraction_hierarchy: &contraction_hierarchy,
    hub_labeling: &hub_labeling,
};

assert_eq!(pathfinder.distance(&query), Some(OrderedFloat(5.0)));
assert_eq!(
    pathfinder.path(&query).unwrap().vertices,
    vec![VertexId::new(0), VertexId::new(1), VertexId::new(2)]
);
```
