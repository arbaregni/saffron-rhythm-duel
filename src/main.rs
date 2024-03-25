mod arrow;
mod lane;
mod targets;
mod ui;
mod shaders;

use bevy::{
    prelude::*,
    window::WindowTheme,
};

pub const WORLD_WIDTH: f32 = 400.0;
pub const WORLD_HEIGHT: f32 = 600.0;

fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .build();

    log::info!("Initializing...");

    App::new()
        .add_systems(Startup, setup)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Rhythm Tutorial".to_string(),
                resolution: (WORLD_WIDTH, WORLD_HEIGHT).into(),
                window_theme: Some(WindowTheme::Dark),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MaterialPlugin::<shaders::CustomMaterial>::default())
        .insert_resource(ClearColor(Color::rgb(27.0 / 255.0, 32.0 / 255.0, 33.0 / 255.0))) // eerie black
        .add_plugins(arrow::ArrowsPlugin)
        .add_plugins(targets::TargetsPlugin)
        .add_plugins(ui::UiPlugin)
        .add_systems(Update, close_on_esc)
        .run()
}

fn setup(mut commands: Commands) {
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = bevy::render::camera::ScalingMode::Fixed {
        width: WORLD_WIDTH,
        height: WORLD_HEIGHT, 
    };
    commands.spawn(cam);
}


fn close_on_esc(_commands: Commands,
                mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
                input: Res<ButtonInput<KeyCode>>) 
{
    if input.just_pressed(KeyCode::Escape) {
        log::info!("exitting");
        app_exit_events.send(bevy::app::AppExit);
    }
}
