use bevy::prelude::*;

use anyhow::Result;
use crate::arrow::{
    chart::Chart,
    spawner::ArrowSpawner,
    timer::BeatTimer,
};
use crate::team_markers::{
    Marker,
    PlayerMarker,
    EnemyMarker,
};

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




fn process_load_chart_events<T: Marker>(
    mut load_chart_events: EventReader<LoadChartEvent<T>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut query: Query<(&mut ArrowSpawner, &mut BeatTimer), With<T>>,
) {
    if load_chart_events.is_empty() {
        return;
    }
    let (mut spawner, mut timer) = match query.get_single_mut() {
        Ok(items) => items,
        Err(e) => {
            bevy::log::warn_once!("can not process load charts events, can not find entity: {e}");
            return;
        }
    };
    load_chart_events
        .read()
        .for_each(|ev| {
            log::info!("consuming load chart event");

            let chart_name = ev.chart_name.as_str();
            let Ok(chart) = parse_chart(chart_name)
                .inspect_err(|e| log::error!("unable to parse {chart_name} due to: {e}"))
                else { return; };

            timer.reset_and_load_settings_for(&chart);
            spawner.set_chart(chart);

            commands.spawn(AudioBundle {
                source: assets.load("sounds/windless-slopes.ogg"),
                ..default()
            });

        });
}

fn parse_chart(name: &str) -> Result<Chart> {
    let path = format!("assets/charts/{name}.json");

    // read the chart from the file
    let text = std::fs::read_to_string(&path)?;

    let chart: Chart = serde_json::from_str(text.as_str())?;

    log::info!("Parsed chart '{}' from {}", chart.chart_name(), path);

    Ok(chart)
}


pub struct ChartLoaderPlugin;
impl Plugin for ChartLoaderPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building ChartLoaderPlugin");
        app
            .add_event::<LoadChartEvent<PlayerMarker>>()
            .add_event::<LoadChartEvent<EnemyMarker>>()
            .add_systems(Update, process_load_chart_events::<PlayerMarker>)
            .add_systems(Update, process_load_chart_events::<EnemyMarker>)
        ;
    }
}
