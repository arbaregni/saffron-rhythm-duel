use bevy::prelude::*;

use crate::input::InputActionEvent;

use crate::team_markers::PlayerMarker;

use crate::layout::{
    SongPanel,
};

use super::{
    DroppedNoteEvent,
    CorrectHitEvent,
    LaneTarget,
    LaneLetter,
};

const DARKENED_DURATION: f32 = 0.25;

#[derive(Component)]
#[component(storage = "SparseSet")] // because we add and remove these often
pub struct DarkeningEffect {
    // timestamp of when this UI object was last triggered
    start_time: f32,
    start_color: Color,
    end_color: Color,
}

pub fn darken_on_press(
    mut commands: Commands,
    time: Res<Time>,
    mut input_events: EventReader<InputActionEvent>,
    lane_targets: Query<(Entity, &LaneTarget)>,
    lane_letters: Query<(Entity, &LaneLetter)>,
) {
    let now = time.elapsed().as_secs_f32();

    for input_action in input_events.read() {
        let InputActionEvent::LaneHit(event_lane) = input_action; // only input action type for now
                                                                  // we would ignore the others
        // get the lane targets
        lane_targets
            .iter()
            .filter(|(_, lane_target)| lane_target.lane == *event_lane)
            .for_each(|(entity, _)| {
                commands.entity(entity)
                      .insert(DarkeningEffect {
                          start_time: now,
                          start_color: event_lane.colors().heavy,
                          end_color: event_lane.colors().light,
                      });
            });

        // get the lane letters
        lane_letters
            .iter()
            .filter(|(_, lane_letter)| lane_letter.lane == *event_lane)
            .for_each(|(entity, _)| {
                commands.entity(entity)
                        .insert(DarkeningEffect {
                            start_time: now,
                            start_color: event_lane.colors().heavy.with_a(LaneLetter::alpha()),
                            end_color: event_lane.colors().light.with_a(LaneLetter::alpha()),
                        });
            });

    }
}

pub fn darken_over_time(
    mut commands: Commands,
    time: Res<Time>,
    mut lane_targets: Query<(Entity, &DarkeningEffect, &mut Sprite), With<LaneTarget>>,
    mut lane_letters: Query<(Entity, &DarkeningEffect, &mut Text), With<LaneLetter>>,
) {
    let now = time.elapsed().as_secs_f32();

    let mut set_color = |id: Entity, effect: &DarkeningEffect, color: &mut Color| {
        let t = (now - effect.start_time) / DARKENED_DURATION;
        *color = effect.start_color * (1.0 - t) + effect.end_color * t;

        if t >= 1.0 {
            // all done: remove the component
            commands.entity(id)
                .remove::<DarkeningEffect>();
        }
    };

    lane_targets
        .iter_mut()
        .for_each(|(id, effect, mut sprite)| {
            set_color(id, effect, &mut sprite.color);
        });

    lane_letters
        .iter_mut()
        .for_each(|(id, effect, mut text)| {
            set_color(id, effect, &mut text.sections[0].style.color);
        });
}

const JOSTLING_DURATION: f32 = 0.3;

#[derive(Component)]
#[component(storage = "SparseSet")] // because they are added and removed
pub struct JostlingEffect {
    start_time: f32,
    pos: Vec3,
    extents: Vec3,
}
pub fn jostle_on_dropped_note(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &LaneLetter, &Transform)>,
    mut dropped_notes: EventReader<DroppedNoteEvent>,
    panel: Query<&SongPanel, With<PlayerMarker>>,
) {
    let now = time.elapsed().as_secs_f32();

    let panel = panel.single();

    for dropped_note in dropped_notes.read() {
        let event_lane = dropped_note.arrow.lane();

        let x_extents = panel.lane_bounds(event_lane).width() / 6.0;

        query
            .iter()
            .filter(|(_, lane_letter, _)| lane_letter.lane == event_lane)
            .for_each(|(id, _, transform)| {
                log::info!("adding jostling effect");
                commands.entity(id)
                        .insert(JostlingEffect {
                            start_time: now,
                            pos: transform.translation, // TODO: make this be the center of the
                                                        // lane to allow stacked jostles
                            extents: Vec3::new(x_extents, 0.0, 0.0)
                        });
            });

    }
}
pub fn animate_jostling(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &JostlingEffect, &mut Transform)>,
) {
    let now = time.elapsed().as_secs_f32();

    fn impulse(t: f32) -> f32 {
        use std::f32::consts::PI;

        let freq = 3.0;
        let decay = 1.0 - t;
        (freq * PI * t).sin() * decay
    }

    query
        .iter_mut()
        .for_each(|(id, effect, mut transform)| {
            let t = (now - effect.start_time) / JOSTLING_DURATION;

            let offset = impulse(t) * effect.extents;
            transform.translation = effect.pos + offset;

            if t >= 1.0 {
                transform.translation = effect.pos;
                commands.entity(id)
                        .remove::<JostlingEffect>();
            }
        });

}

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


