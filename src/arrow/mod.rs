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
    // Set to zero to start at the beginning
    beat_count: u32,
    team: T,
}
impl <T: Marker> LoadChartEvent<T> {
    pub fn create(chart_name: String) -> LoadChartEvent<T> {
        Self {
            chart_name,
            beat_count: 0,
            team: T::marker(),
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

fn _get_audio_bundle<T: Marker>(
    chart: &Chart,
    assets: &AssetServer,
) -> AudioBundle {

    if T::is_local() {
        return AudioBundle::default();
    }


    match chart.sound_file() {
        Some(filename) => {
            let filepath = format!("sounds/{filename}");
            log::info!("loading audio asset from path {filepath}");
            AudioBundle {
                source: assets.load(filepath),
                ..default()
            }
        }
        None => AudioBundle::default()
    }
}
fn process_load_chart_events<T: Marker>(
    mut load_chart_events: EventReader<LoadChartEvent<T>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    time: Res<Time>,
    // spawner_q: Query<&ArrowSpawner<T>>,
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

            let Ok(spawner) = ArrowSpawner::<T>::create(chart_name, time.as_ref())
                .inspect_err(|e| log::error!("could not create arrow spawner: {e}"))
                else { return; };

            let audio_bundle = _get_audio_bundle::<T>(spawner.chart(), assets.as_ref());
            
            let arrow_buf = ArrowBuf::new();

            commands
                .spawn((spawner, arrow_buf, audio_bundle, T::marker()));

            state.set(SongState::Playing(T::marker()));
        });
}

    
fn spawn_arrows<T: Marker>(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: Query<(&mut ArrowSpawner<T>, &mut ArrowBuf), With<T>>,
    panel_query: Query<&SongPanel, With<T>>,
) {
    let panel = panel_query.single();

    // ========================================
    //    create the arrows
    // ========================================
    //
    let (mut spawner, mut arrow_buf) = spawner.single_mut();

    spawner.tick(&time);

    spawner.create_arrows_in(&mut arrow_buf, time.as_ref());

    // =======================================
    //   spawn the arrows
    // =======================================
    for arrow in arrow_buf.drain() {

        let x = panel.lane_bounds(arrow.lane).center().x;
        let y = panel.bounds().top();
        let z = Layer::Arrows.z();
        let pos = Vec3::new(x, y, z);

        let width = panel.lane_bounds(arrow.lane).width();
        let height = Arrow::height();
        let scale = Vec3::new(width, height, 1.0);

        let color = arrow.lane.colors().base;

        let transform = Transform {
            translation: pos,
            scale,
            ..default()
        };
        let sprite = Sprite {
            color,
            ..default()
        };
        let sprite_bundle = SpriteBundle {
            transform,
            sprite,
            ..default()
        };

        log::debug!("spawning arrow: {arrow:#?}");
        commands
            .spawn((arrow, sprite_bundle, T::marker()));

    }

}

/// Put the arrows where they need to be
fn position_arrows<T: Marker>(
    spawner: Query<&ArrowSpawner<T>>,
    mut arrows: Query<(&mut Transform, &Arrow), With<T>>
) {
    let spawner = spawner.single();
    for (mut transform, arrow) in arrows.iter_mut() {

        // calculate the fraction of the way through the lead space we are
        let t = (spawner.beat_fraction() - arrow.beat_fraction()) / spawner.chart().lead_time_beats();

        // Set the y, where when t = 0% we are at the top and when t = 100% we are at the bottom
        transform.translation.y = world().bottom() * t + world().top() * (1.0 - t);
        //                      = (world().bottom() - world().top()) * t + world().top()
    }
}

fn check_for_song_end<T: Marker>(
    _commands: Commands,
    time: Res<Time>,
    arrows: Query<&Arrow, With<T>>,
    spawner_q: Query<&ArrowSpawner<T>>,
    mut state: ResMut<NextState<SongState<T>>>,
) {
    let now = time.elapsed().as_secs_f32();

    let spawner = spawner_q.single();

    let finished_with_beats = spawner.beat_count() > spawner.chart().num_beats();
    let all_arrows_despawned = arrows.is_empty();
    
    let song_end = spawner.song_start() + spawner.chart().total_duration();
    let buffer_time = 1.2 * spawner.chart().lead_time_secs();

    if finished_with_beats && all_arrows_despawned && now > song_end + buffer_time {
        log::info!("set state: not playing song {:?}", T::team());
        state.set(SongState::NotPlaying);
    }
}

fn cleanup_spawner<T: Marker>(
    mut commands: Commands,
    spawner: Query<(Entity, &ArrowSpawner<T>)>,
    mut ending_ev: EventWriter<SongFinishedEvent<T>>,
) {
    spawner
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
                    position_arrows::<T>,
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
