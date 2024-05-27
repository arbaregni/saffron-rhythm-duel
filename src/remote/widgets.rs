use bevy::prelude::*;
use bevy::text::{
    Text2dBounds
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
struct StatusText {

}

pub fn setup_networking_status_text (
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    panel_query: Query<&SongPanel, With<EnemyMarker>>,
) {
    let font = asset_server.load(crate::BASE_FONT_NAME);
    let font_size = 100.0;
    let color = Color::rgb(0.9, 0.9, 0.9); // off white is a nice text color with this
                                           // backgournd

    let panel = panel_query.single();

    let text_content = "'ello govner".to_string();

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

    let status_text = StatusText{};

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

fn update_status_text(
    mut text_q: Query<(&mut Text, &StatusText)>,
    mut comms: ResMut<Comms>,
) {
    let (mut text, _status_text) = text_q.single_mut();

    let Some(status_rx) = comms.status_rx.as_mut() else {
        // nothing to do
        return
    };

    use tokio::sync::mpsc::error::TryRecvError;
    let content = match status_rx.try_recv() {
        Ok(c) => c,
        Err(TryRecvError::Disconnected) => {
            log::error!("status_rx disconnected");
            comms.status_rx = None;
            return
        }
        _ => {
            // nothing to do
            return
        }
    };

    text.sections[0].value = content;
}


/// Widgets for showing the status of connection to the remote player
pub struct NetworkingWidgetsPlugin;
impl Plugin for NetworkingWidgetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(LayoutState::Done), setup_networking_status_text)
            .add_systems(Update, update_status_text)
        ;
    }
}
