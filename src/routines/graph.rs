use std::fmt::Display;

use glam::{Vec2, Vec3Swizzles};
use rustc_hash::FxHashSet;

use super::mesh::EdgeAdditionError;

// Graph edge
// Abstract reprentation of mesh edge
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct AbstractEdge {
    origin: usize,
    destination: usize
}

impl AbstractEdge {
    pub fn new(origin: usize, destination: usize) -> Self {
        Self { origin, destination }
    }

    pub fn reverse(&self) -> Self {
        Self::new(self.destination, self.origin)
    }
}

// Graph node
// Abstract representation of mesh vertex
#[derive(Clone, Default, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Node {
    pub edges: Vec<AbstractEdge>,
}

#[derive(Clone, Default, Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: FxHashSet<AbstractEdge>
}

impl Graph {
    // Add a new node to the graph
    #[allow(dead_code)]
    pub fn add_node(&mut self) {
        self.nodes.push(Node::default());
        log::trace!("Added a node to the graph.");
    }

    // Add multiple new nodes to the graph
    #[allow(dead_code)]
    pub fn add_nodes(&mut self, count: usize) {
        self.nodes.extend((0..count).map(|_| Node::default()));
        log::trace!("Added {count} nodes to the graph.");
    }

    // Add a new edge to the graph
    #[allow(dead_code)]
    pub fn add_edge(&mut self, origin: usize, destination: usize) -> Result<(), EdgeAdditionError> {
        log::trace!("Creating edge from {origin} to {destination}.");

        // Create possible edge
        let edge: AbstractEdge = AbstractEdge::new(origin, destination);

        // Check whether origin and destination exist
        let origin_vertex: Option<&Node> = self.nodes.get(origin);
        let destination_vertex: Option<&Node> = self.nodes.get(destination);

        let added_result: Result<(), EdgeAdditionError> = match origin_vertex {
            None => Err(EdgeAdditionError::FalseOrigin),
            Some(origin_vertex) => match destination_vertex {
                None => Err(EdgeAdditionError::FalseDestination),
                Some(destination_vertex) => {
                    // Check whether they have space for more edges
                    if origin_vertex.edges.len() == 4 {
                        Err(EdgeAdditionError::OriginFull)
                    } else if destination_vertex.edges.len() == 4 {
                        Err(EdgeAdditionError::DestinationFull)
                    } else {
                        // Create the new edge
                        self.edges.insert(edge);
                        self.edges.insert(edge.reverse());
                        Ok(())
                    }
                }
            }
        };

        // Update edge count for origin and destination
        if added_result.is_ok() {
            if let Ok([origin_vertex, destination_vertex]) = self.nodes.get_many_mut([origin, destination]) {
                origin_vertex.edges.push(edge);
                destination_vertex.edges.push(edge.reverse());
                log::trace!("Edge creation successful!");
            }
        };

        added_result
    }
}

// Create graph from mesh
impl From<super::mesh::Mesh> for Graph {
    fn from(mesh: super::mesh::Mesh) -> Self {
        log::debug!("Generating graph from mesh");

        // Initialise graph
        let mut graph: Graph = Graph::default();

        // Load nodes and edges from mesh
        graph.add_nodes(mesh.vertices.len());
        mesh.edges.iter().for_each(|edge| {
            if !graph.edges.contains(&AbstractEdge::new(edge.destination, edge.origin)) {
                graph.add_edge(edge.origin, edge.destination).unwrap(); 
            }
        });
    
        graph
    }
}

// Tree
// Tree representation of a graph
#[derive(Clone, Debug)]
pub struct Tree {
    pub root: usize,
    pub children: Vec<(bool, Tree)>
}

impl Tree {
    // Create a new empty tree from a root
    pub fn new(root: usize) -> Self {
        Self { root, children: Vec::new() }
    }

    // Find children of root in a graph, excluding parents
    // Returns a bool representing whether there are more children
    pub fn find_children(&mut self, 
        parent: usize, 
        nodes: &[Node], 
        visited_nodes: &mut FxHashSet<usize>
    ) -> bool {
        // Get root node
        let root: &Node = nodes.get(self.root).unwrap();

        // Check whether it has children
        if root.edges.is_empty() {
            false
        } else {
            // Find number of children
            let num_children: usize = root.edges.len();

            // Check for grandchildren
            #[allow(clippy::unnecessary_fold)]
            (0..num_children).map(|child_index| {
                // Get child
                let child: usize = root.edges.get(child_index).unwrap().destination;

                // Check whether node has already been visited and update children
                if visited_nodes.insert(child) {
                    // Node has not been visited
                    // Update children
                    self.children.push((false, Tree::new(child)));

                    // There might be grandchildren
                    true                    
                } else if child == parent {
                    // Node is parent, there are no grandchildren
                    false
                } else {
                    // Node has been visited
                    // Update children
                    self.children.push((true, Tree::new(child)));

                    // There are no grandchildren (it would form a cycle)
                    false
                }
            }).fold(false, |verdict, can_continue| verdict || can_continue)
            // Fold used here instead of .any(), as the latter is short-circuiting
            // thus preventing correct execution
        }
    }

    // Find progeny of root
    fn find_progeny(&mut self,
        nodes: &[Node], 
        visited_nodes: &mut FxHashSet<usize>,
        parent: usize,
    ) -> bool {
        // Find number of children
        let num_children: usize = self.children.len();

        // If there are no children, find children
        if num_children == 0 {
            self.find_children(parent, nodes, visited_nodes)
        } else {
            // Find grandchildren
            (0..num_children).any(|child_index| {
                // Get child
                let (is_cycle_breaker, child) = self.children.get_mut(child_index).unwrap();

                // Search for grandchildren
                if *is_cycle_breaker {
                    // If it is a cycle breaking node, stop searching
                    false
                } else {
                    // Otherwise find descendants until there are none left
                    child.find_progeny(nodes, visited_nodes, self.root)
                }
            })
        }
    }
    
    // Create tree from a graph and specified root
    pub fn from_graph(graph: &Graph, root: usize) -> Self {
        // Initialise tree
        let mut tree: Self = Self::new(root);

        // Find all children
        let mut visited_nodes: FxHashSet<usize> = FxHashSet::default();
        visited_nodes.insert(root);
        let mut has_more_descendants: bool = true;
        while has_more_descendants {
            has_more_descendants = tree.find_progeny(&graph.nodes, &mut visited_nodes, root);
        }

        tree
    }

    // Create a path crossing all non-cyclical edges exactly twice
    // Identify cycle breaking nodes
    pub fn find_path(&mut self, 
        mesh: &super::mesh::Mesh, 
        path: &mut Vec<(bool, usize)>, 
        parent: usize
    ) {
        // Add current node to path
        path.push((false, self.root));

        // Sort children to determine what order to visit them in
        self.children.sort_by(|(_1, child1), (_2, child2)| {
            // Get node positions (turned to 2d for simplicity)
            let child1_pos: Vec2 = mesh.vertices.get(child1.root).unwrap().pos.xy();
            let child2_pos: Vec2 = mesh.vertices.get(child2.root).unwrap().pos.xy();
            let root_pos: Vec2 = mesh.vertices.get(self.root).unwrap().pos.xy();
            let parent_pos: Vec2 = mesh.vertices.get(parent).unwrap().pos.xy();

            // Create a reference vector for rotation
            let mut ref_vec: Vec2 = match self.root == parent {
                false => -(root_pos - parent_pos).normalize_or_zero(),
                true => -0.5 * (child1_pos.normalize() + child2_pos.normalize())
            };
            if ref_vec.length_squared() == 0.0 {
                ref_vec = Vec2::NEG_ONE;
            }

            // Calculate angles
            let angle1: f32 = ref_vec.angle_to(child1_pos);
            let angle2: f32 = ref_vec.angle_to(child2_pos);

            // Compare
            angle1.partial_cmp(&angle2).unwrap()

        });

        // Update path for each child
        self.children.iter_mut().for_each(|(is_cycle_breaker, child)| {
            if *is_cycle_breaker {
                // Add cycle-breaking target and return to root
                path.extend_from_slice(&[(true, child.root), (false, self.root)]);
            } else {
                // Find path of child, then return
                child.find_path(mesh, path, self.root);
                path.push((false, self.root));
            }
        });
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut whitespace: String = String::new();
        fn write_layer(
            tree: &Tree,
            f: &mut std::fmt::Formatter<'_>,
            layer: usize,
            whitespace: &mut String,
        ) {
            for _ in 0..layer {
                whitespace.extend_one(" ");
            }
            writeln!(f, "{}Node: {}", whitespace, tree.root).unwrap();
            for maybe_child in tree.children.iter() {
                let local_whitespace: &mut String = &mut whitespace.clone();
                if maybe_child.0 {
                    writeln!(f, "{}  Node: Hairpin (KL w/ {:})", whitespace, maybe_child.1.root).unwrap();
                } else {
                    write_layer(&maybe_child.1, f, layer + 1, local_whitespace);
                }
            }
        }

        write_layer(self, f, 1, &mut whitespace);
        write!(f, "")
    }
}