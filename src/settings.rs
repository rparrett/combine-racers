use bevy::prelude::*;
use std::fmt::Display;

#[derive(Resource, Deref, DerefMut, Debug, Clone, Reflect)]
pub struct MusicSetting(u8);
impl Default for MusicSetting {
    fn default() -> Self {
        Self(50)
    }
}

#[derive(Resource, Deref, DerefMut, Debug, Clone, Reflect)]
pub struct SfxSetting(u8);
impl Default for SfxSetting {
    fn default() -> Self {
        Self(50)
    }
}

#[derive(Resource, Debug, Clone, Default, Reflect)]
pub enum ShadowSetting {
    None,
    Low,
    #[default]
    Medium,
    High,
}
impl ShadowSetting {
    pub fn next(&self) -> Self {
        match self {
            Self::None => Self::Low,
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::None,
        }
    }
}
impl Display for ShadowSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "None",
                Self::Low => "Low",
                Self::Medium => "Medium",
                Self::High => "High",
            }
        )
    }
}

#[derive(Resource, Default, Deref, DerefMut, Debug, Clone, Reflect)]
pub struct LeaderboardSetting(pub Option<bevy_jornet::Player>);
