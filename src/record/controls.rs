use bevy::prelude::*;
use serde::{
    Deserialize,
    Serialize
};
use crate::keycode_serde;
use crate::arrow::{
    ArrowSpawner
};
use crate::team_markers::{
    Marker,
    PlayerMarker
};

#[derive(Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct RecordingControls {
    /// Pauses the playback.
    #[serde(with = "keycode_serde")]
    pub pause: KeyCode,
    /// Moves forward one beat.
    #[serde(with = "keycode_serde")]
    pub forward: KeyCode,
    /// Moves backward one beat.
    #[serde(with = "keycode_serde")]
    pub backward: KeyCode
}
impl Default for RecordingControls {
    fn default() -> Self {
        Self {
            pause: KeyCode::Space,
            forward: KeyCode::ArrowDown,
            backward: KeyCode::ArrowUp,
        }
    }
}

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

fn handle_scroll_actions<T: Marker>(
    mut spawner_q: Query<&mut ArrowSpawner<T>>,
    input: Res<ButtonInput<KeyCode>>,
) {

}

pub struct RecordingControlsPlugin;
impl Plugin for RecordingControlsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, handle_pause_actions::<PlayerMarker>)
        ;
    }
}

