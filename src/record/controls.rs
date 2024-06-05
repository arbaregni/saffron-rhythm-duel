use bevy::prelude::*;
use serde::{
    Deserialize,
    Serialize
};
use crate::user_settings::UserSettings;
use crate::keycode_serde;
use crate::song::{
    ArrowSpawner
};
use crate::team_markers::{
    Marker,
    PlayerMarker
};

#[derive(Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct RecordingKeymap {
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
impl Default for RecordingKeymap {
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
    settings: Res<UserSettings>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let keymap = &settings.keybindings.recording_keymap;
    if !keys.just_pressed(keymap.pause) {
        return;
    }
    for mut spawner in spawner_q.iter_mut() {
        log::info!("toggling pause state on spawner");
        spawner.toggle_is_paused();
    }
}

const SCROLL_SPEED: f32 = 1.0;

fn handle_scroll_actions<T: Marker>(
    mut spawner_q: Query<&mut ArrowSpawner<T>>,
    settings: Res<UserSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let keymap = &settings.keybindings.recording_keymap;
    
    let dy = SCROLL_SPEED * time.delta().as_secs_f32();

    spawner_q
        .iter_mut()
        .for_each(|mut spawner| {
            
            if keys.pressed(keymap.forward) {
                spawner.change_scroll_pos(dy);
            }
            else if keys.pressed(keymap.backward) {
                spawner.change_scroll_pos(-dy);
            }


        });

}

pub struct RecordingControlsPlugin;
impl Plugin for RecordingControlsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, handle_pause_actions::<PlayerMarker>)
            .add_systems(Update, handle_scroll_actions::<PlayerMarker>)
        ;
    }
}

