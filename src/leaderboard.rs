use bevy::prelude::*;
use bevy_jornet::{JornetPlugin, Leaderboard, LeaderboardEvent};

use crate::{GameAssets, GameState, RaceTime};

pub struct LeaderboardPlugin;
impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScoreSaved>();

        if let (Some(id), Some(key)) = (
            option_env!("JORNET_LEADERBOARD_ID"),
            option_env!("JORNET_LEADERBOARD_KEY"),
        ) {
            app.init_resource::<Refreshing>()
                .init_resource::<RefreshTimer>()
                .add_plugin(JornetPlugin::with_leaderboard(id, key))
                .add_system_set(SystemSet::on_enter(GameState::Loading).with_system(create_player))
                .add_system_set(
                    SystemSet::on_enter(GameState::Leaderboard)
                        .with_system(save_score)
                        .with_system(spawn_leaderboard),
                )
                .add_system_set(
                    SystemSet::on_update(GameState::Leaderboard)
                        .with_system(initiate_refresh)
                        .with_system(update_leaderboard)
                        .with_system(buttons)
                        .with_system(play_again_button),
                )
                .add_system_set(SystemSet::on_exit(GameState::Leaderboard).with_system(cleanup));
        }
    }
}

#[derive(Default, Deref, DerefMut)]
struct ScoreSaved(bool);

#[derive(Deref, DerefMut)]
struct RefreshTimer(Timer);
impl Default for RefreshTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(3., false))
    }
}

#[derive(Deref, DerefMut)]
struct Refreshing(bool);
impl Default for Refreshing {
    fn default() -> Self {
        Self(true)
    }
}
#[derive(Component)]
struct LoadingText;
#[derive(Component)]
struct LeaderboardMarker;
#[derive(Component)]
struct ScoresContainer;

#[derive(Component)]
struct PlayAgainButton;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

fn initiate_refresh(leaderboard: Res<Leaderboard>, mut events: EventReader<LeaderboardEvent>) {
    if !events
        .iter()
        .any(|e| matches!(*e, LeaderboardEvent::SendScoreSucceeded))
    {
        return;
    }

    info!("score sending succeeded. refreshing leaderboard.");

    leaderboard.refresh_leaderboard();
}

fn update_leaderboard(
    mut commands: Commands,
    leaderboard: Res<Leaderboard>,
    time: Res<RaceTime>,
    container_query: Query<Entity, With<ScoresContainer>>,
    loading_text_query: Query<Entity, With<LoadingText>>,
    assets: Res<GameAssets>,
    mut events: EventReader<LeaderboardEvent>,
) {
    if !events
        .iter()
        .any(|e| matches!(*e, LeaderboardEvent::RefreshLeaderboardSucceeded))
    {
        return;
    }

    info!("update_leaderboard");

    for entity in loading_text_query.iter() {
        // I am not sure why this needs to be despawn_recursive, but we panic without it.
        // Is despawn_recursive better named `hierarchy_aware_despawn` or something? The
        // LoadingText itself has no children...
        commands.entity(entity).despawn_recursive();
    }

    if let Some(player) = leaderboard.get_player() {
        let container = container_query.single();
        commands.entity(container).despawn_descendants();

        let leaderboard = leaderboard.get_leaderboard();

        // TODO check if leaderboard is empty, which seems to happen occasionally. Spawn a
        // message about that.

        let has_us = leaderboard
            .iter()
            .any(|score| player.name == score.player && score.score == -time.elapsed_secs());

        for (i, score) in leaderboard.iter().enumerate() {
            // When we have a fresh leaderboard (when not refreshing), we assume
            // that our score would be included if it were high enough. So if we
            // haven't already displayed our score, toss it in at the last position.
            let (display_score, display_name, is_us, rank) =
                if !has_us && i == leaderboard.len() - 1 {
                    (time.elapsed_secs(), &player.name, true, "?".to_string())
                } else {
                    let is_us = player.name == score.player && score.score == -time.elapsed_secs();

                    (-score.score, &score.player, is_us, format!("{}", i + 1))
                };

            let row = commands
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(30.),
                            ..default()
                        },
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                })
                .id();

            let rank_text = commands
                .spawn_bundle(TextBundle {
                    text: Text::from_section(
                        rank,
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 30.,
                            color: if is_us { Color::PURPLE } else { Color::WHITE },
                        },
                    ),
                    style: Style {
                        size: Size {
                            width: Val::Px(50.),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                })
                .id();

            let name_text = commands
                .spawn_bundle(TextBundle {
                    text: Text::from_section(
                        display_name,
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 30.,
                            color: if is_us { Color::PURPLE } else { Color::WHITE },
                        },
                    ),
                    style: Style {
                        size: Size {
                            width: Val::Px(300.),
                            ..default()
                        },
                        overflow: Overflow::Hidden,
                        ..default()
                    },
                    ..default()
                })
                .id();

            let score_text = commands
                .spawn_bundle(TextBundle {
                    text: Text::from_section(
                        format!("{:.3}", display_score),
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 30.,
                            color: if is_us { Color::PURPLE } else { Color::WHITE },
                        },
                    ),
                    ..default()
                })
                .id();

            commands
                .entity(row)
                .push_children(&[rank_text, name_text, score_text]);

            commands.entity(container).add_child(row);
        }
    }
}

fn spawn_leaderboard(mut commands: Commands, assets: Res<GameAssets>) {
    info!("spawn_leaderboard");

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
        .insert(LeaderboardMarker)
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
            TextBundle::from_section("Leaderboard", title_text_style).with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
        )
        .id();

    let loading = commands
        .spawn_bundle(
            TextBundle::from_section(
                "Loading...",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 30.0,
                    color: TEXT_COLOR,
                },
            )
            .with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(LoadingText)
        .id();

    let scores_container = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                margin: UiRect {
                    bottom: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(ScoresContainer)
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
        .push_children(&[title, loading, scores_container, play_again]);
}

fn create_player(mut leaderboard: ResMut<Leaderboard>) {
    leaderboard.create_player(None);
}

fn save_score(race_time: Res<RaceTime>, leaderboard: Res<Leaderboard>) {
    leaderboard.send_score(-race_time.elapsed_secs());
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
            state.set(GameState::Playing).unwrap();
        }
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<LeaderboardMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
