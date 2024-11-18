use nanorand::{Rng, WyRand};
use rustc_hash::{FxHashMap, FxHashSet};

use super::mesh::{Edge, Mesh};

pub fn dot_bracket_notation(mesh: &Mesh, path: &[(bool, usize)], motifs: &MotifStorage) -> String {
    let mut brackets: String = String::new();

    let mut visited_edges: FxHashSet<Edge> = FxHashSet::default();
    let mut bracket_counter: [usize; 2] = [0; 2];
    let mut node_visit_counts: Vec<u8> = mesh.vertices.iter().map(|_| 0).collect();

    path.iter().enumerate().for_each(|(index, &(is_hairpin, vertex))| {
        // Process edges
        if let Some(&(is_next_hairpin, next_vertex)) = path.get(index + 1) {
            // Process nodes
            {
                if is_next_hairpin {
                    // Get current node motif size
                    let num_bases: usize = motifs.get(
                        1
                    ).first().unwrap().len();
                    // Nodes always unpaired -> .
                    brackets.extend((0..num_bases).map(|_| "."));
                } else {
                    let node_visits: u8 = *node_visit_counts.get(vertex).unwrap();
                    *node_visit_counts.get_mut(vertex).unwrap() += 1;

                    // Get current node motif size
                    let num_bases: usize = motifs.get(
                        mesh.vertices.get(vertex).unwrap().edge_count
                    ).get(node_visits as usize).unwrap().len();
                    // Nodes always unpaired -> .
                    brackets.extend((0..num_bases).map(|_| "."));
                }
            }

            // Process edges
            {
                // Find edge
                let edge: Edge = *mesh.edges.iter().find(|&edge| 
                    ((edge.destination == next_vertex) && (edge.origin == vertex))
                ).unwrap();

                // Determine edge length
                let num_bases: usize = if is_hairpin || is_next_hairpin {
                    edge.length / 2
                } else {
                    edge.length
                };
                // If edge (or reverse) already visited: Closed parentheses -> paired to prior base
                if !visited_edges.contains(&edge.reverse()) {
                    // If opposite edge not already visited: Open brackets -> paired to future base
                    brackets.extend((0..num_bases).map(|_| {
                        *bracket_counter.first_mut().unwrap() += 1;
                        "("
                    }));
                    visited_edges.insert(edge);
                } else {
                    // If opposite edge not already visited: Closed brackets -> paired to past base
                    brackets.extend((0..num_bases).map(|_| {
                        *bracket_counter.last_mut().unwrap() += 1;
                        ")"
                    }));
                }
            }
        }
    });

    assert_eq!(bracket_counter.first().unwrap(), bracket_counter.last().unwrap());

    brackets
}

pub fn generate_sequence(mesh: &Mesh, path: &[(bool, usize)], motifs: &MotifStorage) -> String {
    let mut sequence: String = String::new();
    
    let mut visited_edges: FxHashMap<Edge, String> = FxHashMap::default();
    let mut node_visit_counts: Vec<u8> = mesh.vertices.iter().map(|_| 0).collect();
    let mut kissing_loop_visits: Vec<u8> = mesh.vertices.iter().map(|_| 0).collect();
    let mut edge_sequences: Vec<String> = Vec::with_capacity(path.len() - 1);
    let mut node_sequences: Vec<String> = Vec::with_capacity(path.len() - 1);
    let rng: &mut WyRand = &mut WyRand::new();

    path.iter().enumerate().for_each(|(index, &(is_hairpin, vertex))| {
        // Last node is initial node, so ignore
        if let Some(&(is_next_hairpin, next_vertex)) = path.get(index + 1) {
            // Process nodes
            {
                if is_next_hairpin {
                    // Get current node visit count, update for future reference
                    let kl_visits: u8 = *kissing_loop_visits.get(vertex).unwrap();

                    // Find hairpin motif
                    let motif: String = motifs.get(
                        1
                    ).first().unwrap().to_string();

                    // Pair if 2nd hald of KL
                    if kl_visits > 0 {
                        node_sequences.push(motif.chars().rev().map(|c| match c {
                            'A' => 'U',
                            'G' => 'C',
                            'C' => 'G',
                            'U' => 'A',
                            _ => 'O'
                        }).collect::<String>());
                    } else {
                        node_sequences.push(motif);
                    }

                    *kissing_loop_visits.get_mut(vertex).unwrap() += 1;
                    *kissing_loop_visits.get_mut(next_vertex).unwrap() += 1;
                } else {
                    // Get current node visit count, update for future reference
                    let node_visits: u8 = *node_visit_counts.get(vertex).unwrap();
                    *node_visit_counts.get_mut(vertex).unwrap() += 1;

                    // Get current node motif
                    let motif: &str = motifs.get(
                        mesh.vertices.get(vertex).unwrap().edge_count
                    ).get(node_visits as usize).unwrap();

                    // Add motif to sequence
                    node_sequences.push(motif.to_string());
                }
            }

            // Process edges
            {
                // Find edge
                let edge: Edge = *mesh.edges.iter().find(|&edge| 
                    ((edge.destination == next_vertex) && (edge.origin == vertex))
                ).unwrap();

                // Determine edge length
                let num_bases: usize = if is_hairpin || is_next_hairpin {
                    edge.length / 2
                } else {
                    edge.length
                };
                // If edge (or reverse) already visited: Closed parentheses -> paired to prior base
                if !visited_edges.contains_key(&edge.reverse()) {
                    // If opposite edge not already visited: Generate sequence
                    let seq: String = (0..num_bases).map(|_| match rng.generate_range(0..4u8){
                        0 => 'A',
                        1 => 'G',
                        2 => 'C',
                        3 => 'U',
                        _ => 'G'
                    }).collect();
                    visited_edges.insert(edge, seq.clone());
                    edge_sequences.push(seq);
                } else {
                    // If opposite edge not already visited: Reverse existing sequence
                    let seq: &str = visited_edges.get(&edge.reverse()).unwrap();
                    edge_sequences.push(seq.chars().rev().map(|c| match c {
                        'A' => 'U',
                        'G' => 'C',
                        'U' => 'A',
                        'C' => 'G',
                        _ => 'O'
                    }).collect());
                }
            }            
        }
    });

    log::debug!("Length difference{:}", path.len() - node_sequences.len());

    (0..path.len() - 1).for_each(|index| {
        sequence.push_str(node_sequences.get(index).unwrap());
        sequence.push_str(edge_sequences.get(index).unwrap());
    });

    sequence
}

pub struct MotifStorage {
    hairpin: String,
    kink: String,
    o3wj: Vec<String>,
    o4wj: Vec<String>,
}

impl MotifStorage {
    fn get(&self, num_edges: u8) -> Vec<&str> {
        match num_edges {
            1 => vec![&self.hairpin],
            2 => vec![&self.kink],
            3 => self.o3wj.iter().map(|s| s.as_str()).collect(),
            4 => self.o4wj.iter().map(|s| s.as_str()).collect(),
            _ => Vec::new()
        }
    }

    pub fn load_from_file(path: &str) -> Self {
        // Read file to string
        log::info!("Loading RNA motif data from {path}");
        let file_contents: String = std::io::read_to_string(
            std::fs::File::open(path).expect("Could not open motifs file"),
        )
        .expect("Could not read motifs file");

        // Load toml data
        log::debug!("Parsing mesh TOML data");
        let toml_data: toml::map::Map<String, toml::Value> = file_contents.parse::<toml::Table>().expect("Invalid format in 'motifs.toml'");

        // Load motifs
        let hairpin: String = toml_data.get("hairpin")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let kink: String = toml_data.get("kink")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let o3wj: Vec<String> = toml_data.get("three-way-junction")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                v.as_str().unwrap().to_string()
            })
            .collect();
        let o4wj: Vec<String> = toml_data.get("four-way-junction")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                v.as_str().unwrap().to_string()
            }).collect();

            Self { hairpin, kink, o3wj, o4wj }
    }
}