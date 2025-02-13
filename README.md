# RRNA CAD

A CAD tool to design ssRNA sequences that cotranscriptionally fold into desired shapes. The sequence generation method is based on motifs inspired by tecto-RNA and [this paper by Li et al.](https://doi.org/10.1038/s41467-018-04652-4).

## Usage

## How it works
rRNA CAD is composed of 4 main modules:
- `graph.rs` deals with transforming a given structure into a path and motifs for the RNA sequence.
- `sequencer.rs` uses the output from `graph.rs` to generate a corresponding sequence.
- `user_interface.rs` contains the code for the user interface, to call graph and sequencer as required. It also deals with the rendering of the structure.
- `io.rs` currently does nothing, it aims to deal with file input/output in the future.

### `graph.rs`

TODO

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
