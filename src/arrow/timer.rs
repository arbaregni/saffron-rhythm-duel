use bevy::prelude::*;
use serde::Serialize;

#[derive(Debug, Copy, Clone, clap::ValueEnum, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FinishBehavior {
    /// Stop everything after song finishes 
    #[default]
    Stop,
    /// Repeat the song on loop
    Repeat,
    /// Record new notes on the song
    Record,
}

pub struct BeatTick {
    /// The count of the beat we are on.
    beat: u32,
}

#[derive(Component)]
pub struct BeatTimer {
    /// What we do when we reach the end of the chart (if we're using a chart)
    pub (in crate::arrow) on_finish: FinishBehavior,
    /// The time stamp when the song started
    pub (in crate::arrow) song_start: f32,
    /// Ticks for every beat
    pub (in crate::arrow) beat_timer: Timer,
    /// Count the number of beat ticks, starting at 0
    /// Indexes into crate list of beats in a chart
    pub (in crate::arrow) beat_count: u32,
}


impl BeatTick {
    pub fn beat(&self) -> u32 {
        self.beat
    }
}

impl BeatTimer {
    pub fn on_finish(&self) -> &FinishBehavior {
        &self.on_finish
    }
    pub fn song_start(&self) -> f32 {
        self.song_start
    }
    pub fn beat_count(&self) -> u32 {
        self.beat_count
    }
    pub fn tick(&mut self, time: &Time) -> Option<BeatTick> {
        let now = time.elapsed().as_secs_f32();

        if now < self.song_start() {
            // not time to start the song yet
            return None;
        }

        self.beat_timer.tick(time.delta());

        if !self.beat_timer.just_finished() {
            // not time for another beat just yet
            return None;
        }

        let beat = self.beat_count();
        self.beat_count += 1;

        Some(BeatTick {
            beat
        })

    }
}

