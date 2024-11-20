#![feature(get_many_mut, extend_one, ascii_char)]

use routines::{
    graph::{Graph, Tree}, 
    mesh::Mesh, 
    sequencer::{generate_sequence, MotifStorage}
};

pub mod routines;

fn main() {
    #[cfg(debug_assertions)]
    simple_logger::init_with_level(log::Level::Debug).expect("Could not initialise logger");
    #[cfg(not(debug_assertions))]
    simple_logger::init_with_level(log::Level::Info).expect("Could not initialise logger");

    // Load motifs
    let motifs: MotifStorage = MotifStorage::load_from_file("config/motifs.toml");

    // Load mesh, create graph and tree
    let mesh: Mesh = Mesh::load_from_file("config/mesh.toml");
    let mesh_consumable: Mesh = mesh.clone();
    let graph: Graph = Graph::from(mesh_consumable);
    let mut tree: Tree = Tree::from_graph(&graph, 3);

    log::info!("Tree: \n{tree}");
    
    // Find path
    let mut path: Vec<(bool, usize)> = Vec::new();
    tree.find_path(&mesh, &mut path, 3);
    log::debug!("Path: {path:?}");

    // Generate sequence
    let [db_sequence, nt_sequence]: [String; 2] = generate_sequence(&mesh, &path, &motifs);
    log::info!("Sequence:\n {nt_sequence}");
    log::info!("Brackets:\n {db_sequence}");    
}
