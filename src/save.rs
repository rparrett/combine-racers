use crate::{
    settings::{LeaderboardSetting, MusicSetting, SfxSetting, ShadowSetting},
    GameState,
};

use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
const SAVE_FILE: &str = "save.ron";
#[cfg(target_arch = "wasm32")]
const LOCAL_STORAGE_KEY: &str = "combine-racers-save";

pub struct SavePlugin;
impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, save_system);
        app.add_systems(OnEnter(GameState::Loading), load_system);
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct SaveFile {
    sfx: SfxSetting,
    music: MusicSetting,
    leaderboard: LeaderboardSetting,
    shadow: ShadowSetting,
}

pub fn load_system(mut commands: Commands) {
    commands.insert_resource(SfxSetting::default());
    commands.insert_resource(MusicSetting::default());
    commands.insert_resource(LeaderboardSetting::default());
    commands.insert_resource(ShadowSetting::default());

    #[cfg(not(target_arch = "wasm32"))]
    {
        let file = match std::fs::File::open(SAVE_FILE) {
            Ok(f) => f,
            Err(_) => return,
        };

        let save_file: SaveFile = match ron::de::from_reader(file) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to deserialize save file: {:?}", e);
                return;
            }
        };

        info!("Loaded settings: {:?}", save_file);

        commands.insert_resource(save_file.sfx);
        commands.insert_resource(save_file.music);
        commands.insert_resource(save_file.leaderboard);
        commands.insert_resource(save_file.shadow);
    }
    #[cfg(target_arch = "wasm32")]
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        let storage = match window.local_storage() {
            Ok(Some(s)) => s,
            _ => return,
        };

        let item = match storage.get_item(LOCAL_STORAGE_KEY) {
            Ok(Some(i)) => i,
            _ => return,
        };

        let save_file: SaveFile = match ron::de::from_str(&item) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to serialize save file: {:?}", e);
                return;
            }
        };

        info!("Loaded settings: {:?}", save_file);

        commands.insert_resource(save_file.sfx);
        commands.insert_resource(save_file.music);
        commands.insert_resource(save_file.leaderboard);
        commands.insert_resource(save_file.shadow);
    }
}

pub fn save_system(
    sfx: Res<SfxSetting>,
    music: Res<MusicSetting>,
    leaderboard: Res<LeaderboardSetting>,
    shadow: Res<ShadowSetting>,
) {
    let sfx_changed = sfx.is_changed() && !sfx.is_added();
    let music_changed = music.is_changed() && !music.is_added();
    let leaderboard_changed = leaderboard.is_changed() && !leaderboard.is_added();
    let shadow_changed = shadow.is_changed() && !shadow.is_added();

    if !sfx_changed && !music_changed && !leaderboard_changed && !shadow_changed {
        return;
    }

    info!("Saving settings.");

    let save_file = SaveFile {
        sfx: sfx.clone(),
        music: music.clone(),
        leaderboard: leaderboard.clone(),
        shadow: shadow.clone(),
    };

    let pretty = PrettyConfig::new();

    #[cfg(not(target_arch = "wasm32"))]
    {
        let file = match std::fs::File::create(SAVE_FILE) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to create save file: {:?}", e);
                return;
            }
        };

        if let Err(e) = ron::ser::to_writer_pretty(file, &save_file, pretty) {
            warn!("Failed to serialize save data: {:?}", e);
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let data = match ron::ser::to_string_pretty(&save_file, pretty) {
            Ok(d) => d,
            Err(e) => {
                warn!("Failed to serialize save data: {:?}", e);
                return;
            }
        };

        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        let storage = match window.local_storage() {
            Ok(Some(s)) => s,
            _ => return,
        };

        if let Err(e) = storage.set_item(LOCAL_STORAGE_KEY, data.as_str()) {
            warn!("Failed to store save file: {:?}", e);
        }
    }
}
