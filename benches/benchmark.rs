#![feature(get_many_mut, extend_one)]
use criterion::{criterion_group, criterion_main, Criterion};
use routines::{graph::{Graph, Tree}, mesh::Mesh};

#[path = "../src/routines/mod.rs"]
mod routines;

fn generate_tree(graph: &Graph) -> Tree {
    Tree::from_graph(&graph, 4)
}
 
fn generate_path(tree: &mut Tree, mesh: &Mesh) -> Vec<(bool, usize)> {
    let mut path: Vec<(bool, usize)> = Vec::new();
    tree.find_path(&mesh, &mut path, 4);
    path
}

pub fn graph_benchmarks(c: &mut Criterion) {
    let mesh: Mesh = Mesh::load_from_file("config/mesh.toml");
    let mesh_consumable: Mesh = mesh.clone();

    let graph: Graph = Graph::from(mesh_consumable);
    let mut tree: Tree = generate_tree(&graph);
    
    c.bench_function("Generate Tree", |b| b.iter(|| generate_tree(&graph)));
    c.bench_function("Generate Path", |b| b.iter(|| generate_path(&mut tree, &mesh)));
}

criterion_group!{
    name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(5000);
    targets = graph_benchmarks
}
criterion_main!(benches);