use bevy::prelude::*;

use crate::lane::Lane;
use crate::WORLD_HEIGHT;

pub const ARROW_SIZE: Vec3 = Vec3::new(Lane::lane_width(), 20.0, 0.0);
pub const ARROW_SPEED: f32 = -250.0;
#[derive(Component,Debug,Copy,Clone)]
pub struct Arrow {
    lane: Lane,
}
impl Arrow {
    pub fn new() -> Arrow {
        Arrow {
            lane: Lane::random()
        }
    }
    pub fn lane(self) -> Lane {
        self.lane
    }
}

#[derive(Resource)]
struct SpawnTimer(Timer);
impl SpawnTimer {
    pub fn new(seconds: f32) -> SpawnTimer {
        let t = Timer::from_seconds(seconds, TimerMode::Repeating);
        SpawnTimer(t)
    }
}

fn spawn_arrows(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let arrow = Arrow::new();

    let pos = Vec3::new(arrow.lane.center_x(), WORLD_HEIGHT / 2.0, 0.0);

    commands
        .spawn((
            arrow,
            SpriteBundle {
                transform: Transform {
                    translation: pos,
                    scale: ARROW_SIZE,
                    ..default()
                },
                sprite: Sprite {
                    color: arrow.lane.colors().base,
                    ..default()
                },
                ..default()
            }
        ));
}

fn move_arrows(time: Res<Time>, mut query: Query<(&mut Transform, &Arrow)>) {
    for (mut transform, _arrow) in query.iter_mut() {
        transform.translation.y += time.delta_seconds() * ARROW_SPEED;
    }
}

pub struct ArrowsPlugin;
impl Plugin for ArrowsPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building Arrow plugin...");
        app
            .add_systems(Startup, setup)
            .add_systems(Update, spawn_arrows)
            .add_systems(Update, move_arrows);
    }
}

fn setup(mut commands: Commands, _materials: ResMut<Assets<ColorMaterial>>, _asset_server: Res<AssetServer>) {
    commands
        .insert_resource(SpawnTimer::new(0.6))
}
