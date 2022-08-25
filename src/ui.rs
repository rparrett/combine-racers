use bevy::prelude::*;
use interpolation::Ease;

use crate::{GameAssets, GameState, RaceTime};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrickTextTimer>()
            .add_system_set(SystemSet::on_exit(GameState::MainMenu).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(fade_trick_text)
                    .with_system(race_time),
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

#[derive(Component)]
pub struct TrickText;
#[derive(Deref, DerefMut)]
pub struct TrickTextTimer(Timer);
impl Default for TrickTextTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2., false))
    }
}
#[derive(Component)]
pub struct RaceTimeText;

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
        .insert(TrickText);
}

fn fade_trick_text(
    time: Res<Time>,
    mut timer: ResMut<TrickTextTimer>,
    mut query: Query<&mut Text, With<TrickText>>,
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

pub fn trick_text(front_flips: u32, back_flips: u32, fakie: bool) -> String {
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

    if front_flips > 0 {
        let mut parts = vec![];

        if fakie {
            parts.push("Fakie");
        }

        if let Some(num) = num_text(front_flips) {
            parts.push(num);
        }

        parts.push("Front Flip!");

        lines.push(parts.join(" "));
    }
    if back_flips > 0 {
        let mut parts = vec![];

        if fakie {
            parts.push("Fakie");
        }

        if let Some(num) = num_text(back_flips) {
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

fn cleanup(
    mut commands: Commands,
    query: Query<Entity, Or<(With<RaceTimeText>, With<TrickText>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
