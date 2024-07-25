use bevy::{
    audio::{AudioSink, Volume},
    pbr::{DirectionalLightShadowMap, ShadowFilteringMethod},
    prelude::*,
};
use bevy_alt_ui_navigation_lite::prelude::*;

use crate::{
    loading::{AudioAssets, GameAssets},
    settings::{MusicSetting, SfxSetting, ShadowSetting},
    ui::{buttons, BUTTON_TEXT, CONTAINER_BACKGROUND, NORMAL_BUTTON},
    GameState, MainCamera, MusicController,
};

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TipIndex>()
            .add_systems(OnEnter(GameState::MainMenu), setup_menu)
            .add_systems(OnExit(GameState::Pipelines), start_music)
            .add_systems(
                Update,
                (
                    sfx_volume,
                    music_volume,
                    shadow_changed,
                    button_actions,
                    buttons.after(NavRequestSystem),
                )
                    .run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnExit(GameState::MainMenu), cleanup_menu);
    }
}

#[derive(Component)]
struct MainMenuMarker;

#[derive(Component)]
struct PlayButton;
#[derive(Component)]
struct MusicSettingButton;
#[derive(Component)]
struct MusicSettingButtonText;
#[derive(Component)]
struct SfxSettingButton;

#[derive(Component)]
struct SfxSettingButtonText;
#[derive(Component)]
struct ShadowSettingButton;

#[derive(Component)]
struct ShadowSettingButtonText;
#[derive(Component)]
struct TipText;
#[derive(Resource, Default, Deref, DerefMut)]
struct TipIndex(usize);
impl TipIndex {
    fn next(&mut self) -> usize {
        let next = self.0;

        self.0 += 1;
        if self.0 > TIPS.len() - 1 {
            self.0 = 0;
        }

        next
    }
}

const TIPS: &[&str] = &[
    "Jump and rotate at the same time to do flips!",
    "Earn even more boost by doing a different trick than the last.",
    "Press escape or select to start over.",
    "Do a double flip for an even longer boost!",
    "Be careful not to bonk your head.",
    "Get a mega-boost by submitting a 5 star rating*",
];

fn setup_menu(
    mut commands: Commands,
    assets: Res<GameAssets>,
    sfx: Res<SfxSetting>,
    music: Res<MusicSetting>,
    shadow: Res<ShadowSetting>,
    mut tip_index: ResMut<TipIndex>,
) {
    info!("setup_menu");

    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(45.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 30.0,
        color: BUTTON_TEXT,
    };
    let title_text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 60.0,
        color: BUTTON_TEXT,
    };
    let subtitle_text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 40.0,
        color: BUTTON_TEXT,
    };

    let container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.)),
                    ..default()
                },
                background_color: CONTAINER_BACKGROUND.into(),
                ..default()
            },
            MainMenuMarker,
        ))
        .id();

    let title = commands
        .spawn(
            TextBundle::from_section("Combine-Racers", title_text_style).with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
        )
        .id();

    let play_button = commands
        .spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            Focusable::default(),
            MenuButton::Play,
            PlayButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Play", button_text_style.clone()));
        })
        .id();

    let audio_settings_title = commands
        .spawn(
            TextBundle::from_section("Audio", subtitle_text_style.clone()).with_style(Style {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            }),
        )
        .id();

    let sfx_button = commands
        .spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            Focusable::default(),
            MenuButton::Sfx,
            SfxSettingButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(format!("SFX {}%", **sfx), button_text_style.clone()),
                SfxSettingButtonText,
            ));
        })
        .id();

    let music_button = commands
        .spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            Focusable::default(),
            MenuButton::Music,
            MusicSettingButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(format!("Music {}%", **music), button_text_style.clone()),
                MusicSettingButtonText,
            ));
        })
        .id();

    let shadow_settings_title = commands
        .spawn(
            TextBundle::from_section("Shadows", subtitle_text_style).with_style(Style {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            }),
        )
        .id();

    let shadow_button = commands
        .spawn((
            ButtonBundle {
                style: button_style,
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            Focusable::default(),
            MenuButton::Shadow,
            ShadowSettingButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(format!("{}", *shadow), button_text_style),
                ShadowSettingButtonText,
            ));
        })
        .id();

    commands.entity(container).push_children(&[
        title,
        play_button,
        audio_settings_title,
        sfx_button,
        music_button,
        shadow_settings_title,
        shadow_button,
    ]);

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(40.),
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..default()
                    },
                    width: Val::Percent(100.),
                    height: Val::Px(50.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ..default()
            },
            MainMenuMarker,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        TIPS[tip_index.next()].to_owned(),
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 40.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..Default::default()
                },
                TipText,
            ));
        });
}

#[derive(Component)]
enum MenuButton {
    Play,
    Sfx,
    Music,
    Shadow,
}

// Seems like bevy-ui-navigation forces us to write this abomination of a megasystem
fn button_actions(
    buttons: Query<&MenuButton>,
    mut events: EventReader<NavEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut music_setting: ResMut<MusicSetting>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<SfxSettingButtonText>>,
        Query<&mut Text, With<MusicSettingButtonText>>,
        Query<&mut Text, With<ShadowSettingButtonText>>,
    )>,
    mut sfx_setting: ResMut<SfxSetting>,
    mut shadow_setting: ResMut<ShadowSetting>,
) {
    // Note: we have a closure here because the `buttons` query is mutable.
    // for immutable queries, you can use `.activated_in_query` which returns an iterator.
    // Do something when player activates (click, press "A" etc.) a `Focusable` button.

    for button in events.nav_iter().activated_in_query(&buttons) {
        match button {
            MenuButton::Play => {
                next_state.set(GameState::Playing);
            }
            MenuButton::Sfx => {
                if **sfx_setting == 0 {
                    **sfx_setting = 100;
                } else {
                    **sfx_setting -= 10;
                }

                for mut text in text_queries.p0().iter_mut() {
                    text.sections[0].value = format!("SFX {}%", **sfx_setting);
                }
            }
            MenuButton::Music => {
                if **music_setting == 0 {
                    **music_setting = 100;
                } else {
                    **music_setting -= 10;
                }

                for mut text in text_queries.p1().iter_mut() {
                    text.sections[0].value = format!("Music {}%", **music_setting);
                }
            }
            MenuButton::Shadow => {
                *shadow_setting = shadow_setting.next();

                for mut text in text_queries.p2().iter_mut() {
                    text.sections[0].value = format!("{}", *shadow_setting);
                }
            }
        }
    }
}

fn sfx_volume(mut commands: Commands, sfx_setting: Res<SfxSetting>, game_audio: Res<AudioAssets>) {
    // Do not run when SfxSetting is first added by SavePlugin
    if !sfx_setting.is_changed() || sfx_setting.is_added() {
        return;
    }

    commands.spawn(AudioBundle {
        source: game_audio.trick.clone(),
        settings: PlaybackSettings::DESPAWN.with_volume(Volume::new(**sfx_setting as f32 / 100.)),
    });
}

fn music_volume(
    music_setting: Res<MusicSetting>,
    music_query: Query<&AudioSink, With<MusicController>>,
) {
    // Do not run when MusicSetting is first added by SavePlugin
    if !music_setting.is_changed() || music_setting.is_added() {
        return;
    }

    for sink in &music_query {
        sink.set_volume(**music_setting as f32 / 100.)
    }
}

fn start_music(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    music_setting: Res<MusicSetting>,
) {
    commands.spawn((
        AudioBundle {
            source: audio_assets.music.clone(),
            settings: PlaybackSettings::LOOP
                .with_volume(Volume::new(**music_setting as f32 / 100.)),
        },
        MusicController,
    ));
}

fn shadow_changed(
    mut commands: Commands,
    shadow_setting: Res<ShadowSetting>,
    camera_query: Query<Entity, With<MainCamera>>,
    mut light_query: Query<&mut DirectionalLight>,
) {
    // Do run when ShadowSetting is first added by SavePlugin
    if !shadow_setting.is_changed() {
        return;
    }

    let mut light = light_query.single_mut();
    let camera_entity = camera_query.single();

    match *shadow_setting {
        ShadowSetting::None => {
            light.shadows_enabled = false;
            commands
                .entity(camera_entity)
                .insert(ShadowFilteringMethod::Hardware2x2);
            commands.insert_resource(DirectionalLightShadowMap { size: 256 });
        }
        ShadowSetting::Low => {
            light.shadows_enabled = true;
            commands
                .entity(camera_entity)
                .insert(ShadowFilteringMethod::Hardware2x2);
            commands.insert_resource(DirectionalLightShadowMap { size: 256 });
        }
        ShadowSetting::Medium => {
            light.shadows_enabled = true;
            commands
                .entity(camera_entity)
                .insert(ShadowFilteringMethod::Gaussian);
            commands.insert_resource(DirectionalLightShadowMap { size: 512 });
        }
        ShadowSetting::High => {
            light.shadows_enabled = true;
            commands
                .entity(camera_entity)
                .insert(ShadowFilteringMethod::Gaussian);
            commands.insert_resource(DirectionalLightShadowMap { size: 1024 });
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MainMenuMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
