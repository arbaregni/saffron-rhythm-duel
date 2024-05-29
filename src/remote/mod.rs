use bevy::prelude::*;

use serde::{
    Deserialize,
    Serialize
};

use crate::lane::{
    Lane
};

pub mod communicate;
pub mod widgets;
pub mod translate;

#[derive(Event)]
#[derive(Debug)]
pub struct RemoteLaneHit {
    lane: Lane
}
impl RemoteLaneHit {
    pub fn lane(&self) -> Lane {
        self.lane
    }
}
/// Message sent from user to user to communicate game state.
/// We will use this for local -> remote and remote -> local
/// since comms are meant to be symmetric
#[derive(Debug, Clone)]
#[derive(Deserialize, Serialize)]
pub enum GameMessage {
    LaneHit {
        lane: Lane
    },
    LoadChart {
        chart_name: String
    },
    CorrectHit {
        lane_hit: crate::input::LaneHit,
        grade: crate::judgement::SuccessGrade,
    },
}


pub struct RemoteUserPlugin;
impl Plugin for RemoteUserPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<RemoteLaneHit>()
            .add_systems(Update, translate::translate_messages_from_remote)
            .add_systems(Update, translate::translate_events_from_local)
            .add_plugins(widgets::NetworkingWidgetsPlugin)
        ;
    }
}
