use bevy::{audio::Volume, prelude::*};
use interpolation::Ease;

use crate::{loading::AudioAssets, settings::SfxSetting, ui::TrickTextMarker, GameState, RaceTime};

pub struct CountdownPlugin;
impl Plugin for CountdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup)
            .add_systems(Update, update.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), cleanup);
    }
}

#[derive(Component)]
struct CountdownTimer {
    countdown: Timer,
    go: Timer,
}

impl Default for CountdownTimer {
    fn default() -> Self {
        Self {
            countdown: Timer::from_seconds(3., TimerMode::Once),
            go: Timer::from_seconds(1., TimerMode::Once),
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(CountdownTimer::default());
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<&mut CountdownTimer>,
    mut text_query: Query<&mut Text, With<TrickTextMarker>>,
    mut race_time: ResMut<RaceTime>,
    game_audio: Res<AudioAssets>,
    audio_setting: Res<SfxSetting>,
) {
    for mut timer in query.iter_mut() {
        if !timer.countdown.finished() {
            if timer.countdown.elapsed_secs() == 0.0 {
                commands.spawn(AudioBundle {
                    source: game_audio.three_two_one.clone(),
                    settings: PlaybackSettings::DESPAWN
                        .with_volume(Volume::new(**audio_setting as f32 / 100.)),
                });
            }

            timer.countdown.tick(time.delta());

            for mut text in text_query.iter_mut() {
                let left =
                    timer.countdown.fraction_remaining() * timer.countdown.duration().as_secs_f32();

                text.sections[0].value = format!("{}", left.ceil());
                text.sections[0].style.color = Color::rgba(1., 0., 0., Ease::cubic_out(left % 1.));
            }

            if timer.countdown.just_finished() {
                for mut text in text_query.iter_mut() {
                    text.sections[0].value = "GO!".to_string();
                }
                timer.go.reset();
                race_time.unpause();
            }
        } else if !timer.go.finished() {
            timer.go.tick(time.delta());
            for mut text in text_query.iter_mut() {
                text.sections[0].style.color =
                    Color::rgba(1., 0., 0., Ease::cubic_out(timer.go.fraction_remaining()));
            }
        }
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<CountdownTimer>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
