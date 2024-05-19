use bevy::prelude::*;

use crate::lane::{
    Lane
};
use crate::{
    CliArgs,
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

fn translate_messages_from_remote(
    mut listener: ResMut<server::Listener>,
    mut remote_lane_hit: EventWriter<RemoteLaneHit>,
) {
    let Some(msg) = listener.message() else {
        return; // nothing to do
    };

    use server::GameMessage::*;
    match msg {
        LaneHit { lane } => {
            log::info!("emitting remote lane hit");
            remote_lane_hit.send(RemoteLaneHit {
                lane
            });
        }
    }

}

pub struct RemoteUserPlugin;
impl Plugin for RemoteUserPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Building remote user plugin");
        app
            .add_event::<RemoteLaneHit>()
            .add_systems(Update, translate_messages_from_remote)
        ;
    }
}
