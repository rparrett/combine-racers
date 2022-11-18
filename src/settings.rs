use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Default, Debug, Deref, DerefMut, Serialize, Deserialize, Clone)]
pub struct KeyboardSetting(KeyboardLayout);
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum KeyboardLayout {
    Qwerty,
    Azerty,
}
impl Default for KeyboardLayout {
    fn default() -> Self {
        Self::Qwerty
    }
}
impl std::fmt::Display for KeyboardLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Qwerty => write!(f, "QWERTY"),
            Self::Azerty => write!(f, "AZERTY"),
        }
    }
}

#[derive(Resource, Deref, DerefMut, Debug, Serialize, Deserialize, Clone)]
pub struct MusicSetting(u8);
impl Default for MusicSetting {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Resource, Deref, DerefMut, Debug, Serialize, Deserialize, Clone)]
pub struct SfxSetting(u8);
impl Default for SfxSetting {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Resource, Default, Deref, DerefMut, Debug, Serialize, Deserialize, Clone)]
pub struct LeaderboardSetting(pub Option<bevy_jornet::Player>);
