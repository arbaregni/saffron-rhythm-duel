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

use crate::team_markers::{
    PlayerMarker
};
use crate::layout::{
    Layer,
    SongPanel
};

use super::{
    CorrectHitEvent,
};


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
const TARGET_SPARKLE_INITIAL_RADIUS: f32 = 0.3;
const TARGET_SPARKLE_FINAL_RADIUS: f32 = 100.0;

fn create_target_sparkle_on_correct_hit(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SparkleMaterial>>,
    time: Res<Time>,
    panel: Query<&SongPanel, With<PlayerMarker>>,
    mut correct_events: EventReader<CorrectHitEvent>,
) {
    let now = time.elapsed().as_secs_f32();
    let panel = panel.single();

    let initial_radius = 10.0;

    for event in correct_events.read() {

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

        let position = Vec3::new(
            panel.lane_bounds(event.lane).center().x,
            panel.target_line_y(),
            Layer::AboveTargets.z()
        );

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
}

fn update_target_sparkles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &TargetSparkle)>,
) {
    let now = time.elapsed().as_secs_f32();

    for (entity, mut transform, target_sparkle) in query.iter_mut() {

        let t = (now - target_sparkle.created_at) / TARGET_SPARKLE_MAX_TIME;

        if t >= 1.0 {
            commands.entity(entity).despawn();
        }

        // expand the radius over time
        // [0,1] -> [initial_radius, final_radius]
        let radius = t * (TARGET_SPARKLE_FINAL_RADIUS - TARGET_SPARKLE_INITIAL_RADIUS) + TARGET_SPARKLE_INITIAL_RADIUS;

        transform.scale = Vec3::splat(radius);

    }
}


pub struct TargetSparklesPlugin;
impl Plugin for TargetSparklesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(Material2dPlugin::<SparkleMaterial>::default())
            .add_systems(Update, create_target_sparkle_on_correct_hit)
            .add_systems(Update, update_target_sparkles)
        ;

    }
}
