mod chart;

use anyhow::{Result, Context};
use bevy::prelude::*;
use serde_json;

use crate::lane::Lane;
use crate::layout::BBox;

use chart::Chart;

fn world() -> BBox {
    crate::world()
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Arrow {
    lane: Lane,
    status: ArrowStatus,
    /// When the arrow is created and first visibile to player
    creation_time: f32,
    /// When the arrow arrives at the judgement line, i.e. the ideal time for the player to hit it
    arrival_time: f32, 
}
impl Arrow {
    pub fn new(lane: Lane, creation_time: f32, arrival_time: f32) -> Arrow {
        Arrow {
            lane,
            status: ArrowStatus::BeforeTarget,
            creation_time,
            arrival_time,
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
#[derive(Debug, Clone)]
struct ArrowSpawner {
    mode: SpawningMode,
    song_start: f32,
    beat_timer: Timer,
    beat_count: u32,
}

#[derive(Debug, Clone)]
enum SpawningMode {
    Chart(Chart),
    Random,
}

impl ArrowSpawner {
    fn try_from(cli: &crate::Cli) -> Result<ArrowSpawner> {
        log::info!("Creating arrow spawner");
        let mode = match cli.chart.as_ref() {
            Some(path) => {
                let friendly_name = path.to_string_lossy();
                use std::fs;
                // parse the chart
                let text = fs::read_to_string(path)
                    .with_context(|| format!("Failed to read chart from path: {friendly_name}"))?;

                let chart: Chart = serde_json::from_str(text.as_str())
                    .with_context(|| format!("File at {friendly_name} could not be parsed as a chart"))?;

                log::info!("Parsed chart '{}' from {}", chart.chart_name(), friendly_name);

                SpawningMode::Chart(chart)
            }
            None => {
                log::info!("No chart specified, using random note generation");
                SpawningMode::Random
            }
        };
        let seconds_per_beat = match &mode {
            SpawningMode::Chart(chart) => chart.beat_duration_secs(),
            SpawningMode::Random => 0.3
        };
        let beat_timer = Timer::from_seconds(seconds_per_beat, TimerMode::Repeating);

        let spawner = ArrowSpawner {
            mode,
            beat_timer,
            song_start: 0.,
            beat_count: 0,
        };
        Ok(spawner)
    }
}


fn spawn_arrows(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<ArrowSpawner>,
) {
    let now = time.elapsed().as_secs_f32();

    if now < spawner.song_start {
        // not time to start the song yet
        return;
    }

    spawner.beat_timer.tick(time.delta());
    if !spawner.beat_timer.just_finished() {
        // not time for another arrow yet, just return
        return;
    }
    let beat = spawner.beat_count;
    spawner.beat_count += 1;

    let mut arrows = vec![]; // todo: store this in a buffer separately so we don't always allocate

    match &spawner.mode {
        SpawningMode::Chart(chart) => {
            let lead_time = chart.lead_time_secs();
            for note in chart.get(beat) {
                let lane = note.lane();
                let arrow = Arrow::new(lane, now, now + lead_time);
                arrows.push(arrow);
            }
        }
        SpawningMode::Random => {
            let lane = Lane::random();
            let lead_time = 1.5; // seconds
            let arrow = Arrow::new(lane, now, now + lead_time);
            arrows.push(arrow);
        }
    };

    for arrow in arrows.into_iter() {
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
}

fn move_arrows(time: Res<Time>, mut query: Query<(&mut Transform, &Arrow)>) {
    let now = time.elapsed().as_secs_f32();
    for (mut transform, arrow) in query.iter_mut() {
        let t = (now - arrow.creation_time) / (arrow.arrival_time - arrow.creation_time);
        transform.translation.y = world().bottom() * t + world().top() * (1.0 - t);
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

pub struct ArrowsPlugin {
    spawner: ArrowSpawner
}
impl ArrowsPlugin {
    pub fn new(cli: &crate::Cli) -> Result<ArrowsPlugin> {
        let spawner = ArrowSpawner::try_from(cli)?;
        Ok(ArrowsPlugin {
            spawner
        })
    }
}

impl Plugin for ArrowsPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building Arrow plugin...");
        app
            //.add_systems(Startup, setup)
            .insert_resource(self.spawner.clone()) // possibly slightly janky?
            .add_systems(Update, spawn_arrows)
            .add_systems(Update, move_arrows)
            .add_systems(Update, despawn_arrows)
        ;
    }
}

