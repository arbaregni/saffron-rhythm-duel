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
    }
}

/// Local GameEvents become GameMessages which are sent to the remote
pub fn translate_events_from_local(
    mut listener: ResMut<Comms>,
    mut lane_hit: EventReader<LaneHit>,
    mut load_chart: EventReader<LoadChartEvent<PlayerMarker>>,
) {
    for ev in lane_hit.read() {
        log::debug!("consuming local lane hit, passing to remote");
        listener.try_send_message(GameMessage::LaneHit {
            lane: ev.lane()
        });
    }
    for ev in load_chart.read() {
        log::debug!("consuming local chart load, passing to remote");
        listener.try_send_message(GameMessage::LoadChart {
            chart_name: ev.chart_name().to_string()
        });
    }
}


