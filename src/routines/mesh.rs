use glam::{vec3a, Vec3A};
use rustc_hash::FxHashSet;

// Vertex
// Defines a point in space
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vertex {
    pub pos: Vec3A,
    pub edge_count: u8,
}
impl Vertex {
    pub fn new(pos: Vec3A) -> Self {
        Self {pos, edge_count: 0}
    }
}

// Edge
// Connects two vertices (origin, destination)
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Edge {
    pub origin: usize,
    pub destination: usize,
    pub length: usize,
}

impl Edge {
    pub fn new(origin: usize, destination: usize, length: usize) -> Self {
        Self { origin, destination, length }
    }

    pub fn reverse(&self) -> Self {
        Self::new(self.destination, self.origin, self.length)
    }
}

// Mesh
// Collection of vertices and edges representing a 3D wireframe structure
#[derive(Clone, Default, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub edges: FxHashSet<Edge>
}

impl Mesh {
    // Add a new vertex to the mesh
    #[allow(dead_code)]
    pub fn add_vertex(&mut self, pos: Vec3A) {
        self.vertices.push(Vertex::new(pos));
        log::trace!("Added a vertex to the mesh at {pos}.");
    }

    // Add multiple vertices to the mesh
    #[allow(dead_code)]
    pub fn add_vertices(&mut self, positions: &[Vec3A]) {
        self.vertices.extend(positions.iter().map(|&pos| Vertex::new(pos)));
        log::trace!("Added {:} vertices to the mesh.", positions.len());
    }

    // Add an edge between two vertices
    #[allow(dead_code)]
    pub fn add_edge(&mut self, origin: usize, destination: usize) -> Result<(), EdgeAdditionError> {
        log::trace!("Creating edge from {origin} to {destination}.");

        // Check whether origin and destination exist
        let origin_vertex: Option<&Vertex> = self.vertices.get(origin);
        let destination_vertex: Option<&Vertex> = self.vertices.get(destination);

        let added_result: Result<(), EdgeAdditionError> = match origin_vertex {
            None => Err(EdgeAdditionError::FalseOrigin),
            Some(origin_vertex) => match destination_vertex {
                None => Err(EdgeAdditionError::FalseDestination),
                Some(destination_vertex) => {
                    // Check whether they have space for more edges
                    if origin_vertex.edge_count == 4 {
                        Err(EdgeAdditionError::OriginFull)
                    } else if destination_vertex.edge_count == 4 {
                        Err(EdgeAdditionError::DestinationFull)
                    } else {
                        // Create the new edge
                        let length: usize = (origin_vertex.pos.distance(destination_vertex.pos)).round() as usize;
                        let edge: Edge = Edge::new(origin, destination, length);
                        self.edges.insert(edge);
                        self.edges.insert(edge.reverse());
                        Ok(())
                    }
                }
            }
        };

        // Update edge count for origin and destination
        if added_result.is_ok() {
            if let Ok([origin_vertex, destination_vertex]) = self.vertices.get_many_mut([origin, destination]) {
                origin_vertex.edge_count += 1;
                destination_vertex.edge_count += 1;
                log::trace!("Edge creation successful!");
            }
        };

        added_result
    }

    // Load mesh from TOML file 
    #[allow(dead_code)]
    pub fn load_from_file(path: &str) -> Self {
        // Read file to string
        log::info!("Loading mesh data from {path}");
        let file_contents: String = std::io::read_to_string(
            std::fs::File::open(path).expect("Could not open motifs file"),
        )
        .expect("Could not read mesh file");

        // Load toml data
        log::debug!("Parsing mesh TOML data");
        let toml_data: toml::map::Map<String, toml::Value> = file_contents.parse::<toml::Table>().expect("Invalid format in 'mesh.toml'");

        // Deserialise vertices
        log::debug!("Parsing vertex data");
        let vertices: Vec<Vertex> = toml_data.get("vertices")
            .expect("No 'vertices' found")
            .as_array()
            .expect("Vertices not in array format")
            .iter()
            .map(|val| {
                let vertex_as_value: &Vec<toml::Value> = val.as_array().expect("Invalid vertex format");
                Vertex::new(vec3a(
                    vertex_as_value.first().unwrap().as_float().unwrap() as f32, 
                    vertex_as_value.get(1).unwrap().as_float().unwrap() as f32,
                    vertex_as_value.last().unwrap().as_float().unwrap() as f32
                ))
            })
            .collect();
        
        // Deserialise edges
        log::debug!("Parsing edge data");
        let edges: Vec<[usize; 2]> = toml_data.get("edges")
        .expect("No 'edges' found")
        .as_array()
        .expect("Edges not in array format")
        .iter()
        .map(|val| {
            let edge_as_value: &Vec<toml::Value> = val.as_array().expect("Invalid edge format");
            [
                edge_as_value.first().unwrap().as_integer().unwrap() as usize, 
                edge_as_value.last().unwrap().as_integer().unwrap() as usize
            ]
        })
        .collect();

        // Create mesh
        log::debug!("Generating mesh");
        let mut mesh: Mesh = Mesh {vertices, ..Default::default()};
        edges.iter().for_each(|edge| {
            mesh.add_edge(edge[0], edge[1]).unwrap();
        });

        mesh
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum EdgeAdditionError {
    FalseOrigin,
    FalseDestination,
    OriginFull,
    DestinationFull,
}