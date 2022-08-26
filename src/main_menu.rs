use bevy::{audio::AudioSink, prelude::*};
use bevy_ui_navigation::prelude::*;

use crate::{
    settings::{KeyboardLayout, KeyboardSetting, MusicSetting, SfxSetting},
    ui::{buttons, BUTTON_TEXT, NORMAL_BUTTON},
    AudioAssets, GameAssets, GameState, MusicController,
};

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::MainMenu).with_system(setup_menu))
            .add_system_set(
                SystemSet::on_update(GameState::MainMenu)
                    .with_system(sfx_volume)
                    .with_system(music_volume)
                    .with_system(button_actions)
                    .with_system(buttons.after(NavRequestSystem)),
            )
            .add_system_set(SystemSet::on_exit(GameState::MainMenu).with_system(cleanup_menu));
    }
}

#[derive(Component)]
struct MainMenuMarker;

#[derive(Component)]
struct PlayButton;
#[derive(Component)]
struct KeyboardSettingButton;

#[derive(Component)]
struct KeyboardSettingButtonText;
#[derive(Component)]
struct MusicSettingButton;
#[derive(Component)]
struct MusicSettingButtonText;
#[derive(Component)]
struct SfxSettingButton;

#[derive(Component)]
struct SfxSettingButtonText;

fn setup_menu(
    mut commands: Commands,
    assets: Res<GameAssets>,
    sfx: Res<SfxSetting>,
    music: Res<MusicSetting>,
    keyboard: Res<KeyboardSetting>,
) {
    info!("setup_menu");

    let button_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(45.0)),
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
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.)),
                ..default()
            },
            color: Color::rgb(0.1, 0.1, 0.1).into(),
            ..default()
        })
        .insert(MainMenuMarker)
        .id();

    let title = commands
        .spawn_bundle(
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
        .spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .insert(Focusable::default())
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section("Play", button_text_style.clone()));
        })
        .insert(MenuButton::Play)
        .insert(PlayButton)
        .id();

    let keyboard_settings_title = commands
        .spawn_bundle(
            TextBundle::from_section("Keyboard", subtitle_text_style.clone()).with_style(Style {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            }),
        )
        .id();

    let qwerty_button = commands
        .spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle::from_section(
                    format!("{}", **keyboard),
                    button_text_style.clone(),
                ))
                .insert(KeyboardSettingButtonText);
        })
        .insert(Focusable::default())
        .insert(MenuButton::Keyboard)
        .insert(KeyboardSettingButton)
        .id();

    let audio_settings_title = commands
        .spawn_bundle(
            TextBundle::from_section("Audio", subtitle_text_style).with_style(Style {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            }),
        )
        .id();

    let sfx_button = commands
        .spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle::from_section(
                    format!("SFX {}%", **sfx),
                    button_text_style.clone(),
                ))
                .insert(SfxSettingButtonText);
        })
        .insert(Focusable::default())
        .insert(MenuButton::Sfx)
        .insert(SfxSettingButton)
        .id();

    let music_button = commands
        .spawn_bundle(ButtonBundle {
            style: button_style,
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .insert(Focusable::default())
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle::from_section(
                    format!("Music {}%", **music),
                    button_text_style,
                ))
                .insert(MusicSettingButtonText);
        })
        .insert(MenuButton::Music)
        .insert(MusicSettingButton)
        .id();

    commands.entity(container).push_children(&[
        title,
        play_button,
        keyboard_settings_title,
        qwerty_button,
        audio_settings_title,
        sfx_button,
        music_button,
    ]);
}

#[derive(Component)]
enum MenuButton {
    Play,
    Keyboard,
    Sfx,
    Music,
}

// Seems like bevy-ui-navigation forces us to write this abomination of a megasystem
fn button_actions(
    buttons: Query<&MenuButton>,
    mut events: EventReader<NavEvent>,
    mut state: ResMut<State<GameState>>,
    mut keyboard_setting: ResMut<KeyboardSetting>,

    mut music_setting: ResMut<MusicSetting>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<KeyboardSettingButtonText>>,
        Query<&mut Text, With<SfxSettingButtonText>>,
        Query<&mut Text, With<MusicSettingButtonText>>,
    )>,
    mut sfx_setting: ResMut<SfxSetting>,
) {
    // Note: we have a closure here because the `buttons` query is mutable.
    // for immutable queries, you can use `.activated_in_query` which returns an iterator.
    // Do something when player activates (click, press "A" etc.) a `Focusable` button.

    for button in events.nav_iter().activated_in_query(&buttons) {
        match button {
            MenuButton::Play => {
                state.set(GameState::Playing).unwrap();
            }
            MenuButton::Keyboard => {
                **keyboard_setting = match **keyboard_setting {
                    KeyboardLayout::Azerty => KeyboardLayout::Qwerty,
                    KeyboardLayout::Qwerty => KeyboardLayout::Azerty,
                };

                for mut text in text_queries.p0().iter_mut() {
                    text.sections[0].value = format!("{}", **keyboard_setting);
                }
            }
            MenuButton::Sfx => {
                if **sfx_setting == 0 {
                    **sfx_setting = 100;
                } else {
                    **sfx_setting -= 10;
                }

                for mut text in text_queries.p1().iter_mut() {
                    text.sections[0].value = format!("SFX {}%", **sfx_setting);
                }
            }
            MenuButton::Music => {
                if **music_setting == 0 {
                    **music_setting = 100;
                } else {
                    **music_setting -= 10;
                }

                for mut text in text_queries.p2().iter_mut() {
                    text.sections[0].value = format!("Music {}%", **music_setting);
                }
            }
        }
    }
}

fn sfx_volume(sfx_setting: Res<SfxSetting>, audio: Res<Audio>, game_audio: Res<AudioAssets>) {
    // Do not run when SfxSetting is first added by SavePlugin
    if !sfx_setting.is_changed() || sfx_setting.is_added() {
        return;
    }

    audio.play_with_settings(
        game_audio.trick.clone(),
        PlaybackSettings::ONCE.with_volume(**sfx_setting as f32 / 100.),
    );
}

fn music_volume(
    music_setting: Res<MusicSetting>,
    audio_sinks: Res<Assets<AudioSink>>,
    controller: Option<Res<MusicController>>,
) {
    // Do not run when MusicSetting is first added by SavePlugin
    if !music_setting.is_changed() || music_setting.is_added() {
        return;
    }

    if let Some(controller) = controller {
        if let Some(sink) = audio_sinks.get(&controller.0) {
            sink.set_volume(**music_setting as f32 / 100.)
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MainMenuMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
