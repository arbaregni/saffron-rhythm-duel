use bevy::prelude::*;
use bevy::text::{
    Text2dBounds
};

use crate::song::{
    SongState,
    LoadChartRequest, 
};
use crate::layout::{
    Layer,
    SongPanel,
    LayoutState,
};
use crate::team_markers::{
    EnemyMarker,
};

use super::{
    communicate::Comms
};

#[derive(Component)]
struct StatusText { }

#[derive(Debug,Clone)]
pub enum NetStatus {
    Disconnected,
    Connected,
    Listening(String),
    Connecting(String),
    Error(String),
}

pub fn setup_networking_status_text (
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    panel_query: Query<&SongPanel, With<EnemyMarker>>,
) {
    let font = asset_server.load(crate::BASE_FONT_NAME);
    let font_size = 60.0;
    let color = Color::rgb(0.9, 0.9, 0.9); // off white is a nice text color with this
                                           // backgournd

    let panel = panel_query.single();

    let text_content = "".to_string();

    let mut pos = panel.bounds().center();
    pos.z = Layer::TextAlerts.z();

    let transform = Transform {
        translation: pos,
        ..default()
    };

    let style = TextStyle { font, font_size, color };
    let text = Text {
        sections: vec![
            TextSection {
                value: text_content,
                style,
            }
        ],
        ..default()
    };

    let status_text = StatusText {
    };

    commands.spawn((
        EnemyMarker,
        status_text,
        Text2dBundle {
            text,
            transform,
            text_2d_bounds: Text2dBounds {
                size: panel.bounds().size().truncate() // clips of the z component
            },
            ..default()
        }
    ));
}

fn update_status_text_on_remote_event(
    mut text_q: Query<(&mut Text, &mut StatusText)>,
    mut load_chart_ev: EventReader<LoadChartRequest<EnemyMarker>>,
    comms: Res<Comms>,
) {
    if load_chart_ev.is_empty() {
        return;
    }
    load_chart_ev.clear();

    if !matches!(comms.net_status(), NetStatus::Connected) {
        return;
    }

    let (mut text, _status_text) = text_q.single_mut();

    // just clear it out when we load a chart
    text.sections[0].value.clear();
}



fn update_status_text(
    mut text_q: Query<(&mut Text, &mut StatusText)>,
    song_state: Res<State<SongState<EnemyMarker>>>,
    mut comms: ResMut<Comms>,
) {
    let (mut text, _status_text) = text_q.single_mut();

    let no_song_yet = matches!(song_state.get(), SongState::NotPlaying);

    if !comms.update_net_status().is_changed() {
        // nothing to do. no sense rewriting anything
        return;
    }

    match comms.net_status() {
        NetStatus::Disconnected => {
            text.sections[0].value.clear();
            text.sections[0].value.push_str("disconnected");
        }
        NetStatus::Connected => {
            text.sections[0].value.clear();
            if no_song_yet {
                text.sections[0].value.push_str("waiting for remote user to select a song");
            }
        }
        NetStatus::Error(content) => {
            text.sections[0].value.clear();
            text.sections[0].value.push_str("[ERROR] ");
            text.sections[0].value.push_str(content.as_str());
        }
        NetStatus::Listening(content) | NetStatus::Connecting(content) => {
            text.sections[0].value.clear();
            text.sections[0].value.push_str(content.as_str());
        }
    }

    if matches!(comms.net_status(), NetStatus::Connected) {
        log::info!("new connection");
    }

}


/// Widgets for showing the status of connection to the remote player
pub struct NetworkingWidgetsPlugin;
impl Plugin for NetworkingWidgetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(LayoutState::Done), setup_networking_status_text)
            .add_systems(Update, update_status_text)
            .add_systems(Update, update_status_text_on_remote_event)
        ;
    }
}
