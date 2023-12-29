use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use bevy_ui_navigation::prelude::*;
use interpolation::Ease;

use crate::{AfterPhysics, Boost, GameAssets, GameSet, GameState, Player, RaceTime, Trick};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrickText>()
            .init_resource::<TrickTextTimer>()
            .add_systems(OnExit(GameState::MainMenu), setup)
            .add_systems(
                Update,
                (fade_trick_text, race_time, trick_text, boost_gauge)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                speedometer_text
                    .after(GameSet::Movement)
                    .run_if(in_state(GameState::Playing))
                    .in_set(AfterPhysics),
            )
            .add_systems(
                Update,
                fade_trick_text.run_if(in_state(GameState::Leaderboard)),
            )
            // Keep displaying game UI until the player is done mentally processing their failure
            // and finally presses that "play again" button.
            .add_systems(OnExit(GameState::Leaderboard), cleanup)
            .add_systems(OnExit(GameState::GameOver), cleanup);
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

pub const BOOST_PX_PER_SECOND: f32 = 30.;
pub const BOOST_NOTCH_PX: f32 = 2.;

#[derive(Component)]
pub struct GameUiMarker;
#[derive(Component)]
pub struct TrickTextMarker;
#[derive(Resource, Deref, DerefMut)]
pub struct TrickTextTimer(Timer);
impl Default for TrickTextTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2., TimerMode::Once))
    }
}
#[derive(Component)]
pub struct RaceTimeText;
#[derive(Component)]
pub struct SpeedometerText;
#[derive(Component)]
pub struct BoostLeftNode;
#[derive(Component)]
pub struct BoostRightNode;
#[derive(Resource, Default, Deref, DerefMut)]
pub struct TrickText(String);

fn setup(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(5.),
                    top: Val::Px(5.),
                    width: Val::Px(120.),
                    height: Val::Px(60.),
                    align_items: AlignItems::FlexEnd,
                    justify_content: JustifyContent::FlexStart,
                    ..Default::default()
                },
                ..default()
            },
            GameUiMarker,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "0.000",
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..Default::default()
                },
                RaceTimeText,
            ));
        });

    commands.spawn((
        TextBundle {
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
            .with_alignment(TextAlignment::Center),
            ..Default::default()
        },
        GameUiMarker,
        TrickTextMarker,
    ));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(5.),
                    bottom: Val::Px(5.),
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..default()
                    },
                    width: Val::Percent(100.),
                    height: Val::Px(60.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ..default()
            },
            GameUiMarker,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "0 kph",
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 60.0,
                            color: TITLE_TEXT,
                        },
                    ),
                    ..Default::default()
                },
                SpeedometerText,
            ));
        });
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(5.),
                    bottom: Val::Px(5.),
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..default()
                    },
                    width: Val::Percent(100.),
                    height: Val::Px(60.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ..default()
            },
            GameUiMarker,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::RowReverse,
                            margin: UiRect::right(Val::Px(70.)),
                            width: Val::Px(0.),
                            height: Val::Px(5.),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        ..default()
                    },
                    BoostLeftNode,
                ))
                .with_children(|parent| {
                    for _ in 0..10 {
                        parent.spawn(NodeBundle {
                            style: Style {
                                min_width: Val::Px(BOOST_PX_PER_SECOND - BOOST_NOTCH_PX),
                                margin: UiRect::left(Val::Px(BOOST_NOTCH_PX)),
                                height: Val::Percent(100.),
                                ..default()
                            },
                            background_color: BOOSTED_TEXT.into(),
                            ..default()
                        });
                    }
                });
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            margin: UiRect::left(Val::Px(70.)),
                            width: Val::Px(0.),
                            height: Val::Px(5.),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        ..default()
                    },
                    BoostRightNode,
                ))
                .with_children(|parent| {
                    for _ in 0..10 {
                        parent.spawn(NodeBundle {
                            style: Style {
                                min_width: Val::Px(BOOST_PX_PER_SECOND - BOOST_NOTCH_PX),
                                margin: UiRect::left(Val::Px(BOOST_NOTCH_PX)),
                                height: Val::Percent(100.),
                                ..default()
                            },
                            background_color: BOOSTED_TEXT.into(),
                            ..default()
                        });
                    }
                });
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
            text.sections[0].value = format!("{:.0} kph", (velocity.linvel.length() * 3.5).round());
            if boost.remaining > 0.0 {
                text.sections[0].style.color = BOOSTED_TEXT
            } else {
                text.sections[0].style.color = TITLE_TEXT
            }
        }
    }
}

fn boost_gauge(
    query: Query<&Boost, Changed<Boost>>,
    mut left_query: Query<&mut Style, (With<BoostLeftNode>, Without<BoostRightNode>)>,
    mut right_query: Query<&mut Style, (With<BoostRightNode>, Without<BoostLeftNode>)>,
) {
    for boost in query.iter() {
        for mut style in left_query.iter_mut() {
            style.width = Val::Px(boost.remaining * BOOST_PX_PER_SECOND);
        }

        for mut style in right_query.iter_mut() {
            style.width = Val::Px(boost.remaining * BOOST_PX_PER_SECOND);
        }
    }
}

pub fn buttons(
    mut interaction_query: Query<
        (&Interaction, &Focusable, &mut BackgroundColor),
        (Or<(Changed<Interaction>, Changed<Focusable>)>, With<Button>),
    >,
) {
    for (interaction, focusable, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
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
