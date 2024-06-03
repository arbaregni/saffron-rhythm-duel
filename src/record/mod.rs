use bevy::prelude::*;

pub mod controls;

pub struct RecordingPlugin;
impl Plugin for RecordingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(controls::RecordingControlsPlugin)
        ;
    }
}

