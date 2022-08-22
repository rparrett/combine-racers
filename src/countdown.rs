use bevy::prelude::*;
use interpolation::Ease;

use crate::{ui::TrickText, GameState};

pub struct CountdownPlugin;
impl Plugin for CountdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
            .add_system_set(SystemSet::on_update(GameState::Playing).with_system(update));
    }
}

#[derive(Component)]
struct CountdownTimer {
    countdown: Timer,
    go: Timer,
}

impl Default for CountdownTimer {
    fn default() -> Self {
        Self {
            countdown: Timer::from_seconds(3., false),
            go: Timer::from_seconds(1., false),
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn().insert(CountdownTimer::default());
}

fn update(
    time: Res<Time>,
    mut query: Query<&mut CountdownTimer>,
    mut text_query: Query<&mut Text, With<TrickText>>,
) {
    for mut timer in query.iter_mut() {
        if !timer.countdown.finished() {
            timer.countdown.tick(time.delta());

            for mut text in text_query.iter_mut() {
                let left =
                    timer.countdown.percent_left() * timer.countdown.duration().as_secs_f32();

                text.sections[0].value = format!("{}", left.ceil());
                text.sections[0].style.color = Color::rgba(1., 0., 0., Ease::cubic_out(left % 1.));
            }

            if timer.countdown.just_finished() {
                // TODO start race timer
                for mut text in text_query.iter_mut() {
                    text.sections[0].value = "GO!".to_string();
                }
                timer.go.reset();
            }
        } else if !timer.go.finished() {
            timer.go.tick(time.delta());
            for mut text in text_query.iter_mut() {
                text.sections[0].style.color =
                    Color::rgba(1., 0., 0., Ease::cubic_out(timer.go.percent_left()));
            }
        }
    }
}
