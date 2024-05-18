mod server;

use bevy::prelude::*;

use crate::lane::{
    Lane
};
use crate::{
    CliArgs,
};

#[derive(Event)]
#[derive(Debug,Clone)]
pub struct RemoteLaneHit {
    lane: Lane,
    // can't trust the time that they would give us
}
impl RemoteLaneHit {
    pub fn lane(&self) -> Lane {
        self.lane
    }
}

#[derive(Component)]
#[derive(Debug,Clone)]
pub struct RemoteListener {
    timer: Timer,
}


fn setup(
    mut commands: Commands,
    cli: Res<CliArgs>,
)
{

    if cli.disable_remote_listener {
        return;
    }

    commands.spawn(RemoteListener{
        timer: Timer::new(
           std::time::Duration::from_secs_f32(0.12),
           TimerMode::Repeating
        )
    });
}



/// Listens on a separate thread for responses from the remote user.
/// Turns the responses into events
///  -> RemoteLaneHit
fn listen_for_remote_events(
    time: Res<Time>,
    mut query: Query<&mut RemoteListener>,
    mut remote_lane_hit: EventWriter<RemoteLaneHit>,
) {

    query
        .iter_mut()
        .for_each(|mut listener| {

            // just fake the network stuff for now
            // TODO: message passing here
            listener.timer.tick(time.delta());
            if listener.timer.just_finished() {
                log::info!("emitting remote lane hit");
                let lane = Lane::random();
                remote_lane_hit.send(RemoteLaneHit {
                    lane
                });
            }
        });
}

pub struct RemoteUserPlugin;
impl Plugin for RemoteUserPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Building remote user plugin");
        app
            .add_event::<RemoteLaneHit>()
            .add_systems(Startup, setup)
            .add_systems(Update, listen_for_remote_events)
            .add_plugins(server::ServerPlugin)
        ;
    }
}
