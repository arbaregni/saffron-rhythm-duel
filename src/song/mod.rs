use std::sync::Arc;

mod chart;
pub use chart::{
    Chart,
    ChartName,
    ChartAssets
};
mod arrow;
pub use arrow::{
    Arrow,
    ArrowStatus,
};
mod spawner;
pub use spawner::{
    ArrowSpawner,
    SyncSpawnerEvent
};

//
// Our imports
//

use bevy::{
    prelude::*,
    sprite::{
        MaterialMesh2dBundle,
        Mesh2dHandle
    }
};

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

/// For the text shown with debug flag --show-beat-numbers
const BEAT_NUMBER_TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);

#[derive(Debug, Clone, Event)]
/// Request to load a new chart
pub struct LoadChartRequest<T: Marker> {
    chart_name: ChartName,
    _team: T,
}
impl <T: Marker> LoadChartRequest<T> {
    pub fn from(chart_name: ChartName) -> LoadChartRequest<T> {
        Self {
            chart_name,
            _team: T::marker(),
        }
    }
    pub fn chart_name(&self) -> &ChartName {
        &self.chart_name
    }
}

#[derive(Debug, Clone, Event)]
pub struct SongFinishedEvent<T: Marker> {
    _team: T,
}
impl <T: Marker> SongFinishedEvent<T> {
    pub fn create(team: T) -> Self {
        Self { _team: team }
    }
}


#[derive(Debug,Clone,Eq,PartialEq,Hash)]
#[derive(States)]
pub enum SongState<T: Marker> {
    NotPlaying,
    SettingUp,
    Playing,
    _Marker(T),
}

fn _get_audio_bundle<T: Marker>(
    chart: &Chart,
    assets: &AssetServer,
) -> AudioBundle {

    if T::is_remote() {
        log::info!("skipping loading audio asset for remote player");
        return AudioBundle::default();
    }

    match chart.sound_file() {
        Some(filename) => {
            let filepath = format!("sounds/{filename}");
            AudioBundle {
                source: assets.load(filepath),
                ..default()
            }
        }
        None => {
            // no audio bundle configured, just use the defaults
            AudioBundle::default()
        }
    }
}

fn _spawn_spawner<T: Marker>(
    spawner: ArrowSpawner<T>,
    assets: &AssetServer,
    commands: &mut Commands,
) -> Entity {

    let audio_bundle = _get_audio_bundle::<T>(spawner.chart(), assets);

    let entity_name = Name::from(
        format!("spawner-{}", T::as_str())
    );

    commands.spawn((
        entity_name,
        spawner, 
        audio_bundle,
        T::marker(),
    )).id()
}

fn process_load_chart_events<T: Marker>(
    mut load_chart_req: EventReader<LoadChartRequest<T>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    time: Res<Time>,
    chart_assets: Res<ChartAssets>,
    mut state: ResMut<NextState<SongState<T>>>,
) {
    // read one event per frame
    load_chart_req
        .read()
        .next()
        .inspect(|ev| {
            log::info!("consuming load chart event");

            let chart = chart_assets.get(ev.chart_name());
            let chart = Arc::clone(chart);
            let spawner = ArrowSpawner::<T>::create(chart, &time);

            _spawn_spawner::<T>(
                spawner,
                &assets,
                &mut commands
            );
            state.set(SongState::SettingUp);
        });
}

fn process_sync_spawner_events<T: Marker>(
    mut sync_spawner_ev: EventReader<SyncSpawnerEvent<T>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    chart_assets: Res<ChartAssets>,
    time: Res<Time>,
    mut spawner_q: Query<&mut ArrowSpawner<T>>,
    mut state: ResMut<NextState<SongState<T>>>,
) {
    use SyncSpawnerEvent::*;

    sync_spawner_ev
        .read()
        .for_each(|ev| match ev {
            Spawning { chart_name, scroll_pos, is_paused, _team } => {
                let scroll_pos = *scroll_pos;
                let is_paused = *is_paused;

                let chart = chart_assets.get(&chart_name);
                let chart = Arc::clone(chart);

                match spawner_q.get_single_mut().ok() {

                    Some(mut spawner) => {
                        spawner.set_chart(chart);
                        spawner.set_scroll_pos(scroll_pos);
                        spawner.set_is_paused(is_paused);
                    }
                    None => {
                        // must create spawner first
                        let mut spawner = ArrowSpawner::<T>::create(chart, &time);
                        spawner.set_scroll_pos(scroll_pos);
                        spawner.set_is_paused(is_paused);

                        _spawn_spawner(
                            spawner,
                            &assets,
                            &mut commands
                        );

                        state.set(SongState::SettingUp);
                    }
                };

            }
            NotSpawning {} => {
                // this will destruct everything
                state.set(SongState::NotPlaying);
            }
        })
}

    
/// Spawns all the arrows of a given song
fn setup_arrows<T: Marker>(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    cli: Res<crate::CliArgs>,
    asset_server: Res<AssetServer>,
    mut spawner_q: Query<&mut ArrowSpawner<T>>,
    panel_query: Query<&SongPanel, With<T>>,
    mut state: ResMut<NextState<SongState<T>>>,
) {
    log::info!("in setup_arrows");

    let panel = panel_query.single();

    let spawner = spawner_q.single_mut();
    spawner
        .as_ref()
        .arrows_to_spawn()
        .for_each(|arrow| {
            let x = panel.lane_bounds(arrow.lane()).center().x;
            let y = panel.bounds().top();
            let z = Layer::Arrows.z();
            let pos = Vec3::new(x, y, z);

           let transform = Transform {
                translation: pos,
                ..default()
            };

            let width = panel.lane_bounds(arrow.lane()).width();
            let height = Arrow::height();

            let rect = Mesh2dHandle(
                meshes.add(Rectangle::new(width, height))
            );
            let color = arrow.lane().colors().base;
            let material = materials.add(color);

            let mesh_bundle = MaterialMesh2dBundle {
                mesh: rect,
                material,
                transform,
                ..default()
            };

            let entity_name = Name::new(
                format!("arrow-{}-{}-{}", T::as_str(), arrow.arrival_beat(), arrow.lane().as_str())
            );

            log::debug!("spawning arrow: {arrow:#?}");
            let mut entity = commands.spawn((
                entity_name,
                arrow.clone(),
                mesh_bundle,
                T::marker(),
            ));

            // helpful debugging
            if cli.show_beat_numbers {

                let text_content = format!("{:.2}", arrow.arrival_beat());

                let font = asset_server.load(crate::BASE_FONT_NAME);
                let font_size = 20.0;
                let color = BEAT_NUMBER_TEXT_COLOR;

                let style = TextStyle {
                    font, font_size, color
                };
                let text = Text {
                    sections: vec![
                        TextSection {
                            value: text_content,
                            style
                        }
                    ],
                    ..default()
                };
                let pos = Vec3::new(
                    0.0,
                    0.0,
                    Layer::AboveArrows.z(),
                );
                let transform = Transform {
                    translation: pos,
                    ..default()
                };
                let text_bundle = Text2dBundle {
                    text,
                    transform,
                    ..default()
                };

                // give the entity the text component
                entity.with_children(|b| {
                    b.spawn((
                        text_bundle,
                    ));
                });

            }
            
            log::debug!("spawned: {:?}", entity.id());

        });

    log::info!("done setting up arrows");

    state.set(SongState::Playing);

}

fn tick_spawner<T: Marker>(
    mut spawner_q: Query<&mut ArrowSpawner<T>>,
    time: Res<Time>
) {
    let mut spawner = spawner_q.single_mut();
    spawner.tick(&time);
}

/// Put the arrows where they need to be
fn position_arrows<T: Marker>(
    spawner: Query<&ArrowSpawner<T>>,
    mut arrows: Query<(&mut Transform, &Arrow), With<T>>
) {
    let spawner = spawner.single();
    for (mut transform, arrow) in arrows.iter_mut() {

        // calculate the fraction of the way through the lead space we are
        let lead_time = spawner.chart().lead_time_beats();
        let finish = arrow.arrival_beat();
        let start = arrow.arrival_beat() - lead_time;

        // calculate the progress towards arriving at the finish line
        let curr = spawner.curr_beat();
        // map [start, finish] to [0, 1]
        //     [0, finish - start]
        let t = (curr - start) / (finish - start);

        // Set the y, where when t = 0% we are at the top and when t = 100% we are at the bottom
        transform.translation.y = world().bottom() * t + world().top() * (1.0 - t);
    }
}


fn check_for_song_end<T: Marker>(
    spawner_q: Query<&ArrowSpawner<T>>,
    mut state: ResMut<NextState<SongState<T>>>,
) {
    let Some(spawner) = spawner_q.get_single().ok() else {
        return;
    };

    if spawner.is_finished() {
        log::info!("set state: not playing song {:?}", T::team());
        state.set(SongState::NotPlaying);
    }
}

fn cleanup_spawner<T: Marker>(
    mut commands: Commands,
    spawner: Query<(Entity, &ArrowSpawner<T>)>,
    arrows: Query<(Entity, &Arrow), With<T>>,
    mut ending_ev: EventWriter<SongFinishedEvent<T>>,
) {
    spawner
        .iter()
        .for_each(|(e, _)| {
            commands.entity(e)
                    .despawn_recursive()
        });

    arrows
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
        let chart_assets = ChartAssets::create()
            .expect("chart assets should load");

        app
            .register_type::<ArrowStatus>()
            .register_type::<Arrow>()
            .register_type::<Chart>()
            .insert_resource(chart_assets)

            // needed for the enemy spawner to keep in sync with remote
            .add_systems(Update, process_sync_spawner_events::<EnemyMarker>)
        ;

        self
            .build_for_team(app, PlayerMarker{})
            .build_for_team(app, EnemyMarker{})
        ;
    }
}
impl ArrowsPlugin {
    fn build_for_team<'s, T: Marker>(&'s self, app: &mut App, _team: T) -> &'s Self {
        let not_playing = in_state(SongState::NotPlaying::<T>);
        let playing = in_state(SongState::Playing::<T>);

        let on_setup = OnEnter(SongState::SettingUp::<T>);
        let on_stop_playing = OnEnter(SongState::NotPlaying::<T>);

        app
            .register_type::<ArrowSpawner<T>>()

            .add_event::<LoadChartRequest<T>>()
            .add_event::<SyncSpawnerEvent<T>>()
            .add_event::<SongFinishedEvent<T>>()
            .insert_state(SongState::NotPlaying::<T>)

            // Load the charts, if we are not playing a song already
            .add_systems(Update, 
                process_load_chart_events::<T>.run_if(not_playing)
            )

            // when we start playing (ideally before) then we load all of the arrows
            .add_systems(on_setup, setup_arrows::<T>)

            // while the song is playing, move the arrow and check for the end
            .add_systems(Update, (
                    tick_spawner::<T>,
                    position_arrows::<T>,
                    check_for_song_end::<T>,
                ).run_if(playing)
            )
            // when we finish, despawn it
            .add_systems(on_stop_playing, cleanup_spawner::<T>)

        ;
        self
    }
}
