#![feature(get_many_mut, extend_one, ascii_char)]

use routines::{
    graph::{Graph, Tree}, mesh::Mesh, rnafold_validator::similarity_test, sequencer::MotifStorage
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
   
    let (mean_accuracy, ([best_result, best_result_db], result_acc)): (f64, ([String;2], f64)) = 
        similarity_test(5000, &mesh, &path, &motifs);
    log::info!("Mean similarity: {mean_accuracy:.2}%");
    log::info!("Highest accuracy: {result_acc:.2}%");
    log::info!("Best sequence: \n{best_result}\n{best_result_db}");
    
    //run_until_successful(&mesh, &path, &motifs, 90.0);
}
