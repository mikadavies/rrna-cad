use crate::{graph::Graph, motifs, pathfinder};


impl Graph {
    // TODO: Differentiate between KL and Kink90 for unknown types
    pub fn to_motifs(&self) -> impl Iterator<Item = Motif> {
        log::trace!("Finding motifs");
        self.iter_vertices().map(|vertex| match vertex.class {
            crate::graph::VertexType::Start => Motif::Hairpin(HairpinID::A),
            crate::graph::VertexType::End => Motif::Hairpin(HairpinID::A),
            crate::graph::VertexType::LoopA => Motif::KL(HairpinID::A),
            crate::graph::VertexType::LoopB => Motif::KL(HairpinID::B),
            crate::graph::VertexType::LoopC => Motif::KL(HairpinID::C),
            crate::graph::VertexType::LoopD => Motif::KL(HairpinID::D),
            crate::graph::VertexType::A => Motif::A,
            crate::graph::VertexType::G => Motif::G,
            crate::graph::VertexType::C => Motif::C,
            crate::graph::VertexType::U => Motif::U,
            crate::graph::VertexType::Unknown => match vertex.connections {
                1 => Motif::Hairpin(HairpinID::D),
                2 => Motif::Kink90,
                3 => Motif::O3WJ,
                4 => Motif::O4WJ,
                _ => Motif::KL(HairpinID::B)
            },
        })
    }

    pub fn find_path(&self) -> Vec<usize> {
        match pathfinder::find_single_path(self.iter_edges(), self.iter_vertices()) {
            Some(single_path) => {
                single_path
            },
            None => {
                let path: Vec<usize> = Vec::new();
                path
            }
        }        
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motif {
    Hairpin(HairpinID),
    Kink90,
    KL(HairpinID),
    O3WJ,
    O4WJ,
    A,
    G,
    C,
    U
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HairpinID {
    A,
    B,
    C,
    D
}

#[allow(dead_code)]
fn get_motif_sequence(motif: Motif) -> Vec<&'static str> {
    log::trace!("Getting motif sequence for motif: {motif:?}");
    // Matches a sequence to the motif
    match motif {
        // 90 degree kink returns [long edge, short edge]
        Motif::Kink90 => vec![motifs::K90D],
        // 3 Way Junction returns (bottom left, top, bottom right)
        Motif::O3WJ => Vec::from(motifs::O3WJ),
        // 4 Way Junction returns (bottom left, top left, top right, bottom right)
        Motif::O4WJ => Vec::from(motifs::O4WJ),
        // Hairpin returns a single sequence
        Motif::Hairpin(id) => match id {
            HairpinID::A => vec![motifs::KL_HAIRPINS[0][0]],
            HairpinID::B => vec![motifs::KL_HAIRPINS[1][0]],
            HairpinID::C => vec![motifs::KL_HAIRPINS[2][0]],
            HairpinID::D => vec![motifs::KL_HAIRPINS[3][0]]
        },
        // Kissing loop returns a pair of matched sequences
        Motif::KL(id) => match id {
            HairpinID::A => Vec::from(motifs::KL_HAIRPINS[0]),
            HairpinID::B => Vec::from(motifs::KL_HAIRPINS[1]),
            HairpinID::C => Vec::from(motifs::KL_HAIRPINS[2]),
            HairpinID::D => Vec::from(motifs::KL_HAIRPINS[3]),
        },
        Motif::A => vec!["A"],
        Motif::G => vec!["G"],
        Motif::C => vec!["C"],
        Motif::U => vec!["U"],
        
    }
}

#[allow(dead_code)]
fn find_complementary_sequence(sequence: &str) -> String {
    log::trace!("Calculating complementary sequence");
    sequence.chars().into_iter().map(|base| {
        if base == 'A' {
            'U'
        } else if base == 'U' {
            'A'
        } else if base == 'G' {
            'C'
        } else if base == 'C' {
            'G'
        } else {
            base
        }
    }).collect()
}