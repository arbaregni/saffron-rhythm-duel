use bevy::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

use crate::lane::Lane;

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
#[derive(Resource)]
pub struct Chart {
    /// The name of the chart
    chart_name: String,

    /// The song file name in assets/songs folder
    sound_file: Option<String>,

    /// How long a beat lasts, in seconds. Controls how fast the beats are generated
    beat_duration_secs: f32,

    /// How many seconds the notes spend scrolling down before they can be hit. Controls how fast
    /// the arrows move.
    lead_time_secs: f32,

    /// Each beat is a list of potential notes to be played
    beats: Vec<Vec<Note>>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct Note {
    /// Which lane does this note come down on?
    lane: Lane 
}

impl Chart {
    pub fn get(&self, beat: u32) -> &[Note] {
        const EMPTY: &'static [Note] = &[];
        match self.beats.get(beat as usize) {
            Some(notes) => notes.as_slice(),
            None => EMPTY
        }
    }
    pub fn chart_name(&self) -> &str {
        self.chart_name.as_str()
    }
    pub fn sound_file(&self) -> Option<&str> {
        self.sound_file.as_ref().map(String::as_str)
    }
    pub fn beat_duration_secs(&self) -> f32 {
        self.beat_duration_secs
    }
    pub fn lead_time_secs(&self) -> f32 {
        self.lead_time_secs
    }
    pub fn num_beats(&self) -> u32 {
        self.beats.len() as u32
    }
    pub fn total_duration(&self) -> f32 {
        self.beat_duration_secs() * (self.num_beats() as f32)
            + self.lead_time_secs()
    }
}

impl Note {
    pub fn lane(&self) -> Lane {
        self.lane
    }
}
