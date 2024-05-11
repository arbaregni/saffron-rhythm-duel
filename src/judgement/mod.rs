mod metrics;
mod lane_box;
mod combo_meter;
mod target_sparkles;
mod animations;

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
fn arrow_drop_line_y() -> f32 {
    // once the arrow is no longer visible, it's too late for the player to click it
    world().bottom() - target_height() * 1.5
}

pub const KEYPRESS_TOLERANCE_SECS: f32 = 0.5; // in seconds

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
    /// The grade the judgment system gave
    pub grade: Grade,
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
    pub opt_hit: Option<(f32, Grade)>,
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

pub const PERFECT_CUTOFF_SECS: f32 = 0.05;
pub const FAIR_CUTOFF_SECS: f32 = 0.15;

#[derive(Debug, Copy, Clone)]
pub enum Grade {
    Perfect,
    Fair,
    Late,
    Early,
}
impl Grade {
    fn from(arrival_time: f32, hit_time: f32) -> Grade {
        let time_diff = (arrival_time - hit_time).abs();
        if time_diff < PERFECT_CUTOFF_SECS {
            Grade::Perfect
        } else if time_diff < FAIR_CUTOFF_SECS {
            Grade::Fair
        } else {
            if arrival_time < hit_time{
                Grade::Early
            } else {
                Grade::Late
            }
        }
    }
    pub fn is_perfect(self) -> bool {
        use Grade::*;
        match self {
            Perfect => true,
            Fair => false,
            Late => false,
            Early => false,
        }
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

/// Listens for Input actions where the user (correctly or incorrectly) attempts to complete a note
fn judge_lane_hits(
    time: Res<Time>,
    mut input_events: EventReader<InputActionEvent>,
    mut query: Query<(&mut Arrow, &mut Sprite)>,
    mut correct_arrow_events: EventWriter<CorrectHitEvent>,
    mut missfire_events: EventWriter<MissfireEvent>,
) {

    let now = time.elapsed().as_secs_f32();

    for input_action in input_events.read() {
        let InputActionEvent::LaneHit(event_lane) = input_action; // only input action type for now
        // 
        // Find the arrow with the closes arrival time 
        //

        use ordered_float::NotNan;
        let search_result = query
            .iter_mut()
            
            // only consider arrows that have not been hit yet
            .filter(|(arrow, _)| arrow.status().is_pending())
            
            // only consider arrows in the lane that was hit
            .filter(|(arrow, _)| arrow.lane() == *event_lane)

            // Get the absolute arrival time of each
            .map(|(arrow, sprite)| {
                let delta_time = arrow.arrival_time() - now;
                let time_diff = delta_time.abs();
                (arrow, sprite, time_diff)
            })

            // Discard the NaNs, everything else can be compared
            .filter_map(|(arrow, sprite, time_diff)| match NotNan::new(time_diff) {
                Ok(time_diff) => Some((arrow, sprite, time_diff)),
                Err(e) => {
                    log::error!("Found NaN while calculating the time to arrival of {arrow:?} - {e:?}");
                    None // discard this arrow
                }
            })
            // Find the minimum
            .min_by_key(|(_, _, diff)| *diff);

        // found a result, we need to send the appropriate event
        match search_result {
            None => {
                // there was a misclick here since the user 
                // pressed down when they should not have

                log::info!("no arrow found in lane - sending missfire event");
                missfire_events.send(MissfireEvent {
                    lane: *event_lane,
                    time_of_hit: now,
                    opt_hit: None,
                });

            }
            Some((mut arrow, mut sprite, _)) => {
                let delta_time = arrow.arrival_time() - now;

                log::info!("arrow found - time to arrival was {delta_time:?}");

                if delta_time.abs() >= KEYPRESS_TOLERANCE_SECS {
                    // too far away to consider this correct
                    log::info!("arrow found but it was too far away - sending missfire event");
                    missfire_events.send(MissfireEvent {
                        lane:  *event_lane,
                        time_of_hit: now,
                        opt_hit: Some((
                            delta_time,
                            Grade::from(arrow.arrival_time(), now)
                        )),
                    });

                } else {
                    arrow.mark_completed();

                    // since it's been completed, move the color closer to grey
                    sprite.color = arrow.lane().colors().greyed;

                    // send the correct hit event
                    log::info!("sending correct hit event");
                    correct_arrow_events.send(CorrectHitEvent {
                        lane: *event_lane,
                        time_of_hit: now,
                        delta_to_target: delta_time,
                        grade: Grade::from(arrow.arrival_time(), now),
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
        if y < arrow_drop_line_y() {

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
            .add_systems(Update, animations::darken_on_press)
            .add_systems(Update, animations::darken_over_time)
            .add_systems(Update, animations::jostle_on_dropped_note)
            .add_systems(Update, animations::animate_jostling)
            .add_systems(Update, animations::play_sound_on_hit)
            .add_systems(Update, animations::play_sound_on_dropped_note)
            
            // Add the plugins
            .add_plugins(lane_box::LaneBoxPlugin)
            .add_plugins(combo_meter::ComboMeterPlugin)
            .add_plugins(target_sparkles::TargetSparklesPlugin)
            .add_plugins(metrics::MetricsPlugin)
        ;
    }
}

