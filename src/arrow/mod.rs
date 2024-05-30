mod chart;
pub use chart::{
    Chart
};
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
#[derive(Debug)]
pub struct LoadChartEvent<T: Marker> {
    chart_name: String,
    team: T,
}
impl <T: Marker> LoadChartEvent<T> {
    pub fn create(chart_name: String, team: T) -> LoadChartEvent<T> {
        Self {
            chart_name,
            team
        }
    }
}
impl <T: Marker> LoadChartEvent<T> {
    pub fn chart_name(&self) -> &str {
        self.chart_name.as_str()
    }
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
pub enum SongState<T: Marker> {
    Playing(T),
    NotPlaying
}

fn process_load_chart_events<T: Marker>(
    mut load_chart_events: EventReader<LoadChartEvent<T>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    time: Res<Time>,
    mut state: ResMut<NextState<SongState<T>>>,
) {
    if load_chart_events.is_empty() {
        return;
    }
    load_chart_events
        .read()
        .for_each(|ev| {
            log::info!("consuming load chart event");

            let chart_name = ev.chart_name.as_str();
            let Ok(chart) = Chart::try_load_from_file(chart_name)
                .inspect_err(|e| log::error!("unable to parse {chart_name} due to: {e}"))
                else { return; };

            let spawner = ArrowSpawner::create(chart, T::team());


            let audio = match (T::is_local(), spawner.chart().sound_file()) {
                (true, Some(filename)) => {
                    let filepath = format!("sounds/{filename}");
                    log::info!("loading audio asset from path {filepath}");
                    AudioBundle {
                        source: assets.load(filepath),
                        ..default()
                    }
                }
                _ => {
                    AudioBundle::default()
                }
            };
            let beat_timer = BeatTimer::create(time.as_ref(), spawner.chart());

            commands.spawn((
                spawner,
                beat_timer,
                audio,
                ArrowBuf::new(),
                T::marker()
            ));
            state.set(SongState::Playing(T::marker()));
        });
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
    time: Res<Time>,
    arrows: Query<&Arrow, With<T>>,
    spawner: Query<(&ArrowSpawner, &BeatTimer), With<T>>,
    mut state: ResMut<NextState<SongState<T>>>,
) {
    let now = time.elapsed().as_secs_f32();

    let (spawner, timer) = spawner.single();

    let finished_with_beats = timer.beat_count() > spawner.chart().num_beats();
    let all_arrows_despawned = arrows.is_empty();
    
    let song_end = timer.song_start() + spawner.chart().total_duration();
    let buffer_time = 1.2 * spawner.chart().lead_time_secs();

    if finished_with_beats && all_arrows_despawned && now > song_end + buffer_time {
        log::info!("set state: not playing song {:?}", T::team());
        state.set(SongState::NotPlaying);
    }
}

fn cleanup_spawner<T: Marker>(
    mut commands: Commands,
    spawners: Query<(Entity, &ArrowSpawner), With<T>>,
    mut ending_ev: EventWriter<SongFinishedEvent<T>>,
) {
    spawners
        .iter()
        .for_each(|(e, _)| {
            commands.entity(e)
                    .despawn_recursive()
        });

    // tell the outside world that we finished
    log::info!("emitting song finished event...");
    ending_ev.send(SongFinishedEvent::create(T::marker()));
}


pub struct ArrowsPlugin;
impl Plugin for ArrowsPlugin {
    fn build(&self, app: &mut App) {
        self.build_for_team(app, PlayerMarker{})
            .build_for_team(app, EnemyMarker{})
        ;
    }
}
impl ArrowsPlugin {
    fn build_for_team<'s, T: Marker>(&'s self, app: &mut App, team: T) -> &'s Self {
        app
            .add_event::<LoadChartEvent<T>>()
            .add_event::<SongFinishedEvent<T>>()
            .insert_state(SongState::NotPlaying::<T>)

            // Load the charts, if we are not playing a song already
            .add_systems(Update, 
                    process_load_chart_events::<T>.run_if(in_state(
                            SongState::NotPlaying::<T>
                    ))
            )

            // while the song is playing, move the arrow and check for the end
            .add_systems(Update, (
                    spawn_arrows::<T>,
                    move_arrows::<T>,
                    check_for_song_end::<T>,
                ).run_if(in_state(
                    SongState::Playing(team.clone())
                ))
            )
            // when we finish, despawn it
            .add_systems(OnEnter(SongState::NotPlaying::<T>), cleanup_spawner::<T>)
        ;
        self
    }
}
