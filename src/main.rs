use bevy::prelude::*;
use bevy_flycam::{MovementSettings, PlayerPlugin};
use rand::Rng;
use bevy_kira_audio::{Audio, AudioPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Colors {
            ..Default::default()
        })
        /*.insert_resource(AmbientLight {
            //brightness: 0.5,
            ..Default::default()
        })*/
        .insert_resource(Sort {
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(AudioPlugin)
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 16.0,          // default: 12.0
        })
        .add_startup_system(setup)
        .add_system(change_color)
        .run();
}

const BOX_WIDTH: f32 = 1.;
const NUMBER_OF_BOXES: u32 = 1000;
const DISTANCE_BETWEEN_BOXES: f32 = 2.;
const MINIMUM_BOX_HEIGHT: f32 = 1.;
const MAXIMUM_BOX_HEIGHT: f32 = 1000.;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut colors: ResMut<Colors>,
    mut sort: ResMut<Sort>,
) {
    colors.ground = materials.add(Color::rgb(0.3, 0.5, 0.3).into());
    colors.unmarked = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    colors.current = materials.add(Color::rgb(0., 1., 0.).into());
    colors.compare = materials.add(Color::rgb(1., 0., 0.).into());
    colors.shortest = materials.add(Color::rgb(0., 0., 1.).into());
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1000000. })),
        material: colors.ground.clone(),
        ..Default::default()
    });
    // cube
    for i in 0..NUMBER_OF_BOXES {
        let height = rand::thread_rng().gen_range(MINIMUM_BOX_HEIGHT..MAXIMUM_BOX_HEIGHT + 1.);
        if i == 0 {
            sort.outer_height = height;
        } else if i == 1 {
            sort.shortest_height = height;
        }
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box {
                    min_x: i as f32,
                    max_x: i as f32 + BOX_WIDTH,
                    max_y: height,
                    ..Default::default()
                })),
                material: colors.unmarked.clone(),
                transform: Transform::from_xyz(
                    i as f32 * BOX_WIDTH * DISTANCE_BETWEEN_BOXES,
                    0.5,
                    0.0,
                ),
                ..Default::default()
            })
            .insert(Pillar {
                id: i as u32,
                height,
            });
    }
    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 10000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 3000.,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(1000., 1000., 1000.),

        ..Default::default()
    });
}

#[derive(Component)]
struct Pillar {
    id: u32,
    height: f32,
}

struct Sort {
    outer_id: u32,
    outer_height: f32,
    inner: u32,
    shortest_id: u32,
    shortest_height: f32,
}

impl Default for Sort {
    fn default() -> Self {
        Sort {
            outer_id: 0,
            outer_height: 0.,
            inner: 1,
            shortest_id: 1,
            shortest_height: 0.,
        }
    }
}

#[derive(Default)]
struct Colors {
    ground: Handle<StandardMaterial>,
    unmarked: Handle<StandardMaterial>,
    current: Handle<StandardMaterial>,
    compare: Handle<StandardMaterial>,
    shortest: Handle<StandardMaterial>,
}

fn change_color(
    mut sort: ResMut<Sort>,
    colors: Res<Colors>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(
        &mut Handle<StandardMaterial>,
        &mut Pillar,
        &mut Handle<Mesh>,
    )>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for (mut material, pillar, _) in query.iter_mut() {
        if pillar.id == sort.outer_id {
            sort.outer_height = pillar.height;
            *material = colors.current.clone();
        } else if pillar.id == sort.inner {
            if pillar.height < sort.shortest_height || sort.inner == sort.outer_id + 1 {
                sort.shortest_id = pillar.id;
                sort.shortest_height = pillar.height;
            }
            *material = colors.compare.clone();
        } else if pillar.id == sort.shortest_id {
            *material = colors.shortest.clone();
        } else {
            *material = colors.unmarked.clone();
        }
    }
    if sort.inner == NUMBER_OF_BOXES {
        let temp = sort.outer_height;
        for (_, mut pillar, mut mesh) in query.iter_mut() {
            if pillar.id == sort.outer_id {
                pillar.height = sort.shortest_height;
                *mesh = meshes.add(Mesh::from(shape::Box {
                    min_x: pillar.id as f32,
                    max_x: pillar.id as f32 + BOX_WIDTH,
                    max_y: sort.shortest_height,
                    ..Default::default()
                }));
                // println!("changed outer ({})", pillar.id);
            } else if pillar.id == sort.shortest_id {
                pillar.height = sort.outer_height;
                *mesh = meshes.add(Mesh::from(shape::Box {
                    min_x: pillar.id as f32,
                    max_x: pillar.id as f32 + BOX_WIDTH,
                    max_y: temp,
                    ..Default::default()
                }));
                // println!("changed shortest ({})", pillar.id);
            }
        }
        sort.outer_id += 1;
        sort.inner = sort.outer_id + 1;
    } else {
        sort.inner += 1;
    }
    if sort.outer_id == NUMBER_OF_BOXES - 1 {
        audio.play(asset_server.load("sort_complete.ogg"));
    }
}
