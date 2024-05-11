use bevy::prelude::*;

use super::{
    CorrectHitEvent,
    DroppedNoteEvent,
    MissfireEvent,
};

#[derive(Resource)]
#[derive(Debug)]
pub struct SongMetrics {
    // see below for definitions
    total_arrows: u32,
    success_arrows: u32,
    streak: u32,

    /// Number of missfires we have seen in a row. Reset if there's another event.
    missfires_in_a_row: u32,
    /// Number of dropped notes we have seen in a row.
    dropped_notes_in_a_row: u32,

    /// The number of notes in the last streak
    last_streak_size: u32,
}


impl SongMetrics {
    pub fn new() -> SongMetrics {
        SongMetrics {
            total_arrows: 0,
            success_arrows: 0,
            streak: 0,
            missfires_in_a_row: 0,
            dropped_notes_in_a_row: 0,

            last_streak_size: 0,
        }
    }

    /// Total number of arrows that have passed the target line.
    pub fn total_arrows(&self) -> u32 {
        self.total_arrows
    }
    /// Number of arrows that the user has correctly intercepted in time.
    pub fn success_arrows(&self) -> u32 {
        self.success_arrows
    }
    /// Number of consecutive arrows the user has gotten correct. 0 if the last hit was incorrect.
    pub fn streak(&self) -> u32 {
        self.streak
    }
    /// Returns true if the last arrow we saw broke the streak
    pub fn just_broke_streak(&self) -> bool {
        let had_streak = self.last_streak_size > 0;
        let did_bad = self.missfires_in_a_row == 1 || self.dropped_notes_in_a_row == 1;
        had_streak && did_bad
    }
}

pub fn update_metrics(
    mut metrics: ResMut<SongMetrics>,
    mut hit_events: EventReader<CorrectHitEvent>,
    mut missfire_events: EventReader<MissfireEvent>,
    mut dropped_events: EventReader<DroppedNoteEvent>,
) {
    // The user presses the note on time.
    // It can still be a little to early or a little to late,
    // but it's within threshold.
    for _correct_hit in hit_events.read() {
        log::info!("metrics - processing hit event");

        metrics.total_arrows += 1;
        metrics.success_arrows += 1;

        metrics.streak += 1;
        metrics.missfires_in_a_row = 0;
        metrics.dropped_notes_in_a_row = 0;

        log::info!("metrics updated - {metrics:#?}");
    }

    // If the user presses a key to early or too late.
    for _missfire in missfire_events.read() {
        log::info!("metrics - processing missfire");

        // does not count towards `total_arrows` since it has not been removed yet

        // this will be caught in dropped_events
        metrics.last_streak_size = metrics.streak;
        metrics.streak = 0;
        metrics.missfires_in_a_row += 1;
        metrics.dropped_notes_in_a_row = 0;

        log::info!("metrics updated - {metrics:#?}");
    }

    // Notes that the player did not hit in time or were never removed
    for _dropped in dropped_events.read() {
        log::info!("metrics - processing dropped note");
        
        metrics.total_arrows += 1;

        metrics.last_streak_size = metrics.streak;
        metrics.streak = 0;
        metrics.missfires_in_a_row = 0;
        metrics.dropped_notes_in_a_row += 1;

        log::info!("metrics updated - {metrics:#?}");
    }

}

pub struct MetricsPlugin;
impl Plugin for MetricsPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Building metrics plugin");
        app
            .insert_resource(SongMetrics::new())
            .add_systems(Update, update_metrics)
        ;
    }
}
