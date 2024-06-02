use bevy::prelude::*;

use crate::judgement::{
    DroppedNoteEvent,
    CorrectHitEvent,
    grading::{
        SuccessGrade,
    }
};

pub fn play_sound_on_hit(
    mut commands: Commands,
    mut hit_events: EventReader<CorrectHitEvent>,
    asset_server: Res<AssetServer>,
) {

    // TODO: should this really trigger an entity for every event?
    for ev in hit_events.read() {

        let sound_name = match ev.grade() {
            SuccessGrade::Perfect => "sounds/correct-2.ogg",
            _ => "sounds/metronome-quartz.ogg",
        };

        commands.spawn(
            AudioBundle {
                source: asset_server.load(sound_name).into(),
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

pub struct SoundAlertsPlugin;
impl Plugin for SoundAlertsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                play_sound_on_hit,
                play_sound_on_dropped_note
            ))
        ;
    }
}
