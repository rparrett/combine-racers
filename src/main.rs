#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod countdown;
mod leaderboard;
mod main_menu;
mod save;
mod settings;
mod ui;

use bevy::{
    gltf::GltfExtras, log::LogSettings, pbr::PointLightShadowMap, prelude::*, time::Stopwatch,
};
use bevy_asset_loader::prelude::*;
#[cfg(feature = "inspector")]
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use countdown::CountdownPlugin;
use interpolation::{Ease, Lerp};
use leaderboard::LeaderboardPlugin;
use leafwing_input_manager::prelude::*;
use main_menu::MainMenuPlugin;
use save::SavePlugin;
use serde::Deserialize;
use settings::{KeyboardLayout, KeyboardSetting, SettingsPlugin};
use std::time::Duration;
use ui::{TrickText, TrickTextTimer, UiPlugin};

const ROT_SPEED: f32 = 8.;
const BASE_SPEED_LIMIT: f32 = 20.;
const BOOST_SPEED_LIMIT: f32 = 30.;
const BASE_BOOST_TIMER: f32 = 2.;

#[derive(Component, Default, Deref, DerefMut)]
struct WheelsOnGround(u8);

#[derive(Component, Default, Deref, DerefMut)]
struct BonkStatus(bool);

#[derive(Component)]
struct Player;
#[derive(Component)]
struct Wheel;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Loading,
    Decorating,
    MainMenu,
    Playing,
    Leaderboard,
}

#[derive(AssetCollection)]
struct GameAssets {
    #[asset(path = "tracktest.glb#Scene0")]
    track: Handle<Scene>,
    #[asset(path = "combine.glb#Scene0")]
    combine: Handle<Scene>,
    #[asset(path = "NanumPenScript-Regular.ttf")]
    font: Handle<Font>,
}
#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(path = "combine-racers-321go.ogg")]
    three_two_one: Handle<AudioSource>,
}

#[derive(Component)]
struct LightContainer;

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

#[derive(Component)]
struct Track;
#[derive(Component)]
struct FinishLine;
#[derive(Deref, DerefMut)]
struct RaceTime(Stopwatch);
impl Default for RaceTime {
    fn default() -> Self {
        let mut watch = Stopwatch::default();
        watch.pause();

        Self(watch)
    }
}
struct Zoom {
    from: f32,
    target: f32,
    timer: Timer,
}
impl Default for Zoom {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.7, false);
        timer.pause();

        Self {
            from: 20.,
            target: 80.,
            timer,
        }
    }
}

struct FinishedEvent;

fn main() {
    let mut app = App::new();

    app.insert_resource(LogSettings {
        filter: "info,bevy_ecs=debug,wgpu_core=warn,wgpu_hal=warn,combine_racers=debug".into(),
        level: bevy::log::Level::DEBUG,
    })
    .insert_resource(PointLightShadowMap { size: 2048 })
    .insert_resource(ClearColor(Color::BLACK))
    .add_state(GameState::Loading)
    .add_state_to_stage(CoreStage::PostUpdate, GameState::Loading)
    .add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Decorating)
            .with_collection::<GameAssets>()
            .with_collection::<AudioAssets>(),
    )
    .add_plugins(DefaultPlugins)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    //.add_plugin(RapierDebugRenderPlugin::default())
    .add_plugin(InputManagerPlugin::<Action>::default())
    .add_plugin(UiPlugin)
    .add_plugin(MainMenuPlugin)
    .add_plugin(CountdownPlugin)
    .add_plugin(LeaderboardPlugin)
    .add_plugin(SettingsPlugin)
    .add_plugin(SavePlugin);

    #[cfg(feature = "inspector")]
    app.add_plugin(WorldInspectorPlugin::new());

    app.init_resource::<RaceTime>()
        .init_resource::<Zoom>()
        .add_event::<FinishedEvent>()
        .add_system_set(SystemSet::on_exit(GameState::Loading).with_system(spawn_camera))
        .add_system_set(SystemSet::on_enter(GameState::Decorating).with_system(setup_game))
        .add_system_set(SystemSet::on_update(GameState::Decorating).with_system(decorate_track))
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_player))
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::on_update(GameState::Playing)
                .with_system(camera_follow)
                .with_system(collision_events)
                .with_system(player_dampening)
                .with_system(track_trick.after(player_dampening))
                .with_system(zoom.after(camera_follow)),
        )
        // Do a limited subset of things in the background while
        // we're showing the leaderboard.
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::on_update(GameState::Leaderboard)
                .with_system(player_dampening)
                .with_system(camera_follow),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(player_movement)
                .with_system(boost)
                .with_system(race_time)
                .with_system(game_finished)
                .with_system(start_zoom),
        )
        .add_system_set(SystemSet::on_exit(GameState::Playing))
        .add_system_set(SystemSet::on_exit(GameState::Leaderboard).with_system(reset))
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
    ToggleZoom,
}

fn spawn_camera(mut commands: Commands, zoom: Res<Zoom>) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., zoom.target),
        ..Default::default()
    });
}

fn decorate_track(
    mut commands: Commands,
    query: Query<(Entity, &GltfExtras, &Children)>,
    mesh_query: Query<(Entity, &Handle<Mesh>), Without<Collider>>,
    meshes: Res<Assets<Mesh>>,
    mut visibility_query: Query<&mut Visibility>,
    mut state: ResMut<State<GameState>>,
) {
    let mut decorated = false;

    for (entity, extras, children) in query.iter() {
        decorated = true;
        #[derive(Deserialize)]
        struct TrackExtra {
            object_type: String,
        }

        if let Ok(v) = serde_json::from_str::<TrackExtra>(&extras.value) {
            if v.object_type == "track" {
                for (mesh_entity, mesh_handle) in mesh_query.iter_many(children) {
                    commands
                        .entity(mesh_entity)
                        .insert(ColliderDebugColor(Color::GREEN))
                        .insert(
                            Collider::from_bevy_mesh(
                                meshes.get(mesh_handle).unwrap(),
                                &ComputedColliderShape::TriMesh,
                            )
                            .unwrap(),
                        )
                        .insert(Track);

                    info!("Added collider to {:?}", entity);
                }
            } else if v.object_type == "finish_line" {
                for (mesh_entity, mesh_handle) in mesh_query.iter_many(children) {
                    commands
                        .entity(mesh_entity)
                        .insert(ColliderDebugColor(Color::GRAY))
                        .insert(
                            Collider::from_bevy_mesh(
                                meshes.get(mesh_handle).unwrap(),
                                &ComputedColliderShape::TriMesh,
                            )
                            .unwrap(),
                        )
                        .insert(Sensor)
                        .insert(FinishLine);
                    info!("Added collider to {:?}", entity);
                }
                if let Ok(mut visibility) = visibility_query.get_mut(entity) {
                    visibility.is_visible = false;
                }
            }
        }
    }

    if decorated {
        state.set(GameState::MainMenu).unwrap();
    }
}

fn setup_game(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn_bundle(SpatialBundle::default())
        .with_children(|parent| {
            parent.spawn_bundle(PointLightBundle {
                point_light: PointLight {
                    shadows_enabled: true,
                    intensity: 500000.,
                    range: 200.,
                    ..default()
                },
                transform: Transform::from_xyz(20., 20., 50.),
                ..default()
            });
        })
        .insert(LightContainer);

    // ambient light
    commands.insert_resource(AmbientLight {
        brightness: 0.2,
        ..default()
    });

    commands.spawn_bundle({
        SceneBundle {
            scene: assets.track.clone(),
            ..default()
        }
    });
}

fn spawn_player(
    mut commands: Commands,
    keyboard: Res<KeyboardSetting>,
    game_assets: Res<GameAssets>,
) {
    let mut axes = LockedAxes::empty();
    axes.insert(LockedAxes::ROTATION_LOCKED_X);
    axes.insert(LockedAxes::ROTATION_LOCKED_Y);
    axes.insert(LockedAxes::TRANSLATION_LOCKED_Z);

    let mut input_map = match **keyboard {
        KeyboardLayout::Qwerty => InputMap::new([
            (KeyCode::Left, Action::Left),
            (KeyCode::A, Action::Left),
            (KeyCode::Right, Action::Right),
            (KeyCode::D, Action::Right),
            (KeyCode::Q, Action::RotateLeft),
            (KeyCode::E, Action::RotateRight),
            (KeyCode::Space, Action::Jump),
            (KeyCode::Z, Action::ToggleZoom),
        ]),
        KeyboardLayout::Azerty => InputMap::new([
            (KeyCode::Left, Action::Left),
            (KeyCode::Q, Action::Left),
            (KeyCode::Right, Action::Right),
            (KeyCode::D, Action::Right),
            (KeyCode::A, Action::RotateLeft),
            (KeyCode::E, Action::RotateRight),
            (KeyCode::Space, Action::Jump),
            (KeyCode::W, Action::ToggleZoom),
        ]),
    };

    input_map.insert_multiple([
        (GamepadButtonType::DPadLeft, Action::Left),
        (GamepadButtonType::DPadRight, Action::Right),
        (GamepadButtonType::LeftTrigger, Action::RotateLeft),
        (GamepadButtonType::RightTrigger, Action::RotateRight),
        (GamepadButtonType::South, Action::Jump),
    ]);

    commands
        .spawn_bundle(SceneBundle {
            scene: game_assets.combine.clone(),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(axes)
        .insert(Velocity::default())
        .insert(WheelsOnGround::default())
        .insert(BonkStatus::default())
        .insert(Collider::cuboid(1., 1., 1.))
        .insert(ColliderDebugColor(Color::ORANGE))
        .insert(ExternalImpulse::default())
        .insert_bundle(InputManagerBundle::<Action> {
            input_map,
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
    race_timer: Res<RaceTime>,
) {
    if race_timer.paused() {
        return;
    }

    for (action_state, mut impulse, mut velocity, wheels, transform) in query.iter_mut() {
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
    mut light: Query<&mut Transform, (With<LightContainer>, Without<Camera3d>, Without<Player>)>,
) {
    for player_transform in player.iter() {
        for mut camera_transform in camera.iter_mut() {
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
        }

        for mut light_transform in light.iter_mut() {
            light_transform.translation.x = player_transform.translation.x;
            light_transform.translation.y = player_transform.translation.y;
        }
    }
}

fn collision_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
    wheel_query: Query<Entity, With<Wheel>>,
    track_query: Query<Entity, With<Track>>,
    finish_line_query: Query<Entity, With<FinishLine>>,
    body_query: Query<Entity, With<Player>>,
    mut player_query: Query<(&mut WheelsOnGround, &mut BonkStatus), With<Player>>,
    mut race_time: ResMut<RaceTime>,
    mut finished_event: EventWriter<FinishedEvent>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                let finish_line = finish_line_query.iter_many([e1, e2]).count() > 0;
                let track = track_query.iter_many([e1, e2]).count() > 0;
                let wheel = wheel_query.iter_many([e1, e2]).count() > 0;
                let body = body_query.iter_many([e1, e2]).count() > 0;

                if wheel && track {
                    for (mut wheels, mut bonk) in player_query.iter_mut() {
                        wheels.0 += 1;
                        **bonk = false;
                    }
                }

                if (body || wheel) && finish_line {
                    race_time.pause();
                    // we have to fire off an event here because you can't
                    // trigger on_exit and on_enter when changing state from
                    // a different stage.
                    finished_event.send(FinishedEvent);
                }

                if body && track {
                    for (_, mut bonk) in player_query.iter_mut() {
                        **bonk = true;
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                let track = track_query.iter_many([e1, e2]).count() > 0;
                let wheel = wheel_query.iter_many([e1, e2]).count() > 0;

                if track && wheel {
                    for (mut wheels, _) in player_query.iter_mut() {
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

fn game_finished(mut events: EventReader<FinishedEvent>, mut state: ResMut<State<GameState>>) {
    if events.iter().count() > 0 {
        state.set(GameState::Leaderboard).unwrap();
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
            &BonkStatus,
            ChangeTrackers<WheelsOnGround>,
            &mut Boost,
        ),
        With<Player>,
    >,
    mut trick_text_timer: ResMut<TrickTextTimer>,
    mut trick_text: Query<&mut Text, With<TrickText>>,
) {
    for (mut rotation, velocity, wheels, bonk, wheels_changed, mut boost) in query.iter_mut() {
        if **bonk {
            rotation.total = 0.;
            rotation.front_flips = 0;
            rotation.back_flips = 0;
        }

        if **wheels == 0 {
            let elapsed = time.delta_seconds();
            let rot = velocity.angvel * elapsed;

            rotation.total += rot.z;

            // TODO back/front reversed when travelling left

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

fn race_time(time: Res<Time>, mut race_time: ResMut<RaceTime>) {
    race_time.tick(time.delta());
}

fn start_zoom(query: Query<&ActionState<Action>, With<Player>>, mut zoom: ResMut<Zoom>) {
    let action_state = query.single();
    if action_state.just_pressed(Action::ToggleZoom) && zoom.timer.paused() {
        (zoom.target, zoom.from) = (zoom.from, zoom.target);

        zoom.timer.reset();
        zoom.timer.unpause();
    }
}

fn zoom(
    time: Res<Time>,
    mut zoom: ResMut<Zoom>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if zoom.timer.paused() {
        return;
    }

    zoom.timer.tick(time.delta());

    let mut camera = camera_query.single_mut();

    let z = zoom
        .from
        .lerp(&zoom.target, &Ease::quadratic_in_out(zoom.timer.percent()));

    camera.translation.z = z;

    if zoom.timer.just_finished() {
        zoom.timer.pause();
    }
}

fn reset(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    mut race_time: ResMut<RaceTime>,
) {
    for entity in player_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    race_time.reset();
}
