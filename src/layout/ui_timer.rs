use bevy::prelude::*;

use crate::song::{
    ArrowSpawner
};
use crate::team_markers::{
    PlayerMarker
};

#[derive(Component)]
pub struct TimeText;

pub fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    log::debug!("seting up time text");

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((TimeText, TextBundle {
                    text: Text::from_section(
                        "Time: 0.0".to_string(),
                        TextStyle {
                            font,
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ),
                    ..default()
                }));
        });
}

pub fn update_time_text(
    spawner: Query<&ArrowSpawner<PlayerMarker>>,
    mut query: Query<(&mut Text, &TimeText)>)
{
    let content = spawner
        .get_single()
        .map(|spawner| {
            let beats = spawner.curr_beat();
            format!("Beat: {beats:.2}")
        })
        .unwrap_or(
            format!("<no song playing>")
        );


    for (mut text, _) in query.iter_mut() {
        text.sections[0].value.clear();
        text.sections[0].value.push_str(content.as_str());
    }

}


