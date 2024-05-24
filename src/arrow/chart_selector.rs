use bevy::prelude::*;

use crate::team_markers::{
    PlayerMarker,
    Marker
};
use crate::arrow::LoadChartEvent;

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
const PRESSED_FILL_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);
const HOVERED_FILL_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);

const NORMAL_BORDER_COLOR: Color = Color::BLACK;

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
#[derive(States)]
enum ChartSelectorState {
    // mark as default just so we start with it on
    #[default]
    Enabled,
    Disabled,
}

#[derive(Component)]
struct SelectChartButton {
    index: usize,
    chart_name: String,
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
   
    let make_button = |parent: &mut ChildBuilder<'_>, text_content: &str, index| {
        parent
            .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_FILL_COLOR.into(),
                        ..default()
                    },
                    SelectChartButton {
                        index,
                        chart_name: text_content.to_string(),
                    }
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    text_content,
                    text_style.clone()
                ));
            });
    };

    let text_contents = chart_selector.selectable.clone();
    
    commands
        .spawn((
            chart_selector,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            }
        ))
        .with_children(|parent| {

            // the buttons
            for (index, text_content) in text_contents.iter().enumerate() {

                make_button(parent, text_content, index);
            }
        });
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
    mut state: ResMut<NextState<ChartSelectorState>>,
    mut load_chart_ev: EventWriter<LoadChartEvent<PlayerMarker>>,
) {
    for (interaction, mut color, _border_color, select_chart) in interactions.iter_mut() {
        use Interaction::*;
        // let mut text = text.get_mut(children[0]).unwrap();
        match *interaction {
            Pressed => {
                *color = Color::BLUE.into();
                // send the thing if it's the first frame
                log::info!("pressed");
                load_chart_ev.send(LoadChartEvent::create(
                    select_chart.chart_name.clone(),
                    PlayerMarker{}
                ));
                state.set(ChartSelectorState::Disabled);
            },
            Hovered => {
                *color = Color::RED.into();
            }
            None => {
                *color = NORMAL_FILL_COLOR.into();
            }
        }


    }

}


pub struct ChartSelectorPlugin;
impl Plugin for ChartSelectorPlugin {
    fn build(&self, app: &mut App) {
        log::info!("building chart selector plugin");
        app
            .init_state::<ChartSelectorState>()
            .add_systems(OnEnter(ChartSelectorState::Enabled), setup_chart_selector::<PlayerMarker>)
            .add_systems(Update, interact_with_buttons.run_if(in_state(ChartSelectorState::Enabled)))
            .add_systems(OnExit(ChartSelectorState::Enabled), despawn_chart_selector::<PlayerMarker>)
        ;
    }
}
