# FasterPaths

The FasterPaths Rust crate provides highly efficient pathfinding algorithms designed for static graphs (i.e., graphs that do not change over time).

## Pathfinding Techniques

Currently, FasterPaths implements two advanced techniques for enhanced performance:

1. **Contraction Hierarchies**
2. **Hub Labels**

Both techniques necessitate some preprocessing, which aims to be as brief as possible.

### Contraction Hierarchies Explained

Contraction Hierarchies simplify the graph by allowing bidirectional search queries on a contracted version of the graph. This approach significantly reduces the number of vertices visited during the search.

### Hub Labels Explained

Hub Labels involve calculating a label for each vertex, representing the shortest path tree of a bidirectional search on a contracted graph. The weight of a shortest path is the minimal weight overlap of two labels.

## Graph Requirements

- Vertices must be continuously numbered starting from zero and are represented as `u32`.
- Edge weights are also `u32` and non-negative.
- Self-edges are not allowed.
