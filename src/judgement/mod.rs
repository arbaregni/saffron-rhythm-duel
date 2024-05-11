mod metrics;
mod lane_box;
mod combo_meter;
mod target_sparkles;

use bevy::prelude::*;

use crate::lane::{
    Lane,
};
use crate::arrow::{
    Arrow,
    ArrowStatus,
};
use crate::layout::BBox;
use crate::input::InputActionEvent;

pub use metrics::{
    SongMetrics
};

fn world() -> BBox {
    crate::world()
}
fn target_line_y() -> f32 {
    world().bottom() + 10.0
}

pub const KEYPRESS_TOLERANCE: f32 = 80.0;

#[derive(Component)]
struct LaneTarget {
    lane: Lane
}
impl LaneTarget {
    pub fn lane(&self) -> Lane {
        self.lane
    }
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
fn setup_targets(mut commands: Commands) {
    for &lane in Lane::all() {
        let lane_target = LaneTarget {
            lane
        };

        let pos = Vec3::new(lane.center_x(), target_line_y(), 0.0);
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
    }

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
            .add_systems(Startup, setup_targets)
            .add_systems(Update, judge_lane_hits)
            .add_systems(Update, despawn_arrows)
            .add_systems(Update, play_sound_on_hit)
            .add_plugins(lane_box::LaneBoxPlugin)
            .add_plugins(combo_meter::ComboMeterPlugin)
            .add_plugins(target_sparkles::TargetSparklesPlugin)
            .add_plugins(metrics::MetricsPlugin)
        ;
    }
}

