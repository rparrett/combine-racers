use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_pipelines_ready::{PipelinesReady, PipelinesReadyPlugin};

use crate::GameState;

pub struct LoadingPlugin;

#[derive(Component)]
pub struct PipelinesMarker;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "track_1.glb#Scene0")]
    pub track: Handle<Scene>,
    #[asset(path = "combine.glb#Scene0")]
    pub combine: Handle<Scene>,
    #[asset(path = "bg.png")]
    pub background: Handle<Image>,
    #[asset(path = "NanumPenScript-Tweaked.ttf")]
    pub font: Handle<Font>,
}
#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "7th-race-aiteru-sawato.ogg")]
    pub music: Handle<AudioSource>,
    #[asset(path = "combine-racers-321go.ogg")]
    pub three_two_one: Handle<AudioSource>,
    #[asset(path = "combine-racers-trick.ogg")]
    pub trick: Handle<AudioSource>,
    #[asset(path = "combine-racers-bonk.ogg")]
    pub bonk: Handle<AudioSource>,
}

#[cfg(not(target_arch = "wasm32"))]
const EXPECTED_PIPELINES: usize = 15;
#[cfg(target_arch = "wasm32")]
const EXPECTED_PIPELINES: usize = 13;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PipelinesReadyPlugin)
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .load_collection::<GameAssets>()
                    .load_collection::<AudioAssets>()
                    .continue_to_state(GameState::Decorating),
            )
            .add_systems(
                Update,
                (
                    check_pipelines.run_if(in_state(GameState::Pipelines)),
                    log_pipelines.run_if(resource_changed::<PipelinesReady>),
                ),
            )
            .add_systems(OnExit(GameState::Pipelines), cleanup)
            .add_systems(OnEnter(GameState::Pipelines), setup_pipelines);
    }
}

fn setup_pipelines(mut commands: Commands) {
    commands.spawn((
        PipelinesMarker,
        TextBundle::from_section("Loading Pipelines...".to_string(), TextStyle::default()),
    ));
}

fn check_pipelines(ready: Res<PipelinesReady>, mut next_state: ResMut<NextState<GameState>>) {
    if ready.get() >= EXPECTED_PIPELINES {
        next_state.set(GameState::MainMenu);
    }
}

fn log_pipelines(pipelines: Res<PipelinesReady>) {
    info!("Pipelines: {}/{}", pipelines.get(), EXPECTED_PIPELINES);
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<PipelinesMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
