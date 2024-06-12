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
        ChartName,
        ChartAssets,
    }
};

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

    pub fn get_sync_state(&self) -> SpawnerSyncableState {
        SpawnerSyncableState {
            chart_name: Some(self.chart().chart_name().clone()),
            scroll_pos: Some(self.scroll_pos()),
            is_paused: Some(self.is_paused),
        }
    }
    pub fn from_syncable_state(ev: SpawnerSyncableState, chart_assets: &ChartAssets, latency_tolerance: f32, time: &Time) -> ArrowSpawner<T> {
        let empty = chart_assets.empty();
        let mut spawner = ArrowSpawner::create(empty, time);
        spawner.load_from_syncable_state(ev, chart_assets, latency_tolerance);
        spawner
    }
    pub fn load_from_syncable_state(&mut self, ev: SpawnerSyncableState, chart_assets: &ChartAssets, latency_tolerance: f32) {
        let SpawnerSyncableState { chart_name, scroll_pos, is_paused, } = ev;

        chart_name
            // Only change it on a new chart
            .filter(|chart_name| self.chart().chart_name() != chart_name)
            .map(|chart_name| {
                let chart = chart_assets.get(&chart_name);
                Arc::clone(chart)
            })
            .then(|chart| {
                log::warn!("changing charts, this could cause arrows to desync");
                self.chart = chart;
            });

        scroll_pos
            // Only change if the jump is big enough
            .filter(|scroll_pos| (scroll_pos - self.scroll_pos).abs() >= latency_tolerance)
            .then(|scroll_pos| {
                self.scroll_pos = scroll_pos; 
            });

        is_paused
            .then(|is_paused| {
                self.is_paused = is_paused;
            });
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnerSyncableState {
    chart_name: Option<ChartName>,
    scroll_pos: Option<f32>,
    is_paused: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub enum SyncSpawnerEvent<T: Marker> {
    NotSpawning,
    Spawning(SpawnerSyncableState, T)
}

trait OptionExt<T> : Sized {
    fn to_option(self) -> Option<T>;
    fn then<F: FnMut(T)>(self, mut f: F) {
        match self.to_option() {
            Some(x) => f(x),
            None => {},
        };
    }
}
impl <T> OptionExt<T> for Option<T> {
    fn to_option(self) -> Option<T> { self }
}


