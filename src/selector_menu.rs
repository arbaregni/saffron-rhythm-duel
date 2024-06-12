use bevy::prelude::*;

use crate::team_markers::{
    PlayerMarker,
    Marker
};
use crate::song::{
    ChartAssets,
    ChartName,
    LoadChartRequest,
    SongFinishedEvent,
};

#[derive(Debug)]
#[derive(Component)]
pub struct ChartSelector {
    selectable: Vec<ChartName>,
    curr_selected: Option<usize>,
}
impl ChartSelector {
    fn create(selectable: Vec<ChartName>) -> Self {
        Self {
            selectable,
            curr_selected: None
        }
    }
    fn selected_chart_name(&self) -> Option<&ChartName> {
        self.curr_selected
            .and_then(|index| {
                self.selectable.get(index)
            })
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
    /// We are not on
    Disabled,
}

#[derive(Component)]
struct SelectChartButton {
    index: usize,
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
    charts: Res<ChartAssets>,
) {
    let font = asset_server.load(crate::BASE_FONT_NAME);

    let selectable = charts.chart_names()
        .cloned()
        .collect();

    let chart_selector = ChartSelector::create(selectable);

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
        border_color: NORMAL_BORDER_COLOR.into(),
        background_color: NORMAL_FILL_COLOR.into(),
        ..default()
    };

    let buttons: Vec<_> = chart_selector.selectable
        .iter()
        .enumerate()
        .map(|(index, chart_name)| {
            let select = SelectChartButton {
                index,
            };
            let text = TextBundle::from_section(
                format!("{}", chart_name),
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
    mut load_chart_ev: EventWriter<LoadChartRequest<PlayerMarker>>,
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
            load_chart_ev.send(LoadChartRequest::from(
                chart_name.clone()
            ));
            state.set(ChartSelectorState::Disabled);
        }
    }
}

pub struct ChartSelectorPlugin;
impl Plugin for ChartSelectorPlugin {
    fn build(&self, app: &mut App) {
        use ChartSelectorState::*;

        let selecting = in_state(SelectingChart);

        app
            .insert_state(SelectingChart)

            .add_systems(OnEnter(SelectingChart), setup_chart_selector::<PlayerMarker>)
            .add_systems(Update, (
                interact_with_buttons
            ).run_if(selecting))
            .add_systems(OnExit(SelectingChart), despawn_chart_selector::<PlayerMarker>)
            .add_systems(Update, 
                enable_chart_selector_on_song_end::<PlayerMarker>
            )
        ;
    }
}
