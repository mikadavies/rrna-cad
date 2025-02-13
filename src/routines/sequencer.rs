use std::borrow::Cow;

use glam::Vec3A;
use nanorand::{Rng, WyRand};
use rustc_hash::FxHashMap;

use super::graph::Tree;

// TODO
pub fn generate_sequence(rna_path: &[usize], tree: &Tree, node_coordinates: &[Vec3A]) -> String {
    let mut sequence: String = String::new();

    let rng: &mut WyRand = &mut WyRand::new();
    let node_types: FxHashMap<usize, NodeType> = get_node_types(tree);

    // TODO
    // For each node in path, associate sequence from node_types
    // Get distance to next node in path
    // Generate random sequence with given length
    let mut visited_edges: FxHashMap<(usize, usize), String> = FxHashMap::default();
    let mut visited_kls: FxHashMap<usize, String> = FxHashMap::default();
    rna_path
        .iter()
        .enumerate()
        .for_each(|(path_index, &node_index)| {
            if let Some(&next_node_index) = rna_path.get(path_index + 1) {
                // Generate node sequence and append to path
                let node_type: NodeType = *node_types.get(&node_index).unwrap();
                match node_type {
                    NodeType::KissingLoop(other) => {
                        if visited_kls.contains_key(&other) {
                            // Generate antisense sequence for complement
                            let existing_seq: &str = visited_kls.get(&other).unwrap();
                            sequence.extend(existing_seq.chars().rev().map(|c| match c {
                                'A' => 'U',
                                'G' => 'C',
                                'C' => 'G',
                                'U' => 'A',
                                _ => 'O',
                            }));
                        } else {
                            let node_sequence: String = generate_node_sequence(node_type, rng);
                            visited_kls.insert(node_index, node_sequence.clone());
                            sequence.push_str(&node_sequence);
                        }
                    }
                    _ => sequence.push_str(&generate_node_sequence(node_type, rng)),
                }
                // Generate edge sequence
                if let Some(sense_sequence) = visited_edges.get(&(next_node_index, node_index)) {
                    // Generate antisense sequence
                    // Replace 20% of AU bonds as GU bonds for better DNA amplification
                    sequence.extend(sense_sequence.chars().rev().map(|c| match c {
                        'A' => 'U',
                        'G' => 'C',
                        'C' => 'G',
                        'U' => match rng.generate_range(0..10u8) {
                            0 | 1 => 'G',
                            _ => 'A',
                        },
                        _ => 'O',
                    }));
                } else {
                    log::debug!("Node coordinates: {node_coordinates:?}");
                    let seq: String = generate_edge_sequence(
                        node_coordinates
                            .get(node_index)
                            .map(Cow::Borrowed)
                            .unwrap_or_else(|| {
                                let parent: usize = tree.nodes.get(&node_index).unwrap().0;
                                let target: usize = tree
                                    .nodes
                                    .get(match tree.cycle_breakers.get(&node_index) {
                                        // Find corresponding cycle-breaker node
                                        // If current node is the key, other is the value
                                        Some(cycle_breaker) => cycle_breaker,
                                        // If the curret node is the value
                                        // Find the pair with this node as the value and extract the key
                                        None => {
                                            tree.cycle_breakers
                                                .iter()
                                                .find(|(_key, val)| **val == node_index)
                                                .unwrap()
                                                .0
                                        }
                                    })
                                    .unwrap()
                                    .0;
                                Cow::Owned(
                                    0.5 * (node_coordinates.get(parent).unwrap()
                                        + node_coordinates.get(target).unwrap()),
                                )
                            })
                            .distance(
                                *node_coordinates
                                    .get(next_node_index)
                                    .map(Cow::Borrowed)
                                    .unwrap_or_else(|| {
                                        let parent: usize = tree.nodes.get(&node_index).unwrap().0;
                                        let target: usize = tree
                                            .nodes
                                            .get(match tree.cycle_breakers.get(&next_node_index) {
                                                // Find corresponding cycle-breaker node
                                                // If current node is the key, other is the value
                                                Some(cycle_breaker) => cycle_breaker,
                                                // If the curret node is the value
                                                // Find the pair with this node as the value and extract the key
                                                None => {
                                                    tree.cycle_breakers
                                                        .iter()
                                                        .find(|(_key, val)| {
                                                            **val == next_node_index
                                                        })
                                                        .unwrap()
                                                        .0
                                                }
                                            })
                                            .unwrap()
                                            .0;
                                        //log::debug!("Parent: {parent}, Child: {target}");
                                        if parent == usize::MAX {
                                            log::error!("This isn't supposed to happen!");
                                            Cow::Owned(
                                                0.5 * (node_coordinates.get(parent).unwrap()
                                                    + node_coordinates.get(target).unwrap()),
                                            )
                                        } else {
                                            Cow::Owned(
                                                0.5 * (node_coordinates.get(parent).unwrap()
                                                    + node_coordinates.get(target).unwrap()),
                                            )
                                        }
                                    }),
                            )
                            .round() as usize,
                        rng,
                    );
                    visited_edges.insert((node_index, next_node_index), seq.clone());
                    sequence.push_str(&seq);
                }
            }
        });

    sequence
}

fn generate_edge_sequence(length: usize, rng: &mut WyRand) -> String {
    (0..length)
        .map(|_| match rng.generate_range(0u8..5u8) {
            0 => 'A',
            1 => 'U',
            2 => 'C',
            _ => 'G',
        })
        .collect()
}

fn generate_node_sequence(node_type: NodeType, rng: &mut WyRand) -> String {
    match node_type {
        NodeType::Hairpin | NodeType::KissingLoop(_) => {
            let mut seq: String = "GC".to_string();
            let first_half: String = (0..3)
                .map(|_| match rng.generate_range(0u8..4u8) {
                    0 => 'A',
                    1 => 'U',
                    2 => 'C',
                    _ => 'G',
                })
                .collect();
            let second_half = first_half.chars().rev().skip(1);
            seq.push_str(&first_half);
            seq.extend(second_half);
            seq.push_str("GC");
            seq
        }
        NodeType::Kink => "AAAA".to_string(),
        NodeType::OpenJunction => "CGUUUCG".to_string(),
    }
}

fn get_node_types(tree: &Tree) -> FxHashMap<usize, NodeType> {
    tree.nodes
        .iter()
        .map(|(&node_id, (_parent, children))| {
            (
                node_id,
                if let Some(complement) = tree.cycle_breakers.get(&node_id) {
                    NodeType::KissingLoop(*complement)
                } else if let Some((key, _val)) = tree
                    .cycle_breakers
                    .iter()
                    .find(|(_key, val)| **val == node_id)
                {
                    NodeType::KissingLoop(*key)
                } else {
                    match children.len() {
                        0 => NodeType::Hairpin,
                        1 => NodeType::Kink,
                        _ => NodeType::OpenJunction,
                    }
                },
            )
        })
        .collect()
}

#[derive(Clone, Copy)]
enum NodeType {
    KissingLoop(usize),
    Hairpin,
    Kink,
    OpenJunction,
}
