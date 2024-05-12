use bevy::prelude::*;

pub struct RecordingPlugin;
impl Plugin for RecordingPlugin {
    fn build(&self, _app: &mut App) {
        log::info!("building Recording plugin...");
    }
}

