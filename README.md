# Faster Paths

This crate aims to provide different pathfinding algorithms.
Currently, there are three pathfinding algorithms:
1. Dijkstra
2. Contraction Hierarchies
3. Hub Labeling

## Dijkstra

Classic Dijkstra. Both hash-based and vec-based `SearchState`s are provided.
The hash-based search state is good for *local* searches, while the vec-based
one is better for *global* searches.

## Contraction Hierarchies

This requires some preprocessing.
Two preprocessing methods are provided: serial and parallel contraction.
For now, I can only recommend parallel contraction, but serial contraction is
the classic paper version.
A Contraction Hierarchy requires *a bit* more space than the underlying graph.

## Hub Labeling

This requires some preprocessing.
A Hub Labeling can be created from a Contraction Hierarchy and can be seen as
the precomputed search spaces discovered during Contraction Hierarchy queries.
A Hub Labeling requires *a lot* more space than the underlying graph.

For querying, both the Contraction Hierarchy and Hub Labeling are needed.
Distance queries are basically instantaneous, but path queries are, for most
types and sizes of graphs, similar in performance to Contraction Hierarchy
queries (because the use the same mechanism for the path unpacking).