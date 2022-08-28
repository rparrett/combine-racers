use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use bevy_ui_navigation::prelude::*;
use interpolation::Ease;

use crate::{Boost, GameAssets, GameState, Player, RaceTime, Trick};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrickText>()
            .init_resource::<TrickTextTimer>()
            .add_system_set(SystemSet::on_exit(GameState::MainMenu).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(fade_trick_text)
                    .with_system(race_time)
                    .with_system(speedometer_text)
                    .with_system(trick_text),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Leaderboard).with_system(fade_trick_text),
            )
            // Keep displaying game UI until the player is done mentally processing their failure
            // and finally presses that "play again" button.
            .add_system_set(SystemSet::on_exit(GameState::Leaderboard).with_system(cleanup))
            .add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(cleanup));
    }
}

pub const FOCUSED_BUTTON: Color = Color::rgb(0.25, 0.0, 0.25);
pub const FOCUSED_HOVERED_BUTTON: Color = Color::rgb(0.35, 0.0, 0.35);
pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
pub const BUTTON_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);
pub const TITLE_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);
pub const BOOSTED_TEXT: Color = Color::rgb(0.55, 0.0, 0.55);
pub const OUR_SCORE_TEXT: Color = Color::rgb(0.55, 0.0, 0.55);
pub const CONTAINER_BACKGROUND: Color = Color::rgb(0.1, 0.1, 0.1);

#[derive(Component)]
pub struct GameUiMarker;
#[derive(Component)]
pub struct TrickTextMarker;
#[derive(Deref, DerefMut)]
pub struct TrickTextTimer(Timer);
impl Default for TrickTextTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2., false))
    }
}
#[derive(Component)]
pub struct RaceTimeText;
#[derive(Component)]
pub struct SpeedometerText;
#[derive(Default, Deref, DerefMut)]
pub struct TrickText(String);

fn setup(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(5.),
                    top: Val::Px(5.),
                    ..default()
                },
                size: Size {
                    width: Val::Px(120.),
                    height: Val::Px(60.),
                },
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::FlexStart,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(GameUiMarker)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::from_section(
                        "0.000",
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..Default::default()
                })
                .insert(RaceTimeText);
        });

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    bottom: Val::Auto,
                    top: Val::Px(120.),
                },
                align_self: AlignSelf::Center,
                ..Default::default()
            },
            text: Text::from_section(
                "",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 60.0,
                    color: Color::NONE,
                },
            )
            .with_alignment(TextAlignment::CENTER),
            ..Default::default()
        })
        .insert(GameUiMarker)
        .insert(TrickTextMarker);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(5.),
                    bottom: Val::Px(5.),
                    ..default()
                },
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    ..default()
                },
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Px(60.),
                },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(GameUiMarker)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::from_section(
                        "0%",
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..Default::default()
                })
                .insert(SpeedometerText);
        });
}

fn fade_trick_text(
    time: Res<Time>,
    mut timer: ResMut<TrickTextTimer>,
    mut query: Query<&mut Text, With<TrickTextMarker>>,
) {
    timer.tick(time.delta());
    if !timer.finished() {
        for mut text in query.iter_mut() {
            text.sections[0].style.color =
                Color::rgba(1., 0., 0., Ease::cubic_out(timer.percent_left()))
        }
    } else if timer.just_finished() {
        for mut text in query.iter_mut() {
            text.sections[0].style.color = Color::rgba(1., 0., 0., 0.)
        }
    }
}

pub fn get_trick_text(trick: &Trick) -> String {
    fn num_text(num: u32) -> Option<&'static str> {
        match num {
            0 | 1 => None,
            2 => Some("Double"),
            3 => Some("Triple"),
            4 => Some("Quad"),
            _ => Some("Mega"),
        }
    }

    let mut lines = vec![];

    if trick.front_flips > 0 {
        let mut parts = vec![];

        if trick.fakie {
            parts.push("Fakie");
        }

        if let Some(num) = num_text(trick.front_flips) {
            parts.push(num);
        }

        parts.push("Front Flip!");

        lines.push(parts.join(" "));
    }
    if trick.back_flips > 0 {
        let mut parts = vec![];

        if trick.fakie {
            parts.push("Fakie");
        }

        if let Some(num) = num_text(trick.back_flips) {
            parts.push(num);
        }

        parts.push("Back Flip!");

        lines.push(parts.join(" "));
    }

    lines.join("\n")
}

fn race_time(time: Res<RaceTime>, mut query: Query<&mut Text, With<RaceTimeText>>) {
    if !time.is_changed() {
        return;
    }

    for mut text in query.iter_mut() {
        text.sections[0].value = format!("{:.3}", time.elapsed_secs());
    }
}

fn trick_text(
    mut timer: ResMut<TrickTextTimer>,
    mut text_node: Query<&mut Text, With<TrickTextMarker>>,
    text: Res<TrickText>,
) {
    if !text.is_changed() {
        return;
    }

    for mut node in text_node.iter_mut() {
        node.sections[0].value.clone_from(&**text);
        node.sections[0].style.color = Color::rgba(1., 0., 0., 1.)
    }

    timer.reset();
}

fn speedometer_text(
    query: Query<(&Velocity, &Boost), With<Player>>,
    mut text_query: Query<&mut Text, With<SpeedometerText>>,
) {
    for (velocity, boost) in query.iter() {
        for mut text in text_query.iter_mut() {
            text.sections[0].value = format!("{:.0} kph", (velocity.linvel.length() * 3.5).floor());
            if boost.remaining > 0.0 {
                text.sections[0].style.color = BOOSTED_TEXT
            } else {
                text.sections[0].style.color = TITLE_TEXT
            }
        }
    }
}

pub fn buttons(
    mut interaction_query: Query<
        (&Interaction, &Focusable, &mut UiColor),
        (Or<(Changed<Interaction>, Changed<Focusable>)>, With<Button>),
    >,
) {
    for (interaction, focusable, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                if matches!(focusable.state(), FocusState::Focused) {
                    *color = FOCUSED_HOVERED_BUTTON.into()
                } else {
                    *color = HOVERED_BUTTON.into();
                };
            }
            Interaction::None => {
                if matches!(focusable.state(), FocusState::Focused) {
                    *color = FOCUSED_BUTTON.into()
                } else {
                    *color = NORMAL_BUTTON.into();
                };
            }
        }
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<GameUiMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
