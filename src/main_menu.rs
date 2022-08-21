//! This example illustrates how to use [`States`] to control transitioning from a `Menu` state to
//! an `InGame` state.

use bevy::prelude::*;

use crate::{GameAssets, GameState};

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::MainMenu).with_system(setup_menu))
            .add_system_set(SystemSet::on_update(GameState::MainMenu).with_system(menu))
            .add_system_set(SystemSet::on_exit(GameState::MainMenu).with_system(cleanup_menu));
    }
}

#[derive(Component)]
struct MainMenuMarker;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

fn setup_menu(mut commands: Commands, assets: Res<GameAssets>) {
    info!("setup_menu");

    // TODO move to startup system out of state
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 100.0),
        ..Default::default()
    });

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
        color: TEXT_COLOR,
    };
    let title_text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 60.0,
        color: TEXT_COLOR,
    };
    let subtitle_text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 40.0,
        color: TEXT_COLOR,
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
            TextBundle::from_section("Combine-Racers", title_text_style.clone()).with_style(
                Style {
                    margin: UiRect {
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    ..default()
                },
            ),
        )
        .id();

    let play_button = commands
        .spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section("Play", button_text_style.clone()));
        })
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
            parent.spawn_bundle(TextBundle::from_section(
                "QWERTY",
                button_text_style.clone(),
            ));
        })
        .id();

    let azerty_button = commands
        .spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "AZERTY",
                button_text_style.clone(),
            ));
        })
        .id();

    let audio_settings_title = commands
        .spawn_bundle(
            TextBundle::from_section("Audio", subtitle_text_style.clone()).with_style(Style {
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
            parent.spawn_bundle(TextBundle::from_section("SFX", button_text_style.clone()));
        })
        .id();

    let music_button = commands
        .spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section("Music", button_text_style.clone()));
        })
        .id();

    commands.entity(container).push_children(&[
        title,
        play_button,
        keyboard_settings_title,
        qwerty_button,
        azerty_button,
        audio_settings_title,
        sfx_button,
        music_button,
    ]);
}

fn menu(
    mut state: ResMut<State<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                state.set(GameState::Playing).unwrap();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MainMenuMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
