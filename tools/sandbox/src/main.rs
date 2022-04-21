use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use rand::distributions::{Distribution, Uniform};
use simula_camera::flycam::*;
use simula_core::{
    force_graph::{NodeData, NodeIndex, SimulationParameters},
    signal::{SignalController, SignalFunction, SignalGenerator},
};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    force_graph::{ForceGraph, ForceGraphBundle},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{Lines, LinesBundle, LinesPlugin},
    signal::{
        signal_control_lines, signal_generator_lines, SignalControlLine, SignalGeneratorLine,
    },
    voxels::{Voxel, Voxels, VoxelsBundle, VoxelsPlugin},
};

fn main() {
    App::new()
        .register_type::<SignalGenerator>()
        .register_type::<SignalFunction>()
        .register_type::<SignalController<f32>>()
        .register_type::<ForceGraph<SandboxNodeData, SandboxEdgeData>>()
        .register_type::<SimulationParameters>()
        .register_type::<SandboxNode>()
        .insert_resource(WindowDescriptor {
            title: "[Simbotic] Simula - Sandbox".to_string(),
            width: 940.,
            height: 528.,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.125, 0.12, 0.13)))
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(FlyCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(VoxelsPlugin)
        .add_startup_system(setup)
        .add_system(debug_info)
        .add_system(line_test)
        .add_system(signal_generator_lines)
        .add_system(signal_control_lines)
        .add_system(rotate_system)
        .add_system(force_graph_test)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(2.0, 0.01, 2.0),
        ..Default::default()
    });

    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(-2.0, 0.0, -2.0),
        ..Default::default()
    });

    // grid
    commands.spawn_bundle(GridBundle {
        grid: Grid {
            size: 10,
            divisions: 10,
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default()
    });

    // axes
    commands.spawn_bundle(AxesBundle {
        axes: Axes {
            size: 1.,
            inner_offset: 5.,
        },
        transform: Transform::from_xyz(0.0, 0.01, 0.0),
        ..Default::default()
    });

    // x - axis
    commands
        .spawn_bundle(AxesBundle {
            axes: Axes {
                size: 3.,
                inner_offset: 0.,
            },
            transform: Transform::from_xyz(7.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(Rotate {
            axis: Vec3::new(1.0, 0.0, 0.0),
            angle: 1.0,
        })
        .insert(RandomLines);

    // y - axis
    commands
        .spawn_bundle(AxesBundle {
            axes: Axes {
                size: 3.,
                inner_offset: 0.,
            },
            transform: Transform::from_xyz(0.0, 7.0, 0.0),
            ..Default::default()
        })
        .insert(Rotate {
            axis: Vec3::new(0.0, 1.0, 0.0),
            angle: 1.0,
        })
        .insert(RandomLines);

    // z - axis
    commands
        .spawn_bundle(AxesBundle {
            axes: Axes {
                size: 3.,
                inner_offset: 0.,
            },
            transform: Transform::from_xyz(0.0, 0.0, -7.0),
            ..Default::default()
        })
        .insert(Rotate {
            axis: Vec3::new(0.0, 0.0, 1.0),
            angle: 1.0,
        })
        .insert(RandomLines);

    let theta = std::f32::consts::FRAC_PI_4;
    let light_transform = Mat4::from_euler(EulerRot::ZYX, 0.0, std::f32::consts::FRAC_PI_2, -theta);
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            illuminance: 5000.,
            ..Default::default()
        },
        transform: Transform::from_matrix(light_transform),
        ..Default::default()
    });

    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 1.5, 8.0),
            ..Default::default()
        })
        .insert(FlyCamera {
            sensitivity: 100.,
            ..Default::default()
        });

    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "\nFPS: ".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 12.0,
                    color: Color::rgb(0.0, 1.0, 0.0),
                },
            }],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });

    // voxels

    let voxels: Vec<Voxel> = vec![
        Voxel {
            position: Vec3::new(6., 0., 0.),
            size: 0.5,
            color: Color::RED,
        },
        Voxel {
            position: Vec3::new(0., 6., 0.),
            size: 0.5,
            color: Color::GREEN,
        },
        Voxel {
            position: Vec3::new(0., 0., -6.),
            size: 0.5,
            color: Color::rgba(0.0, 0.0, 1.0, 0.2),
        },
    ];

    commands
        .spawn_bundle(VoxelsBundle {
            voxels: Voxels { voxels },
            ..Default::default()
        })
        .insert(Rotate {
            axis: Vec3::new(0.0, 1.0, 0.0),
            angle: 1.0,
        });

    let rod_mesh = simula_viz::rod::Rod {
        ..Default::default()
    };
    let rod_mesh = simula_viz::rod::RodMesh::from(rod_mesh);

    commands.spawn().insert_bundle(PbrBundle {
        mesh: meshes.add(rod_mesh.mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::PINK,
            ..Default::default()
        }),
        transform: Transform::from_xyz(5.0, 0.0, -5.0),
        ..Default::default()
    });

    commands.spawn_scene(asset_server.load("models/metric_plane/metric_plane_8x8.gltf#Scene0"));

    commands
        .spawn()
        .insert_bundle((
            Transform::from_xyz(-2.5, 0.0, 2.5),
            GlobalTransform::default(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("models/metric_box/metric_box_1x1.gltf#Scene0"));
        });

    // generator signals

    let points: Vec<Vec3> = (-100i32..=100)
        .map(|i| Vec3::new((i as f32) * 0.01, 0.0, 0.0))
        .collect();

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 3.0, 0.0),
            ..Default::default()
        })
        .insert(SignalGenerator {
            func: SignalFunction::Sine,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 2.8, 0.0),
            ..Default::default()
        })
        .insert(SignalGenerator {
            func: SignalFunction::Square,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 2.6, 0.0),
            ..Default::default()
        })
        .insert(SignalGenerator {
            func: SignalFunction::Triangle,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 2.4, 0.0),
            ..Default::default()
        })
        .insert(SignalGenerator {
            func: SignalFunction::Sawtooth,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 2.2, 0.0),
            ..Default::default()
        })
        .insert(SignalGenerator {
            func: SignalFunction::Pulse,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..Default::default()
        })
        .insert(SignalGenerator {
            func: SignalFunction::WhiteNoise,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 1.8, 0.0),
            ..Default::default()
        })
        .insert(SignalGenerator {
            func: SignalFunction::GaussNoise,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 1.6, 0.0),
            ..Default::default()
        })
        .insert(SignalGenerator {
            func: SignalFunction::DigitalNoise,
            amplitude: 0.1,
            frequency: 3.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        });

    // control signals

    commands
        .spawn_bundle(LinesBundle {
            transform: Transform::from_xyz(0.0, 4.0, 0.0),
            ..Default::default()
        })
        .insert(SignalGenerator {
            func: SignalFunction::Pulse,
            amplitude: 1.0,
            frequency: 1.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        })
        .insert(SignalController::<f32> {
            kp: 0.1,
            ki: 0.0,
            kd: 0.0,
            ..Default::default()
        })
        .insert(SignalControlLine {
            points: points.clone(),
        });

    // force graph

    let mut graph_bundle = ForceGraphBundle::<SandboxNodeData, SandboxEdgeData> {
        transform: Transform::from_xyz(0.0, 3.5, 0.0),
        ..Default::default()
    };
    let graph = &mut graph_bundle.graph.graph;

    commands
        .spawn()
        .insert(SandboxGraph)
        .with_children(|parent| {
            let root_index = graph.add_node(NodeData::<SandboxNodeData> {
                is_anchor: true,
                ..Default::default()
            });

            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::UVSphere {
                        radius: 0.1,
                        ..Default::default()
                    })),
                    material: materials.add(Color::GOLD.into()),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    ..Default::default()
                })
                .insert(SandboxNode {
                    node_index: root_index,
                });

            for _ in 0..10 {
                let node_index = graph.add_node(NodeData::<SandboxNodeData> {
                    position: Vec3::new(rand::random(), rand::random(), rand::random()) * 0.01,
                    mass: 1.0,
                    ..Default::default()
                });

                graph.add_edge(root_index, node_index, Default::default());

                parent
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::UVSphere {
                            radius: 0.1,
                            ..Default::default()
                        })),
                        material: materials.add(Color::ALICE_BLUE.into()),
                        transform: Transform::from_xyz(0.0, 0.5, 0.0),
                        ..Default::default()
                    })
                    .insert(SandboxNode { node_index });

                let parent_index = node_index;
                for _ in 0..3 {
                    let node_index = graph.add_node(NodeData::<SandboxNodeData> {
                        position: Vec3::new(rand::random(), rand::random(), rand::random()) * 0.01,
                        mass: 1.0,
                        ..Default::default()
                    });

                    graph.add_edge(parent_index, node_index, Default::default());

                    parent
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::UVSphere {
                                radius: 0.1,
                                ..Default::default()
                            })),
                            material: materials.add(Color::ALICE_BLUE.into()),
                            transform: Transform::from_xyz(0.0, 0.5, 0.0),
                            ..Default::default()
                        })
                        .insert(SandboxNode { node_index });
                }
            }
        })
        .insert_bundle(graph_bundle);
}

fn debug_info(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            for mut text in query.iter_mut() {
                text.sections[0].value = format!("{:.2}", average);
            }
        }
    };
}

#[derive(Component)]
struct RandomLines;

#[derive(Component)]
struct Rotate {
    axis: Vec3,
    angle: f32,
}

fn rotate_system(time: Res<Time>, mut query: Query<(&Rotate, &mut Transform)>) {
    for (rotate, mut transform) in query.iter_mut() {
        transform.rotate(Quat::from_axis_angle(
            rotate.axis,
            rotate.angle * time.delta_seconds(),
        ));
    }
}

fn line_test(mut lines: Query<&mut Lines, With<RandomLines>>) {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(0f32..1f32);

    for mut lines in lines.iter_mut() {
        for _ in 0..20 {
            let x = die.sample(&mut rng) * 0.2 - 0.1;
            let z = die.sample(&mut rng) * 0.2 - 0.1;
            let start = Vec3::new(x, -0.1, z);
            let end = Vec3::new(x, 0.1, z);

            let color = Color::Hsla {
                hue: die.sample(&mut rng) * 360.0,
                lightness: 0.5,
                saturation: 1.0,
                alpha: 1.0,
            };
            lines.line_colored(start, end, color);
        }
    }
}

#[derive(Reflect, Component, Default, Clone, PartialEq)]
#[reflect(Component)]
pub struct SandboxNode {
    #[reflect(ignore)]
    node_index: NodeIndex,
}

#[derive(Reflect, Component, Default, Clone, PartialEq)]
#[reflect(Component)]
pub struct SandboxNodeData;

#[derive(Reflect, Component, Default, Clone, PartialEq)]
#[reflect(Component)]
pub struct SandboxEdgeData;

#[derive(Reflect, Component, Default, Clone, PartialEq)]
#[reflect(Component)]
pub struct SandboxGraph;

fn force_graph_test(
    time: Res<Time>,
    mut graphs: Query<
        (
            &mut ForceGraph<SandboxNodeData, SandboxEdgeData>,
            &Children,
            &mut Lines,
        ),
        With<SandboxGraph>,
    >,
    mut nodes: Query<(&mut Transform, &SandboxNode)>,
) {
    for (mut graph, children, mut lines) in graphs.iter_mut() {
        graph.graph.parameters = graph.parameters.clone();
        graph.graph.update(time.delta());
        graph.graph.visit_edges(|a, b, _| {
            lines.line_colored(a.position(), b.position(), Color::DARK_GRAY);
        });
        for child in children.iter() {
            if let Ok((mut transform, node)) = nodes.get_mut(*child) {
                let node = &graph.graph.get_graph()[*node.node_index];
                transform.translation = node.position();
            }
        }
    }
}
