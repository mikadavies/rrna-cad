#![allow(dead_code)]

use glam::Vec3;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Graph {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>
}

impl Graph {
    pub fn add_vertex(&mut self, position: Vec3, class: VertexType) {
        log::trace!("Adding vertex (type: {class:?}) at position: {position:?}.");
        self.vertices.push(Vertex { position, class, connections: 0 , edges: [None; 4]});
    }

    pub fn add_edge(&mut self, origin: usize, destination: usize) {
        log::trace!("Adding edge from {origin} to {destination}.");
        let vorigin: Vertex = *self.vertices.get_mut(origin).unwrap();
        let vdestination: Vertex = *self.vertices.get_mut(destination).unwrap();

        let origin_connections: u8 = self.vertices.get(origin).unwrap().connections;
        let destination_connections: u8 = self.vertices.get(destination).unwrap().connections;

        let can_origin_add: bool = match vorigin.class {
            VertexType::Unknown => origin_connections < 4,
            VertexType::Start => origin_connections < 1,
            VertexType::End => origin_connections < 1,
            VertexType::LoopA => origin_connections < 1,
            VertexType::LoopB => origin_connections < 1,
            VertexType::LoopC => origin_connections < 1,
            VertexType::LoopD => origin_connections < 1,
            VertexType::A => origin_connections < 2,
            VertexType::G => origin_connections < 2,
            VertexType::C => origin_connections < 2,
            VertexType::U => origin_connections < 2,
        };

        if !can_origin_add {
            log::warn!("Could not add edge from {origin} to {destination}. Reason: origin has no free connections.");
        } else {
            let can_destination_add: bool = match vdestination.class {
                VertexType::Unknown => destination_connections < 4,
                VertexType::Start => destination_connections < 1,
                VertexType::End => destination_connections < 1,
                VertexType::LoopA => destination_connections < 1,
                VertexType::LoopB => destination_connections < 1,
                VertexType::LoopC => destination_connections < 1,
                VertexType::LoopD => destination_connections < 1,
                VertexType::A => destination_connections < 2,
                VertexType::G => destination_connections < 2,
                VertexType::C => destination_connections < 2,
                VertexType::U => destination_connections < 2,
            };
            if !can_destination_add {
                log::warn!("Could not add edge from {origin} to {destination}. Reason: origin has no free connections.");
            } else {
                self.add_edge_unchecked(origin, destination);
            }
        }



    }

    fn add_edge_unchecked(&mut self, origin: usize, destination: usize) {
        {
            // Add edge
            let vorigin: Vertex = *self.vertices.get(origin).unwrap();
            let vdestination: Vertex = *self.vertices.get(destination).unwrap();
            self.edges.push(Edge{origin, destination, length: vorigin.position.distance(vdestination.position)});
        }{
            // Update origin
            let vorigin: &mut Vertex = self.vertices.get_mut(origin).unwrap();
            vorigin.connections += 1;
            let local_edge_index: usize = vorigin.edges.iter().position(|&x| x.is_none()).unwrap();
            *vorigin.edges.get_mut(local_edge_index).unwrap() = Some(self.edges.len() - 1);
        }{
            // Update Destination
            let vdestination: &mut Vertex = self.vertices.get_mut(destination).unwrap();
            vdestination.connections += 1;
            let local_edge_index: usize = vdestination.edges.iter().position(|&x| x.is_none()).unwrap();
            *vdestination.edges.get_mut(local_edge_index).unwrap() = Some(self.edges.len() - 1);
        }
    }

    pub fn to_ron(&self, path: &str) {
        log::info!("Writing graph to {path}");
        ron::ser::to_writer_pretty(
            std::fs::File::create(path).expect("Could not create graph write file"), 
            self, 
            PrettyConfig::default()
        ).expect("Could not write graph to RON");
    }

    pub fn from_ron(path: &str) -> Option<Self> {
        log::info!("Reading graph from {path}");
        let reader: Option<std::fs::File> = match std::fs::File::open(path) {
            Ok(file) => Some(file),
            Err(e) => {
                log::error!("Could not open graph file. {e}");
                None
            }
        };
        if reader.is_some() {
            match ron::de::from_reader(reader.unwrap()) {
                Ok(graph) => Some(graph),
                _ => {
                    log::error!("Could not read graph RON. Invalid file format?");
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn iter_vertices(&self) -> impl Iterator<Item = Vertex> {
        self.vertices.clone().into_iter()
    }

    pub fn iter_edges(&self) -> impl Iterator<Item = Edge> {
        self.edges.clone().into_iter()
    }

}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub class: VertexType,
    pub connections: u8,
    pub edges: [Option<usize>; 4]
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub origin: usize,
    pub destination: usize,
    pub length: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum VertexType {
    Unknown,
    Start,
    End,
    LoopA,
    LoopB,
    LoopC,
    LoopD,
    A,
    G,
    C,
    U,
}