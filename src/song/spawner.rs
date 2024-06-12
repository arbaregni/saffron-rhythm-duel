use std::sync::Arc;

use bevy::prelude::*;
use serde::{
    Deserialize,
    Serialize
};

use crate::team_markers::Marker;

use crate::song::{
    arrow::Arrow,
    chart::{
        Chart,
        ChartName
    }
};

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub enum SyncSpawnerEvent<T: Marker> {
    NotSpawning,
    Spawning {
        chart_name: ChartName,
        scroll_pos: f32,
        is_paused: bool,
        _team: T,
    }
}

#[derive(Component, Reflect)]
#[derive(Debug, Clone)]
pub struct ArrowSpawner<T: Marker> {
    /// How we will spawn the arrows
    chart: Arc<Chart>,

    /// The timer marking off the beat
    spawn_timer: Timer,

    /// Literally no fucking clue what this is anymore.
    /// Basically just counts the number of ticks but also used to scroll around
    scroll_pos: f32,

    /// The local timestamp when the song started
    song_start: f32,

    /// True if we are paused and not making new notes
    is_paused: bool,

    /// The team we are spawning for.
    _team: T,
}
impl <T: Marker> ArrowSpawner<T> {
    /// Creates an arrow spawner
    pub fn create(chart: Arc<Chart>, time: &Time) -> Self {
        use std::time::Duration;

        let duration = Duration::from_secs_f32(chart.beat_duration_secs());
        let spawn_timer = Timer::new(duration, TimerMode::Repeating);

        let now = time.elapsed().as_secs_f32();

        Self {
            chart,
            spawn_timer,
            song_start: now,
            scroll_pos: 0.0,
            is_paused: false,
            _team: T::marker(),
        }
    }

    pub fn change_scroll_pos(&mut self, dy: f32) {
        self.scroll_pos += dy;
    }

    pub fn tick(&mut self, time: &Time) {
        let now = time.elapsed().as_secs_f32();

        if self.is_paused {
            return
        }

        if now < self.song_start() {
            // not time to start the song yet
            return
        }

        self.spawn_timer.tick(time.delta());

        if !self.spawn_timer.just_finished() {
            // not time for another beat just yet
            return
        }

        self.change_scroll_pos(1.0);
    }

    pub fn song_start(&self) -> f32 {
        self.song_start
    }
    pub fn chart(&self) -> &Chart {
        &self.chart
    }

    // it's fine, used by networking
    pub fn scroll_pos(&self) -> f32 {
        self.scroll_pos + self.spawn_timer.fraction()
    }
    pub fn set_scroll_pos(&mut self, scroll_pos: f32) {
        log::info!("setting scroll pos");
        self.scroll_pos = scroll_pos;
    }
    pub fn set_is_paused(&mut self, is_paused: bool) {
        self.is_paused = is_paused;
    }
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }
    pub fn set_chart(&mut self, chart: Arc<Chart>) {
        log::warn!("warning: changing the chart. this may cause arrows to become desynced");
        self.chart = chart;
    }


    /// Returns the current beat that is passing through the target line
    pub fn curr_beat(&self) -> f32 {
        let frac = self.spawn_timer.fraction();
        let scroll = self.scroll_pos + frac;
        let lead_time = self.chart().lead_time_beats();
        // we want to give the arrows a bit of lead time, so we subtract it off here.
        // Example:
        // Say we have a lead time of 10 beats.
        // When we start the song, we want the first note (beat 0), to appear at the top of the
        // song panel and travel down. After 10 beats, the first note should pass the target line.
        //
        scroll - lead_time
    }

    pub fn toggle_is_paused(&mut self) {
        self.is_paused = !self.is_paused;
    }

    /// Iterate over the arrows needed to fulfill this song.
    pub fn arrows_to_spawn(&self) -> impl Iterator<Item = Arrow> + '_ {
        self.chart()
            .beats_iter()
            // enumerate it so we get the beat count information
            .enumerate()
            // take out all of the individual notes in a beat
            .flat_map(|(beat_count, notes)| {
                notes
                    .iter()
                    .map(move |note| (beat_count, note))
            })
            .map(|(beat_count, note)| {
                let lane = note.lane();
                // arrives at the finish line based on the index in the song.
                let arrives = beat_count as f32;
                Arrow::new(
                    lane,
                    arrives
                )
            })

    }

    pub fn is_finished(&self) -> bool {
        // if there's something we need to spawn, then we can't be finished
        if self.curr_beat() < self.chart().last_beat() {
            return false
        }

        log::info!("next beat to spawn: {:.4} >= {:.4}, ending song now",
             self.curr_beat(), self.chart().last_beat()
        );
        true
    } 
}


