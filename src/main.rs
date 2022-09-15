#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod countdown;
mod game_over;
mod leaderboard;
mod main_menu;
mod music_fade_in;
mod random_name;
mod save;
mod settings;
mod ui;

use std::f32::consts::TAU;

use bevy::{
    audio::AudioSink, log::LogSettings, pbr::PointLightShadowMap, prelude::*, time::Stopwatch,
};
use bevy_asset_loader::prelude::*;
#[cfg(feature = "inspector")]
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_ui_navigation::{systems::InputMapping, DefaultNavigationPlugins};
use countdown::CountdownPlugin;
use game_over::GameOverPlugin;
use interpolation::{Ease, Lerp};
use leaderboard::{get_leaderboard_credentials, LeaderboardPlugin};
use leafwing_input_manager::{axislike::AxisType, prelude::*};
use main_menu::MainMenuPlugin;
use music_fade_in::MusicFadeInPlugin;
use save::SavePlugin;
use settings::{KeyboardLayout, KeyboardSetting, SfxSetting};
use ui::{TrickText, UiPlugin};

const ROT_SPEED: f32 = 8.;
const JUMP_IMPULSE: f32 = 175.;
const DRIVE_FORCE: f32 = 400.;
const BASE_SPEED_LIMIT: f32 = 20.;
const BOOST_SPEED_LIMIT: f32 = 30.;
const BASE_BOOST_TIMER: f32 = 2.;

#[derive(Component, Default, Deref, DerefMut)]
struct WheelsOnGround(u8);
#[derive(Component, Default, Deref, DerefMut)]
struct JumpWheelsOnGround(u8);

#[derive(Component, Debug, Default, Deref, DerefMut)]
struct BonkStatus(bool);

#[derive(Component)]
struct Player;
#[derive(Component)]
struct Wheel;
/// A special wheel, slightly larger than the normal wheel. When at
/// least one JumpWheel is touching the track, the player is allowed
/// to jump.
///
/// This works around some frustrating jank where the player might be
/// "mid-air" for 5 frames at a time while travelling on flat ground.
#[derive(Component)]
struct JumpWheel;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Loading,
    Decorating,
    MainMenu,
    Playing,
    Leaderboard,
    GameOver,
}

#[derive(AssetCollection)]
struct GameAssets {
    #[asset(path = "track_1.glb#Scene0")]
    track: Handle<Scene>,
    #[asset(path = "combine.glb#Scene0")]
    combine: Handle<Scene>,
    #[asset(path = "NanumPenScript-Tweaked.ttf")]
    font: Handle<Font>,
}
#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(path = "7th-race-aiteru-sawato.ogg")]
    music: Handle<AudioSource>,
    #[asset(path = "combine-racers-321go.ogg")]
    three_two_one: Handle<AudioSource>,
    #[asset(path = "combine-racers-trick.ogg")]
    trick: Handle<AudioSource>,
    #[asset(path = "combine-racers-bonk.ogg")]
    bonk: Handle<AudioSource>,
}

struct MusicController(Handle<AudioSink>);

#[derive(Component)]
struct LightContainer;

#[derive(Component, Deref, DerefMut)]
struct SpeedLimit(f32);

#[derive(Component, Default)]
struct TrickStatus {
    rotation: f32,
    front_flips: u32,
    back_flips: u32,
    start_x: f32,
    hang_time: f32,
}
impl TrickStatus {
    fn reset(&mut self) {
        self.rotation = 0.;
        self.front_flips = 0;
        self.back_flips = 0;
        self.hang_time = 0.;
    }
}
#[derive(Component, Default, Deref, DerefMut)]
struct LastTrick(Trick);
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Trick {
    front_flips: u32,
    back_flips: u32,
    fakie: bool,
}

#[derive(Component, Default)]
struct Boost {
    remaining: f32,
}

#[derive(Component)]
struct Track;
#[derive(Component)]
struct FinishLine;
#[derive(Component)]
struct PlaceholderCombine;
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
#[derive(Component, Deref, DerefMut)]
struct JumpCooldown(bool);
impl Default for JumpCooldown {
    fn default() -> Self {
        Self(true)
    }
}

struct FinishedEvent;

const LAVA: f32 = -200.;

fn main() {
    let mut app = App::new();

    app.insert_resource(LogSettings {
        filter: "info,bevy_ecs=debug,wgpu_core=warn,wgpu_hal=warn,combine_racers=debug".into(),
        level: bevy::log::Level::DEBUG,
    })
    .insert_resource(WindowDescriptor {
        fit_canvas_to_parent: true,
        ..default()
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
    .insert_resource(InputMapping {
        keyboard_navigation: true,
        ..default()
    })
    .add_plugin(InputManagerPlugin::<Action>::default())
    .add_plugins(DefaultNavigationPlugins)
    .add_plugin(UiPlugin)
    .add_plugin(MainMenuPlugin)
    .add_plugin(CountdownPlugin)
    .add_plugin(LeaderboardPlugin)
    .add_plugin(GameOverPlugin)
    .add_plugin(SavePlugin)
    .add_plugin(MusicFadeInPlugin);

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
        // we're showing the leaderboard or game over screen
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::on_update(GameState::Leaderboard)
                .with_system(player_dampening)
                .with_system(camera_follow),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::on_update(GameState::GameOver)
                .with_system(player_dampening)
                .with_system(camera_follow),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(player_movement)
                .with_system(boost)
                .with_system(race_time)
                .with_system(game_finished)
                .with_system(start_zoom)
                .with_system(reset_action)
                .with_system(bonk_sound)
                .with_system(death),
        )
        .add_system_set(SystemSet::on_exit(GameState::Playing))
        .add_system_set(SystemSet::on_exit(GameState::Leaderboard).with_system(reset))
        .add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(reset))
        .run();
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Left,
    Right,
    LeftRight,
    RotateLeft,
    RotateRight,
    Jump,
    ToggleZoom,
    Reset,
}

fn spawn_camera(mut commands: Commands, zoom: Res<Zoom>) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., zoom.target),
        ..Default::default()
    });
}

fn decorate_track(
    mut commands: Commands,
    mesh_query: Query<(Entity, &Name, &Handle<Mesh>), Without<Collider>>,
    meshes: Res<Assets<Mesh>>,
    mut visibility_query: Query<&mut Visibility>,
    mut state: ResMut<State<GameState>>,
) {
    fn chop_name(name: &str) -> Option<&str> {
        name.rsplitn(2, '.').last()
    }

    let mut decorated = false;

    for (mesh_entity, name, mesh_handle) in mesh_query.iter() {
        match chop_name(&**name) {
            Some("Track") => {
                decorated = true;

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

                info!("Added track collider to {:?}", mesh_entity);
            }
            Some("FinishLineCollider") => {
                decorated = true;

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

                if let Ok(mut visibility) = visibility_query.get_mut(mesh_entity) {
                    visibility.is_visible = false;
                }

                info!("Added finish line collider to {:?}", mesh_entity);
            }
            _ => {}
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

    // this is super dumb, but spawning the combine stops the world in web
    // builds and ruins the race start countdown. so we'll spawn it here
    // instead when it's less disruptive.
    commands
        .spawn_bundle({
            SceneBundle {
                scene: assets.combine.clone(),
                // the thing has to be "visible" for this to work, so hide it in the track.
                transform: Transform::from_xyz(0., -4., 0.).with_scale(Vec3::splat(0.001)),
                ..default()
            }
        })
        .insert(PlaceholderCombine);
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
            (KeyCode::Escape, Action::Reset),
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
            (KeyCode::Escape, Action::Reset),
        ]),
    };

    input_map.insert_multiple([
        (GamepadButtonType::DPadLeft, Action::Left),
        (GamepadButtonType::DPadRight, Action::Right),
        (GamepadButtonType::LeftTrigger, Action::RotateLeft),
        (GamepadButtonType::RightTrigger, Action::RotateRight),
        (GamepadButtonType::South, Action::Jump),
        (GamepadButtonType::North, Action::ToggleZoom),
        (GamepadButtonType::Select, Action::Reset),
    ]);

    // TODO replace with SingleAxis::negative_only when LWIM is updated
    input_map.insert_multiple([
        (
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
                negative_low: -0.3,
                positive_low: f32::MAX,
                value: None,
            },
            Action::Left,
        ),
        (
            SingleAxis {
                axis_type: AxisType::Gamepad(GamepadAxisType::LeftStickX),
                negative_low: f32::MIN,
                positive_low: 0.3,
                value: None,
            },
            Action::Right,
        ),
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
        .insert(JumpWheelsOnGround::default())
        .insert(JumpCooldown::default())
        .insert(BonkStatus::default())
        .insert(Collider::cuboid(1., 1., 1.))
        .insert(ColliderDebugColor(Color::ORANGE))
        .insert(ExternalImpulse::default())
        .insert(ExternalForce::default())
        .insert_bundle(InputManagerBundle::<Action> {
            input_map,
            ..default()
        })
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(SpeedLimit(BASE_SPEED_LIMIT))
        .insert(Boost::default())
        .insert(TrickStatus::default())
        .insert(LastTrick::default())
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
            parent
                .spawn_bundle(TransformBundle {
                    local: Transform::from_translation(Vec3::new(-1.5, -0.5, 0.)),
                    ..default()
                })
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Collider::ball(1.1))
                .insert(ColliderDebugColor(Color::ORANGE))
                .insert(ColliderMassProperties::Density(0.0))
                .insert(Sensor)
                .insert(JumpWheel);
            parent
                .spawn_bundle(TransformBundle {
                    local: Transform::from_translation(Vec3::new(1.5, -0.5, 0.)),
                    ..default()
                })
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Collider::ball(1.1))
                .insert(ColliderDebugColor(Color::ORANGE))
                .insert(ColliderMassProperties::Density(0.0))
                .insert(Sensor)
                .insert(JumpWheel);
        });
}

fn player_movement(
    time: Res<Time>,
    mut query: Query<
        (
            &ActionState<Action>,
            &mut ExternalForce,
            &mut ExternalImpulse,
            &mut Velocity,
            &WheelsOnGround,
            &JumpWheelsOnGround,
            &mut JumpCooldown,
            &Transform,
        ),
        With<Player>,
    >,
    race_timer: Res<RaceTime>,
) {
    if race_timer.paused() {
        return;
    }

    for (
        action_state,
        mut force,
        mut impulse,
        mut velocity,
        wheels,
        jump_wheels,
        mut jump_cooldown,
        transform,
    ) in query.iter_mut()
    {
        force.force = Vec3::ZERO;

        if action_state.pressed(Action::Left) && **jump_wheels >= 1 {
            force.force = transform.rotation * -Vec3::X * DRIVE_FORCE;
        }
        if action_state.pressed(Action::Right) && **jump_wheels >= 1 {
            force.force = transform.rotation * Vec3::X * DRIVE_FORCE;
        }
        if action_state.pressed(Action::RotateLeft) {
            velocity.angvel += Vec3::Z * ROT_SPEED * time.delta_seconds();
        }
        if action_state.pressed(Action::RotateRight) {
            velocity.angvel += -Vec3::Z * ROT_SPEED * time.delta_seconds();
        }
        if action_state.just_pressed(Action::Jump) && **jump_wheels >= 1 && !**jump_cooldown {
            // We don't want a jump from an angled ramp to impart any impulse in the backwards
            // direction, slowing the player down.
            //
            // So use only the y component of the current rotation for the jump impulse.
            // But to enable "wall jumping," use the x component if we're close to vertical.
            //
            // An alternative to explore if this turns out to be janky would be to differentiate
            // between "tracks" and "ramps" and use the old jumping behavior on non-ramps.

            let up = transform.up();
            let deg = up.angle_between(Vec3::NEG_X).to_degrees();

            if (deg < 20. || deg > 340.) || (deg > 160. && deg < 200.) {
                impulse.impulse = Vec3::new(up.x.signum() * JUMP_IMPULSE, 0., 0.);
            } else {
                impulse.impulse = Vec3::new(0., up.y.signum() * JUMP_IMPULSE, 0.);
            }

            **jump_cooldown = true;
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
    wheel_query: Query<Entity, With<Wheel>>,
    jump_wheel_query: Query<Entity, With<JumpWheel>>,
    track_query: Query<Entity, With<Track>>,
    finish_line_query: Query<Entity, With<FinishLine>>,
    body_query: Query<Entity, With<Player>>,
    mut player_query: Query<
        (
            &mut WheelsOnGround,
            &mut JumpWheelsOnGround,
            &mut BonkStatus,
            &mut JumpCooldown,
        ),
        With<Player>,
    >,
    mut race_time: ResMut<RaceTime>,
    mut finished_event: EventWriter<FinishedEvent>,
    mut trick_text: ResMut<TrickText>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                let finish_line = finish_line_query.iter_many([e1, e2]).count() > 0;
                let track = track_query.iter_many([e1, e2]).count() > 0;
                let wheel = wheel_query.iter_many([e1, e2]).count() > 0;
                let jump_wheel = jump_wheel_query.iter_many([e1, e2]).count() > 0;
                let body = body_query.iter_many([e1, e2]).count() > 0;

                if jump_wheel && track {
                    for (_, mut wheels, _, mut jump_cooldown) in player_query.iter_mut() {
                        wheels.0 += 1;

                        if wheels.0 == 2 {
                            **jump_cooldown = false;
                        }
                    }
                }

                if wheel && track {
                    for (mut wheels, _, mut bonk, _) in player_query.iter_mut() {
                        wheels.0 += 1;

                        if wheels.0 == 2 {
                            // don't use **bonk, it will trigger change detection
                            if bonk.0 {
                                **bonk = false;
                            }
                        }
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
                    for (_, _, mut bonk, _) in player_query.iter_mut() {
                        // don't use **bonk, it will trigger change detection
                        if !bonk.0 {
                            **trick_text = "BONK!".to_string();
                            **bonk = true;
                        }
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                let track = track_query.iter_many([e1, e2]).count() > 0;
                let wheel = wheel_query.iter_many([e1, e2]).count() > 0;
                let jump_wheel = jump_wheel_query.iter_many([e1, e2]).count() > 0;

                if track && wheel {
                    for (mut wheels, _, _, _) in player_query.iter_mut() {
                        wheels.0 -= 1;
                    }
                }

                if track && jump_wheel {
                    for (_, mut wheels, _, _) in player_query.iter_mut() {
                        wheels.0 -= 1;
                    }
                }
            }
        }
    }
}

fn game_finished(mut events: EventReader<FinishedEvent>, mut state: ResMut<State<GameState>>) {
    if events.iter().count() > 0 {
        if get_leaderboard_credentials().is_some() {
            state.set(GameState::Leaderboard).unwrap();
        } else {
            state.set(GameState::GameOver).unwrap();
        }
    }
}

fn player_dampening(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &SpeedLimit, &JumpWheelsOnGround), With<Player>>,
) {
    for (mut velocity, speed_limit, wheels) in query.iter_mut() {
        let elapsed = time.delta_seconds();
        velocity.angvel *= 0.1f32.powf(elapsed);

        // clamp to speed limit
        if velocity.linvel.length() > **speed_limit && **wheels > 0 {
            velocity.linvel =
                (velocity.linvel * 0.1f32.powf(elapsed)).clamp_length_min(**speed_limit);
        }
    }
}

fn track_trick(
    time: Res<Time>,
    mut query: Query<
        (
            &mut TrickStatus,
            &mut LastTrick,
            &Velocity,
            &Transform,
            &WheelsOnGround,
            ChangeTrackers<WheelsOnGround>,
            &BonkStatus,
            &mut Boost,
        ),
        With<Player>,
    >,
    mut trick_text: ResMut<TrickText>,
    audio: Res<Audio>,
    game_audio: Res<AudioAssets>,
    audio_setting: Res<SfxSetting>,
) {
    for (
        mut trick_status,
        mut last_trick,
        velocity,
        transform,
        wheels,
        wheels_changed,
        bonk,
        mut boost,
    ) in query.iter_mut()
    {
        if **bonk {
            trick_status.reset();
        }

        if **wheels == 0 {
            // if we just left the ground, make a note of our starting
            // position so we can determine if we went forward or backward
            if wheels_changed.is_changed() {
                trick_status.start_x = transform.translation.x;
                trick_status.hang_time = 0.;
            }

            let elapsed = time.delta_seconds();
            let rot = velocity.angvel * elapsed;

            trick_status.rotation += rot.z;
            trick_status.hang_time += elapsed;

            if trick_status.rotation > TAU {
                trick_status.back_flips += 1;
                trick_status.rotation -= TAU;
            } else if trick_status.rotation < -TAU {
                trick_status.front_flips += 1;
                trick_status.rotation += TAU;
            }
        } else if wheels_changed.is_changed() {
            // if a wheel just hit the ground

            // round up the remainder of the rotation generously, because
            // the player may have launched from angled ramp. maybe we
            // should keep track of the launch angle?
            if trick_status.rotation > 260.0_f32.to_radians() {
                trick_status.back_flips += 1;
            } else if trick_status.rotation < -260.0_f32.to_radians() {
                trick_status.front_flips += 1;
            }

            let flips = trick_status.front_flips + trick_status.back_flips;

            if flips > 0 {
                let fakie = transform.translation.x < trick_status.start_x;

                let trick = Trick {
                    front_flips: trick_status.front_flips,
                    back_flips: trick_status.back_flips,
                    fakie,
                };

                let fresh_bonus = if trick != **last_trick { 1. } else { 0. };

                let boost_duration = BASE_BOOST_TIMER + (flips - 1) as f32 * 1. + fresh_bonus;

                boost.remaining += boost_duration;

                info!("boost +{} ({})", boost_duration, boost.remaining);

                **trick_text = ui::get_trick_text(&trick);

                **last_trick = trick.clone();

                audio.play_with_settings(
                    game_audio.trick.clone(),
                    PlaybackSettings::ONCE.with_volume(**audio_setting as f32 / 100.),
                );
            }

            trick_status.reset();
        }
    }
}

fn boost(time: Res<Time>, mut query: Query<(&mut Boost, &mut SpeedLimit), With<Player>>) {
    for (mut boost, mut speed_limit) in query.iter_mut() {
        if boost.remaining <= 0. {
            return;
        }

        if speed_limit.0 == BASE_SPEED_LIMIT {
            **speed_limit = BOOST_SPEED_LIMIT;
            info!("speed limit now {}", **speed_limit);
        }

        boost.remaining -= time.delta_seconds();
        if boost.remaining <= 0. {
            boost.remaining = 0.;
            **speed_limit = BASE_SPEED_LIMIT;
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

fn bonk_sound(
    audio: Res<Audio>,
    game_audio: Res<AudioAssets>,
    audio_setting: Res<SfxSetting>,
    bonk_query: Query<&BonkStatus, (Changed<BonkStatus>, With<Player>)>,
) {
    for bonk in &bonk_query {
        if **bonk {
            audio.play_with_settings(
                game_audio.bonk.clone(),
                PlaybackSettings::ONCE.with_volume(**audio_setting as f32 / 100.),
            );
        }
    }
}

fn death(
    query: Query<&Transform, With<Player>>,
    mut state: ResMut<State<GameState>>,
    mut race_time: ResMut<RaceTime>,
) {
    for transform in &query {
        if transform.translation.y < LAVA {
            race_time.pause();
            state.set(GameState::GameOver).unwrap();
        }
    }
}

fn reset_action(
    query: Query<&ActionState<Action>, With<Player>>,
    mut state: ResMut<State<GameState>>,
    mut race_time: ResMut<RaceTime>,
) {
    let action_state = query.single();
    if action_state.just_pressed(Action::Reset) {
        race_time.pause();
        state.set(GameState::GameOver).unwrap();
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
