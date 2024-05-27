use bevy::prelude::*;

use crate::lane::Lane;
use crate::team_markers::{
    Team,
    Marker,
};

use super::chart::Chart;


#[derive(Event)]
#[derive(Debug, Clone)]
pub struct SongFinishedEvent<T: Marker> {
    team: T,
}
impl <T: Marker> SongFinishedEvent<T> {
    pub fn create(team: T) -> Self {
        Self { team }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Arrow {
    pub (in crate::arrow) lane: Lane,
    pub (in crate::arrow) status: ArrowStatus,
    /// When the arrow is created and first visibile to player
    pub (in crate::arrow) creation_time: f32,
    /// When the arrow arrives at the judgement line, i.e. the ideal time for the player to hit it
    pub (in crate::arrow) arrival_time: f32, 
    /// Which beat number this was created for
    pub (in crate::arrow) beat_number: u32,
}
impl Arrow {
    pub fn new(lane: Lane, creation_time: f32, arrival_time: f32, beat_number: u32) -> Arrow {
        Arrow {
            lane,
            status: ArrowStatus::Pending,
            creation_time,
            arrival_time,
            beat_number,
        }
    }
    pub fn height() -> f32 {
        20.0 // hard coded for now
    }
    pub fn lane(&self) -> Lane {
        self.lane
    }
    pub fn status(&self) -> ArrowStatus {
        self.status
    }
    pub fn mark_completed(&mut self) {
        self.status = ArrowStatus::Completed;
    }
    pub fn creation_time(&self) -> f32 {
        self.creation_time
    }
    pub fn arrival_time(&self) -> f32 {
        self.arrival_time
    }
    pub fn beat_number(&self) -> u32 {
        self.beat_number
    }
}


#[derive(Debug, Copy, Clone)]
pub enum ArrowStatus {
    /// Has been generated, but not clicked
    Pending,
    /// Has been clicked
    Completed,
}
impl ArrowStatus {
    /// Is this arrow still on the  board?
    /// I.e. can the user still click it.
    pub fn is_pending(self) -> bool {
        match self {
            ArrowStatus::Pending => true,
            ArrowStatus::Completed => false,
        }
    }
}

#[derive(Component)]
#[derive(Debug, Clone)]
pub struct ArrowSpawner {
    /// How we will spawn the arrows
    chart: Option<Chart>,
    /// The team we are spawning for.
    team: Team,
}
impl ArrowSpawner {
    /// Creates an arrow spawner
    pub fn create(team: Team) -> Self {
        Self {
            chart: None,
            team,
        }
    }

    pub fn chart(&self) -> Option<&Chart> {
        self.chart.as_ref()
    }
    pub fn team(&self) -> Team {
        self.team
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

    pub fn set_chart(&mut self, chart: Chart) {
        self.chart = Some(chart);
    }

    /// Creates the arrows ands appends them to the given buffer
    pub fn create_arrows_in(&self, buf: &mut Vec<Arrow>, time: &Time, beat: u32) {
        let now = time.elapsed().as_secs_f32();

        let Some(chart) = self.chart.as_ref() else {
            return;
        };

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




