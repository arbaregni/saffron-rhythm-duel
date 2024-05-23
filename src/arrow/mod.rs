mod chart;
mod spawner;
mod timer;
mod chart_loader;

use bevy::prelude::*;

use crate::team_markers::{
    PlayerMarker,
    EnemyMarker,
    Team,
    Marker,
    EntityCommandsExt,
};
use crate::layout::{
    BBox,
    Layer,
    SongPanel,
};


pub use spawner::{
    ArrowSpawner,
    ArrowBuf,
    Arrow,
};
pub use timer::{
    BeatTimer,
    FinishBehavior,
};
pub use chart_loader::{
    LoadChartEvent
};

fn world() -> BBox {
    crate::world()
}

impl <'a, 'w, 's, T: Marker> crate::layout::SongPanelSetupContext<'a, 'w, 's, T> {
    pub fn setup_arrow_spawner(self) -> Self {
        log::info!("Creating arrow spawner");

        let spawner = ArrowSpawner::create(self.marker.as_team());

        let seconds_per_beat = spawner.chart()
            .map(|chart| chart.beat_duration_secs())
            .unwrap_or(self.cli.fallback_beat_duration);

        let on_finish = self.cli.on_finish.clone();

        let duration = std::time::Duration::from_secs_f32(seconds_per_beat);
        let beat_timer = Timer::new(duration, TimerMode::Repeating);

        self.commands.spawn((
            spawner,
            BeatTimer {
                song_start: 3.0, // seconds
                beat_count: 0,
                beat_timer,
                on_finish,
            },
            ArrowBuf::new(),
            self.marker.clone(),
        ));


        self
    }
}


fn spawn_arrows(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner_query: Query<(&ArrowSpawner, &mut BeatTimer, &mut ArrowBuf)>,
    player_panel: Query<&SongPanel, With<PlayerMarker>>,
    enemy_panel: Query<&SongPanel, With<EnemyMarker>>,
) {
    let player_panel = player_panel.single();
    let enemy_panel = enemy_panel.single();

    // ========================================
    //    create the arrows
    // ========================================

    for (spawner, mut beat_timer, mut arrow_buf) in spawner_query.iter_mut() {

        let Some(beat_tick) = beat_timer.tick(&time) else {
            // the clock did not tick, no arrow yet.
            // move on to the next spawner
            continue;
        };
        let beat = beat_tick.beat();

        arrow_buf.buf.clear();
        spawner.create_arrows_in(&mut arrow_buf.buf, time.as_ref(), beat);

        // =======================================
        //   spawn the arrows
        // =======================================
        let panel = match spawner.team() {
            Team::Player => player_panel,
            Team::Enemy => enemy_panel,
        };

        for arrow in arrow_buf.buf.drain(..) {

            let x = panel.lane_bounds(arrow.lane).center().x;
            let y = panel.bounds().top();
            let z = Layer::Arrows.z();
            let pos = Vec3::new(x, y, z);

            let width = panel.lane_bounds(arrow.lane).width();
            let height = Arrow::height();
            let scale = Vec3::new(width, height, 1.0);

            let color = arrow.lane.colors().base;

            let sprite = SpriteBundle {
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
            };
            commands
                .spawn((arrow, sprite))
                .assign_team_marker(spawner.team());
        }

    // =======================================
    //   check for end conditions
    // =======================================
    /*
    let is_ending = match &spawner.mode {
        SpawningMode::Chart(chart) => {
            spawner.beat_count >= chart.num_beats()
        },
        SpawningMode::Recording(_) => {
            false
        },
        SpawningMode::Random => {
            false
        }
    };

    // check if we need to loop
    
    if is_ending && matches!(&spawner.on_finish, FinishBehavior::Repeat) {
        spawner.beat_count = 0;
    }
    */
               
    }

}

fn move_arrows(time: Res<Time>, mut query: Query<(&mut Transform, &Arrow)>) {
    let now = time.elapsed().as_secs_f32();
    for (mut transform, arrow) in query.iter_mut() {
        let t = (now - arrow.creation_time) / (arrow.arrival_time - arrow.creation_time);
        transform.translation.y = world().bottom() * t + world().top() * (1.0 - t);
    }
}

pub struct ArrowsPlugin;
impl Plugin for ArrowsPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building Arrow plugin...");
        app
            .add_systems(Update, spawn_arrows)
            .add_systems(Update, move_arrows)
            .add_plugins(chart_loader::ChartLoaderPlugin)
        ;
    }
}

