use bevy::prelude::*;

use super::{
    DroppedNoteEvent,
    CorrectHitEvent,
};

pub fn play_sound_on_hit(
    mut commands: Commands,
    mut hit_events: EventReader<CorrectHitEvent>,
    asset_server: Res<AssetServer>,
) {

    // TODO: should this really trigger an entity for every event?
    for _ in hit_events.read() {
        commands.spawn(
            AudioBundle {
                source: asset_server.load("sounds/metronome-quartz.ogg").into(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    ..default()
                }
            }
        );
    }
}

pub fn play_sound_on_dropped_note(
    mut commands: Commands,
    mut drop_events: EventReader<DroppedNoteEvent>,
    asset_server: Res<AssetServer>,
) {
    if drop_events.is_empty() {
        return;
    }
    drop_events.clear();
    commands.spawn(
            AudioBundle {
                source: asset_server.load("sounds/blocky-land-wood-3.ogg").into(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    ..default()
                }
            }
        );
}


