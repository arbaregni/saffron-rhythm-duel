use bevy::prelude::*;

use crate::input::{
    LaneHit
};
use crate::team_markers::{
    PlayerMarker,
    EnemyMarker
};
use crate::arrow::{
    LoadChartEvent
};

use super::{
    communicate::{
        Comms
    },
    RemoteLaneHit,
    GameMessage
};
use crate::judgement::{
    CorrectHitEvent
};


/// GameMessages from remote become local game events
pub fn translate_messages_from_remote(
    mut listener: ResMut<Comms>,
    mut remote_lane_hit: EventWriter<RemoteLaneHit>,
    mut remote_load_chart: EventWriter<LoadChartEvent<EnemyMarker>>,
) {
    let Some(msg) = listener.try_recv_message() else {
        return; // nothing to do
    };

    use GameMessage::*;
    match msg {
        LaneHit { lane } => {
            log::debug!("emitting remote lane hit");
            remote_lane_hit.send(RemoteLaneHit {
                lane
            });
        }
        LoadChart { chart_name } => {
            log::debug!("emitting remote chart load");
            remote_load_chart.send(LoadChartEvent::create(
                chart_name,
                EnemyMarker{},
            ));
        }
        CorrectHit { .. } => {
            log::debug!("emitting remote correct hit");
            // TODO
            log::warn!("TODO: emit remote correct hit");
        }
    }
}

/// Local GameEvents become GameMessages which are sent to the remote
pub fn translate_events_from_local(
    mut comms: ResMut<Comms>,
    mut lane_hit_ev: EventReader<LaneHit>,
    mut load_chart_ev: EventReader<LoadChartEvent<PlayerMarker>>,
    mut correct_hit_ev: EventReader<CorrectHitEvent>,
) {
    for ev in lane_hit_ev.read() {
        log::debug!("consuming local lane hit, passing to remote");
        comms.try_send_message(GameMessage::LaneHit {
            lane: ev.lane()
        });
    }
    for ev in load_chart_ev.read() {
        log::debug!("consuming local chart load, passing to remote");
        comms.try_send_message(GameMessage::LoadChart {
            chart_name: ev.chart_name().to_string()
        });
    }
    for ev in correct_hit_ev.read() {
        log::debug!("consuming local correct hit, passing to remote");
        comms.try_send_message(GameMessage::CorrectHit {
            lane_hit: ev.lane_hit.clone(),
            grade: ev.grade.clone(),
        });
    }
}


