use bevy::prelude::*;

use crate::lane::Lane;

#[derive(Component, Debug, Clone)]
pub struct Arrow {
    pub (in crate::song) lane: Lane,
    pub (in crate::song) status: ArrowStatus,
    /// When the arrow is created and first visibile to player
    pub (in crate::song) creation_time: f32,
    /// Which beat number this was created for
    pub (in crate::song) beat_number: u32,
    /// Which beat this is supposed to arrive at
    pub arrival_beat: f32,
}
impl Arrow {
    pub fn new(lane: Lane, creation_time: f32, beat_number: u32, arrival_beat: f32) -> Arrow {
        Arrow {
            lane,
            status: ArrowStatus::Pending,
            creation_time,
            beat_number,
            arrival_beat,
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
    pub fn marked_dropped(&mut self) {
        self.status = ArrowStatus::Dropped;
    }
    pub fn creation_time(&self) -> f32 {
        self.creation_time
    }
    pub fn beat_number(&self) -> u32 {
        self.beat_number
    }
    pub fn beat_fraction(&self) -> f32 {
        self.beat_number() as f32
    }
    pub fn arrival_beat(&self) -> f32 {
        self.arrival_beat
    }
}


#[derive(Debug, Copy, Clone)]
pub enum ArrowStatus {
    /// Has been generated, but not clicked
    Pending,
    /// Has been clicked
    Completed,
    /// Can never be clicked again
    Dropped,
}
impl ArrowStatus {
    /// Is this arrow still on the  board?
    /// I.e. can the user still click it.
    pub fn is_pending(self) -> bool {
        match self {
            ArrowStatus::Pending => true,
            ArrowStatus::Completed => false,
            ArrowStatus::Dropped => false,
        }
    }
}
