use bevy::prelude::*;

use crate::lane::Lane;

use crate::layout::BBox;

fn world() -> BBox {
    crate::world()
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Arrow {
    lane: Lane,
    status: ArrowStatus
}
impl Arrow {
    pub fn new() -> Arrow {
        Arrow {
            lane: Lane::random(),
            status: ArrowStatus::BeforeTarget,
        }
    }
    pub fn lane(self) -> Lane {
        self.lane
    }
    pub fn size() -> Vec3 {
        Vec3::new(Lane::lane_width(), 20.0, 0.0)
    }
    pub fn status(self) -> ArrowStatus {
        self.status
    }
    pub fn set_status(&mut self, status: ArrowStatus) {
        self.status = status;
    }
    pub fn speed(self) -> f32 {
        -400.0
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ArrowStatus {
    /// Before it should be hit
    BeforeTarget,
    /// In the middle of the target
    InTarget,
    /// After it passes through the target
    AfterTarget,
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

    let pos = Vec3::new(arrow.lane.center_x(), world().top(), 0.0);

    commands
        .spawn((
            arrow,
            SpriteBundle {
                transform: Transform {
                    translation: pos,
                    scale: Arrow::size(),
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
    for (mut transform, arrow) in query.iter_mut() {
        transform.translation.y += time.delta_seconds() * arrow.speed();
    }
}

fn despawn_arrows(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Arrow)>
) {
    for (entity, transform, _arrow) in query.iter() {
        let y = transform.translation.y;
        if y < world().bottom() - 100.0 {
            log::info!("despawning arrow");
            commands.entity(entity).despawn();
        }
    }

}

pub struct ArrowsPlugin;
impl Plugin for ArrowsPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building Arrow plugin...");
        app
            .add_systems(Startup, setup)
            .add_systems(Update, spawn_arrows)
            .add_systems(Update, move_arrows)
            .add_systems(Update, despawn_arrows)
        ;
    }
}

fn setup(mut commands: Commands, _materials: ResMut<Assets<ColorMaterial>>, _asset_server: Res<AssetServer>) {
    commands
        .insert_resource(SpawnTimer::new(0.5))
}
