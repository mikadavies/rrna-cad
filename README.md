# rRNA CAD

A CAD tool to design ssRNA sequences that cotranscriptionally fold into desired shapes. The sequence generation method is based on motifs inspired by tecto-RNA and [this paper by Li et al.](https://doi.org/10.1038/s41467-018-04652-4).

## Table of contents
- [Usage](#usage)
- [How it works](#how-it-works)
  - [3D structure to RNA path (`graph.rs`)](#graphrs)
  - [Sequence generation (`sequencer.rs`)](#sequencerrs)
  - [User interface (`user_interface.rs`)](#userinterfacers)
  - [File I/O (`io.rs`)](#iors)
- [Benchmarks](#benchmarks)
- [Reliability / Accuracy](#reliability)
- [Related literature](#related-literature)

## Usage

TODO

## How it works
rRNA CAD is composed of 4 main modules:
- `graph.rs` deals with transforming a given structure into a path and motifs for the RNA sequence.
- `sequencer.rs` uses the output from `graph.rs` to generate a corresponding sequence.
- `user_interface.rs` contains the code for the user interface, to call graph and sequencer as required. It also deals with the rendering of the structure.
- `io.rs` currently does nothing, it aims to deal with file input/output in the future.

### `graph.rs`

#### Terminology
**Edge**<br>
An edge between two vertices, it has an origin and a destination. It is represented as a tuple of 2 unique IDs of the form `(origin, destination)`.

**Vertex / Node**<br>
A vertex in your structure, a point in 3D space. Each vertex has a unique identifier, and must be part of an edge. As such, each vertex has at least one parent or one child.

#### `construct_tree(edges)`
This is a function that takes an array of edges as an input. These are defined by the user, and connect vertices to each other. In this function, vertices are only referenced by their unique identifiers, and there is therefore no physical or dimensional information. The function iterates over all edges, constructing a tree describing the hierarchical relationship between vertices.

In cases where there is a cycle (i.e. a child vertex is the parent of one of its parent vertices), two new vertices are created, representing a kissing-loop in the RNA structure. Each vertex in the edge is assigned its corresponding one of the new vertices, and the uIDs of the new vertices are copied to a `cycle_breakers` array.

This function produces a `Tree` as an output. The tree has three components:
- `edges`: An array of edges, like the input.
- `nodes`: An array of vertices, or nodes, storing the hierarchical data of the structure. Each node has a uID and contains the uID of a parent node and the uIDs its children nodes.
- `cycle_breakers`: An array of cycle-breaking pairs. It contains the uIDS of kissing-loop nodes created to avoid cyclical structures, which would be impossible for many ssRNA configurations.

#### `sort_tree_edges(tree, node_coordinates)`
A function that takes a `Tree` and an array of 3D points as inputs, this uses the vertex hierarchy from the tree and coordinates of the vertices to sort the children of each node. This ensures the RNA path is physically possible. Due to the dimensionality of the structures, sorting edges by is not necessarily straightforward.

To overcome problems with non-planar edges, all vertices are projected onto a randomly selected plane. If any edge is perpendicular to the plane, a new plane is determined. The edges are then sorted based on the clockwise rotation between their in-plane vector and the parent to current-node vector.

This function mutates the tree in-place, and there is no output.

#### `find_rna_path(tree, node_coordinates)`
A function that traverses the tree edges with a path analogous to the path an RNA sequence would have to take (i.e. each edge is visited exactly twice, sensewise and antisensewise).The function then returns an array with the uIDs of the traversed nodes in the order they were visited.

The function initially sorts the edges using `sort_tree_edges`, and thus the children of each node are visited in order. For a given node, the function visits its children in order, recursively. Once a node has no more unvisited children, the function returns to the node's parent, and repeats.

### `sequencer.rs`

TODO

### `user_interface.rs`

TODO

### `io.rs`

Currently does nothing.

## Benchmarks

TODO

## Reliability

Preliminary tests using ViennaRNA to optimise the secondary structure have shown promising results. Further tests needed.

## Related Literature
- Li, M., Zheng, M., Wu, S. *et al*. In vivo production of RNA nanostructures via programmed folding of single-stranded RNAs. *Nat Commun* **9**, 2196 (2018). https://doi.org/10.1038/s41467-018-04652-4
