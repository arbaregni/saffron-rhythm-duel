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

use crate::input::{
    LaneHit
};
use crate::team_markers::PlayerMarker;

use crate::layout::{
    SongPanel,
    Layer,
};
    

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
    created_at: f32
}

const LANE_BOX_MAX_TIME: f32 = 0.4;

fn create_lane_box_on_press(
    mut commands: Commands,
    mut input_events: EventReader<LaneHit>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LaneBoxMaterial>>,
    panel: Query<&SongPanel, With<PlayerMarker>>,
) {
    let now = time.elapsed().as_secs_f32();

    let panel = panel.single();

    let initial_alpha = 0.1;

    for ev in input_events.read() {
        let lane = ev.lane();

        let mut pos = panel.lane_bounds(lane).center();
        pos.z = Layer::SongEffects.z();

        let color = lane.colors().light.with_a(initial_alpha);

        let rect = panel.lane_bounds(lane).to_rectangle();
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


fn animate_lane_boxes(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &LaneBox)>,
) {
    let now = time.elapsed().as_secs_f32();
    for (entity, lane_box) in query.iter() {
        let t = (now - lane_box.created_at) / LANE_BOX_MAX_TIME;

        // kill anything that's past it's lifetime
        if t >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }

    }

    // manage the life time of each of the things
}

pub struct LaneBoxPlugin;
impl Plugin for LaneBoxPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building Targets plugin...");
        app
            .add_plugins(Material2dPlugin::<LaneBoxMaterial>::default())
            .add_systems(Update, create_lane_box_on_press)
            .add_systems(Update, animate_lane_boxes)
        ;
    }
}
