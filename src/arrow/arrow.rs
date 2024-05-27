use bevy::prelude::*;

use crate::lane::Lane;

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
