#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod ui;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;
use std::time::Duration;
use ui::{TrickText, TrickTextTimer, UiPlugin};

const ROT_SPEED: f32 = 8.;
const BASE_SPEED_LIMIT: f32 = 20.;
const BOOST_SPEED_LIMIT: f32 = 30.;
const BASE_BOOST_TIMER: f32 = 2.;

#[derive(Component, Deref, DerefMut)]
struct WheelsOnGround(u8);

#[derive(Component)]
struct Player;
#[derive(Component)]
struct Wheel;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    Next,
}

#[derive(AssetCollection)]
struct GameAssets {
    #[allow(dead_code)]
    #[asset(path = "tracktest.glb")]
    track: Handle<Scene>,
    #[asset(path = "NanumPenScript-Regular.ttf")]
    font: Handle<Font>,
}

#[derive(Component)]
struct VertexDebugger;

#[derive(Component, Deref, DerefMut)]
struct SpeedLimit(f32);

#[derive(Component, Default)]
struct Rotation {
    total: f32,
    front_flips: u32,
    back_flips: u32,
}

#[derive(Component)]
struct Boost {
    timer: Timer,
}
impl Default for Boost {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(BASE_BOOST_TIMER, false);
        timer.tick(Duration::from_secs_f32(BASE_BOOST_TIMER));

        Self { timer }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_state(GameState::AssetLoading)
        .add_state_to_stage(CoreStage::PostUpdate, GameState::AssetLoading)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Next)
                .with_collection::<GameAssets>(),
        )
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugin(UiPlugin)
        //.add_plugin(WireframePlugin)
        .add_system_set(
            SystemSet::on_enter(GameState::Next)
                .with_system(setup_physics)
                .with_system(setup_graphics),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::on_update(GameState::Next)
                .with_system(camera_follow)
                .with_system(display_events)
                .with_system(player_dampening)
                .with_system(track_trick.after(player_dampening)),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Next)
                .with_system(player_movement)
                .with_system(boost),
        )
        .run();
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Left,
    Right,
    RotateLeft,
    RotateRight,
    Jump,
}

fn setup_graphics(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 100.0),
        ..Default::default()
    });

    let mesh_handle = asset_server.load("tracktest.glb#Mesh0/Primitive0");

    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        })
        .insert(ColliderDebugColor(Color::GREEN))
        .insert(
            Collider::from_bevy_mesh(
                meshes.get(&mesh_handle).unwrap(),
                &ComputedColliderShape::TriMesh,
            )
            .unwrap(),
        );

    let mesh_handle = asset_server.load("tracktest.glb#Mesh1/Primitive0");

    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        })
        .insert(ColliderDebugColor(Color::GREEN))
        .insert(
            Collider::from_bevy_mesh(
                meshes.get(&mesh_handle).unwrap(),
                &ComputedColliderShape::TriMesh,
            )
            .unwrap(),
        );

    let mesh_handle = asset_server.load("tracktest.glb#Mesh2/Primitive0");

    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        })
        .insert(ColliderDebugColor(Color::GREEN))
        .insert(
            Collider::from_bevy_mesh(
                meshes.get(&mesh_handle).unwrap(),
                &ComputedColliderShape::TriMesh,
            )
            .unwrap(),
        );

    let mesh_handle = asset_server.load("tracktest.glb#Mesh3/Primitive0");

    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        })
        .insert(ColliderDebugColor(Color::GREEN))
        .insert(
            Collider::from_bevy_mesh(
                meshes.get(&mesh_handle).unwrap(),
                &ComputedColliderShape::TriMesh,
            )
            .unwrap(),
        );
}

pub fn setup_physics(mut commands: Commands) {
    let mut axes = LockedAxes::empty();
    axes.insert(LockedAxes::ROTATION_LOCKED_X);
    axes.insert(LockedAxes::ROTATION_LOCKED_Y);
    axes.insert(LockedAxes::TRANSLATION_LOCKED_Z);

    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(0., 0., 0.)))
        .insert(RigidBody::Dynamic)
        .insert(axes)
        .insert(Velocity::default())
        .insert(WheelsOnGround(0))
        .insert(Collider::cuboid(1., 1., 1.))
        .insert(ColliderDebugColor(Color::ORANGE))
        .insert(ExternalImpulse::default())
        .insert_bundle(InputManagerBundle::<Action> {
            input_map: InputMap::new([
                (KeyCode::Left, Action::Left),
                (KeyCode::A, Action::Left),
                (KeyCode::Right, Action::Right),
                (KeyCode::D, Action::Right),
                (KeyCode::Q, Action::RotateLeft),
                (KeyCode::E, Action::RotateRight),
                (KeyCode::Space, Action::Jump),
            ]),
            ..default()
        })
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(SpeedLimit(BASE_SPEED_LIMIT))
        .insert(Boost::default())
        .insert(Rotation::default())
        .insert(Player)
        .with_children(|parent| {
            parent
                .spawn_bundle(TransformBundle {
                    local: Transform::from_translation(Vec3::new(-1.5, -0.5, 0.)),
                    ..default()
                })
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Collider::ball(1.))
                .insert(ColliderDebugColor(Color::ORANGE))
                .insert(Friction::coefficient(0.1))
                .insert(Restitution::coefficient(0.0))
                .insert(Wheel);
            parent
                .spawn_bundle(TransformBundle {
                    local: Transform::from_translation(Vec3::new(1.5, -0.5, 0.)),
                    ..default()
                })
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Collider::ball(1.))
                .insert(ColliderDebugColor(Color::ORANGE))
                .insert(Friction::coefficient(0.1))
                .insert(Restitution::coefficient(0.0))
                .insert(Wheel);
        });
}

// Query for the `ActionState` component in your game logic systems!
fn player_movement(
    time: Res<Time>,
    mut query: Query<
        (
            &ActionState<Action>,
            &mut ExternalImpulse,
            &mut Velocity,
            &WheelsOnGround,
            &Transform,
        ),
        With<Player>,
    >,
) {
    for (action_state, mut impulse, mut velocity, wheels, transform) in query.iter_mut() {
        // Each action has a button-like state of its own that you can check
        if action_state.pressed(Action::Left) && **wheels >= 1 {
            impulse.impulse = transform.rotation * -Vec3::X * 500. * time.delta_seconds();
        }
        if action_state.pressed(Action::Right) && **wheels >= 1 {
            impulse.impulse = transform.rotation * Vec3::X * 500. * time.delta_seconds();
        }
        if action_state.pressed(Action::RotateLeft) {
            velocity.angvel += Vec3::Z * ROT_SPEED * time.delta_seconds();
        }
        if action_state.pressed(Action::RotateRight) {
            velocity.angvel += -Vec3::Z * ROT_SPEED * time.delta_seconds();
        }
        if action_state.just_pressed(Action::Jump) && wheels.0 >= 1 {
            impulse.impulse = transform.rotation * Vec3::Y * 175.;
        }
    }
}

fn camera_follow(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
) {
    for player_transform in player.iter() {
        for mut camera_transform in camera.iter_mut() {
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
            camera_transform.translation.z = 100.;
        }
    }
}

fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
    wheel_query: Query<Entity, With<Wheel>>,
    mut player_query: Query<&mut WheelsOnGround, With<Player>>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                for _ in wheel_query.iter_many([e1, e2]) {
                    for mut wheels in player_query.iter_mut() {
                        wheels.0 += 1;
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                for _ in wheel_query.iter_many([e1, e2]) {
                    for mut wheels in player_query.iter_mut() {
                        wheels.0 -= 1;
                    }
                }
            }
        }
    }

    for contact_force_event in contact_force_events.iter() {
        println!("Received contact force event: {:?}", contact_force_event);
    }
}

fn player_dampening(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &SpeedLimit, &WheelsOnGround), With<Player>>,
) {
    for (mut velocity, speed_limit, wheels) in query.iter_mut() {
        let elapsed = time.delta_seconds();
        velocity.angvel *= 0.1f32.powf(elapsed);

        if velocity.linvel.length() > **speed_limit && **wheels > 0 {
            velocity.linvel *= 0.2f32.powf(elapsed);
        }
    }
}

fn track_trick(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Rotation,
            &Velocity,
            &WheelsOnGround,
            ChangeTrackers<WheelsOnGround>,
            &mut Boost,
        ),
        With<Player>,
    >,
    mut trick_text_timer: ResMut<TrickTextTimer>,
    mut trick_text: Query<&mut Text, With<TrickText>>,
) {
    for (mut rotation, velocity, wheels, wheels_changed, mut boost) in query.iter_mut() {
        if **wheels == 0 {
            let elapsed = time.delta_seconds();
            let rot = velocity.angvel * elapsed;

            rotation.total += rot.z;

            if rotation.total > std::f32::consts::TAU {
                rotation.back_flips += 1;
                rotation.total -= std::f32::consts::TAU;
            } else if rotation.total < -std::f32::consts::TAU {
                rotation.front_flips += 1;
                rotation.total += std::f32::consts::TAU;
            }
        } else if wheels_changed.is_changed() {
            // super generous, because player may have launched from angled ramp
            if rotation.total > 280.0_f32.to_radians() {
                rotation.back_flips += 1;
            } else if rotation.total < -280.0_f32.to_radians() {
                rotation.front_flips += 1;
            }
            if rotation.front_flips > 0 || rotation.back_flips > 0 {
                info!(
                    "FLIP! fwd{} rev{} (leftover: {})",
                    rotation.front_flips,
                    rotation.back_flips,
                    rotation.total.to_degrees()
                );

                let flips = rotation.front_flips + rotation.back_flips;

                boost.timer.reset();
                boost.timer.set_duration(Duration::from_secs_f32(
                    BASE_BOOST_TIMER + (flips - 1) as f32 * 1.,
                ));

                for mut text in trick_text.iter_mut() {
                    text.sections[0].value =
                        ui::trick_text(rotation.front_flips, rotation.back_flips);
                    text.sections[0].style.color = Color::rgba(1., 0., 0., 1.)
                }
                trick_text_timer.reset();
            }

            rotation.total = 0.;
            rotation.front_flips = 0;
            rotation.back_flips = 0;
        }
    }
}

fn boost(time: Res<Time>, mut query: Query<(&mut Boost, &mut SpeedLimit), With<Player>>) {
    for (mut boost, mut speed_limit) in query.iter_mut() {
        boost.timer.tick(time.delta());
        if boost.timer.just_finished() {
            **speed_limit = BASE_SPEED_LIMIT;
            info!("just finished");
            info!("speed limit now {}", **speed_limit);
        } else if !boost.timer.finished() {
            **speed_limit = BOOST_SPEED_LIMIT;
            info!("speed limit now {}", **speed_limit);
        }
    }
}
