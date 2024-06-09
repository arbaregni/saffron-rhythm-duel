use std::sync::Arc;

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

#[derive(Reflect)]
#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
struct ChartData {
    /// The user friendly name of the chart
    chart_name: String,

    /// Plain text about the chart.
    description: Option<String>,

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Reflect)]
pub struct ChartName {
    name: String
}

#[derive(Debug,Clone)]
#[derive(Reflect)]
pub struct Chart  {
    /// The data stored in the .json file
    data: ChartData,
    /// the filename, without the .json
    name: ChartName
}


#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
#[derive(Reflect)]
pub struct Note {
    /// Which lane does this note come down on?
    lane: Lane 
}



impl Chart {
    pub fn load_charts_in(buf: &mut Vec<Arc<Chart>>) -> Result<()> {
        use std::fs;
        let path = "assets/charts/";
        let dir = fs::read_dir(path)
            .with_context(|| format!("while reading chart directory at {path}"))?;

        for entry_or_err in dir {
            let Ok(entry) = entry_or_err
                .inspect_err(|e| log::error!("unable to read entry in chart directory: {e}"))
                else { continue; };

            let filepath = entry.path();

            let extension = filepath.extension().and_then(|s| s.to_str());
            if extension != Some("json") {
                continue;
            }

            // gets the filename, without the .json
            let Some(filename) = filepath.file_stem().and_then(|s| s.to_str()) else {
                log::error!("filepath without filestem: {filepath:?}");
                continue;
            };

            // this is fine because we have validated that the asset exists
            let name = ChartName { name: filename.to_string() };

            let Ok(chart) = Chart::try_load_from_name(name) 
                .inspect_err(|e| log::error!("while loading chart: {e:?}"))
                else { continue; };

            let chart = Arc::new(chart);


            buf.push(chart);
        }

        Ok(())
    }
    pub fn try_load_from_name(name: ChartName) -> Result<Chart> {
        let path = format!("assets/charts/{}.json", name.name);

        // read the chart from the file
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading from file {path}"))?;

        let chart_data: ChartData = serde_json::from_str(text.as_str())
            .context("parsing json")?;

        log::info!("Parsed chart '{}' from {}", name, path);

        let chart = Chart {
            data: chart_data,
            name
        };

        Ok(chart)
    }
    pub fn chart_name(&self) -> &ChartName {
        &self.name
    }
    pub fn friendly_name(&self) -> &str {
        self.data.chart_name.as_str()
    }
    pub fn sound_file(&self) -> Option<&str> {
        self.data.sound_file.as_ref().map(String::as_str)
    }
    /// Returns the number of seconds per beat (i.e. how fast they are generated)
    pub fn beat_duration_secs(&self) -> f32 {
        self.data.beat_duration_secs
    }
    /// Returns for how many beats arrows are visible
    pub fn lead_time_beats(&self) -> f32 {
        self.data.lead_time_beats
    }
    /// last beat that we see passing through target line
    pub fn last_beat(&self) -> f32 {
        let beats = self.data.beats.len() as f32;
        beats + self.lead_time_beats()
    }

    /// Iterate over all beats in the chart
    pub fn beats_iter(&self) -> impl Iterator<Item = &[Note]> + '_ {
        log::info!("in beats_iter");
        self.data
            .beats
            .iter()
            .map(|b| b.as_slice())
    }
}

impl std::fmt::Display for ChartName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Note {
    pub fn lane(&self) -> Lane {
        self.lane
    }
}


