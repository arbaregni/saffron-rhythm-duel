mod chart;
mod arrow;
pub use arrow::{
    Arrow,
};
mod spawner;
pub use spawner::{
    ArrowSpawner,
    ArrowBuf,
};
mod timer;
pub use timer::{
    BeatTimer,
    FinishBehavior,
};
mod chart_loader;
pub use chart_loader::{
    LoadChartEvent
};

//
// Our imports
//
use bevy::prelude::*;

use crate::team_markers::{
    PlayerMarker,
    EnemyMarker,
    Marker,
};
use crate::layout::{
    BBox,
    Layer,
    SongPanel,
};

fn world() -> BBox {
    crate::world()
}

#[derive(Event)]
#[derive(Debug, Clone)]
pub struct SongFinishedEvent<T: Marker> {
    team: T,
}
impl <T: Marker> SongFinishedEvent<T> {
    pub fn create(team: T) -> Self {
        Self { team }
    }
}


#[derive(Debug,Clone,Eq,PartialEq,Hash)]
#[derive(States)]
pub(in crate::arrow) enum SongState<T: Marker> {
    Playing(T),
    NotPlaying
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

fn spawn_arrows<T: Marker>(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner_query: Query<(&ArrowSpawner, &mut BeatTimer, &mut ArrowBuf), With<T>>,
    panel_query: Query<&SongPanel, With<T>>,
) {
    let panel = panel_query.single();

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
                .spawn((arrow, sprite, T::marker()));

        }

    }

}

fn move_arrows<T: Marker>(
    time: Res<Time>,
    mut arrows: Query<(&mut Transform, &Arrow), With<T>>
) {
    let now = time.elapsed().as_secs_f32();
    for (mut transform, arrow) in arrows.iter_mut() {
        let t = (now - arrow.creation_time) / (arrow.arrival_time - arrow.creation_time);
        transform.translation.y = world().bottom() * t + world().top() * (1.0 - t);
    }
}

fn check_for_song_end<T: Marker>(
    _commands: Commands,
    _time: Res<Time>,
    arrows: Query<&Arrow, With<T>>,
    spawner: Query<(&ArrowSpawner, &BeatTimer), With<T>>,
    mut ending_ev: EventWriter<SongFinishedEvent<T>>,
    mut state: ResMut<NextState<SongState<T>>>,
) {

    let (spawner, timer) = spawner.single();

    let finished_with_beats = timer.beat_count() > spawner.chart().map(|c| c.num_beats()).unwrap_or(0);
    let all_arrows_despawned = arrows.is_empty();

    if finished_with_beats && all_arrows_despawned {

        log::info!("emitting song finished event...");

        ending_ev.send(SongFinishedEvent::create(T::marker()));
        state.set(SongState::NotPlaying);
    }
}

pub struct ArrowsPlugin;
impl Plugin for ArrowsPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building Arrow plugin...");
        self.build_for_team(app, PlayerMarker{})
            .build_for_team(app, EnemyMarker{})
        ;
        app
            .add_plugins(chart_loader::ChartLoaderPlugin)
        ;
    }
}
impl ArrowsPlugin {
    fn build_for_team<'s, T: Marker>(&'s self, app: &mut App, team: T) -> &'s Self {
        app
            .add_event::<SongFinishedEvent<T>>()
            .insert_state(SongState::NotPlaying::<T>)
            .add_systems(Update, (
                    spawn_arrows::<T>,
                    move_arrows::<T>,
                    check_for_song_end::<T>,
                ).run_if(in_state(
                    SongState::Playing(team.clone())
                ))
            )
        ;
        self
    }
}
