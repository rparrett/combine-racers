use std::cmp::Ordering;

use bevy::prelude::*;
use bevy_jornet::{
    CreatePlayerEvent, JornetPlugin, Leaderboard, RefreshLeaderboardEvent, SendScoreEvent,
};
use bevy_ui_navigation::prelude::*;

use crate::{
    random_name::random_name,
    settings::LeaderboardSetting,
    ui::{buttons, BUTTON_TEXT, CONTAINER_BACKGROUND, NORMAL_BUTTON, OUR_SCORE_TEXT, TITLE_TEXT},
    GameAssets, GameState, RaceTime,
};

pub struct LeaderboardPlugin;
impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        if let Some((id, key)) = get_leaderboard_credentials() {
            app.init_resource::<ScoreSaved>()
                .init_resource::<Refreshing>()
                .init_resource::<RefreshTimer>()
                .add_plugin(JornetPlugin::with_leaderboard(id, key))
                .add_system(save_leaderboard_setting)
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
                        .with_system(button_actions)
                        .with_system(buttons.after(NavRequestSystem)),
                )
                .add_system_set(SystemSet::on_exit(GameState::Leaderboard).with_system(cleanup));
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct ScoreSaved(bool);

#[derive(Resource, Deref, DerefMut)]
struct RefreshTimer(Timer);
impl Default for RefreshTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(3., TimerMode::Once))
    }
}

#[derive(Resource, Deref, DerefMut)]
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
#[derive(Component)]
enum LeaderboardButton {
    PlayAgain,
}

pub fn get_leaderboard_credentials() -> Option<(&'static str, &'static str)> {
    if let (Some(id), Some(key)) = (
        option_env!("JORNET_LEADERBOARD_ID"),
        option_env!("JORNET_LEADERBOARD_KEY"),
    ) {
        Some((id, key))
    } else {
        None
    }
}

fn save_leaderboard_setting(
    mut leaderboard_setting: ResMut<LeaderboardSetting>,
    mut events: EventReader<CreatePlayerEvent>,
    leaderboard: Res<Leaderboard>,
) {
    if !events
        .iter()
        .any(|e| matches!(*e, CreatePlayerEvent::Success))
    {
        return;
    }

    if let Some(player) = leaderboard.get_player() {
        leaderboard_setting.0 = Some(player.clone());
    }
}

fn initiate_refresh(leaderboard: Res<Leaderboard>, mut events: EventReader<SendScoreEvent>) {
    if !events.iter().any(|e| matches!(*e, SendScoreEvent::Success)) {
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
    mut events: EventReader<RefreshLeaderboardEvent>,
) {
    if !events
        .iter()
        .any(|e| matches!(*e, RefreshLeaderboardEvent::Success))
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

        let mut leaderboard = leaderboard.get_leaderboard();
        leaderboard
            .sort_unstable_by(|s1, s2| s1.score.partial_cmp(&s2.score).unwrap_or(Ordering::Equal));
        leaderboard.truncate(10);

        let has_us = leaderboard
            .iter()
            .any(|score| player.name == score.player && score.score == time.elapsed_secs());

        for (i, score) in leaderboard.iter().enumerate() {
            // When we have a fresh leaderboard (when not refreshing), we assume
            // that our score would be included if it were high enough. So if we
            // haven't already displayed our score, toss it in at the last position.
            let (display_score, display_name, is_us, rank) =
                if !has_us && i == leaderboard.len() - 1 {
                    (time.elapsed_secs(), &player.name, true, "?".to_string())
                } else {
                    let is_us = player.name == score.player && score.score == time.elapsed_secs();

                    (score.score, &score.player, is_us, format!("{}", i + 1))
                };

            let row = commands
                .spawn(NodeBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(30.),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                })
                .id();

            let rank_text = commands
                .spawn(TextBundle {
                    text: Text::from_section(
                        rank,
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 30.,
                            color: if is_us { OUR_SCORE_TEXT } else { TITLE_TEXT },
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
                .spawn(TextBundle {
                    text: Text::from_section(
                        display_name,
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 30.,
                            color: if is_us { OUR_SCORE_TEXT } else { TITLE_TEXT },
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
                .spawn(TextBundle {
                    text: Text::from_section(
                        format!("{:.3}", display_score),
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 30.,
                            color: if is_us { OUR_SCORE_TEXT } else { TITLE_TEXT },
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
        color: TITLE_TEXT,
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
        color: BUTTON_TEXT,
    };

    let root = commands
        .spawn((
            NodeBundle {
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
                ..default()
            },
            LeaderboardMarker,
        ))
        .id();

    let container = commands
        .spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.)),
                ..default()
            },
            background_color: CONTAINER_BACKGROUND.into(),
            ..default()
        })
        .id();

    let title = commands
        .spawn(
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
        .spawn((
            TextBundle::from_section(
                "Loading...",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 30.0,
                    color: TITLE_TEXT,
                },
            )
            .with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
            LoadingText,
        ))
        .id();

    let scores_container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect {
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            ScoresContainer,
        ))
        .id();

    let play_again = commands
        .spawn((
            ButtonBundle {
                style: button_style,
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            Focusable::default(),
            LeaderboardButton::PlayAgain,
            PlayAgainButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Play Again",
                button_text_style.clone(),
            ));
        })
        .id();

    commands.entity(root).push_children(&[container]);

    commands
        .entity(container)
        .push_children(&[title, loading, scores_container, play_again]);
}

fn create_player(
    mut leaderboard: ResMut<Leaderboard>,
    leaderboard_setting: Res<LeaderboardSetting>,
) {
    if let Some(player) = &**leaderboard_setting {
        info!("as_playering with {:?}", player);

        leaderboard.as_player(player.clone());
    } else {
        info!("creating new player");
        leaderboard.create_player(Some(&random_name()));
    }
}

fn save_score(race_time: Res<RaceTime>, leaderboard: Res<Leaderboard>) {
    info!("sending score. player is: {:?}", leaderboard.get_player());
    leaderboard.send_score(race_time.elapsed_secs());
}

fn button_actions(
    buttons: Query<&LeaderboardButton>,
    mut events: EventReader<NavEvent>,
    mut state: ResMut<State<GameState>>,
) {
    for button in events.nav_iter().activated_in_query(&buttons) {
        match button {
            LeaderboardButton::PlayAgain => {
                state.set(GameState::MainMenu).unwrap();
            }
        }
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<LeaderboardMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
