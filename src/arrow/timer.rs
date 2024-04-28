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

#[derive(Event)]
pub struct BeatTickEvent {
    /// The count of the beat we are on.
    beat: u32,
}

#[derive(Resource)]
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


impl BeatTickEvent {
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

}


pub fn check_for_beat(
    time: Res<Time>,
    mut beat_timer: ResMut<BeatTimer>,
    mut events: EventWriter<BeatTickEvent>,
)
{
    let now = time.elapsed().as_secs_f32();

    if now < beat_timer.song_start() {
        // not time to start the song yet
        return;
    }

    beat_timer.beat_timer.tick(time.delta());
    if !beat_timer.beat_timer.just_finished() {
        // not time for another beat (set of arrows) yet, just return
        return;
    }
    let beat = beat_timer.beat_count;
    beat_timer.beat_count += 1;

    events.send(BeatTickEvent {
        beat
    });
}
