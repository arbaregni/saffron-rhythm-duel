use bevy::prelude::*;

use crate::team_markers::{
    PlayerMarker,
    Marker
};
use crate::arrow::{
    LoadChartEvent,
    LoadChartResponse,
    SongFinishedEvent
};

#[derive(Debug)]
#[derive(Component)]
pub struct ChartSelector {
    selectable: Vec<String>,
    curr_selected: Option<usize>,
}
impl ChartSelector {
    fn create() -> Self {
        let mut selectable = Vec::with_capacity(16);

        // initialize the selectable list
        find_selectable_charts(&mut selectable);
        Self {
            selectable,
            curr_selected: None
        }
    }
    fn selected_chart_name(&self) -> Option<&str> {
        self.curr_selected
            .and_then(|index| {
                self.selectable.get(index)
            })
            .map(|s| s.as_str())
    }
}

fn find_selectable_charts(buf: &mut Vec<String>) {
    use std::fs;
    let path = "assets/charts/";
    let Ok(dir) = fs::read_dir(path)
        .inspect_err(|e| log::error!("unable to read chart directory: {e}"))
        else { return; };

    for entry_or_err in dir {
        let Ok(entry) = entry_or_err
            .inspect_err(|e| log::error!("unable to read entry in chart directory: {e}"))
            else { continue; };

        let filepath = entry.path();

        let extension = filepath.extension().and_then(|s| s.to_str());
        if extension != Some("json") {
            continue;
        }

        // gets the filename, without the .json
        let Some(filename) = filepath.file_stem().and_then(|s| s.to_str()) else {
            log::error!("filepath without filestem: {filepath:?}");
            continue;
        };

        buf.push(filename.to_string());
    }

}

const NORMAL_FILL_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);
const PRESSED_FILL_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
const HOVERED_FILL_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);

const NORMAL_BORDER_COLOR: Color = Color::BLACK;

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
#[derive(States)]
enum ChartSelectorState {
    /// User is currently picking a chart
    SelectingChart,
    /// We are attempting to honor the request
    LoadingChart,
    /// We are not on
    Disabled,
}

#[derive(Component)]
struct SelectChartButton {
    index: usize,
    chart_name: String,
}

fn enable_chart_selector_on_song_end<T: Marker>(
    mut song_end_ev: EventReader<SongFinishedEvent<T>>,
    mut state: ResMut<NextState<ChartSelectorState>>,
) {
    if song_end_ev.is_empty() {
        return; // Nothing to do
    }
    song_end_ev.clear();
    state.set(ChartSelectorState::SelectingChart);
}



fn setup_chart_selector<T: Marker>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load(crate::BASE_FONT_NAME);

    let chart_selector = ChartSelector::create();

    let text_style = TextStyle {
        font_size: 36.0,
        color: TEXT_COLOR,
        font,
        ..default()
    };

    let button_style = Style {
        width: Val::Px(300.0),
        height: Val::Px(130.0),
        border: UiRect::all(Val::Px(5.0)),
        // horizontally center child text
        justify_content: JustifyContent::Center,
        // vertically center child text
        align_items: AlignItems::Center,
        ..default()
    };
    let button_bundle = ButtonBundle {
        style: button_style,
        border_color: Color::BLACK.into(),
        background_color: NORMAL_FILL_COLOR.into(),
        ..default()
    };

    let buttons: Vec<_> = chart_selector.selectable
        .iter()
        .enumerate()
        .map(|(index, chart_name)| {
            let select = SelectChartButton {
                index,
                chart_name: chart_name.to_string(),
            };
            let text = TextBundle::from_section(
                chart_name,
                text_style.clone()
            );
            commands
                .spawn((
                    button_bundle.clone(),
                    select
                ))
                .with_children(|p| {
                    p.spawn(text);
                })
                .id()
        })
        .collect();

    let chart_selector_style = Style {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Column,
        ..default()
    };

    let chart_selector_node = NodeBundle {
        style: chart_selector_style,
        ..default()
    };

    commands
        .spawn((
            chart_selector,
            chart_selector_node,
        ))
        .push_children(buttons.as_slice());
    }

fn despawn_chart_selector<T: Marker>(
    mut commands: Commands,
    chart_selector: Query<(Entity, &ChartSelector)>
) {
    for (e, _) in chart_selector.iter() {
        commands.entity(e)
                .despawn_recursive();
    }
}

fn interact_with_buttons(
    mut interactions: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &SelectChartButton,
        ),
        (Changed<Interaction>, With<Button>)
    >,
    mut chart_selector: Query<&mut ChartSelector>,
    mut state: ResMut<NextState<ChartSelectorState>>,
    mut load_chart_ev: EventWriter<LoadChartEvent<PlayerMarker>>,
) {
    let mut chart_selector = chart_selector.single_mut(); // otherwise, this system would have nothing
                                                          // to do

    let mut do_load_chart = false;

    interactions.iter_mut()
        .for_each(|(interaction, mut color, _border_color, select_button)| {
            match *interaction {
                Interaction::Pressed => {
                    *color = PRESSED_FILL_COLOR.into();
                     chart_selector.curr_selected = Some(select_button.index);

                     // signal for us to send the event on a button click
                     do_load_chart = true;
                }
                Interaction::Hovered => {
                    *color = HOVERED_FILL_COLOR.into();
                    chart_selector.curr_selected = Some(select_button.index);
                }
                Interaction::None => {
                    *color = NORMAL_FILL_COLOR.into();
                }
            }
        });

    if do_load_chart {
        if let Some(chart_name) = chart_selector.selected_chart_name() {
            log::info!("emitting load chart event");
            load_chart_ev.send(LoadChartEvent::create(
                chart_name.to_string(),
            ));
            state.set(ChartSelectorState::LoadingChart);
        }
    }
}
fn process_load_chart_resp(
    mut load_chart_resp: EventReader<LoadChartResponse<PlayerMarker>>,
    mut state: ResMut<NextState<ChartSelectorState>>,
) {
    use ChartSelectorState::*;
    load_chart_resp
        .read()
        .for_each(|resp| {
            match &resp.response {
                Ok(()) => {
                    state.set(Disabled)
                }
                Err(e) => {
                    log::error!("unable to load: {e:?}");
                    state.set(SelectingChart)
                }
            }
        })
}

pub struct ChartSelectorPlugin;
impl Plugin for ChartSelectorPlugin {
    fn build(&self, app: &mut App) {
        use ChartSelectorState::*;
        app
            .insert_state(SelectingChart)
            .add_systems(Update, enable_chart_selector_on_song_end::<PlayerMarker>)
            .add_systems(OnEnter(SelectingChart), setup_chart_selector::<PlayerMarker>)
            .add_systems(Update, interact_with_buttons.run_if(in_state(SelectingChart)))
            .add_systems(Update, process_load_chart_resp.run_if(in_state(LoadingChart)))
            .add_systems(OnExit(SelectingChart), despawn_chart_selector::<PlayerMarker>)
        ;
    }
}
