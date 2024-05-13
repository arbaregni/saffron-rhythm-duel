use bevy::prelude::*;

use crate::lane::Lane;
use crate::team_markers::{
    Team,
};

use super::chart::Chart;


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
    pub (in crate::arrow) mode: SpawningMode,
    /// Scratch space for spawning the arrows
    pub (in crate::arrow) arrow_buf: Vec<Arrow>,
    /// The team we are spawning for.
    pub (in crate::arrow) team: Team,
}
impl ArrowSpawner {
    pub fn mode(&self) -> &SpawningMode {
        &self.mode
    }
    /// Creates the arrows ands appends them to the given buffer
    pub fn create_arrows_in(&self, buf: &mut Vec<Arrow>, time: &Time, beat: u32) {
        use SpawningMode::*;

        let now = time.elapsed().as_secs_f32();

        match self.mode() {
            Chart(chart) => {
                let lead_time = chart.lead_time_secs();
                for note in chart.get(beat) {
                    let lane = note.lane();
                    let arrow = Arrow::new(lane, now, now + lead_time, beat);
                    buf.push(arrow);
                }
            }
            SpawningMode::Random => {
                let lane = Lane::random();
                let lead_time = 1.5; // seconds
                let arrow = Arrow::new(lane, now, now + lead_time, beat);
                buf.push(arrow);
            }

            SpawningMode::Recording(_) => {
                // nothing to do
            }
        };

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

#[derive(Debug, Clone)]
pub enum SpawningMode {
    Chart(Chart),
    Recording(Chart),
    Random,
}



