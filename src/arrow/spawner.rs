use bevy::prelude::*;

use anyhow::{
    Result,
    Context
};

use crate::team_markers::{
    Marker,
};

use super::{
    chart::Chart,
    arrow::Arrow
};

#[derive(Component)]
#[derive(Debug, Clone)]
pub struct ArrowSpawner<T: Marker> {
    /// How we will spawn the arrows
    chart: Chart,
    /// The timer marking off the beat
    beat_timer: Timer,
    /// The number of beat tickets. Indexes into the list of beats in a chart.
    beat_count: u32,
    /// The local timestamp when the song started
    song_start: f32,
    /// The team we are spawning for.
    team: T,
}
impl <T: Marker> ArrowSpawner<T> {
    /// Creates an arrow spawner
    pub fn create(chart_name: &str, time: &Time) -> Result<Self> {
        use std::time::Duration;

        let chart = Chart::try_load_from_file(chart_name)
            .with_context(|| format!("loading chart with name {chart_name}"))?;

        let duration = Duration::from_secs_f32(chart.beat_duration_secs());
        let beat_timer = Timer::new(duration, TimerMode::Repeating);

        let now = time.elapsed().as_secs_f32();

        Ok(Self {
            chart,
            beat_timer,
            song_start: now,
            beat_count: 0,
            team: T::marker(),
        })
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

    pub fn song_start(&self) -> f32 {
        self.song_start
    }
    pub fn beat_count(&self) -> u32 {
        self.beat_count
    }
    pub fn chart(&self) -> &Chart {
        &self.chart
    }

    /// Populates `buf` with a list of chart names that the user can select.
    /// Returns Ok or a user friendly error.
    pub fn selectable_chart_names(buf: &mut Vec<String>) -> Result<(), String> {
        use std::fs;
        let parent_path = "assets/charts/";
        
        let paths = fs::read_dir(parent_path)
            .map_err(|e| {
                format!("Unable to read available chart names from `assets/charts/`: {e}")
            })?;

        buf.clear();
        for path in paths {
            let path = path
                .map(|p| p.path().display().to_string())
                .unwrap_or("<unable to read>".to_string());
            buf.push(path);
        }

        Ok(())
    }

    /// Creates the arrows ands appends them to the given buffer
    pub fn create_arrows_in(&self, buf: &mut Vec<Arrow>, time: &Time, beat: u32) {
        let now = time.elapsed().as_secs_f32();
        let chart = self.chart();

        let lead_time = chart.lead_time_secs();
        for note in chart.get(beat) {
            let lane = note.lane();
            let arrow = Arrow::new(lane, now, now + lead_time, beat);
            buf.push(arrow);
        }
    }

}


#[derive(Component)]
/// Component that holds scratch space for spawning arrows
pub struct ArrowBuf {
    pub buf: Vec<Arrow>
}
impl ArrowBuf {
    pub fn new() -> Self {
        Self {
            // capacity for 4 arrows because we will have at most 1 per lane
            buf: Vec::with_capacity(4)
        }
    }
}

#[derive(Debug)]
pub struct BeatTick {
    /// The count of the beat we are on.
    beat: u32,
}

impl BeatTick {
    pub fn beat(&self) -> u32 {
        self.beat
    }
}
