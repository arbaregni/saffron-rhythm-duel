use bevy::prelude::*;

use crate::lane::{
    Lane
};
use crate::input::{
    LaneHit
};

pub mod server;

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

/// GameMessages from remote become local game events
fn translate_messages_from_remote(
    mut listener: ResMut<server::Listener>,
    mut remote_lane_hit: EventWriter<RemoteLaneHit>,
) {
    let Some(msg) = listener.try_recv_message() else {
        return; // nothing to do
    };

    use server::GameMessage::*;
    match msg {
        LaneHit { lane } => {
            log::debug!("emitting remote lane hit");
            remote_lane_hit.send(RemoteLaneHit {
                lane
            });
        }
    }
}

/// Local GameEvents become GameMessages which are sent to the remote
fn translate_events_from_local(
    mut listener: ResMut<server::Listener>,
    mut lane_hit: EventReader<LaneHit>,
) {
    use server::GameMessage;
    for ev in lane_hit.read() {
        log::debug!("consuming local lane hit, passing to remote");
        listener.try_send_message(GameMessage::LaneHit {
            lane: ev.lane()
        });
    }
}

pub struct RemoteUserPlugin;
impl Plugin for RemoteUserPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Building remote user plugin");
        app
            .add_event::<RemoteLaneHit>()
            .add_systems(Update, translate_messages_from_remote)
            .add_systems(Update, translate_events_from_local)
        ;
    }
}
