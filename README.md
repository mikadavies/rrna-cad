# Rusty RNA (RRNA)

## Table of contents
- [Rusty RNA (RRNA)](#rusty-rna-rrna)
  - [Table of contents](#table-of-contents)
  - [What is RRNA](#what-is-rrna)
  - [TODO](#todo)
  - [Documentation](#documentation)
    - [How it works](#how-it-works)
    - [RRNA File Syntax](#rrna-file-syntax)

## What is RRNA

RRNA is a CAD tool for designing RNA structures, in 2D or 3D, written in Rust.

As it is currently being developped, it has no useful features yet. However it aims to achieve or implement the following

- [ ] Easy to use GUI
- [ ] Accurate RNA structure prediction
- [ ] Fast RNA sequence generation
- [x] Enable imports from RRNA file instead of requiring editor
- [x] Export schematics to RRNA file
- [ ] Produce RNA sequence as text, illustrative diagram, and interaction diagrams

## TODO
- [ ] Split schematic to a single path
- [ ] Turn 3D schematic to 2D schematic
- [ ] Turn path to RNA sequence
- [ ] Turn schematic to interaction diagram

## Documentation

### How it works

RRNA is a rather naive CAD solution for RNA. After creating a wireframe structure in the editor, or loading one from a file, RRNA matches each vertex with a predefined motif, based on the number of connections the vertex has.

It then attempts to find a single path passing through every edge exactly twice. If this is done successfully, RRNA generates a random duplex-forming sequence, except where nucleotides are specified by the user or the motifs.

More details on the way RRNA works can be found [here](docs/how_it_works.md).

### RRNA File Syntax

RRNA files use the Rusty Object Notation (RON), and are effectively the serialised version of the `Graph` object representing your structure. As such, they start with `(` and end with `)`.

Within these brackets, there are two components: `vertices` and `edges`, where `vertices` is an array of all the vertices, and `edges` is an array of all the edges. The RRNA notation for a vertex follows the form:
```
(
    position: (x, y),
    class: Class,
    connections: Number of connections
)
```
And the RRNA notation for an edge has the form:
```

(
    origin: Index of origin vertex
    destination: Index of destination vertex
    length: Distance between origin and destination
)
```

Therefore, a structure with 4 vertices of which the type is unknown, layed out as shown in the following diagram:

<img src="docs_images/sample_graph_diagram.png" alt="drawing" style="width: 300px; margin-left: auto; margin-right: auto;"/>

Would be written as such in the RRNA file:

```
(
    vertices: [
        (
            position: (0.0, 0.0, 0.0),
            class: Unknown,
            connections: 2,
        ),
        (
            position: (1.0, 1.0, 1.0),
            class: Unknown,
            connections: 1,
        ),
        (
            position: (-1.0, -1.0, -1.0),
            class: Unknown,
            connections: 1,
        ),
        (
            position: (0.0, 1.0, 0.0),
            class: Unknown,
            connections: 2,
        ),
    ],
    edges: [
        (
            origin: 0,
            destination: 1,
            length: 1.7320508,
        ),
        (
            origin: 0,
            destination: 3,
            length: 1.0,
        ),
        (
            origin: 2,
            destination: 3,
            length: 2.4494898,
        ),
    ],
)
```