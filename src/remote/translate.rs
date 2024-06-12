use bevy::prelude::*;

use crate::input::LaneHit;

use crate::team_markers::{
    PlayerMarker,
    EnemyMarker
};
use crate::song::{LoadChartRequest, SyncSpawnerEvent};

use super::{
    communicate::Comms,
    GameMessage
};
use crate::judgement::{
    CorrectHitEvent,
    RawCorrectHitEvent
};
use crate::input::RemoteLaneHit;

/// GameMessages from remote become local game events
pub fn translate_messages_from_remote(
    time: Res<Time>,
    mut listener: ResMut<Comms>,
    mut remote_lane_hit: EventWriter<RemoteLaneHit>,
    mut remote_load_chart: EventWriter<LoadChartRequest<EnemyMarker>>,
    mut remote_correct_hit: EventWriter<RawCorrectHitEvent<EnemyMarker>>,
    mut remote_sync_state: EventWriter<SyncSpawnerEvent<EnemyMarker>>,
) {
    let Some(msg) = listener.try_recv_message() else {
        return; // nothing to do
    };

    let now = time.elapsed().as_secs_f32();

    use GameMessage::*;
    match msg {
        LaneHit { lane, beat } => {
            log::debug!("emitting remote lane hit");
            remote_lane_hit.send(RemoteLaneHit::from(
                lane,
                beat,
                now
            ));
        }
        LoadChart { chart_name } => {
            log::debug!("emitting remote chart load");
            remote_load_chart.send(LoadChartRequest::from(
                chart_name
            ));
        }
        CorrectHit { lane, beat, grade } => {
            log::debug!("emitting remote correct hit");
            remote_correct_hit.send(RawCorrectHitEvent {
                lane_hit: RemoteLaneHit::from(lane, beat, now),
                grade,
            });
        }
        SyncSpawnerState(ev) => {
            log::debug!("emitting remote correct hit");
            remote_sync_state.send(ev);
        }
    }
}

/// Local GameEvents become GameMessages which are sent to the remote
pub fn translate_events_from_local(
    mut comms: ResMut<Comms>,
    mut lane_hit_ev: EventReader<LaneHit>,
    mut load_chart_ev: EventReader<LoadChartRequest<PlayerMarker>>,
    mut correct_hit_ev: EventReader<CorrectHitEvent>,
) {
    for ev in lane_hit_ev.read() {
        log::debug!("consuming local lane hit, passing to remote");
        comms.try_send_message(GameMessage::LaneHit {
            lane: ev.lane(),
            beat: ev.beat(),
        });
    }
    for ev in load_chart_ev.read() {
        log::debug!("consuming local chart load, passing to remote");
        comms.try_send_message(GameMessage::LoadChart {
            chart_name: ev.chart_name().clone(),
        });
    }
    for ev in correct_hit_ev.read() {
        log::debug!("consuming local correct hit, passing to remote");
        comms.try_send_message(GameMessage::CorrectHit {
            lane: ev.lane_hit.lane(),
            beat: ev.lane_hit.beat(),
            grade: ev.grade.clone(),
        });
    }
}


