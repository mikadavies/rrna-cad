use glam::Vec3A;
use rustc_hash::FxHashSet;
use three_d::{
    Camera, ClearState, Context, CpuMaterial, CpuMesh, DirectionalLight, FrameOutput, GUI, Gm,
    InnerSpace, Mat4, Mesh, OrbitControl, PhysicalMaterial, Quat, Srgba, Window, WindowSettings,
    degrees,
    egui::{Response, SidePanel, TopBottomPanel},
    vec3,
};

use super::{
    graph::{Tree, construct_tree, find_rna_path},
    sequencer::generate_sequence,
};

// TODO
pub fn run_gui() {
    let window: Window = Window::new(WindowSettings {
        title: "RRNA CAD".to_string(),
        initial_size: Some((800, 600)),
        ..Default::default()
    })
    .unwrap();

    let context: Context = window.gl();

    // RNA stuff
    let mut sequence: String = "No sequence generated yet...".to_string();
    let mut node_coordinates: Vec<Vec3A> = Vec::new();
    let mut edges: FxHashSet<(usize, usize)> = FxHashSet::default();

    // 3D rendering stuff
    let mut camera: Camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 0.0, -10.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(60.0),
        0.1,
        1000.0,
    );

    let mut orbit_control: OrbitControl = OrbitControl::new(camera.target(), 1.0, 100.0);

    let mut rendered_nodes: Vec<Gm<Mesh, PhysicalMaterial>> = Vec::new();
    let mut rendered_edges: Vec<Gm<Mesh, PhysicalMaterial>> = Vec::new();

    let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(-20.0, -20.5, -20.5));
    let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(20.0, 20.5, 20.5));

    // UI-specific editables
    let mut node_coordinates_text: String = "[-10.0, 0.0, 0.0]\n[10.0, 0.0, 0.0]\n".to_string();
    let mut edges_text: String = "(0, 1)".to_string();

    let mut gui: GUI = GUI::new(&context);
    window.render_loop(move |mut frame_input| {
        let mut panel_width: f32 = 0.0;
        // Describe GUI
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                SidePanel::left("side_panel").show(gui_context, |ui| {
                    ui.heading("Structure Vertices");
                    ui.text_edit_multiline(&mut node_coordinates_text);

                    ui.heading("Structure Edges");
                    ui.text_edit_multiline(&mut edges_text);

                    let btn_genseq: Response = ui.button("Generate sequence");
                    let btn_vis: Response = ui.button("Visualise");
                    if btn_vis.clicked() {
                        update_node_coordinates(
                            &mut node_coordinates,
                            &node_coordinates_text,
                            &mut sequence,
                        );
                        if try_update_edges(&mut edges, &edges_text, &node_coordinates).is_err() {
                            sequence = "Error: Invalid edge data".to_string();
                        }
                        update_rendered_object(
                            &mut rendered_nodes,
                            &node_coordinates,
                            &mut rendered_edges,
                            &edges,
                            &context,
                        );
                    }

                    if btn_genseq.clicked() && !edges_text.is_empty() {
                        update_node_coordinates(
                            &mut node_coordinates,
                            &node_coordinates_text,
                            &mut sequence,
                        );
                        if try_update_edges(&mut edges, &edges_text, &node_coordinates).is_err() {
                            sequence = "Error: Invalid edge data".to_string();
                        } else {
                            log::debug!("Nodes: {node_coordinates:?}");
                            log::debug!("Edges: {edges:?}");

                            let mut tree: Tree = construct_tree(
                                &edges.iter().copied().collect::<Vec<(usize, usize)>>(),
                            );
                            let path: Vec<usize> = find_rna_path(&mut tree, &node_coordinates);
                            log::debug!("Path: {path:?}");
                            log::debug!("Last coords: {node_coordinates:?}");
                            log::debug!("Tree: {tree:?}");
                            sequence = generate_sequence(&path, &tree, &node_coordinates);
                        }
                    } else if btn_genseq.clicked() && edges.is_empty() {
                        sequence = "Error: Invalid shape".to_string();
                    }
                    panel_width = gui_context.used_rect().width();
                });
                TopBottomPanel::bottom("bottom_panel").show(gui_context, |ui| {
                    ui.heading("Generated RNA Sequence");
                    ui.code_editor(&mut sequence.as_str());
                    if ui.button("Copy to clipboard").clicked() {
                        ui.output_mut(|o| o.copied_text = sequence.clone());
                    }
                });
            },
        );

        // Draw 3D scene
        camera.set_viewport(frame_input.viewport);
        orbit_control.handle_events(&mut camera, &mut frame_input.events);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.4, 0.4, 0.4, 1.0, 1.0))
            .render(
                &camera,
                rendered_nodes.iter().chain(rendered_edges.iter()),
                &[&light0, &light1],
            );

        // Render GUI to screen
        frame_input.screen().write(|| gui.render()).unwrap();

        FrameOutput::default()
    });
}

fn update_node_coordinates(coordinates: &mut Vec<Vec3A>, input: &str, sequence: &mut String) {
    // Parse the input coordinate string to Vec3A coordinate values

    let mut new_coords: Vec<Vec3A> = Vec::with_capacity(coordinates.len());

    input
        .chars()
        .filter(|c| !(c.is_whitespace() || *c == '['))
        .collect::<String>()
        .split(']')
        .for_each(|coords| {
            if !coords.is_empty() {
                let num_coords = coords
                    .split(',')
                    .map(|val| match val.parse::<f32>() {
                        Ok(coord) => coord,
                        Err(_) => f32::NAN,
                    })
                    .collect::<Vec<f32>>();

                if num_coords.contains(&f32::NAN) {
                    *sequence = "Failed to parse input coordinates...".to_string();
                } else {
                    new_coords.push(Vec3A::from_slice(&num_coords));
                }
            }
        });

    *coordinates = new_coords;
}

fn try_update_edges(
    edges: &mut FxHashSet<(usize, usize)>,
    input: &str,
    nodes: &[Vec3A],
) -> Result<(), InvalidNodeDataError> {
    let mut new_edges: FxHashSet<(usize, usize)> = FxHashSet::default();

    if input
        .chars()
        .filter(|c| !(c.is_whitespace() || *c == '('))
        .collect::<String>()
        .split(')')
        .map(|edge| {
            if !edge.is_empty() {
                let indices: Vec<usize> = edge
                    .split(',')
                    .map(|val| match val.parse::<usize>() {
                        Ok(index) => {
                            if index > nodes.len() {
                                usize::MAX
                            } else {
                                index
                            }
                        }
                        Err(_) => usize::MAX,
                    })
                    .collect();

                if indices.contains(&usize::MAX) {
                    Err(InvalidNodeDataError)
                } else {
                    new_edges.insert((*indices.first().unwrap(), *indices.last().unwrap()));
                    Ok(())
                }
            } else {
                Ok(())
            }
        })
        .any(|res| res.is_err())
    {
        Err(InvalidNodeDataError)
    } else {
        *edges = new_edges;
        Ok(())
    }
}

struct InvalidNodeDataError;

fn update_rendered_object(
    rendered_nodes: &mut Vec<Gm<Mesh, PhysicalMaterial>>,
    nodes: &[Vec3A],
    rendered_edges: &mut Vec<Gm<Mesh, PhysicalMaterial>>,
    edges: &FxHashSet<(usize, usize)>,
    context: &Context,
) {
    rendered_nodes.clear();
    rendered_edges.clear();
    nodes.iter().for_each(|pos| {
        let mut sphere: Gm<Mesh, PhysicalMaterial> = create_sphere(context);
        sphere.set_transformation(Mat4::from_translation(vec3(pos.x, pos.y, pos.z)));
        rendered_nodes.push(sphere);
    });
    edges.iter().for_each(|&(origin, destination)| {
        if origin < nodes.len() && destination < nodes.len() {
            let pos_origin: Vec3A = *nodes.get(origin).unwrap();
            let pos_destination: Vec3A = *nodes.get(destination).unwrap();

            let p1 = vec3(pos_origin.x, pos_origin.y, pos_origin.z);
            let p2 = vec3(pos_destination.x, pos_destination.y, pos_destination.z);
            let transform = Mat4::from_translation(p1)
                * Into::<Mat4>::into(Quat::from_arc(
                    vec3(1.0, 0.0, 0.0),
                    (p2 - p1).normalize(),
                    None,
                ))
                * Mat4::from_nonuniform_scale((p1 - p2).magnitude(), 0.5, 0.5);

            let mut cylinder = create_cylinder(context);
            cylinder.set_transformation(transform);
            rendered_edges.push(cylinder);
        }
    });
}

fn create_cylinder(context: &Context) -> Gm<Mesh, PhysicalMaterial> {
    Gm::new(
        Mesh::new(context, &CpuMesh::cylinder(8)),
        PhysicalMaterial::new_opaque(
            context,
            &CpuMaterial {
                albedo: three_d::Srgba {
                    r: 0,
                    g: 0,
                    b: 5,
                    a: 255,
                },
                ..Default::default()
            },
        ),
    )
}

fn create_sphere(context: &Context) -> Gm<Mesh, PhysicalMaterial> {
    Gm::new(
        Mesh::new(context, &CpuMesh::sphere(16)),
        PhysicalMaterial::new_opaque(
            context,
            &CpuMaterial {
                albedo: three_d::Srgba {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
                ..Default::default()
            },
        ),
    )
}
