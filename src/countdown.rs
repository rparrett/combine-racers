use bevy::prelude::*;
use interpolation::Ease;

use crate::{settings::SfxSetting, ui::TrickTextMarker, AudioAssets, GameState, RaceTime};

pub struct CountdownPlugin;
impl Plugin for CountdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(GameState::Playing)))
            .add_system(update.in_set(OnUpdate(GameState::Playing)))
            .add_system(cleanup.in_schedule(OnExit(GameState::Playing)));
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
    time: Res<Time>,
    mut query: Query<&mut CountdownTimer>,
    mut text_query: Query<&mut Text, With<TrickTextMarker>>,
    mut race_time: ResMut<RaceTime>,
    audio: Res<Audio>,
    game_audio: Res<AudioAssets>,
    audio_setting: Res<SfxSetting>,
) {
    for mut timer in query.iter_mut() {
        if !timer.countdown.finished() {
            if timer.countdown.elapsed_secs() == 0.0 {
                audio.play_with_settings(
                    game_audio.three_two_one.clone(),
                    PlaybackSettings::ONCE.with_volume(**audio_setting as f32 / 100.),
                );
            }

            timer.countdown.tick(time.delta());

            for mut text in text_query.iter_mut() {
                let left =
                    timer.countdown.percent_left() * timer.countdown.duration().as_secs_f32();

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
                    Color::rgba(1., 0., 0., Ease::cubic_out(timer.go.percent_left()));
            }
        }
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<CountdownTimer>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
