mod graph;
mod motifs;
mod motif_assembler;
mod pathfinder;
mod plotter;

fn main() {
    simple_logger::init_with_level(log::Level::Trace).expect("Could not initiate logger.");

    let graph: graph::Graph = graph::Graph::from_ron("./data/test_graph.rrna").unwrap();

    graph.to_schematic("./schematics/test-graph.svg");
}
