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
};

//
// Our imports
//
use anyhow::{
    Result,
    Context
};
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

#[derive(Event)]
#[derive(Debug)]
/// Request to load a new chart
pub struct LoadChartRequest<T: Marker> {
    chart_name: String,
    // Set to zero to start at the beginning
    #[allow(dead_code)]
    beat_count: u32,
    #[allow(dead_code)]
    team: T,
}
impl <T: Marker> LoadChartRequest<T> {
    pub fn create(chart_name: String) -> LoadChartRequest<T> {
        Self {
            chart_name,
            beat_count: 0,
            team: T::marker(),
        }
    }
}
impl <T: Marker> LoadChartRequest<T> {
    pub fn chart_name(&self) -> &str {
        self.chart_name.as_str()
    }
}

#[derive(Event,Debug)]
/// Response to the attempt to load a new chart
pub struct LoadChartResponse<T: Marker> {
    /// Either OK and the chart was loaded, or Err with a message to the user on why.
    pub response: Result<()>,
    _team: T
}

#[derive(Event)]
#[derive(Debug, Clone)]
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
            log::info!("loading audio asset from path {filepath}");
            AudioBundle {
                source: assets.load(filepath),
                ..default()
            }
        }
        None => {
            log::warn!("no audio bundle configured");
            AudioBundle::default()
        }
    }
}

fn process_load_chart_events<T: Marker>(
    mut load_chart_req: EventReader<LoadChartRequest<T>>,
    mut load_chart_resp: EventWriter<LoadChartResponse<T>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    time: Res<Time>,
    // spawner_q: Query<&ArrowSpawner<T>>,
    mut state: ResMut<NextState<SongState<T>>>,
) {
    if load_chart_req.is_empty() {
        return;
    }
    
    let mut load_chart_impl = |chart_name| -> Result<()> {
        let spawner = ArrowSpawner::<T>::create(chart_name, time.as_ref())
                .with_context(|| format!("while attempting to load chart name '{chart_name}'"))?;

        let audio_bundle = _get_audio_bundle::<T>(spawner.chart(), assets.as_ref());
        
        commands
            .spawn((spawner, audio_bundle, T::marker()));

        state.set(SongState::SettingUp);

        Ok(())
    };

    load_chart_req
        .read()
        .for_each(|ev| {
            log::info!("consuming load chart event");
            let chart_name = ev.chart_name.as_str();
            let resp = load_chart_impl(chart_name);
            load_chart_resp.send(LoadChartResponse {
                response: resp,
                _team: T::marker()
            });
        });
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

            let bundle = MaterialMesh2dBundle {
                mesh: rect,
                material,
                transform,
                ..default()
            };
            log::debug!("spawning arrow: {arrow:#?}");
            let mut entity = commands.spawn((
                arrow.clone(),
                bundle,
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
        //                      = (world().bottom() - world().top()) * t + world().top()
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
        self.build_for_team(app, PlayerMarker{})
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
            .add_event::<LoadChartRequest<T>>()
            .add_event::<LoadChartResponse<T>>()
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
