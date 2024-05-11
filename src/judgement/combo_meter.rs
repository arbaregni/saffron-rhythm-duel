use bevy::prelude::*;

use super::{
    metrics,
    CorrectHitEvent,
    DroppedNoteEvent,
    MissfireEvent,
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
    let Some(_correct_hit) = correct_events.read().last() else {
        return; // nothing to do
    };


    // TODO: advanced feedback here
    //
    let content = match song_metrics.streak() {
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
            let content = &format!("SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-PREFIX-COMBO! x{n}");
            set_feedback_text_content(content, time, query, FeedbackStyle::Success);
            return;
        }
    };
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
    let Some(_missfire) = missfire_events.read().last() else {
        // nothing to do
        return;
    };

    // TODO: advanced feedback here
    let mut content = "Butter Fingers";

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

const TEXT_COLOR_FOR_SUCCESS: Color = Color::rgb(1.0, 1.0, 1.0); // white
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
