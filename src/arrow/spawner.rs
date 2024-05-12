use bevy::prelude::*;

use crate::lane::Lane;

use super::chart::Chart;

#[derive(Component, Debug, Copy, Clone)]
pub struct Arrow {
    pub (in crate::arrow) lane: Lane,
    pub (in crate::arrow) status: ArrowStatus,
    /// When the arrow is created and first visibile to player
    pub (in crate::arrow) creation_time: f32,
    /// When the arrow arrives at the judgement line, i.e. the ideal time for the player to hit it
    pub (in crate::arrow) arrival_time: f32, 
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

#[derive(Resource)]
#[derive(Debug, Clone)]
pub struct ArrowSpawner {
    /// How we will spawn the arrows
    pub (in crate::arrow) mode: SpawningMode,
    /// Scratch space for spawning the arrows
    pub (in crate::arrow) arrow_buf: Vec<Arrow>,
}

#[derive(Debug, Clone)]
pub enum SpawningMode {
    Chart(Chart),
    Recording(Chart),
    Random,
}

impl Arrow {
    pub fn new(lane: Lane, creation_time: f32, arrival_time: f32) -> Arrow {
        Arrow {
            lane,
            status: ArrowStatus::Pending,
            creation_time,
            arrival_time,
        }
    }
    pub fn height() -> f32 {
        20.0 // hard coded for now
    }
    pub fn lane(self) -> Lane {
        self.lane
    }
    pub fn status(self) -> ArrowStatus {
        self.status
    }
    pub fn mark_completed(&mut self) {
        self.status = ArrowStatus::Completed;
    }
    pub fn creation_time(self) -> f32 {
        self.creation_time
    }
    pub fn arrival_time(self) -> f32 {
        self.arrival_time
    }
}


impl ArrowSpawner {
    pub fn mode(&self) -> &SpawningMode {
        &self.mode
    }
}


