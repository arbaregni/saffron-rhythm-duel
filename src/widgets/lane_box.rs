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
    LaneHit,
};
use crate::team_markers::{
    PlayerMarker,
    EnemyMarker,
    Marker,
};
use crate::lane::{
    Lane
};
use crate::layout::{
    SongPanel,
    Layer,
};
use crate::remote::{
    RemoteLaneHit
};
    

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct LaneBoxMaterial {
    /// The base color
    #[uniform(0)]
    color: Color,
    /// The game time when we started animating
    #[uniform(1)]
    start_time: f32, 
    /// How long we plan to go for
    #[uniform(2)]
    duration: f32,
}

impl Material2d for LaneBoxMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lane_box_material_2d.wgsl".into()
    }
}

const LANE_BOX_MAX_TIME: f32 = 0.3;
const LANE_BOX_INITIAL_ALPHA: f32 = 0.4;

/// Short lived component that is created to indicate a key press
#[derive(Component)]
#[component(storage = "SparseSet")]
struct LaneBox {
    created_at: f32,
    duration: f32,
}
pub struct LaneBoxCreationArgs<'a, 'w, 's, T> {
    commands: &'a mut Commands<'w, 's>,
    time: &'a Time,
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<LaneBoxMaterial>,
    lane: Lane,
    panel: &'a SongPanel,
    marker: T,
}
impl LaneBox {
    pub fn create<'a, 'w, 's, T: Component + Marker>(args: LaneBoxCreationArgs<'a, 'w, 's, T>) {
        let LaneBoxCreationArgs {
            commands, time, meshes, materials, lane, panel, marker
        } = args;

        let now = time.elapsed().as_secs_f32();

        let mut pos = panel.lane_bounds(lane).center();
        pos.z = Layer::SongEffects.z();

        let color = lane.colors().light.with_a(LANE_BOX_INITIAL_ALPHA);

        let rect = panel.lane_bounds(lane).to_rectangle();
        let mesh = Mesh2dHandle(meshes.add(rect));

        let material = materials.add(LaneBoxMaterial {
            color,
            start_time: now,
            duration: LANE_BOX_MAX_TIME,
        });

        let lane_box = LaneBox {
            created_at: now,
            duration: LANE_BOX_MAX_TIME,
        };

        let mesh_bundle = MaterialMesh2dBundle {
            mesh,
            transform: Transform {
                translation: pos,
                ..default()
            },
            material,
            ..default()
        };

        commands.spawn((
            lane_box,
            mesh_bundle,
            marker,
        ));

    }
}

fn create_lane_box_on_press(
    // needed to spawn lane box
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LaneBoxMaterial>>,

    // needed to spawn inside a panel
    player_panel: Query<&SongPanel, With<PlayerMarker>>,
    enemy_panel: Query<&SongPanel, With<EnemyMarker>>,

    // listen for the triggers
    mut input_events: EventReader<LaneHit>,
    mut remote_input_events: EventReader<RemoteLaneHit>,
) {
    let player_panel = player_panel.single();
    let enemy_panel = enemy_panel.single();

    for ev in input_events.read() {
        let lane = ev.lane();

        LaneBox::create(LaneBoxCreationArgs {
            commands: &mut commands,
            time: time.as_ref(),
            meshes: meshes.as_mut(),
            materials: materials.as_mut(),
            lane,
            panel: player_panel,
            marker: PlayerMarker
        });

    }

    for ev in remote_input_events.read() {
        let lane = ev.lane();
        LaneBox::create(LaneBoxCreationArgs {
            commands: &mut commands,
            time: time.as_ref(),
            meshes: meshes.as_mut(),
            materials: materials.as_mut(),
            lane,
            panel: enemy_panel,
            marker: EnemyMarker
        });

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
