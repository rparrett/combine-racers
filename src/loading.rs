use bevy::{
    prelude::*,
    render::{
        render_resource::{CachedPipelineState, PipelineCache},
        Render, RenderApp, RenderSet,
    },
};
use bevy_asset_loader::prelude::*;
use crossbeam_channel::Receiver;

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

#[derive(Resource)]
struct PipelineStatus(Receiver<bool>);

#[cfg(not(target_arch = "wasm32"))]
const EXPECTED_PIPELINES: usize = 14;
#[cfg(target_arch = "wasm32")]
const EXPECTED_PIPELINES: usize = 12;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = crossbeam_channel::bounded(1);

        app.insert_resource(PipelineStatus(rx));

        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Decorating),
        )
        .add_collection_to_loading_state::<_, GameAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading)
        .add_systems(
            Update,
            pipelines_done.run_if(in_state(GameState::Pipelines)),
        )
        .add_systems(OnExit(GameState::Pipelines), cleanup)
        .add_systems(OnEnter(GameState::Pipelines), setup_pipelines);

        let renderer_app = app.sub_app_mut(RenderApp);
        let mut done = false;
        renderer_app.add_systems(
            Render,
            (move |cache: Res<PipelineCache>| {
                if done {
                    return;
                }

                let ready = cache
                    .pipelines()
                    .filter(|pipeline| matches!(pipeline.state, CachedPipelineState::Ok(_)))
                    .count();

                debug!("pipelines ready: {}/{}", ready, EXPECTED_PIPELINES);

                if ready >= EXPECTED_PIPELINES {
                    let _ = tx.send(true);
                    done = true
                }
            })
            .in_set(RenderSet::Cleanup),
        );
    }
}

fn setup_pipelines(mut commands: Commands) {
    commands.spawn((
        PipelinesMarker,
        TextBundle::from_section("Loading Pipelines...".to_string(), TextStyle::default()),
    ));
}

fn pipelines_done(status: Res<PipelineStatus>, mut next_state: ResMut<NextState<GameState>>) {
    if status.0.try_recv().unwrap_or_default() {
        next_state.set(GameState::MainMenu);
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<PipelinesMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
