use bevy::prelude::*;

use super::{
    metrics,
    CorrectHitEvent,
    DroppedNoteEvent,
    MissfireEvent,
    Grade,
    SongMetrics,
};

#[derive(Component)]
pub struct FeedbackText {
    last_updated: f32,
    initial_scale: f32,
}
impl FeedbackText {
    pub fn new() -> Self {
        Self {
            last_updated: 0.0,
            initial_scale: 1.0,
        }
    }
}


/// Setups the resources for the feedback text.
fn setup_feedback_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load(crate::BASE_FONT_NAME);
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
                    FeedbackText::new(),
                    TextBundle {
                        text,
                        ..default()
                    }
                ));
        });

}

    
/// Display a message to the user when they hit a note correctly.
fn set_feedback_content_on_correct_hit(
    time: Res<Time>,
    song_metrics: Res<SongMetrics>,
    query: Query<(&mut Text, &mut FeedbackText)>,
    mut correct_events: EventReader<CorrectHitEvent>,
) {
    // we just want to know if there have been correct events, we'll handle them all now
    let Some(correct_hit) = correct_events.read().last() else {
        return; // nothing to do
    };


    let mut content = match &correct_hit.grade {
        Grade::Perfect => {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            [
                "Perfect!",
                "Wonderful!",
                "Fantastic!!",
                "Outstanding!!!",
            ]
                .choose(&mut rng)
                .copied()
                .expect("at least one option")
        }
        Grade::Fair => {
            "Fair"
        },
        Grade::Early => {
            "Early"
        },
        Grade::Late => {
            "Late"
        }
    };


    let streak_begin = 3;
    if correct_hit.grade.is_perfect() && song_metrics.streak() >= streak_begin {
        content = match song_metrics.streak() - streak_begin {
            0 => "SUPER-!",
            1 => "SUPER-POWER-!",
            2 => "SUPER-POWER-NINJA-!",
            3 => "SUPER-POWER-NINJA-TURBO-!",
            4 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-!",
            5 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-!",
            6 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-!",
            7 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-!",
            8 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-!",
            9 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-!",
            10 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-!",
            11 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-PREFIX-!",
            12 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-PREFIX-COMBO!",
            n => {
                let n = n - 12;
                let content = &format!("SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-PREFIX-COMBO! x{n}");
                set_feedback_text_content(content, time, query, FeedbackStyle::Success);
                return;
            }
        };

    }

    
    set_feedback_text_content(content, time, query, FeedbackStyle::Success);
}


/// Displays a message to the user when they missfire.
/// That is, either click it to early or too late to be considered a hit.
fn set_feedback_content_on_missfire(
    time: Res<Time>,
    song_metrics: Res<SongMetrics>,
    query: Query<(&mut Text, &mut FeedbackText)>,
    mut missfire_events: EventReader<MissfireEvent>,
) {
    // We read to the last missfire event, if there was one
    let Some(missfire) = missfire_events.read().last() else {
        // nothing to do
        return;
    };

    let mut content = match &missfire.opt_hit {
        Some((_, Grade::Early)) => {
            "Too early!"
        }
        Some((_, Grade::Late)) => {
            "Too late!"
        },
        _ => {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            ["Butter fingers", "Whoops", "Turn off sticky keys", "Try again", "Not that time!", "Oops!", "Missclicked"]
                .choose(&mut rng)
                .copied()
                .expect("at least one option")
        },
    };

    if song_metrics.just_broke_streak() {
        content = "Streak broken!";
    }

    set_feedback_text_content(content, time, query, FeedbackStyle::Failure);
}

/// Displays message to the user when they don't hit a note.
fn set_feedback_content_on_dropped_note(
    time: Res<Time>,
    song_metrics: Res<SongMetrics>,
    query: Query<(&mut Text, &mut FeedbackText)>,
    mut dropped_note_events: EventReader<DroppedNoteEvent>,
) {
    let Some(_dropped_note) = dropped_note_events.read().last() else {
        // nothing to do
        return;
    };
    log::info!("combo meter - consumed dropped note");

    let mut content = "Miss";

    if song_metrics.just_broke_streak() {
        content = "Streak broken!";
    }
    set_feedback_text_content(content, time, query, FeedbackStyle::Failure);
}


enum FeedbackStyle {
    Success, // for correct hit events
    Failure, // for misses and drops
}


const TEXT_SCALE_FOR_SUCCESS: f32 = 1.2;
const TEXT_SCALE_FOR_FAILURE: f32 = 1.5;

const TEXT_COLOR_FOR_SUCCESS: Color = Color::rgb(0.9, 0.9, 0.9); // off-white
const TEXT_COLOR_FOR_FAILURE: Color = Color::rgb(171.0 / 256.0, 32.0 / 256.0, 46.0 / 256.0); // red

/// Sets the feedback text contents
fn set_feedback_text_content(
    content: &str,
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut FeedbackText)>,
    style: FeedbackStyle,
) {
    let now = time.elapsed().as_secs_f32();

    let (mut text, mut feedback) = query.single_mut();

    feedback.last_updated = now;
    text.sections[0].value.clear();
    text.sections[0].value.push_str(content);

    match style {
        FeedbackStyle::Success => {
            feedback.initial_scale = TEXT_SCALE_FOR_SUCCESS;
            text.sections[0].style.color = TEXT_COLOR_FOR_SUCCESS;
        }
        FeedbackStyle::Failure => {
            feedback.initial_scale = TEXT_SCALE_FOR_FAILURE;
            text.sections[0].style.color = TEXT_COLOR_FOR_FAILURE;
        }
    }
}

const TEXT_SCALE_END: f32 = 1.0;
const FEEDBACK_TEXT_MAX_SHOW_TIME: f32 = 0.25; // seconds

/// Animates out the feedback text over time.
fn update_feedback_text(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut FeedbackText, &mut Transform)>
) {
    let now = time.elapsed().as_secs_f32();
    let (mut text, feedback, mut transform) = query.single_mut();

    let t = (now - feedback.last_updated) / FEEDBACK_TEXT_MAX_SHOW_TIME;

    if t >= 1.0 {
        text.sections[0].value.clear();
        return;
    }
    let alpha = 1.0 - t;
    text.sections[0].style.color.set_a(alpha);

    let scale_factor = feedback.initial_scale * (1.0 - t) + TEXT_SCALE_END * t;
    transform.scale = Vec3::splat(scale_factor);
}


pub struct ComboMeterPlugin;
impl Plugin for ComboMeterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_feedback_text)
            .add_systems(Update, update_feedback_text)

            // We need to schedule these after the metrics update so that we can
            // get the latest information on the frame the events are published
            .add_systems(Update, set_feedback_content_on_correct_hit
                                    .after(metrics::update_metrics))
            .add_systems(Update, set_feedback_content_on_missfire
                                    .after(metrics::update_metrics))
            .add_systems(Update, set_feedback_content_on_dropped_note
                                    .after(metrics::update_metrics))// issue where this isn't
                                                                    // working... :(

        ;


    }
}
