use bevy::prelude::*;
use bevy::{
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};
use crate::WORLD_HEIGHT;

use crate::lane::Lane;
use crate::arrow::{
    ARROW_SIZE,
    Arrow
};

pub const KEYPRESS_TOLERANCE: f32 = 40.0;
pub const TARGET_LINE_Y: f32 = -WORLD_HEIGHT / 2.0 + 10.0;

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
                        translation: Vec3::new(lane.center_x(), TARGET_LINE_Y, 0.0),
                        scale: ARROW_SIZE,
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

fn arrow_hit_listener(
    _commands: Commands,
    mut query: Query<(&mut Sprite, &LaneTarget)>,
    mut arrow_hits: EventReader<ArrowHitEvent>
) {
    // is each lane being hit by an arrow right now?
    let mut l1 = false;
    let mut l2 = false;
    let mut r1 = false;
    let mut r2 = false;
    for hit in arrow_hits.read() {
        let is_hit = match hit.kind {
            ArrowHitKind::Enter => true,
            ArrowHitKind::Exit => false,
        };
        match hit.lane {
            Lane::L1 => { l1 = is_hit; }
            Lane::L2 => { l2 = is_hit; }
            Lane::R1 => { r1 = is_hit; }
            Lane::R2 => { r2 = is_hit; }
        };
    }
    // update the lane target appropriately
    for (mut sprite, lane_target) in query.iter_mut() {
        let is_hit = match lane_target.lane() {
            Lane::L1 => l1,
            Lane::L2 => l2,
            Lane::R1 => r1,
            Lane::R2 => r2,
        };
        let color = match is_hit {
            true => lane_target.lane().colors().heavy,
            false => lane_target.lane().colors().light,
        };
        sprite.color = color;
    }
}


fn despawn_arrows(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &Transform, &Arrow)>,
    input: Res<ButtonInput<KeyCode>>,
    mut correct_arrow_events: EventWriter<CorrectArrowEvent>,
    mut arrow_hit_events: EventWriter<ArrowHitEvent>,
) {
    let bottom = -WORLD_HEIGHT / 2.0;

    for (entity, transform, arrow) in query.iter() {
        let pos = transform.translation.y;

        if pos < TARGET_LINE_Y + KEYPRESS_TOLERANCE {

            arrow_hit_events.send(ArrowHitEvent {
                lane: arrow.lane(),
                arrow: arrow.clone(),
                time: time.elapsed().as_secs_f32(),
                kind: ArrowHitKind::Enter,
            });

            let key = arrow.lane().keycode();
            if input.just_pressed(key) { 
                log::info!("we have a hit ! in lane: {:?}", arrow.lane());

                correct_arrow_events.send(CorrectArrowEvent {
                    lane: arrow.lane(),
                    time: time.elapsed().as_secs_f32(),
                });

                commands.entity(entity).despawn();
                continue;
            }
        }

        if pos < TARGET_LINE_Y - KEYPRESS_TOLERANCE {
            log::info!("arrow exitted a hit");
            arrow_hit_events.send(ArrowHitEvent {
                lane: arrow.lane(),
                arrow: arrow.clone(),
                time: time.elapsed().as_secs_f32(),
                kind: ArrowHitKind::Exit,
            });
        }
        if pos < bottom {
            commands.entity(entity).despawn();
            log::info!("failed");
        }
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
    mut query: Query<(Entity, &mut Sprite, &LaneBox)>,
) {
    let now = time.elapsed().as_secs_f32();

    for &lane in Lane::all() {
        let key = lane.keycode();
        if input.just_pressed(key) {

            let pos = Vec3::new(lane.center_x(), 0.0, 0.0);
            let scale = Vec3::new(Lane::lane_width(), WORLD_HEIGHT, 0.0);

            let color = lane.colors().light.with_a(0.5);

            // spawn the thing
            commands.spawn((
                    LaneBox {
                        lane,
                        created_at: now,
                    },
                    SpriteBundle {
                        transform: Transform {
                            translation: pos,
                            scale,
                            ..default()
                        },
                        sprite: Sprite {
                            color,
                            ..default()
                        },
                        ..default()
                    }));

        }
    }

    for (entity, mut sprite, lane_box) in query.iter_mut() {
        let t = (now - lane_box.created_at) / LANE_BOX_MAX_TIME;

        // kill anything that's past it's lifetime
        if t >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // transparency fade

        // map t: [0, 1] to [0.5, 0.0]
        let alpha = -t / 2.0 + 1.0;
        sprite.color.set_a(alpha);

    }


    // manage the life time of each of the things
}

#[derive(Component)]
struct TargetSparkle {
    created_at: f32,
}
const TARGET_SPARKLE_MAX_TIME: f32 = 0.3;

fn target_sparkle_on_correct_hit(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<SparkleMaterial>>,
    mut color_mat: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut correct_events: EventReader<CorrectArrowEvent>,
    mut query: Query<(Entity, &mut Transform, &TargetSparkle)>,
) {
    let now = time.elapsed().as_secs_f32();

    let initial_radius = Lane::lane_width() / 2.0;

    for event in correct_events.read() {

        log::info!("correct event, spawning a little funny guy");

        let sparkle = TargetSparkle {
            created_at: now,
        };

        let color = event.lane.colors().base;
        let material = color_mat.add(color);


        let mesh = bevy::sprite::Mesh2dHandle(meshes.add(Circle {
            radius: 1.0
        }));
        let scale = Vec3::splat(initial_radius);

        let position = Vec3::new(event.lane.center_x(), TARGET_LINE_Y, 0.0);


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

    for (entity, mut transform, target_sparkle) in query.iter_mut() {

        let t = (now - target_sparkle.created_at) / TARGET_SPARKLE_MAX_TIME;

        if t >= 1.0 {
            commands.entity(entity).despawn();
        }

        // expand the radius over time
        let radius = Lane::lane_width() / 2.0 * (t + 1.0);



    }
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
        "shaders/custom_material_2d.wgsl".into()
    }
}

pub struct TargetsPlugin;
impl Plugin for TargetsPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building Targets plugin...");
        app
            .add_plugins(Material2dPlugin::<SparkleMaterial>::default())
            .add_systems(Startup, setup_targets)
            .add_event::<CorrectArrowEvent>()
            .add_event::<ArrowHitEvent>()
            .add_systems(Update, create_lane_box_on_press)
            .add_systems(Update, target_sparkle_on_correct_hit)
            .add_systems(Update, despawn_arrows)
            .add_systems(Update, arrow_hit_listener);
    }
}

