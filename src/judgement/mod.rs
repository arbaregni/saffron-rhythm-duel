mod metrics;
mod lane_box;
mod combo_meter;
mod target_sparkles;

use bevy::prelude::*;

use crate::{
    Config
};
use crate::lane::{
    Lane,
};
use crate::arrow::{
    Arrow,
};
use crate::layout::{
    BBox,
    Layer
};
use crate::input::InputActionEvent;

pub use metrics::{
    SongMetrics
};

fn world() -> BBox {
    crate::world()
}
fn target_height() -> f32 {
    20.0 // the arrow height
}
fn target_line_y() -> f32 {
    world().bottom() + 0.5 * target_height()
}
fn lane_text_y() -> f32 {
    target_line_y() + 20.0 // 
}


pub const KEYPRESS_TOLERANCE: f32 = 80.0;

#[derive(Component)]
struct LaneTarget {
    lane: Lane,
}
impl LaneTarget {
    pub fn lane(&self) -> Lane {
        self.lane
    }
}

const LANE_LETTER_ALPHA: f32 = 0.3;
#[derive(Component)]
struct LaneLetter {
    lane: Lane
}

/// Represents when the user hits the lane and there is a nearby note
#[derive(Event)]
pub struct CorrectHitEvent {
    /// Which lane it happened in
    pub lane: Lane,
    /// When the hit occured (game time)
    pub time_of_hit: f32,
    /// The signed distance from the target line to the nearest note
    pub delta_to_target: f32,
}

#[derive(Event)]
/// Event representing when the user attempts to complete a note, but are too early or late to be
/// considered 'correct'
pub struct MissfireEvent {
    /// Which lane it happened in
    pub lane: Lane,
    /// When the hit occured (game time)
    pub time_of_hit: f32,
    /// The signed distance from the target line to the nearest note,
    /// if we can identify one.
    pub opt_delta_to_target: Option<f32>,
}

#[derive(Event)]
/// Event representing when an arrow never gets hit by the player
pub struct DroppedNoteEvent {
    arrow: Arrow,
}
impl DroppedNoteEvent {
    /// The arrow that was never hit.
    pub fn arrow(&self) -> &Arrow {
        &self.arrow
    }
}

// Draws the targets on the target line
fn setup_targets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<Config>,
) {
    for &lane in Lane::all() {
        let lane_target = LaneTarget {
            lane,
        };

        let z = Layer::Targets.z();
        let pos = Vec3::new(lane.center_x(), target_line_y(), z);
        let transform = Transform {
            translation: pos,
            scale: Arrow::size(),
            ..default()
        };

        let color = lane.colors().light;
        let sprite = Sprite {
            color,
            ..default()
        };

        commands
            .spawn((
                lane_target,
                SpriteBundle {
                    transform,
                    sprite,
                    ..default()
                }
            ));

        // spawn a letter above the lane

        {
            let text_content = config.keybindings.key_name(lane).to_uppercase();

            let font = asset_server.load(crate::BASE_FONT_NAME);
            let font_size = 50.0;
            let color = lane.colors().light.with_a(LANE_LETTER_ALPHA);
            

            let z = Layer::AboveTargets.z();
            let transform = Transform {
                translation: Vec3::new(lane.center_x(), target_line_y() + 50.0, z),
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

            commands.spawn((
                LaneLetter {
                    lane
                },
                Text2dBundle {
                    text,
                    transform,
                    ..default()
                }
            ));
             
        }

    }

}

const DARKENED_DURATION: f32 = 0.25;

#[derive(Component)]
#[component(storage = "SparseSet")] // because we add and remove these often
struct DarkeningEffect {
    // timestamp of when this UI object was last triggered
    start_time: f32,
    start_color: Color,
    end_color: Color,
}

fn darken_on_press(
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
                            start_color: event_lane.colors().heavy.with_a(LANE_LETTER_ALPHA),
                            end_color: event_lane.colors().light.with_a(LANE_LETTER_ALPHA),
                        });
            });

    }
}

fn darken_over_time(
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
struct JostlingEffect {
    start_time: f32,
    pos: Vec3,
    extents: Vec3,
}
fn jostle_on_dropped_note(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &LaneLetter, &Transform)>,
    mut dropped_notes: EventReader<DroppedNoteEvent>,
) {
    let now = time.elapsed().as_secs_f32();

    for dropped_note in dropped_notes.read() {
        let event_lane = dropped_note.arrow.lane();

        let x_extents = Lane::lane_width() / 6.0;

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
fn animate_jostling(
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

/// Listens for Input actions where the user (correctly or incorrectly) attempts to complete a note
fn judge_lane_hits(
    time: Res<Time>,
    mut input_events: EventReader<InputActionEvent>,
    mut query: Query<(&Transform, &mut Arrow)>,
    mut correct_arrow_events: EventWriter<CorrectHitEvent>,
    mut missfire_events: EventWriter<MissfireEvent>,
) {

    let now = time.elapsed().as_secs_f32();

    for input_action in input_events.read() {
        let InputActionEvent::LaneHit(event_lane) = input_action; // only input action type for now
        // 
        // Find the closest arrow to the target line
        //

        let mut search_result = None;
        let mut smallest_dist = f32::INFINITY;

        for (transform, arrow) in query.iter_mut() {
            let pos = transform.translation.y;

            if !arrow.status().is_pending() {
                // only consider arrows that have not been hit yet
                continue;
            }

            if arrow.lane() != *event_lane {
                // do not consider this arrow, it is not in the right lane
                continue;
            }

            let dist = (target_line_y() - pos).abs();

            // progressively choose the closest arrow
            if dist < smallest_dist {
                search_result = Some((transform, arrow));
                smallest_dist = dist;
            }
        }


        // found a result, we need to send the appropriate event
        match search_result {
            None => {
                // there was a misclick here since the user 
                // pressed down when they should not have

                log::info!("sending missfire event");
                missfire_events.send(MissfireEvent {
                    lane: *event_lane,
                    time_of_hit: now,
                    opt_delta_to_target: None,
                });

            }
            Some((transform, mut arrow)) => {
                arrow.mark_completed();

                let delta_to_target = target_line_y() - transform.translation.y;

                if delta_to_target.abs() >= KEYPRESS_TOLERANCE {

                    // too far away to consider this correct
                    log::info!("sending missfire event");
                    missfire_events.send(MissfireEvent {
                        lane:  *event_lane,
                        time_of_hit: now,
                        opt_delta_to_target: Some(delta_to_target),
                    });

                } else {

                    // send the correct hit event
                    log::info!("sending correct hit event");
                    correct_arrow_events.send(CorrectHitEvent {
                        lane: *event_lane,
                        time_of_hit: now,
                        delta_to_target: (target_line_y() - transform.translation.y),
                    });
                }
            }
        }

        // all done


    }

}

fn despawn_arrows(
    mut commands: Commands,
    mut events: EventWriter<DroppedNoteEvent>,
    query: Query<(Entity, &Transform, &Arrow)>
) {
    for (entity, transform, arrow) in query.iter() {
        let y = transform.translation.y;
        if y < world().bottom() - KEYPRESS_TOLERANCE {

            // it's low enough to despawn
            commands.entity(entity).despawn();

            if arrow.status().is_pending() {
                log::info!("emitting DroppedNoteEvent");
                events.send(DroppedNoteEvent {
                    arrow: *arrow
                });
            }

        }
    }

}


fn play_sound_on_hit(
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


pub struct TargetsPlugin;
impl Plugin for TargetsPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building Targets plugin...");
        app
            .add_event::<CorrectHitEvent>()
            .add_event::<MissfireEvent>()
            .add_event::<DroppedNoteEvent>()
            
            // Add the systems
            .add_systems(Startup, setup_targets)
            .add_systems(Update, judge_lane_hits)
            .add_systems(Update, despawn_arrows)
            .add_systems(Update, play_sound_on_hit)
            .add_systems(Update, darken_on_press)
            .add_systems(Update, darken_over_time)
            .add_systems(Update, jostle_on_dropped_note)
            .add_systems(Update, animate_jostling)
            
            // Add the plugins
            .add_plugins(lane_box::LaneBoxPlugin)
            .add_plugins(combo_meter::ComboMeterPlugin)
            .add_plugins(target_sparkles::TargetSparklesPlugin)
            .add_plugins(metrics::MetricsPlugin)
        ;
    }
}

