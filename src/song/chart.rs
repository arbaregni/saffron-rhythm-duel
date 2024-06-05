use anyhow::{
    Result,
    Context
};
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

    /// How long a beat lasts, in seconds. Controls how fast the beats are generated
    beat_duration_secs: f32,

    /// How many beats the notes spend scrolling down before they can be hit. Controls how fast
    /// the arrows move.
    lead_time_beats: f32,

    /// Song end beats. Defaults to zero
    song_end_beats: Option<f32>,

    /// Each beat is a list of potential notes to be played
    beats: Vec<Vec<Note>>,

    /// The song file name in assets/songs folder
    sound_file: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct Note {
    /// Which lane does this note come down on?
    lane: Lane 
}

impl Chart {
    pub fn try_load_from_file(filename: &str) -> Result<Chart> {
        let path = format!("assets/charts/{filename}.json");

        // read the chart from the file
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading from file {path}"))?;

        let chart: Chart = serde_json::from_str(text.as_str())
            .context("parsing json")?;

        log::info!("Parsed chart '{}' from {}", chart.chart_name(), path);

        Ok(chart)
    }
    pub fn chart_name(&self) -> &str {
        self.chart_name.as_str()
    }
    pub fn sound_file(&self) -> Option<&str> {
        self.sound_file.as_ref().map(String::as_str)
    }
    /// Returns the number of seconds per beat (i.e. how fast they are generated)
    pub fn beat_duration_secs(&self) -> f32 {
        self.beat_duration_secs
    }
    /// Returns for how many beats arrows are visible
    pub fn lead_time_beats(&self) -> f32 {
        self.lead_time_beats
    }
    /// last beat that we see passing through target line
    pub fn last_beat(&self) -> f32 {
        let beats = self.beats.len() as f32;
        beats + self.lead_time_beats()
    }

    /// Iterate over all beats in the chart
    pub fn beats_iter(&self) -> impl Iterator<Item = &[Note]> + '_ {
        log::info!("in beats_iter");
        self.beats
            .iter()
            .map(|b| b.as_slice())
    }
}

impl Note {
    pub fn lane(&self) -> Lane {
        self.lane
    }
}
