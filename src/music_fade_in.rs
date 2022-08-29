use bevy::{audio::AudioSink, prelude::*};
use interpolation::{Ease, Lerp};

use crate::{settings::MusicSetting, AudioAssets, GameState, MusicController};

/// Fades in music at the start of the game. This is accomplished in a peculiar way in order to
/// work around an issue on the web where the audio buffer processing is seemingly paused while
/// the main thread stalls and then races to catch up, resulting in very funky sounding music.
///
/// So we start the music immediately, but fade in from an inaudible volume and hopefully the
/// situation has resolved itself by the time the player starts hearing the music.
pub struct MusicFadeInPlugin;
impl Plugin for MusicFadeInPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicFadeTimer>()
            .add_system_set(SystemSet::on_enter(GameState::Decorating).with_system(start_music))
            .add_system_set(SystemSet::on_update(GameState::MainMenu).with_system(fade_music))
            .add_system_set(SystemSet::on_update(GameState::Playing).with_system(fade_music));
    }
}

const FADE_IN_TIME: f32 = 4.;
const FADE_IN_SILENCE: f32 = 3.;
const SILENCE_VOLUME: f32 = 0.01;

#[derive(Deref, DerefMut)]
pub struct MusicFadeTimer(Timer);
impl Default for MusicFadeTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(10., false))
    }
}

fn start_music(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    audio_sinks: Res<Assets<AudioSink>>,
    audio: Res<Audio>,
) {
    let handle = audio_sinks.get_handle(audio.play_with_settings(
        audio_assets.music.clone(),
        PlaybackSettings::LOOP.with_volume(0.),
    ));
    commands.insert_resource(MusicController(handle));
}

fn fade_music(
    music_setting: Res<MusicSetting>,
    audio_sinks: Res<Assets<AudioSink>>,
    controller: Option<Res<MusicController>>,
    mut timer: ResMut<MusicFadeTimer>,
    time: Res<Time>,
) {
    // Stop fading if player has interacted with the music settings
    if music_setting.is_changed() && !music_setting.is_added() {
        let dur = timer.duration();
        timer.set_elapsed(dur);
        return;
    }

    if let Some(controller) = controller {
        if let Some(sink) = audio_sinks.get(&controller.0) {
            timer.tick(time.delta());
            if !timer.finished() {
                let timer_pct_silence = FADE_IN_SILENCE / FADE_IN_TIME;
                let timer_pct_fade = 1. - timer_pct_silence;

                let to = **music_setting as f32 / 100.;
                let pct = (timer.percent() - timer_pct_silence).max(0.) / timer_pct_fade;

                let vol = SILENCE_VOLUME.lerp(&to, &Ease::quadratic_in(pct));

                sink.set_volume(vol);
            } else if timer.just_finished() {
                sink.set_volume(**music_setting as f32 / 100.);
            }
        }
    }
}
