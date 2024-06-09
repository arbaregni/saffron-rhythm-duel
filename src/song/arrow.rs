use bevy::prelude::*;

use crate::lane::Lane;

#[derive(Component, Debug, Clone)]
#[derive(Reflect)]
pub struct Arrow {
    lane: Lane,
    status: ArrowStatus,
    /// Which beat this is supposed to arrive at (i.e. when you hit it)
    beat: f32,
}
impl Arrow {
    pub fn new(lane: Lane, arrival_beat: f32) -> Arrow {
        Arrow {
            lane,
            status: ArrowStatus::Pending,
            beat: arrival_beat,
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
    pub fn mark_dropped(&mut self) {
        self.status = ArrowStatus::Dropped;
    }
    /// The beat that when this arrow passes the target line
    pub fn arrival_beat(&self) -> f32 {
        self.beat
    }
}


#[derive(Debug, Copy, Clone)]
#[derive(Reflect)]
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

