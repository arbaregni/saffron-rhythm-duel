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
    spawn_timer: Timer,

    /// the last beat that we actually spawned
    last_spawned_beat: Option<u32>,

    /// the current desired beat to spawn
    beat_count: u32,

    /// The local timestamp when the song started
    song_start: f32,
    /// True if we are paused and not making new notes
    is_paused: bool,
    /// The team we are spawning for.
    _team: T,
}
impl <T: Marker> ArrowSpawner<T> {
    /// Creates an arrow spawner
    pub fn create(chart_name: &str, time: &Time) -> Result<Self> {
        use std::time::Duration;

        let chart = Chart::try_load_from_file(chart_name)
            .with_context(|| format!("loading chart with name {chart_name}"))?;

        let duration = Duration::from_secs_f32(chart.beat_duration_secs());
        let spawn_timer = Timer::new(duration, TimerMode::Repeating);

        let now = time.elapsed().as_secs_f32();

        Ok(Self {
            chart,
            spawn_timer,
            song_start: now,
            last_spawned_beat: None,
            beat_count: 0,
            is_paused: false,
            _team: T::marker(),
        })
    }

    pub fn tick(&mut self, time: &Time) -> Option<BeatTick> {
        let now = time.elapsed().as_secs_f32();

        if self.is_paused {
            return None;
        }

        if now < self.song_start() {
            // not time to start the song yet
            return None;
        }

        self.spawn_timer.tick(time.delta());

        if !self.spawn_timer.just_finished() {
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
    /// Returns the number of beats that have passed
    pub fn beat_count(&self) -> u32 {
        self.beat_count
    }
    /// Returns the beats that we have spawned, including fractional beats
    pub fn beat_fraction(&self) -> f32 {
        let frac = self.spawn_timer.fraction();
        self.beat_count() as f32 + frac
    }
    /// Returns the current beat that is passing through the target line
    pub fn curr_beat(&self) -> f32 {
        self.beat_fraction() - self.chart().lead_time_beats()
    }

    pub fn chart(&self) -> &Chart {
        &self.chart
    }

    pub fn toggle_is_paused(&mut self) {
        self.is_paused = !self.is_paused;
    }

    fn next_beat_to_spawn(&self) -> Option<u32> {
        match self.last_spawned_beat {
            None => Some(0),
            Some(b) => {
                if b < self.beat_count {
                    Some(b + 1)
                } else {
                    None
                }
            }
        }
    }
    /// Creates the arrows ands appends them to the given buffer
    pub fn create_arrows_in(&mut self, buf: &mut ArrowBuf, time: &Time) {
        let now = time.elapsed().as_secs_f32();

        while let Some(beat) = self.next_beat_to_spawn() {
            self.chart
                .get(beat)
                .into_iter()
                .for_each(|note| {
                    let lane = note.lane();
                    let arrival = beat as f32 + self.chart().lead_time_beats();
                    let arrow = Arrow::new(lane, now, beat, arrival);
                    buf.push(arrow);
                });
            self.last_spawned_beat = Some(beat); 
        }
    }
    pub fn is_finished(&self) -> bool {
        // if there's something we need to spawn, then we can't be finished
        if self.next_beat_to_spawn().is_some() {
            return false
        }

        if self.curr_beat() < self.chart().last_beat() {
            return false
        }

        log::info!("next beat to spawn: next beat to spawn is {:?} and {:.4} >= {:.4}, ending song now",
            self.next_beat_to_spawn(), self.curr_beat(), self.chart().last_beat()
        );
        true
    }

    pub fn move_forward(&mut self) {
        self.beat_count += 1;
    }
    pub fn move_backward(&mut self) {
        self.beat_count -= 1;
    }

}


#[derive(Component)]
/// Component that holds scratch space for spawning arrows
pub struct ArrowBuf {
    buf: Vec<Arrow>
}
impl ArrowBuf {
    pub fn new() -> Self {
        Self {
            // capacity for 4 arrows because we will have at most 1 per lane
            buf: Vec::with_capacity(4)
        }
    }
    pub fn push(&mut self, arrow: Arrow) {
        self.buf.push(arrow);
    }
    pub fn drain(&mut self) -> impl Iterator<Item = Arrow> + '_ {
        self.buf.drain(..)
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
