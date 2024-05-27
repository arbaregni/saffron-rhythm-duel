use bevy::prelude::*;

use super::{
    CorrectHitEvent,
    IncorrectHitEvent,
    DroppedNoteEvent,
    MissfireEvent,
    FailingGrade
};

#[derive(Debug)]
enum SongEvent {
    CorrectHit(CorrectHitEvent),
    IncorrectHit(IncorrectHitEvent),
    DroppedNote(DroppedNoteEvent),
    Missfire(MissfireEvent),
}
impl From<CorrectHitEvent> for SongEvent {
    fn from(event: CorrectHitEvent) -> SongEvent {
        SongEvent::CorrectHit(event)
    }
}
impl From<IncorrectHitEvent> for SongEvent {
    fn from(event: IncorrectHitEvent) -> SongEvent {
        SongEvent::IncorrectHit(event)
    }
}
impl From<DroppedNoteEvent> for SongEvent {
    fn from(event: DroppedNoteEvent) -> SongEvent {
        SongEvent::DroppedNote(event)
    }
}
impl From<MissfireEvent> for SongEvent {
    fn from(event: MissfireEvent) -> SongEvent {
        SongEvent::Missfire(event)
    }
}


#[derive(Resource)]
#[derive(Debug)]
pub struct SongMetrics {
    history: Vec<SongEvent>,

    /// Count the total number of arrows that have passed the target line
    total_arrows: u32,

    /// Count the number of CorrectHitEvents
    correct_hits: u32,
    /// Count the number of IncorrectHitEvents
    incorrect_hits: u32,
    /// Count the number of MissfireEvents
    missfires: u32,
    /// Count the number of DroppedNoteEvents
    dropped_notes: u32,

    /// Of the incorrect hits, how many were early?
    early: u32,
    /// Of the incorrect hits, how many were late?
    late: u32,

    /// Number of 'perfect' grades in a row.
    streak: u32,
    /// True if the last event we saw broke the streak.
    just_broke_streak: bool,
}


impl SongMetrics {
    pub fn new() -> SongMetrics {
        SongMetrics {
            // start with a big capacity because we expect to fill it up
            history: Vec::with_capacity(1000),

            // fill everything else with 0
            total_arrows: 0,
            correct_hits: 0,
            incorrect_hits: 0,
            missfires: 0,
            dropped_notes: 0,
            early: 0,
            late: 0,
            streak: 0,
            just_broke_streak: false,
        }
    }
    fn last_event(&self) -> Option<&SongEvent> {
        self.history.last()
    }
    /// Total number of arrows that have passed the target line.
    pub fn total_arrows(&self) -> u32 {
        self.total_arrows
    }
    /// Number of arrows that the user has correctly intercepted in time.
    pub fn success_arrows(&self) -> u32 {
        self.correct_hits
    }
    /// Number of consecutive arrows the user has gotten correct. 0 if the last hit was incorrect.
    pub fn streak(&self) -> u32 {
        self.streak
    }
    /// Returns true if the last arrow we saw broke the streak
    pub fn just_broke_streak(&self) -> bool {
        self.just_broke_streak
    }
}

pub fn update_metrics(
    mut metrics: ResMut<SongMetrics>,
    mut correct_hit_events: EventReader<CorrectHitEvent>,
    mut incorrect_hit_events: EventReader<IncorrectHitEvent>,
    mut missfire_events: EventReader<MissfireEvent>,
    mut dropped_events: EventReader<DroppedNoteEvent>,
) {
    metrics.just_broke_streak = false;

    let reset_streak = |metrics: &mut SongMetrics| {
        let had_streak = metrics.streak() > 0;
        metrics.streak = 0;
        if had_streak {
            metrics.just_broke_streak = true;
        }
    };


    for correct_hit in correct_hit_events.read() {
        log::debug!("metrics - processing correct hit event");

        metrics.total_arrows   += 1;
        metrics.correct_hits   += 1;
        metrics.incorrect_hits += 0;
        metrics.missfires      += 0;
        metrics.dropped_notes  += 0;

        if correct_hit.grade.is_perfect() {
            metrics.streak += 1;
        } else {
            reset_streak(metrics.as_mut());
        }

        log::debug!("metrics updated - {metrics:#?}");
    }

    for incorrect_hit in incorrect_hit_events.read() {
        log::debug!("metrics - processing incorrect hit event");

        metrics.correct_hits   += 0;
        metrics.incorrect_hits += 1;
        metrics.missfires      += 0;
        metrics.dropped_notes  += 0;

        use FailingGrade::*;
        match &incorrect_hit.grade {
            Early => {
                metrics.early += 1;
            }
            Late => {
                metrics.late += 1;
            }
        }

        reset_streak(metrics.as_mut());

        log::debug!("metrics updated - {metrics:#?}");
    }

    // If the user presses a key to early or too late.
    for _missfire in missfire_events.read() {
        log::debug!("metrics - processing missfire");

        // does not count towards `total_arrows` since it has not been removed yet
        // would be caught in dropped events
        // metrics.total_arrows   += 1;
        metrics.correct_hits   += 0;
        metrics.incorrect_hits += 0;
        metrics.missfires      += 1;
        metrics.dropped_notes  += 0;

        reset_streak(metrics.as_mut());

        log::debug!("metrics updated - {metrics:#?}");
    }

    // Notes that the player did not hit in time or were never removed
    for _dropped in dropped_events.read() {
        log::debug!("metrics - processing dropped note");
        
        metrics.total_arrows += 1;
        metrics.correct_hits   += 0;
        metrics.incorrect_hits += 0;
        metrics.missfires      += 0;
        metrics.dropped_notes  += 1;

        reset_streak(metrics.as_mut());

        log::debug!("metrics updated - {metrics:#?}");
    }

}

pub struct MetricsPlugin;
impl Plugin for MetricsPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Building metrics plugin");
        app
            .insert_resource(SongMetrics::new())
            .add_systems(Update, update_metrics
                                     .after(super::judge_lane_hits)
                                     .after(super::despawn_arrows)
             )
        ;
    }
}
