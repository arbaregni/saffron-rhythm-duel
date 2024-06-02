use bevy::prelude::*;

use serde::{
    Deserialize,
    Serialize
};

use crate::team_markers::{
    PlayerMarker,
    EnemyMarker,
    Marker,
};

use crate::arrow::{
    Arrow,
};
use crate::input::{
    RawLaneHit,
    LaneHit
};

pub const KEYPRESS_TOLERANCE_SECS: f32 = 0.5; // in seconds
                                              
/// Represents when the user hits the lane when an arrow is passing the target line, and it
/// completes that arrow.
#[derive(Event)]
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct RawCorrectHitEvent<T: Marker> {
    /// The lane hit
    pub lane_hit: RawLaneHit<T>,
    /// The grade the judgment system gave
    pub grade: SuccessGrade,
}
impl <T: Marker> RawCorrectHitEvent<T> {
    pub fn grade(&self) -> SuccessGrade {
        self.grade
    }
}
pub type CorrectHitEvent = RawCorrectHitEvent<PlayerMarker>;
pub type RemoteCorrectHitEvent = RawCorrectHitEvent<EnemyMarker>;

/// Represents when the user hits the lane, and there is a nearby note,
/// But we don't want to count it as 'completing' that note.
#[derive(Event)]
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct RawIncorrectHitEvent<T: Marker> {
    /// THe lane hit
    pub lane_hit: RawLaneHit<T>,
    /// The grade the judgement system gave
    pub grade: FailingGrade,
}
pub type IncorrectHitEvent = RawIncorrectHitEvent<PlayerMarker>;
pub type RemoteIncorrectHitEvent = RawIncorrectHitEvent<EnemyMarker>;

/// Event representing when the user attempts to complete a note, but are too early or late to be
/// considered 'correct'
#[derive(Event)]
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct RawMissfireEvent<T: Marker> {
    /// The lane hit that originated this missfire
    pub lane_hit: RawLaneHit<T>,
}
pub type MissfireEvent = RawMissfireEvent<PlayerMarker>;
pub type RemoteMissfireEvent = RawMissfireEvent<EnemyMarker>;

#[derive(Debug,Copy,Clone,Deserialize,Serialize)]
pub enum SuccessGrade {
    Perfect,
    Good,
    Fair,
}
impl SuccessGrade {
    pub fn is_perfect(self) -> bool {
        use SuccessGrade::*;
        match self {
            Perfect => true,
            Good | Fair => false,
        }
    }
}

#[derive(Debug,Copy,Clone,Deserialize,Serialize)]
pub enum FailingGrade {
    Early,
    Late,
}


#[derive(Resource)]
pub struct JudgementSettings {
    // passing grades are perfect, good, and fair
    perfect_cutoff: f32,
    good_cutoff: f32,
    fair_cutoff: f32,

    // if it failed, we just say early or late.
    // There's also the possibility that we couldn't find a note whatsoever
}


#[derive(Debug, Copy, Clone)]
pub enum Grade {
    Success(SuccessGrade),
    Fail(FailingGrade)
}

impl JudgementSettings {
    pub fn new() -> Self {
        Self {
            perfect_cutoff: 0.10, // fractions of a beat
            good_cutoff:    0.20,
            fair_cutoff:    0.40,
        }
    }
    pub fn judge(&self, lane_hit: &LaneHit, arrow: &Arrow) -> Grade {
        let hit_time = lane_hit.beat();
        let arrival_time = arrow.arrival_beat();
        let diff = (arrival_time - hit_time).abs();

        log::debug!("grading arrow {arrow:?} (arrival time = {arrival_time:.2}) vs lane_hit {lane_hit:?}");

        if diff < self.perfect_cutoff {
            log::debug!("meets perfect cutoff, {:.2} < {:.2}", diff, self.perfect_cutoff);
            Grade::Success(SuccessGrade::Perfect)
        } else if diff < self.good_cutoff {
            log::debug!("meets good cutofff, {:.2} < {:.2}", diff, self.good_cutoff);
            Grade::Success(SuccessGrade::Good)
        } else if diff < self.fair_cutoff {
            log::debug!("meets fair cutoff, {:.2} < {:.2}", diff, self.fair_cutoff);
            Grade::Success(SuccessGrade::Fair)
        } else {
            log::debug!("failed to hit note");
            if hit_time < arrival_time {
                // hit before it arrived
                Grade::Fail(FailingGrade::Early)
            } else {
                Grade::Fail(FailingGrade::Late)
            }

        }

    } 
}


