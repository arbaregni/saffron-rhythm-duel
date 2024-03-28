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

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct LaneBoxMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    created_at: f32,
    #[uniform(2)]
    life_length: f32,
}

impl Material2d for LaneBoxMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lane_box_material_2d.wgsl".into()
    }
}

#[derive(Component)]
struct LaneBox {
    lane: Lane,
    created_at: f32
}

const LANE_BOX_MAX_TIME: f32 = 0.4;

fn create_lane_box_on_press(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LaneBoxMaterial>>,
    mut query: Query<(Entity, &LaneBox)>,
) {
    let now = time.elapsed().as_secs_f32();

    let initial_alpha = 0.1;

    for &lane in Lane::all() {
        let key = lane.keycode();
        if input.just_pressed(key) {

            let pos = Vec3::new(lane.center_x(), 0.0, 0.0);

            let color = lane.colors().light.with_a(initial_alpha);

            let rect = Rectangle::new(Lane::lane_width(), world().height());
            let mesh = Mesh2dHandle(meshes.add(rect));

            let created_at = now;

            let material = materials.add(LaneBoxMaterial {
                color,
                created_at,
                life_length: LANE_BOX_MAX_TIME,
            });

            log::info!("key press detected, creating lane box...");
            commands.spawn((
                LaneBox {
                    lane,
                    created_at: now,
                },
                MaterialMesh2dBundle {
                    mesh,
                    transform: Transform {
                        translation: pos,
                        ..default()
                    },
                    material,
                    ..default()
                }));

          }
    }

    for (entity, lane_box) in query.iter_mut() {
        let t = (now - lane_box.created_at) / LANE_BOX_MAX_TIME;

        // kill anything that's past it's lifetime
        if t >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }

    }


    // manage the life time of each of the things
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SparkleMaterial {
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
}
impl Material2d for SparkleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/target_sparkle_material_2d.wgsl".into()
    }
}

#[derive(Component)]
struct TargetSparkle {
    created_at: f32,
}
const TARGET_SPARKLE_MAX_TIME: f32 = 0.3;

fn target_sparkle_on_correct_hit(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SparkleMaterial>>,
    time: Res<Time>,
    mut correct_events: EventReader<CorrectArrowEvent>,
    mut query: Query<(Entity, &mut Transform, &TargetSparkle)>,
) {
    let now = time.elapsed().as_secs_f32();

    let initial_radius = 10.0;

    for event in correct_events.read() {

        log::info!("correct event, spawning a little funny guy");

        let sparkle = TargetSparkle {
            created_at: now,
        };

        let color = event.lane.colors().base;
        let material = materials.add(SparkleMaterial {
            color,
            color_texture: None // Some(asset_server.load("icon.png"))
        });


        let mesh = Mesh2dHandle(meshes.add(Circle {
            radius: 1.0
        }));
        let scale = Vec3::splat(initial_radius);

        let position = Vec3::new(event.lane.center_x(), target_line_y(), 0.0);


        commands.spawn((sparkle,
            MaterialMesh2dBundle {
                mesh,
                transform: Transform {
                    translation: position,
                    scale,
                    ..default()
                },
                material,
                ..default()
            }));

    }

    let final_radius = 100.0;

    for (entity, mut transform, target_sparkle) in query.iter_mut() {

        let t = (now - target_sparkle.created_at) / TARGET_SPARKLE_MAX_TIME;

        if t >= 1.0 {
            commands.entity(entity).despawn();
        }

        // expand the radius over time
        // [0,1] -> [initial_radius, final_radius]
        let radius = t * (final_radius - initial_radius) + initial_radius;

        transform.scale = Vec3::splat(radius);

    }
}


#[derive(Component)]
pub struct FeedbackText {
    last_updated: f32,
}

const FEEDBACK_TEXT_MAX_SHOW_TIME: f32 = 0.7; // seconds

fn setup_feedback_text(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font_size = 100.0;
    let color = Color::rgb(0.9, 0.9, 0.9);

    let style = TextStyle { font, font_size, color };
    let text = Text {
        sections: vec![
            TextSection {
                value: "".to_string(),
                style,
            }
        ],
        ..default()
    };

    commands.spawn(
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    FeedbackText {
                        last_updated: 0f32,
                    },
                    TextBundle {
                        text,
                        ..default()
                    }
                ));
        });

}

fn update_feedback_text(
    time: Res<Time>,
    song_metrics: Res<SongMetrics>,
    mut correct_events: EventReader<CorrectArrowEvent>,
    mut query: Query<(&mut Text, &mut FeedbackText)>,
) {
    let now = time.elapsed().as_secs_f32();

    let Some((mut text, mut feedback)) = query.iter_mut().nth(0) else {
        log::warn!("no feedback item found");
        return;
    };

    for _event in correct_events.read() {


        // TODO: advanced feedback here
        //
        feedback.last_updated = now;
        text.sections[0].value.clear();
        let text_value = match song_metrics.streak {
            0 => "",
            1..=2 => "Good",
            3 => "Nice",
            4 => "Great!",
            5 => "Amazing!",
            6 => "Wonderful!",
            7 => "Fantastic!!",
            8..=19 => "Outstanding!!!",

            20 => "SUPER-!",
            21 => "SUPER-POWER-!",
            22 => "SUPER-POWER-NINJA-!",
            23 => "SUPER-POWER-NINJA-TURBO-!",
            24 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-!",
            25 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-!",
            26 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-!",
            27 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-!",
            28 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-!",
            29 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-!",
            30 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-!",
            31 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-PREFIX-!",
            32 => "SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-PREFIX-COMBO!",

            n => {
                let n = n - 32;
                text.sections[0].value.push_str(&format!("SUPER-POWER-NINJA-TURBO-NEO-HYPER-MEGA-MULTI-ALPHA-META-EXTRA-UBER-PREFIX-COMBO! x{n}"));
                continue;
            }
        };

        text.sections[0].value.clear();
        text.sections[0].value.push_str(text_value);

    }

    let t = (now - feedback.last_updated) / FEEDBACK_TEXT_MAX_SHOW_TIME;
    if t >= 1.0 {
        text.sections[0].value.clear();
        return;
    }
    let alpha = 1.0 - t;
    text.sections[0].style.color.set_a(alpha);
}


pub struct TargetsPlugin;
impl Plugin for TargetsPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building Targets plugin...");
        app
            .add_plugins(Material2dPlugin::<SparkleMaterial>::default())
            .add_plugins(Material2dPlugin::<LaneBoxMaterial>::default())
            .insert_resource(SongMetrics::new())
            .insert_resource(LaneTargetStates::new())
            .add_event::<CorrectArrowEvent>()
            .add_event::<ArrowHitEvent>()
            .add_systems(Startup, setup_targets)
            .add_systems(Startup, setup_feedback_text)
            .add_systems(Update, create_lane_box_on_press)
            .add_systems(Update, target_sparkle_on_correct_hit)
            .add_systems(Update, despawn_arrows)
            .add_systems(Update, update_feedback_text);
    }
}

