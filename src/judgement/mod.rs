mod metrics;
mod lane_box;
mod feedback_text;
mod target_sparkles;
mod animations;
mod lane_widgets;
pub use lane_widgets::{
    LaneTarget,
    LaneLetter,
};


use bevy::prelude::*;

use crate::team_markers::{
    PlayerMarker
};

use crate::lane::{
    Lane,
};
use crate::arrow::{
    Arrow,
};
use crate::layout::{
    SongPanel,
    BBox,
};
use crate::input::InputActionEvent;

pub use metrics::{
    SongMetrics
};

fn world() -> BBox {
    crate::world()
}

pub const KEYPRESS_TOLERANCE_SECS: f32 = 0.5; // in seconds

/// Represents an attempt from the user to hit a lane.
#[derive(Debug,Clone)]
pub struct LaneHit {
    /// Which lane it happened in
    pub lane: Lane,
    /// When the hit occured (game time)
    pub time_of_hit: f32,
}

/// Represents when the user hits the lane when an arrow is passing the target line, and it
/// completes that arrow.
#[derive(Event)]
#[derive(Debug,Clone)]
pub struct CorrectHitEvent {
    /// The lane hit
    pub lane_hit: LaneHit,
    /// The grade the judgment system gave
    pub grade: SuccessGrade,
}

/// Represents when the user hits the lane, and there is a nearby note,
/// But we don't want to count it as 'completing' that note.
#[derive(Event)]
#[derive(Debug,Clone)]
pub struct IncorrectHitEvent {
    /// THe lane hit
    lane_hit: LaneHit,
    /// The grade the judgement system gave
    pub grade: FailingGrade,
}

/// Event representing when the user attempts to complete a note, but are too early or late to be
/// considered 'correct'
#[derive(Event)]
#[derive(Debug,Clone)]
pub struct MissfireEvent {
    /// The lane hit that originated this missfire
    lane_hit: LaneHit,
}

#[derive(Debug, Copy, Clone)]
pub enum SuccessGrade {
    Perfect,
    Good,
    Fair,
}
impl SuccessGrade {
    pub fn is_perfect(self) -> bool {
        use SuccessGrade::*;
        match self {
            Perfect => true,
            Good | Fair => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FailingGrade {
    Early,
    Late,
}


#[derive(Resource)]
pub struct JudgementSettings {
    // passing grades are perfect, good, and fair
    perfect_cutoff: f32,
    good_cutoff: f32,
    fair_cutoff: f32,

    // if it failed, we just say early or late.
    // There's also the possibility that we couldn't find a note whatsoever
}


#[derive(Debug, Copy, Clone)]
pub enum Grade {
    Success(SuccessGrade),
    Fail(FailingGrade)
}

impl JudgementSettings {
    pub fn new() -> Self {
        Self {
            perfect_cutoff: 0.05,
            good_cutoff:    0.06,
            fair_cutoff:    0.08,
        }
    }
    pub fn judge(&self, lane_hit: &LaneHit, arrow: &Arrow) -> Grade {
        let hit_time = lane_hit.time_of_hit;
        let arrival_time = arrow.arrival_time();

        let diff = (arrival_time - hit_time).abs();
        if diff < self.perfect_cutoff {
            Grade::Success(SuccessGrade::Perfect)
        } else if diff < self.good_cutoff {
            Grade::Success(SuccessGrade::Good)
        } else if diff < self.fair_cutoff {
            Grade::Success(SuccessGrade::Fair)
        } else {
            
            if hit_time < arrival_time {
                // hit before it arrived
                Grade::Fail(FailingGrade::Early)
            } else {
                Grade::Fail(FailingGrade::Late)
            }

        }

    } 
}



/// Listens for Input actions where the user (correctly or incorrectly) attempts to complete a note
/// Consumes LaneHit events and creates
///   -> CorrectHitEvent
///   -> IncorrectHitEvent
///   -> MissfireEvent
fn judge_lane_hits(
    time: Res<Time>,
    mut input_events: EventReader<InputActionEvent>,
    mut query: Query<(&mut Arrow, &mut Sprite)>,
    mut correct_arrow_events: EventWriter<CorrectHitEvent>,
    mut incorrect_arrow_events: EventWriter<IncorrectHitEvent>,
    mut missfire_events: EventWriter<MissfireEvent>,
    judgement: Res<JudgementSettings>,
) {

    let now = time.elapsed().as_secs_f32();

    for input_action in input_events.read() {
        let InputActionEvent::LaneHit(event_lane) = input_action; // only input action type for now
                                                                
        let lane_hit = LaneHit {
            lane: *event_lane,
            time_of_hit: now
        };

        // ---------------------------------------------- 
        // Find the arrow with the closest arrival time 
        // ---------------------------------------------- 

        use ordered_float::NotNan;
        let search_result = query
            .iter_mut()
            
            // only consider arrows that have not been hit yet
            .filter(|(arrow, _)| arrow.status().is_pending())
            
            // only consider arrows in the lane that was hit
            .filter(|(arrow, _)| arrow.lane() == lane_hit.lane)

            // Get the absolute arrival time of each
            .map(|(arrow, sprite)| {
                let delta_time = arrow.arrival_time() - lane_hit.time_of_hit;
                let time_diff = delta_time.abs();
                (arrow, sprite, time_diff)
            })

            // Discard the NaNs, everything else can be compared
            .filter_map(|(arrow, sprite, time_diff)| match NotNan::new(time_diff) {
                Ok(time_diff) => Some((arrow, sprite, time_diff)),
                Err(e) => {
                    log::error!("Found NaN while calculating the time to arrival of {arrow:?} - {e:?}");
                    None // discard this arrow
                }
            })
            // Find the minimum
            .min_by_key(|(_, _, diff)| *diff);


        // ---------------------------------------------- 
        // If we did not find anything, then that means it was a missfire.
        // so we can send that off now and skip to the next lane hit
        let Some((mut arrow, mut sprite, _time_diff)) = search_result else {
            log::info!("No arrow found, sending a missfire event");
            missfire_events.send(MissfireEvent {
                lane_hit
            });
            continue;
        };

        // ---------------------------------------------- 
        //   found an arrow, send it off to get judged
        // ---------------------------------------------- 
        log::info!("arrow found, judging now...");
        let grade = judgement.judge(&lane_hit, arrow.as_ref());


        // ---------------------------------------------- 
        //    Process the grade
        // ---------------------------------------------- 
        match grade {
            Grade::Success(grade) => {
                // send the correct hit event

                log::info!("marking arrow as completed");
                arrow.mark_completed();
                sprite.color = lane_hit.lane.colors().greyed;
                log::info!("sending correct hit event");
                correct_arrow_events.send(CorrectHitEvent {
                    lane_hit,
                    grade,
                });
            }
            Grade::Fail(grade) => {
                log::info!("sending incorrect hit event");
                incorrect_arrow_events.send(IncorrectHitEvent {
                    lane_hit,
                    grade,
                });
                
            }
        }

        // all done


    }

}

/// Event representing when an arrow never gets hit by the player
#[derive(Event)]
#[derive(Debug,Clone)]
pub struct DroppedNoteEvent {
    arrow: Arrow,
}
impl DroppedNoteEvent {
    /// The arrow that was never hit.
    pub fn arrow(&self) -> &Arrow {
        &self.arrow
    }
}

/// Despawns old arrows if they fall out of the screen and emits `DroppedNoteEvent`
fn despawn_arrows(
    mut commands: Commands,
    mut events: EventWriter<DroppedNoteEvent>,
    panel: Query<&SongPanel, With<PlayerMarker>>,
    query: Query<(Entity, &Transform, &Arrow), With<PlayerMarker>>
) {
    let panel = panel.single();

    for (entity, transform, arrow) in query.iter() {
        let y = transform.translation.y;
        if y < panel.arrow_drop_line_y() {

            // it's low enough to despawn
            commands.entity(entity).despawn();

            if arrow.status().is_pending() {
                log::info!("emitting DroppedNoteEvent");
                events.send(DroppedNoteEvent {
                    arrow: *arrow
                });
            }

        }
    }

}


pub struct TargetsPlugin;
impl Plugin for TargetsPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building Targets plugin...");
        app
            .add_event::<CorrectHitEvent>()
            .add_event::<IncorrectHitEvent>()
            .add_event::<MissfireEvent>()
            .add_event::<DroppedNoteEvent>()

            .insert_resource::<JudgementSettings>(JudgementSettings::new())
            
            // Add the systems
            .add_systems(Update, judge_lane_hits)
            .add_systems(Update, despawn_arrows)
            .add_systems(Update, animations::darken_on_press)
            .add_systems(Update, animations::darken_over_time)
            .add_systems(Update, animations::jostle_on_dropped_note)
            .add_systems(Update, animations::animate_jostling)
            .add_systems(Update, animations::play_sound_on_hit)
            .add_systems(Update, animations::play_sound_on_dropped_note)
            
            // Add the plugins
            .add_plugins(lane_box::LaneBoxPlugin)
            .add_plugins(feedback_text::FeedbackTextPlugin)
            .add_plugins(target_sparkles::TargetSparklesPlugin)
            .add_plugins(metrics::MetricsPlugin)
        ;
    }
}

