#![windows_subsystem = "windows"]

pub mod routines;

use routines::user_interface::run_gui;

pub fn main() {
    #[cfg(debug_assertions)]
    {
        use glam::{Vec3A, vec3a};
        use routines::{
            graph::{Tree, find_rna_path},
            sequencer::generate_sequence,
        };

        simple_logger::init_with_level(log::Level::Debug).unwrap();

        fn _create_test_edges() -> Vec<(usize, usize)> {
            vec![
                (4, 0),
                (4, 1),
                (4, 2),
                (4, 3),
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
            ]
        }

        fn _create_tree_vertices() -> Vec<Vec3A> {
            vec![
                vec3a(-10.0, 10.0, 0.0),
                vec3a(10.0, 10.0, 0.0),
                vec3a(10.0, -10.0, 0.0),
                vec3a(-10.0, -10.0, 0.0),
                vec3a(0.0, 0.0, 20.0),
            ]
        }

        let mut tree: Tree = routines::graph::construct_tree(&_create_test_edges());
        log::debug!("Tree:");
        log::debug!("Nodes: {:?}", tree.nodes);
        log::debug!("Cycle-Breaker Nodes: {:?}", tree.cycle_breakers);
        log::debug!("Edges: {:?}", tree.edges);
        log::debug!("Sorted Tree Nodes: {:?}", tree.nodes);
        let coordinates: Vec<Vec3A> = _create_tree_vertices();
        let path: Vec<usize> = find_rna_path(&mut tree, &coordinates);
        log::info!("RNA Path: {path:?}",);
        let sequence: String = generate_sequence(&path, &tree, &coordinates);
        log::info!("Sequence: {sequence}");
    }

    #[cfg(not(debug_assertions))]
    simple_logger::init_with_level(log::Level::Info).unwrap();

    run_gui();
}
