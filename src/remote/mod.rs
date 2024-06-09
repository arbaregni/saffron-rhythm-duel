use bevy::prelude::*;
use bevy::utils::Duration;

use serde::{
    Deserialize,
    Serialize
};

use crate::{
    CliArgs,
    user_settings::UserSettings
};
use crate::team_markers::{
    PlayerMarker
};

use crate::lane::{
    Lane
};

use crate::judgement::{
    grading::{
        SuccessGrade,
    }
};
use crate::song::{
    ChartName
};

pub mod communicate;
pub mod widgets;
pub mod translate;

use communicate::Comms;

/// Message sent from user to user to communicate game state.
/// We will use this for local -> remote and remote -> local
/// since comms are meant to be symmetric
#[derive(Debug, Clone)]
#[derive(Deserialize, Serialize)]
pub enum GameMessage {
    LaneHit {
        lane: Lane,
        beat: f32,
    },
    LoadChart {
        chart_name: ChartName,
        scroll_pos: f32,
    },
    CorrectHit {
        lane: Lane,
        grade: SuccessGrade,
        beat: f32,
    },
}

fn setup_comms(
    mut commands: Commands,
    cli: Res<CliArgs>,
    settings: Res<UserSettings>,
) {

    let Ok(comms) = Comms::try_init(cli.as_ref(), settings.as_ref())
        .inspect_err(|e| {
            log::error!("unable to initialize comms: {e:?}");
        })
        else { return; };

    commands
        .insert_resource(comms);
}

const CHART_SYNC_DURATION: Duration = Duration::from_secs(1);

fn sync_chart_progress(
    mut comms: ResMut<Comms>,
    spawner_q: Query<&crate::song::ArrowSpawner<PlayerMarker>>
) {
    // TODO: also send over the lack of arrow spawning
    let Some(spawner) = spawner_q.get_single().ok() else {
        return;
    };
    log::info!("syncing chart progress...");
    comms.try_send_message(GameMessage::LoadChart {
        chart_name: spawner.chart().chart_name().clone(),
        scroll_pos:  spawner.scroll_pos()
    });
}

pub struct RemoteUserPlugin;
impl Plugin for RemoteUserPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_comms)
            .add_systems(Update, (
                    translate::translate_messages_from_remote,
                    translate::translate_events_from_local,
                    sync_chart_progress.run_if(
                        bevy::time::common_conditions::on_timer(CHART_SYNC_DURATION)
                    )
            ))
            .add_plugins(widgets::NetworkingWidgetsPlugin)
        ;
    }
}
