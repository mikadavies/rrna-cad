use rustc_hash::FxHashSet;

use crate::graph::{Edge, Vertex};

pub fn find_single_path(edges: impl Iterator<Item=Edge>, vertices: impl Iterator<Item=Vertex>) -> Option<Vec<usize>> {
    // Depth-first search (DFS)
    let edges_vec: Vec<Edge> = edges.collect();
    let vertices_vec: Vec<Vertex> = vertices.collect();

    let mut current_vertex: usize = 0;
     
    let mut current_edge: usize = 0;
    let mut visited_edges: FxHashSet<usize> = FxHashSet::default();
    let mut visited_vertices: FxHashSet<usize> = FxHashSet::default();

    {
        let is_current_edge: Option<usize> = vertices_vec.get(current_vertex)
            .unwrap()
            .edges
            .iter().filter(|&index| index.is_some())
            .map(|&index| index.unwrap())
            .find(|index| !visited_edges.contains(index));

        if is_current_edge.is_some() {
            current_edge = is_current_edge.unwrap();
            visited_edges.insert(current_edge);
            
        }
    }

    Some(Vec::new())
}