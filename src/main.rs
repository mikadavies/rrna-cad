#![feature(get_many_mut, extend_one)]

use routines::{
    graph::{Graph, Tree},
    mesh::Mesh,
};

pub mod routines;

fn main() {
    #[cfg(debug_assertions)]
    simple_logger::init_with_level(log::Level::Debug).expect("Could not initialise logger");
    #[cfg(not(debug_assertions))]
    simple_logger::init_with_level(log::Level::Info).expect("Could not initialise logger");

    let mesh: Mesh = Mesh::load_from_file("config/mesh.toml");
    let mesh_consumable: Mesh = mesh.clone();
    let graph: Graph = Graph::from(mesh_consumable);
    let mut tree: Tree = Tree::from_graph(&graph, 4);

    log::info!("Tree: \n{tree}");
    let mut path: Vec<(bool, usize)> = Vec::new();
    tree.find_path(&mesh, &mut path, 4);
    log::info!("Path: {path:?}");
}
