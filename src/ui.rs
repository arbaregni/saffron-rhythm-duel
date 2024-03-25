use bevy::{
    prelude::*
};

#[derive(Component)]
struct TimeText;

fn setup_ui(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

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

fn update_time_text(time: Res<Time>, mut query: Query<(&mut Text, &TimeText)>) {
    let secs = time.elapsed().as_secs_f64();

    for (mut text, _) in query.iter_mut() {
        text.sections[0].value = format!("Time: {:.2}", secs);
    }

}

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_ui)
            .add_systems(Update, update_time_text);
    }
}
