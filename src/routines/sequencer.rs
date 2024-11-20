use nanorand::{Rng, WyRand};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{graph::AbstractEdge, mesh::{Edge, Mesh}};

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
                if is_hairpin {
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

pub fn generate_sequence(mesh: &Mesh, path: &[(bool, usize)], motifs: &MotifStorage) -> [String; 2] {
    // Get dot bracket notation
    let db_sequence: String = dot_bracket_notation(mesh, path, motifs);
    // Initialise nucleotide sequence
    let mut nt_sequence: String = db_sequence.clone();

    // Assign motif sequences to each node of the path
    let mut node_visit_counter: Vec<u8> = mesh.vertices.iter().map(|_| 0).collect();
    let motif_path: Vec<&str> = path.iter().enumerate().map(|(i, &(is_hairpin, index))| {
        if i + 1 < path.len() {
            if is_hairpin {
                motifs.hairpin.as_str()
            } else {
                log::debug!("Index: {index}");
                log::debug!("Visits: {:}", *node_visit_counter.get(index).unwrap());
                let num_edges: u8 = mesh.vertices.get(index).unwrap().edge_count;
                let motif: &str = *motifs.get(num_edges).get(
                    *node_visit_counter.get(index).unwrap() as usize
                ).unwrap();
                *node_visit_counter.get_mut(index).unwrap() += 1;
                motif
            }
        } else {
            ""
        }
    }).collect();

    // Find nodes in db sequence
    let mut motif_char_indices: Vec<[usize; 2]> = Vec::with_capacity(nt_sequence.len());
    let mut edge_char_indices: Vec<[usize; 2]> = Vec::with_capacity(mesh.edges.len());
    db_sequence.char_indices().for_each(|(i, current)| {
        // Find motif indices
        {
            // Update record if we start on a "."
            if (i == 0) && (current == '.') {
                motif_char_indices.push([i,0]);
            }

            // Update based on character change
            if let Some(next) = db_sequence.chars().nth(i+1) {
                // Start recording if entering unpaired region
                if next != current && next == '.' {
                    // Move to next motif
                    motif_char_indices.push([i+1, 0]);
                // Stop recording if exiting unpaired region
                } else if next != current && next != '.' {
                    *motif_char_indices.last_mut().unwrap().last_mut().unwrap() = i+1;
                }
            }
        }
        // Find edge indices
        {
            // Update record if we start on a "."
            if (i == 0) && (current == '(' || current == ')') {
                edge_char_indices.push([i,0]);
            }

            // Update based on character change
            if let Some(next) = db_sequence.chars().nth(i+1) {
                // Start recording if entering unpaired region
                if next != current && (next == '(' || next == ')') {
                    // Move to next motif
                    edge_char_indices.push([i+1, 0]);
                // Stop recording if exiting unpaired region
                } else if next != current && (next != '(' || next != ')') {
                    *edge_char_indices.last_mut().unwrap().last_mut().unwrap() = i+1;
                }
            }
        }

    });

    // Match edges
    let mut visited_edges: FxHashMap<AbstractEdge, usize> = FxHashMap::default();
    let edges: Vec<isize> = path.iter().enumerate().filter_map(|(index, &(_is_hairpin, vertex))| {
        if let Some(&(_is_next_hairpin, next_vertex)) = path.get(index + 1) {
            if let Some(&i) = visited_edges.get(&AbstractEdge::new(next_vertex, vertex)) {
                Some(-(i as isize))
            }
            else {
                visited_edges.insert(AbstractEdge::new(vertex, next_vertex), index + 1);
                Some(index as isize + 1)
            }
        } else {
            None
        }
    }).collect();
    log::debug!("Edges: {edges:?}");

    // Generate edge sequences
    let rng: &mut WyRand = &mut WyRand::new(); 
    let mut edge_sequences: Vec<String> = Vec::with_capacity(visited_edges.len());

    // Place sequences
    let mut last_end: usize = 0;
    let mut edge_index: usize = 0;
    motif_char_indices.iter().enumerate().for_each(|(i, indices)| {
        let start: usize = *indices.first().unwrap();
        let end: usize = *indices.last().unwrap();

        // Replace motifs
        nt_sequence.replace_range(
            start..end, 
            motif_path.get(i).unwrap()
        );

        // Ensure GC base-pairs entering motif
        if start > 0 {
            nt_sequence.replace_range(start-1..start, "G");
        }
        if end < nt_sequence.len() {
            nt_sequence.replace_range(end..end+1, "C");
        }

        // Deal with edge
        if start - last_end > 0 {
            let edge_range: std::ops::Range<usize> = (last_end+1)..(start-1);
            let edge_id: isize = *edges.get(edge_index).unwrap();
            if edge_id.is_positive() {
                let sequence: String = generate_random_sequence(rng, edge_range.clone());
                nt_sequence.replace_range(edge_range, &sequence);
                edge_sequences.push(sequence)
            } else {
                if let Some(seq) = edge_sequences.get(edge_id.abs() as usize - 1) {
                    let sequence: String = complement_sequence(seq);
                    nt_sequence.replace_range(edge_range, &sequence);
                    edge_sequences.push(sequence);
                }
            }
            edge_index += 1;
        }

        if i + 1 == motif_char_indices.len() {
            let edge_range: std::ops::Range<usize> = (end+1)..(nt_sequence.len());
            let edge_id: isize = *edges.get(edge_index).unwrap();
            if edge_id.is_positive() {
                let sequence: String = generate_random_sequence(rng, edge_range.clone());
                nt_sequence.replace_range(edge_range, &sequence);
                edge_sequences.push(sequence)
            } else {
                if let Some(seq) = edge_sequences.get(edge_id.abs() as usize - 1) {
                    let sequence: String = complement_sequence(seq);
                    nt_sequence.replace_range(edge_range, &sequence);
                    edge_sequences.push(sequence);
                }
            }
            edge_index += 1;
        }

        last_end = end;
    });

    nt_sequence.push('G');

    [db_sequence, nt_sequence]
}

fn generate_random_sequence(rng: &mut WyRand, range: std::ops::Range<usize>) -> String {
    range.map(|_| match rng.generate_range(0..7u8) {
        0 => 'A',
        1 | 2 => 'U',
        3   => 'C',
        _ => 'G'
    }).collect()
}

fn complement_sequence(sequence: &str) -> String {
    sequence.chars().rev().map(|c| match c {
        'A' => 'U',
        'G' => 'C',
        'U' => 'A',
        _ => 'G'
    }).collect()
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