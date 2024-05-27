use bevy::prelude::*;

use crate::arrow::chart::Chart;

#[derive(Debug)]
pub struct BeatTick {
    /// The count of the beat we are on.
    beat: u32,
}

#[derive(Component)]
pub struct BeatTimer {
    /// The time stamp when the song started
    pub song_start: f32,
    /// Ticks for every beat
    pub beat_timer: Timer,
    /// Count the number of beat ticks, starting at 0
    /// Indexes into crate list of beats in a chart
    pub beat_count: u32,
}


impl BeatTick {
    pub fn beat(&self) -> u32 {
        self.beat
    }
}

impl BeatTimer {
    pub fn create(time: &Time, chart: &Chart) -> BeatTimer {
        use std::time::Duration;

        let duration = Duration::from_secs_f32(chart.beat_duration_secs());
        let beat_timer = Timer::new(duration, TimerMode::Repeating);

        BeatTimer {
            song_start: time.elapsed().as_secs_f32(),
            beat_timer,
            beat_count: 0
        }
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
    pub fn reset_and_load_settings_for(&mut self, chart: &Chart) {
        self.beat_count = 0; 

        let duration = chart.beat_duration_secs();
        let duration = std::time::Duration::from_secs_f32(duration);
        self.beat_timer = Timer::new(duration, TimerMode::Repeating);

    }
}

