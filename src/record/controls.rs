use bevy::prelude::*;

use crate::arrow::{
    ArrowSpawner
};
use crate::team_markers::{
    Marker,
    PlayerMarker
};

fn handle_pause_actions<T: Marker>(
    mut spawner_q: Query<&mut ArrowSpawner<T>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    for mut spawner in spawner_q.iter_mut() {
        log::info!("toggling pause state on spawner");
        spawner.toggle_is_paused();
    }
}

pub struct RecordingControlsPlugin;
impl Plugin for RecordingControlsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, handle_pause_actions::<PlayerMarker>)
        ;
    }
}

