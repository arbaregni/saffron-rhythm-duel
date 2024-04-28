use bevy::prelude::*;

pub struct RecordingPlugin;
impl Plugin for RecordingPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building Recording plugin...");
    }
}

