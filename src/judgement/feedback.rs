use bevy::prelude::*;

use super::{
    SongMetrics,
    CorrectArrowEvent
};

#[derive(Component)]
pub struct FeedbackText {
    last_updated: f32,
}

const FEEDBACK_TEXT_MAX_SHOW_TIME: f32 = 0.7; // seconds

fn setup_feedback_text(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font_size = 100.0;
    let color = Color::rgb(0.9, 0.9, 0.9);

    let style = TextStyle { font, font_size, color };
    let text = Text {
        sections: vec![
            TextSection {
                value: "".to_string(),
                style,
            }
        ],
        ..default()
    };

    commands.spawn(
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    FeedbackText {
                        last_updated: 0f32,
                    },
                    TextBundle {
                        text,
                        ..default()
                    }
                ));
        });

}

fn update_feedback_text(
    time: Res<Time>,
    song_metrics: Res<SongMetrics>,
    mut correct_events: EventReader<CorrectArrowEvent>,
    mut query: Query<(&mut Text, &mut FeedbackText)>,
) {
    let now = time.elapsed().as_secs_f32();

    let Some((mut text, mut feedback)) = query.iter_mut().nth(0) else {
        log::warn!("no feedback item found");
        return;
    };

    for _event in correct_events.read() {


        // TODO: advanced feedback here
        //
        feedback.last_updated = now;
        text.sections[0].value.clear();
        let text_value = match song_metrics.streak {
            0 => "",
            1..=2 => "Good",
            3 => "Nice",
            4 => "Great!",
            5 => "Amazing!",
            6 => "Wonderful!",
            7 => "Fantastic!!",
            8..=19 => "Outstanding!!!",

            20 => "SUPER-!",
            21 => "SUPER-POWER-!",
            22 => "SUPER-POWER-NINJA-!",
            23 => "SUPER-POWER-NINJA-TURBO-!",
            24 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-!",
            25 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-!",
            26 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-!",
            27 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-!",
            28 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-!",
            29 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-!",
            30 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-!",
            31 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-PREFIX-!",
            32 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-PREFIX-COMBO!",

            n => {
                let n = n - 32;
                text.sections[0].value.push_str(&format!("SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-PREFIX-COMBO! x{n}"));
                continue;
            }
        };

        text.sections[0].value.clear();
        text.sections[0].value.push_str(text_value);

    }

    let t = (now - feedback.last_updated) / FEEDBACK_TEXT_MAX_SHOW_TIME;
    if t >= 1.0 {
        text.sections[0].value.clear();
        return;
    }
    let alpha = 1.0 - t;
    text.sections[0].style.color.set_a(alpha);
}

pub struct FeedbackPlugin;
impl Plugin for FeedbackPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_feedback_text)
            .add_systems(Update, update_feedback_text)
        ;


    }
}
