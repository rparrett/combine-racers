use crate::{
    settings::{LeaderboardSetting, MusicSetting, SfxSetting, ShadowSetting},
    GameState,
};

use bevy::prelude::*;
use bevy_simple_prefs::{Prefs, PrefsPlugin};

pub struct SavePlugin;
impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PrefsPlugin::<SaveFile> {
            #[cfg(not(target_arch = "wasm32"))]
            filename: "save.ron".to_string(),
            #[cfg(target_arch = "wasm32")]
            filename: "combine-racers-save".to_string(),
            ..default()
        });
    }
}

#[derive(Prefs, Reflect, Default)]
struct SaveFile {
    sfx: SfxSetting,
    music: MusicSetting,
    leaderboard: LeaderboardSetting,
    shadow: ShadowSetting,
}
