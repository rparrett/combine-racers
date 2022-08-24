use bevy::prelude::*;

use crate::{GameAssets, GameState};

pub struct GameOverPlugin;
impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(spawn))
            .add_system_set(
                SystemSet::on_update(GameState::GameOver)
                    .with_system(buttons)
                    .with_system(play_again_button),
            )
            .add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(cleanup));
    }
}

#[derive(Component)]
struct GameOverMarker;

#[derive(Component)]
struct PlayAgainButton;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

fn spawn(mut commands: Commands, assets: Res<GameAssets>) {
    let title_text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 60.0,
        color: TEXT_COLOR,
    };
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

    let root = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(0.),
                    left: Val::Px(0.),
                    ..default()
                },
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(GameOverMarker)
        .id();

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
        .id();

    let title = commands
        .spawn_bundle(
            TextBundle::from_section("Game Over", title_text_style).with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
        )
        .id();

    let play_again = commands
        .spawn_bundle(ButtonBundle {
            style: button_style,
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Play Again",
                button_text_style.clone(),
            ));
        })
        .insert(PlayAgainButton)
        .id();

    commands.entity(root).push_children(&[container]);

    commands
        .entity(container)
        .push_children(&[title, play_again]);
}

fn buttons(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
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

fn play_again_button(
    mut state: ResMut<State<GameState>>,
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<PlayAgainButton>),
    >,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Clicked {
            state.set(GameState::MainMenu).unwrap();
        }
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<GameOverMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
