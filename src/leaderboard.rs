use bevy::prelude::*;
use bevy_jornet::{JornetPlugin, Leaderboard};

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
                .add_system_set(
                    SystemSet::on_enter(GameState::AssetLoading).with_system(create_player),
                )
                .add_system_set(
                    SystemSet::on_enter(GameState::Leaderboard)
                        .with_system(save_score)
                        .with_system(spawn_leaderboard),
                )
                .add_system_set(
                    SystemSet::on_update(GameState::Leaderboard)
                        .with_system(initiate_refresh)
                        .with_system(update_leaderboard),
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
struct RefreshingText;
#[derive(Component)]
struct LeaderboardMarker;
#[derive(Component)]
struct ScoresContainer;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

fn initiate_refresh(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<RefreshTimer>,
    leaderboard: Res<Leaderboard>,
    mut refreshing: ResMut<Refreshing>,
) {
    timer.tick(time.delta());
    if timer.just_finished() {
        info!("initiating refresh");
        leaderboard.refresh_leaderboard();
        **refreshing = true;
    }
}

fn update_leaderboard(
    mut commands: Commands,
    leaderboard: Res<Leaderboard>,
    time: Res<RaceTime>,
    mut refreshing: ResMut<Refreshing>,
    container_query: Query<Entity, With<ScoresContainer>>,
    mut refreshing_text_query: Query<&mut Text, With<RefreshingText>>,
    assets: Res<GameAssets>,
) {
    if refreshing.is_changed() {
        for mut text in refreshing_text_query.iter_mut() {
            text.sections[0].value = if **refreshing {
                "Refreshing...".to_string()
            } else {
                "".to_string()
            };
        }
    }

    if leaderboard.is_changed() {
        info!("leaderboard changed");

        if let Some(player) = leaderboard.get_player() {
            let container = container_query.single();
            commands.entity(container).despawn_descendants();

            let leaderboard = leaderboard.get_leaderboard();

            let mut displayed_our_score = false;

            for (i, score) in leaderboard.iter().enumerate() {
                // bleh
                let is_us = player.name == score.player;

                // When we have a fresh leaderboard (when not refreshing), we assume
                // that our score would be included if it were high enough. So if we
                // haven't already displayed our score, toss it in at the last position.

                let (display_score, display_name, you) =
                    if !**refreshing && i == leaderboard.len() - 1 && !displayed_our_score {
                        info!("{} {} <-- you", time.elapsed_secs(), player.name);

                        (time.elapsed_secs(), &player.name, true)
                    } else {
                        info!(
                            "{} {} {}",
                            1. / score.score,
                            &score.player,
                            if is_us {
                                "<-- You".to_string()
                            } else {
                                "".to_string()
                            }
                        );

                        if is_us && score.score == 1. / time.elapsed_secs() {
                            displayed_our_score = true;
                        }

                        (1. / score.score, &score.player, is_us)
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

                let name_container = commands
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size {
                                width: Val::Px(250.),
                                ..default()
                            },
                            overflow: Overflow::Hidden,
                            ..default()
                        },
                        color: Color::NONE.into(),
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
                                color: if you { Color::PURPLE } else { Color::WHITE },
                            },
                        ),
                        ..default()
                    })
                    .id();

                commands.entity(name_container).push_children(&[name_text]);

                let score_text = commands
                    .spawn_bundle(TextBundle {
                        text: Text::from_section(
                            format!("{:.3}", display_score),
                            TextStyle {
                                font: assets.font.clone(),
                                font_size: 30.,
                                color: if you { Color::PURPLE } else { Color::WHITE },
                            },
                        ),
                        ..default()
                    })
                    .id();

                commands
                    .entity(row)
                    .push_children(&[name_container, score_text]);

                commands.entity(container).add_child(row);
            }
        }

        **refreshing = false;
    }
}

fn spawn_leaderboard(mut commands: Commands, assets: Res<GameAssets>) {
    info!("spawn_leaderboard");

    let title_text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 60.0,
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
        .insert(LeaderboardMarker)
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

    let refreshing = commands
        .spawn_bundle(
            TextBundle::from_section(
                "Refreshing...",
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
        .insert(RefreshingText)
        .id();

    let scores_container = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(ScoresContainer)
        .id();

    commands
        .entity(container)
        .push_children(&[title, refreshing, scores_container]);
}

fn create_player(mut leaderboard: ResMut<Leaderboard>) {
    info!("b4 create_player {:?}", leaderboard.is_changed());
    leaderboard.create_player(None);
    // grab a leaderboard at the start so we have something to display while
    // we wait for it to refresh when the game ends.
    leaderboard.refresh_leaderboard();
    info!("after create_player {:?}", leaderboard.is_changed());
}

fn save_score(
    race_time: Res<RaceTime>,
    leaderboard: Res<Leaderboard>,
    mut refreshing: ResMut<Refreshing>,
) {
    info!("b4 sendscore: {:?}", leaderboard.is_changed());
    leaderboard.send_score(1. / race_time.elapsed_secs());
    info!("afters sendscore: {:?}", leaderboard.is_changed());
    **refreshing = true;
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<LeaderboardMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
