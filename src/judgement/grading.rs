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

use crate::song::{
    Arrow,
};
use crate::input::{
    RawLaneHit,
    LaneHit
};


// these are all in fractions of a beat
const DEFAULT_PERFECT_CUTOFF: f32 = 0.10;
const DEFAULT_GOOD_CUTOFF: f32    = 0.20;
const DEFAULT_FAIR_CUTOFF: f32    = 0.40;

                                              
/// Represents when the user hits the lane when an arrow is passing the target line, and it
/// completes that arrow.
#[derive(Event)]
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct RawCorrectHitEvent<T: Marker> {
    /// The lane hit
    pub lane_hit: RawLaneHit<T>,
    /// The arrow position when the hit happened
    pub arrow_pos: Vec3,
    /// The grade the judgment system gave
    pub grade: SuccessGrade,
}
impl <T: Marker> RawCorrectHitEvent<T> {
    pub fn grade(&self) -> SuccessGrade {
        self.grade
    }
}
pub type CorrectHitEvent = RawCorrectHitEvent<PlayerMarker>;
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
            perfect_cutoff: DEFAULT_PERFECT_CUTOFF,
            good_cutoff:    DEFAULT_GOOD_CUTOFF,
            fair_cutoff:    DEFAULT_FAIR_CUTOFF,
        }
    }
    pub fn judge(&self, lane_hit: &LaneHit, arrow: &Arrow) -> Grade {
        let hit_time = lane_hit.beat();
        let arrival_time = arrow.arrival_beat();
        let diff = (arrival_time - hit_time).abs();


        let grade = 
            if diff < self.perfect_cutoff {
                Grade::Success(SuccessGrade::Perfect)
            } else if diff < self.good_cutoff {
                Grade::Success(SuccessGrade::Good)
            } else if diff < self.fair_cutoff {
                Grade::Success(SuccessGrade::Fair)
            } else {
                if hit_time < arrival_time {
                    // hit before it arrived
                    Grade::Fail(FailingGrade::Early)
                } else {
                    Grade::Fail(FailingGrade::Late)
                }
            };

        log::debug!("grading {arrow:?} vs lane_hit {lane_hit:?} --> received {grade:?} due to a diff of {diff:.4}");

        grade
    } 
}


