/* === NOTES ===
As this is a prototype and proof of concept, I have use [.unwrap()] on a few occasions.
These are used when there should be no cases that would cause the unwrap call to panic.
Once the app is completed, I will implement proper error handling for the unwrap cases.
However, if an unwrap does cause the system to panic, then something is broken somewhere.
*/

use std::collections::{VecDeque, hash_map::Entry};

use glam::{Quat, Vec2, Vec3A, Vec3Swizzles, vec3a};
use nanorand::{Rng, WyRand};
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug)]
pub struct Tree {
    pub edges: FxHashSet<(usize, usize)>, // (origin, destination)
    pub nodes: FxHashMap<usize, (usize, Vec<usize>)>, // node_id, (parent, children)
    pub cycle_breakers: FxHashMap<usize, usize>, // node_id, node_id
}

pub fn construct_tree(edges: &[(usize, usize)]) -> Tree {
    log::debug!("Edges: {edges:?}");
    // Initialise tree components
    let mut tree_edges: FxHashSet<(usize, usize)> = FxHashSet::default();
    let mut tree_nodes: FxHashMap<usize, (usize, Vec<usize>)> = FxHashMap::default();
    let mut cycle_breakers: FxHashMap<usize, usize> = FxHashMap::default();

    // Find largest node index in registered edges
    // To break cycles, create new nodes with greater index than current max
    let max_id_pair: &(usize, usize) = edges
        .iter()
        .max_by(|a, b| a.0.max(a.1).cmp(&b.0.max(b.1)))
        .unwrap();
    let mut max_id: usize = max_id_pair.0.max(max_id_pair.1);

    // Iterate over input edges, adding them to the tree edges
    // Add nodes involved to tree nodes
    // Find and break cycles
    edges.iter().for_each(|&(origin, destination)| {
        log::debug!("Origin: {origin} | Destination: {destination}");
        // Check if destination is already in the tree
        if tree_nodes.contains_key(&destination) && tree_nodes.contains_key(&origin) {
            // If the destination already exists, a cycle is formed
            // break the cycle by creating intermediary nodes and edges
            let new_node_a: usize = max_id + 1;
            let new_node_b: usize = max_id + 2;

            // Create edge from origin to new node A
            tree_edges.insert((origin, new_node_a));
            // Create corresponding node entry (no children)
            tree_nodes.insert(new_node_a, (origin, Vec::new()));

            // Repeat for the destination and new node B
            tree_edges.insert((destination, new_node_b));
            tree_nodes.insert(new_node_b, (destination, Vec::new()));

            // Add new nodes A and B to the cycle breaker nodes
            cycle_breakers.insert(new_node_a, new_node_b);

            // Update the max node ID
            max_id += 2;

            // Update the origin and destination nodes
            tree_nodes.get_mut(&origin).unwrap().1.push(new_node_a);
            tree_nodes.get_mut(&destination).unwrap().1.push(new_node_b);
        } else {
            // If the destination does not already exist
            // Create a new edge
            tree_edges.insert((origin, destination));

            // Add relevant nodes
            // If the origin entry does not exist, add it
            if let Entry::Vacant(entry) = tree_nodes.entry(origin) {
                // This should only occur when inserting the first node
                // Therefore it hass no parent, max usize value used to describe this
                entry.insert((usize::MAX, vec![destination]));
            } else {
                // If the origin exists, update its children
                tree_nodes.get_mut(&origin).unwrap().1.push(destination);
            }
            tree_nodes.insert(destination, (origin, Vec::new()));
        }
    });

    // Construct tree
    Tree {
        edges: tree_edges,
        nodes: tree_nodes,
        cycle_breakers,
    }
}

// Sort the children for each node based on the parent
fn sort_tree_edges(tree: &mut Tree, node_coordinates: &[Vec3A]) {
    let rng: &mut WyRand = &mut WyRand::new();
    // Get "effective nodes" for directions, replacing cycle-breaker nodes with the intended nodes
    let node_count: usize = node_coordinates.len() - 1; // -1 as Rust in 0-indexed and this is to find cycle-breakers
    // We don't check cycle-breaker nodes, as they have no children and can be removed from the search.
    // Likewise, we can remove nodes with only one child, as there is nothing to sort against
    let node_children_order = tree
        .nodes
        .clone()
        .iter_mut()
        .filter(|(node, (_, children))| (**node <= node_count) || (children.len() > 1))
        .map(|(node, (parent, children))| {
            children.iter_mut().for_each(|child| {
                if *child > node_count {
                    // If the condition is fulfilled, the child is a cycle-breaker
                    // Therefore, it has no assigned position that can be used for sorting
                    // To find the intended target on the cyclical graph
                    // just need to find the parent of the cycle-breaker paired to the current child
                    *child = tree
                        .nodes
                        .get(match tree.cycle_breakers.get(child) {
                            // Find corresponding cycle-breaker node
                            // If current node is the key, other is the value
                            Some(cycle_breaker) => cycle_breaker,
                            // If the curret node is the value
                            // Find the pair with this node as the value and extract the key
                            None => {
                                tree.cycle_breakers
                                    .iter()
                                    .find(|(_key, val)| **val == *child)
                                    .unwrap()
                                    .0
                            }
                        })
                        .unwrap()
                        .0;
                }
            });
            // Once the children have been corrected, find the respective positions
            let node_pos: &Vec3A = node_coordinates.get(*node).unwrap();
            // In the case of the starter node, it has no parent. Thus a new vector is generated, otherwise position is fetched
            let parent_pos: &Vec3A = if *parent == usize::MAX {
                &(node_pos + Vec3A::ONE)
            } else {
                node_coordinates.get(*parent).unwrap()
            };
            let children_pos: Vec<&Vec3A> = children
                .iter()
                .map(|child| node_coordinates.get(*child).unwrap())
                .collect();

            // Sort children
            let sorted_indices: Vec<usize> =
                sort_relative_positions(node_pos, parent_pos, &children_pos, rng);

            // Apply sorting
            (*node, (*parent, sorted_indices))
        })
        .collect::<FxHashMap<usize, (usize, Vec<usize>)>>();

    // Apply sorting to the tree
    tree.nodes
        .iter_mut()
        .filter(|(node, (_, children))| (**node <= node_count) || (children.len() > 1))
        .for_each(|(node, (_parent, children))| {
            let (_, indices): &(usize, Vec<usize>) = node_children_order.get(node).unwrap();
            let ordered_children: Vec<usize> = indices
                .iter()
                .map(|&item_index| *children.get(item_index).unwrap())
                .collect();
            *children = ordered_children;
        });
}

fn sort_relative_positions(
    current: &Vec3A,
    parent: &Vec3A,
    to_sort: &[&Vec3A],
    rng: &mut WyRand,
) -> Vec<usize> {
    // Create a reference vector for the sorting
    // By default it is the vector from the parent to the current node.
    let mut ref_vector: Vec3A = (current - parent).normalize_or_zero();

    // Define the displacement vectors from the current node to the child node
    let child_displacements: Vec<Vec3A> = to_sort.iter().map(|&child| child - current).collect();

    // This reference vector is unsuitable if any of the vectors that need sorting are perpendicular to it
    // So we check for perpendicularity.
    // While loop, as the new vector is randomly generated, which does not guarantee it will be suitable
    while child_displacements
        .iter()
        .any(|&vec_to_child| ref_vector.dot(vec_to_child) == 0.0)
    {
        // In the case where one of the vectors is perpendicular, select a random vector
        ref_vector = vec3a(rng.generate(), rng.generate(), rng.generate()).normalize_or_zero();
    }

    // Find rotation transforming the reference vector to the Y vector
    // Use this to project our vectors onto the XY plane
    // This just simplifies the ordering, as a float angle between 3D vectors is not easy to work with
    let rotation: Quat = Quat::from_rotation_arc(ref_vector.into(), glam::Vec3::Y);

    // Sort the positions relative to the input and reference vector
    let mut indices: Vec<usize> = (0..child_displacements.len()).collect();
    radsort::sort_by_key(&mut indices, |&i| {
        let child_vec: &Vec3A = child_displacements.get(i).unwrap();

        // Rotate displacements to XY plane, and only select X and Y components
        let proj_a: Vec2 = (rotation * *child_vec).xy();

        // Calculate angles between reference vector and relevant projection
        let mut angle_a: f32 = Vec2::Y.angle_to(proj_a);

        // Make sure angle signs are consistent (sometimes they're not)
        // Do this by assigning negative on left and positive on right
        // This simulates clockwise angle, in a way
        if proj_a.x.is_sign_negative() {
            angle_a = -angle_a.abs();
        } else if proj_a.x.is_sign_positive() {
            angle_a = angle_a.abs();
        }

        // Compare
        angle_a
    });

    indices
}

pub fn find_rna_path(tree: &mut Tree, node_coordinates: &[Vec3A]) -> Vec<usize> {
    // Sort the tree to avoid overlapping and crossing segments
    sort_tree_edges(tree, node_coordinates);

    // Clone the tree nodes (we will remove completed interaction through mutation)
    let mut nodes: FxHashMap<usize, (usize, VecDeque<usize>)> = tree
        .nodes
        .iter()
        .map(|(index, (parent, children))| (*index, (*parent, VecDeque::from(children.clone()))))
        .collect();

    // Start at the node with the most children
    let mut current_node_index: usize = *nodes
        .iter()
        .max_by(|(_, (_, a)), (_, (_, b))| a.len().cmp(&b.len()))
        .unwrap()
        .0;

    // Go down the edge to the first child recursively to form the path
    // Until the path length is met
    let target_length: usize = tree.edges.len() * 2 + 1; // 2 nodes per edge, edge travelled twice
    let mut current_length: usize = 0;
    let mut path: Vec<usize> = Vec::with_capacity(target_length);
    while current_length < target_length {
        // Add current node
        path.push(current_node_index);
        current_length += 1;

        // Get current node children
        let (parent, children): &mut (usize, VecDeque<usize>) =
            nodes.get_mut(&current_node_index).unwrap();

        // Go to the next node
        // If the node has no children, go back to the parent node
        if children.is_empty() {
            // If node is root node and has no parent, stop pathfinding
            if *parent == usize::MAX {
                if current_length < target_length {
                    log::warn!(
                        "Break clause activated at length {current_length} / {target_length}"
                    );
                }
                //break;
            } else {
                let local_node_index: usize = current_node_index;
                current_node_index = *parent;
                nodes.remove(&local_node_index);
            }
        } else {
            // Otherwise, the next node is the first child
            current_node_index = children.pop_front().unwrap();
        }
    }

    path
}
