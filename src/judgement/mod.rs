pub mod metrics;
pub mod grading;

use bevy::prelude::*;

use crate::team_markers::{
    PlayerMarker,
    EnemyMarker,
};

use crate::arrow::{
    Arrow,
    ArrowStatus,
};
use crate::layout::{
    SongPanel,
    BBox,
};
use crate::input::{
    LaneHit
};

pub use metrics::{
    SongMetrics
};
pub use grading::{
    CorrectHitEvent,
    IncorrectHitEvent,
    MissfireEvent,
    RawCorrectHitEvent,
    RawIncorrectHitEvent,
    RawMissfireEvent,
    JudgementSettings
};

fn world() -> BBox {
    crate::world()
}


/// Listens for Input actions where the user (correctly or incorrectly) attempts to complete a note
/// Consumes LaneHit events and creates
///   -> CorrectHitEvent
///   -> IncorrectHitEvent
///   -> MissfireEvent
fn judge_lane_hits(
    // consumes input events
    mut input_events: EventReader<LaneHit>,

    // needed to do the judgment
    mut arrow_q: Query<&mut Arrow>,
    judgement: Res<JudgementSettings>,

    // outputs one of the judgement events
    mut correct_arrow_events: EventWriter<CorrectHitEvent>,
    mut incorrect_arrow_events: EventWriter<IncorrectHitEvent>,
    mut missfire_events: EventWriter<MissfireEvent>,
) {
    for lane_hit in input_events.read() {
               
        // ---------------------------------------------- 
        // Find the arrow with the closest arrival time 
        // ---------------------------------------------- 

        use ordered_float::NotNan;
        let search_result = arrow_q
            .iter_mut()
            
            // only consider arrows that have not been hit yet. honestly don't know if this is even
            // the right call
            .filter(|arrow| !matches!(arrow.status(), ArrowStatus::Completed))
            
            // only consider arrows in the lane that was hit
            .filter(|arrow| arrow.lane() == lane_hit.lane())

            // Get the absolute arrival time of each
            .map(|arrow| {
                let delta_time = arrow.arrival_beat() - lane_hit.beat();
                let time_diff = delta_time.abs();
                (arrow, time_diff)
            })

            // Discard the NaNs, everything else can be compared
            .filter_map(|(arrow, time_diff)| match NotNan::new(time_diff) {
                Ok(time_diff) => Some((arrow, time_diff)),
                Err(e) => {
                    log::error!("Found NaN while calculating the time to arrival of {arrow:?} - {e:?}");
                    None // discard this arrow
                }
            })
            // Find the minimum
            .min_by_key(|(_, diff)| *diff);


        // ---------------------------------------------- 
        // If we did not find anything, then that means it was a missfire.
        // so we can send that off now and skip to the next lane hit
        let Some((mut arrow, _time_diff)) = search_result else {
            log::debug!("No arrow found, sending a missfire event");
            missfire_events.send(MissfireEvent {
                lane_hit: lane_hit.clone()
            });
            continue;
        };

        // ---------------------------------------------- 
        //   found an arrow, send it off to get judged
        // ---------------------------------------------- 
        let grade = judgement.judge(&lane_hit, arrow.as_ref());

        log::debug!("arrow found: {arrow:?}, grade = {grade:?}...");


        // ---------------------------------------------- 
        //    Process the grade
        // ---------------------------------------------- 
        match grade {
            grading::Grade::Success(grade) => {
                // send the correct hit event

                log::debug!("marking arrow as completed");
                arrow.mark_completed();
                log::debug!("sending correct hit event");
                correct_arrow_events.send(CorrectHitEvent {
                    lane_hit: lane_hit.clone(),
                    grade,
                });
            }
            grading::Grade::Fail(grade) => {
                log::debug!("sending incorrect hit event");
                incorrect_arrow_events.send(IncorrectHitEvent {
                    lane_hit: lane_hit.clone(),
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
fn emit_dropped_notes(
    mut events: EventWriter<DroppedNoteEvent>,
    panel: Query<&SongPanel, With<PlayerMarker>>,
    mut query: Query<(&Transform, &mut Arrow), With<PlayerMarker>>
) {
    let panel = panel.single();

    query
        .iter_mut()
        .filter(|(transform, _)| {
            let y = transform.translation.y;
            y < panel.arrow_drop_line_y()
        })
        .filter(|(_, arrow)| arrow.status().is_pending())
        .for_each(|(_, mut arrow)| {
            log::debug!("emitting DroppedNoteEvent");
            events.send(DroppedNoteEvent {
                arrow: arrow.clone(),
            });
            arrow.mark_completed();
        });

}


pub struct JudgementPlugin;
impl Plugin for JudgementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<RawCorrectHitEvent::<PlayerMarker>>()
            .add_event::<RawIncorrectHitEvent::<PlayerMarker>>()
            .add_event::<RawMissfireEvent::<PlayerMarker>>()

            .add_event::<RawCorrectHitEvent::<EnemyMarker>>()
            .add_event::<RawIncorrectHitEvent::<EnemyMarker>>()
            .add_event::<RawMissfireEvent::<EnemyMarker>>()

            .add_event::<DroppedNoteEvent>()

            .insert_resource::<JudgementSettings>(JudgementSettings::new())
            
            // Add the systems
            .add_systems(Update, judge_lane_hits)
            .add_systems(Update, emit_dropped_notes)
            
            // Add the plugins
            .add_plugins(metrics::MetricsPlugin)
        ;
    }
}

