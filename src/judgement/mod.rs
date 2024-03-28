mod lane_box;
mod combo_meter;
mod target_sparkles;

use bevy::prelude::*;
use bevy::{
    reflect::TypePath,
    render::{
        render_resource::{
            AsBindGroup, ShaderRef
        },
    },
    sprite::{
        Material2d,
        Material2dPlugin,
        MaterialMesh2dBundle,
        Mesh2dHandle
    },
};

use crate::lane::{
    Lane,
    LaneMap
};
use crate::arrow::{ Arrow };
use crate::layout::BBox;

fn world() -> BBox {
    crate::world()
}
fn target_line_y() -> f32 {
    world().bottom() + 10.0
}

pub const KEYPRESS_TOLERANCE: f32 = 40.0;

#[derive(Component)]
struct LaneTarget {
    lane: Lane
}
impl LaneTarget {
    pub fn lane(&self) -> Lane {
        self.lane
    }
}

#[derive(Event)]
pub struct CorrectArrowEvent {
    pub lane: Lane,
    pub time: f32,
}
#[derive(Event)]
pub struct ArrowHitEvent {
    pub lane: Lane,
    pub time: f32,
    pub kind: ArrowHitKind,
    pub arrow: Arrow,
}
#[derive(Debug,Copy,Clone)]
pub enum ArrowHitKind {
    Enter, Exit
}

fn setup_targets(mut commands: Commands) {
    for &lane in Lane::all() {
        let lane_target = LaneTarget {
            lane
        };

        commands
            .spawn((
                lane_target,
                SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(lane.center_x(), target_line_y(), 0.0),
                        scale: Arrow::size(),
                        ..default()
                    },
                    sprite: Sprite {
                        color: lane.colors().light,
                        ..default()
                    },
                    ..default()
                }
            ));
    }

}

#[derive(Resource)]
struct LaneTargetStates {
    targets: LaneMap<TargetState>
}
impl LaneTargetStates {
    pub fn new() -> LaneTargetStates {
        Self {
            targets: LaneMap::new()
        }
    }
}
#[derive(Default)]
enum TargetState {
    #[default]
    Absent,
    Occupied(Arrow),
}

#[derive(Resource)]
struct SongMetrics {
    /// Total number of arrows that have passed the target line.
    total_arrows: u32,
    /// Number of arrows that the user has correctly intercepted in time.
    success_arrows: u32,
    /// Number of consecutive arrows the user has gotten correct.
    streak: u32
}
impl SongMetrics {
    fn new() -> SongMetrics {
        SongMetrics {
            total_arrows: 0,
            success_arrows: 0,
            streak: 0,
        }
    }
    fn record_success(&mut self) {
        self.total_arrows += 1;
        self.success_arrows += 1;
        self.streak += 1;
    }
    fn record_failure(&mut self) {
        self.total_arrows += 1;
        self.streak = 0;
    }
}

fn despawn_arrows(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &Transform, &Arrow)>,
    input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut song_metrics: ResMut<SongMetrics>,
    mut correct_arrow_events: EventWriter<CorrectArrowEvent>,
    mut arrow_hit_events: EventWriter<ArrowHitEvent>,
    mut lane_target_states: ResMut<LaneTargetStates>,
) {

    let now = time.elapsed().as_secs_f32();

    let mut play_sound = false;

    for (entity, transform, arrow) in query.iter() {
        let pos = transform.translation.y;

        if pos < target_line_y() + KEYPRESS_TOLERANCE {

            lane_target_states.targets[arrow.lane()] = TargetState::Occupied(arrow.clone());

            arrow_hit_events.send(ArrowHitEvent {
                lane: arrow.lane(),
                arrow: arrow.clone(),
                time: now, 
                kind: ArrowHitKind::Enter,
            });

            let key = arrow.lane().keycode();
            if input.just_pressed(key) { 
                log::info!("we have a hit ! in lane: {:?}", arrow.lane());

                song_metrics.record_success();
                correct_arrow_events.send(CorrectArrowEvent {
                    lane: arrow.lane(),
                    time: now
                });

                commands.entity(entity).despawn();
                play_sound = true;
                continue;
            }
        }

        if pos < target_line_y() - KEYPRESS_TOLERANCE {
            log::info!("arrow exitted a hit");
            lane_target_states.targets[arrow.lane()] = TargetState::Absent;
            arrow_hit_events.send(ArrowHitEvent {
                lane: arrow.lane(),
                arrow: arrow.clone(),
                time: time.elapsed().as_secs_f32(),
                kind: ArrowHitKind::Exit,
            });
        }

        if pos < world().bottom() {
            log::info!("failed");
            song_metrics.record_failure();
            play_sound = true;
            commands.entity(entity).despawn();
        }
    }

    // also play a fun little sound every time something happens
    
    if play_sound {

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
            .insert_resource(SongMetrics::new())
            .insert_resource(LaneTargetStates::new())
            .add_event::<CorrectArrowEvent>()
            .add_event::<ArrowHitEvent>()
            .add_systems(Startup, setup_targets)
            .add_systems(Update, despawn_arrows)
            .add_plugins(lane_box::LaneBoxPlugin)
            .add_plugins(combo_meter::ComboMeterPlugin)
            .add_plugins(target_sparkles::TargetSparklesPlugin)
        ;
    }
}

