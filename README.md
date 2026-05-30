# Faster Paths

[![Crates.io](https://img.shields.io/crates/v/faster_paths.svg)](https://crates.io/crates/faster_paths)
[![Docs.rs](https://docs.rs/faster_paths/badge.svg)](https://docs.rs/faster_paths)
[![Build](https://github.com/christianstaib/faster_paths3/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/christianstaib/faster_paths3/actions/workflows/build.yml)
[![Tests](https://github.com/christianstaib/faster_paths3/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/christianstaib/faster_paths3/actions/workflows/test.yml)

> **Blazingly fast, parallel creation of Contraction Hierarchies** for shortest path queries in microseconds and Hub Labeling for shortest path distance queries in nanoseconds, *almost* independent of graph size. Bring your own weight datatypes!

This crate aims to provide different pathfinding algorithms.
Currently, there are three pathfinding algorithms:
1. Dijkstra (slow, as expected)
2. Contraction Hierarchies (fast & versatile)
3. Hub Labeling (blazingly fast distance queries)

## Installation

Faster Paths is available on [crates.io](https://crates.io/crates/faster_paths).
If you want to use it, just add it to your `Cargo.toml`:

```toml
[dependencies]
faster_paths = "0.1.0"
```

## Data Types

The basic building block of the graph you want to work with is `WeightedEdge`.
A `WeightedEdge` has a `tail` and a `head`, both of type `Vertex`.
`Vertex` is a strong typedef for `u32` and can be instantiated with
`Vertex::new(u32)`.

It also has a generic `weight`. The weight type needs to implement the crate's
`Distance` requirements, including `Ord` and `Add`. Possible weight types are
`u32`, `i32`, `u64`, `i64`, `OrderedFloat<f32>`, `OrderedFloat<f64>`, or your
own custom weight type =).

Edge weights must be non-negative. The shortest-path algorithms in this crate
are not designed for graphs with negative edge weights.

## Contraction Hierarchies

This requires some preprocessing.
Two preprocessing methods are provided: serial and parallel contraction.
For now, I can only recommend parallel contraction, but serial contraction is
the classic paper version.
A Contraction Hierarchy requires *a bit* more space than the underlying graph.

Minimal usage:

```rust
use faster_paths::{
    contraction_hierarchy::{ContractionHierarchyPathfinder, contract_graph_parallel},
    graph::WeightedEdge,
    path::Query,
    pathfinder::ShortestPathFinder,
    types::Vertex,
};
use ordered_float::OrderedFloat;

let edges = vec![
    WeightedEdge {
        tail: Vertex::new(0),
        head: Vertex::new(1),
        weight: OrderedFloat(2.0),
    },
    WeightedEdge {
        tail: Vertex::new(0),
        head: Vertex::new(2),
        weight: OrderedFloat(10.0),
    },
    WeightedEdge {
        tail: Vertex::new(1),
        head: Vertex::new(2),
        weight: OrderedFloat(3.0),
    },
];

let contraction_hierarchy = contract_graph_parallel(&edges);
let mut pathfinder = ContractionHierarchyPathfinder::new(&contraction_hierarchy);

let query = Query {
    source: Vertex::new(0),
    target: Vertex::new(2),
};

assert_eq!(pathfinder.distance(&query), Some(OrderedFloat(5.0)));
assert_eq!(
    pathfinder.path(&query).unwrap().vertices,
    vec![Vertex::new(0), Vertex::new(1), Vertex::new(2)]
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
queries (because they use the same mechanism for path unpacking).

Minimal usage, after building a Contraction Hierarchy as above:
During the creation of the Hub Labeling (during the *pruning* to be exact) the
algorithm needs to determine if two weights are *equal*. As a simple equal
comparison has the possibility to not work with all types, you need to provide
an epsilon that fits your datatype and domain.

```rust
let epsilon = OrderedFloat(1e-6);
let hub_labeling =
    HubLabeling::try_from_contraction_hierarchy(&contraction_hierarchy, epsilon).unwrap();

let mut pathfinder = HubLabelingPathfinder::new(&contraction_hierarchy, &hub_labeling);

assert_eq!(pathfinder.distance(&query), Some(OrderedFloat(5.0)));
assert_eq!(
    pathfinder.path(&query).unwrap().vertices,
    vec![Vertex::new(0), Vertex::new(1), Vertex::new(2)]
);

```

## Benchmarking

The benchmarks use the [DIMACS Challenge 9 road networks](https://www.diag.uniroma1.it/challenge9/download.shtml), code to run them yourself can be found [here](https://github.com/christianstaib/faster_paths_benchmark).
The Contraction Hierarchy and Dijkstra measurements below were run on an M1 MacBook Air with 8 cores and 16 GB of RAM.
On larger graphs, path unpacking dominates path-query time, which has optimizatin potential.

| Graph          | # vertices | # edges | Dijkstra distance avg | Dijkstra path avg | CH construction | CH distance avg | CH path avg |
| :------------- | ---------: | ------: | --------------------: | ----------------: | --------------: | --------------: | ----------: |
| USA-road-d.NY  |       264k |    734k |               10.16ms |           10.22ms |           1.54s |        101.65µs |    126.99µs |
| USA-road-d.CAL |      1.89M |   4.66M |               98.02ms |           97.95ms |           7.51s |        148.00µs |    254.48µs |
| USA-road-d.USA |      23.9M |   58.3M |                 1.49s |             1.49s |         119.24s |        568.20µs |      1.16ms |
| USA-road-t.NY  |       264k |    734k |               11.19ms |           11.22ms |           1.24s |         52.23µs |     66.34µs |
| USA-road-t.CAL |      1.89M |   4.66M |               97.52ms |           97.53ms |           5.86s |         69.50µs |    145.24µs |
| USA-road-t.USA |      23.9M |   58.3M |                 1.50s |             1.48s |          77.11s |        132.25µs |    578.67µs |

The Hub Labeling benchmarks were run on a different system with 2x AMD EPYC 9454 and 2.5 TB of RAM on the wonderful [BwUniCluster 3.0](https://www.bwhpc.de/).
Hub Labeling reuses the Contraction Hierarchy for path unpacking, so the path timings include the same, dominating time hit.
The HL construction time in the table are to be seen in addition to the CH construction time.
But if you need an distance oracle for your graph, the super fast distance query time might be nice fore you, but ensure you have enough RAM, as the size of the Hub Labeling can easily be hundreds of times larger as the underlying graph.
However, for some super dense graphs, the Contraction Hierarchy and the Hub Labeling can actually be smaller than the underlying graph, as i explored in my [bachelor thesis](http://dx.doi.org/10.18419/opus-15429).


| Graph          | CH construction | HL construction | Label size | HL distance |  HL path |
| -------------- | --------------: | --------------: | ---------: | ----------: | -------: |
| USA-road-d.NY  |        930.46ms |           2.60s |     110.02 |      1.03µs |  19.13µs |
| USA-road-d.CAL |           5.18s |          31.68s |     136.13 |      1.12µs |  80.99µs |
| USA-road-d.USA |          71.62s |         500.87s |     241.62 |     11.10µs | 746.96µs |
| USA-road-t.NY  |        788.69ms |           1.90s |      73.42 |    838.00ns |  13.65µs |
| USA-road-t.CAL |           4.82s |          22.97s |      95.35 |    871.00ns |  56.29µs |
| USA-road-t.USA |          64.83s |         291.66s |     110.84 |      1.28µs | 389.00µs |

## Credits & Acknowledgements
- The name of this crate was inspired by the wonderful [fast_paths](https://github.com/easbar/fast_paths) whose path unpacking is a lot faster than mine =).
- During my understanding of Contraction Hierarchies [this](https://jlazarsfeld.github.io/ch.150.project) guide by John Lazarsfeld helped me a lot!

