use bevy::prelude::*;

use serde::{
    Deserialize,
    Serialize
};

use crate::{
    CliArgs,
    settings::UserSettings
};

use crate::lane::{
    Lane
};

use crate::judgement::{
    SuccessGrade,
};

pub mod communicate;
pub mod widgets;
pub mod translate;

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
        chart_name: String
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

    let Ok(comms) = communicate::Comms::try_init(cli.as_ref(), settings.as_ref())
        .inspect_err(|e| {
            log::error!("unable to initialize comms: {e:?}");
        })
        else { return; };

    commands
        .insert_resource(comms);
}

pub struct RemoteUserPlugin;
impl Plugin for RemoteUserPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_comms)
            .add_systems(Update, translate::translate_messages_from_remote)
            .add_systems(Update, translate::translate_events_from_local)
            .add_plugins(widgets::NetworkingWidgetsPlugin)
        ;
    }
}
