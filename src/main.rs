mod arrow;
mod lane;
mod targets;
mod ui;
mod shaders;
mod layout;

use bevy::{
    prelude::*,
    window::WindowTheme,
};

use layout::BBox;

pub const WORLD_WIDTH: f32 = 400.0;
pub const WORLD_HEIGHT: f32 = 600.0;

pub fn world() -> BBox {
    BBox::from_size(800.0, 600.0)
}

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
                resolution: (world().width(), world().height()).into(),
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = bevy::render::camera::ScalingMode::Fixed {
        width: world().width(),
        height: world().height(), 
    };
    commands.spawn(cam);

    commands.spawn(AudioBundle {
        source: asset_server.load("sounds/Windless Slopes.ogg"),
        ..default()
    });

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
