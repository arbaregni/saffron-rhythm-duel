#![allow(dead_code)]

mod arrow;
mod lane;
mod judgement;
mod ui;
mod layout;
mod record;
mod input;

use std::path::PathBuf;

use anyhow::{Result, Context};
use bevy::{
    prelude::*,
    window::WindowTheme,
};
use clap::{
    Parser,
};
use serde::Deserialize;

use layout::BBox;

pub const WORLD_WIDTH: f32 = 400.0;
pub const WORLD_HEIGHT: f32 = 600.0;

pub const BACKGROUND_COLOR: Color = Color::rgb(27.0 / 255.0, 32.0 / 255.0, 33.0 / 255.0); // eerie black 
pub fn world() -> BBox {
    BBox::from_size(800.0, 600.0)
}

#[derive(Parser)]
#[derive(Resource)]
#[derive(Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long, value_name = "FILE")]
    chart: Option<PathBuf>,

    #[arg(long, value_name = "FILE", default_value = "assets/config.toml")]
    config: PathBuf,

    #[arg(short, long, value_enum, default_value_t)]
    on_finish: arrow::FinishBehavior,

    #[arg(short, long, default_value = "0.3")]
    fallback_beat_duration: f32,

    #[arg(short, long)]
    debug: bool,
}


#[derive(Debug)]
#[derive(Resource)]
#[derive(Deserialize)]
struct Config {
    keybindings: KeyBindings
}
#[derive(Debug)]
#[derive(Deserialize)]
#[allow(non_snake_case)]
struct KeyBindings {
    lane_hit_L1: String,
    lane_hit_L2: String,
    lane_hit_R1: String,
    lane_hit_R2: String,
}


fn main() -> Result<()> {
    let cli = CliArgs::parse();

    pretty_env_logger::formatted_timed_builder()
        .filter_level(if cli.debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .build();


    log::info!("Reading config file...");
    let config_str = std::fs::read_to_string(&cli.config)
        .with_context(|| format!("Parsing config file"))?;
    let config: Config = toml::from_str(config_str.as_ref())?;

    log::info!("Initializing...");

    App::new()
        .insert_resource(cli)
        .insert_resource(config)
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
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(arrow::ArrowsPlugin)
        .add_plugins(judgement::TargetsPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(input::InputPlugin)
        .add_systems(Update, close_on_esc)
        .run();
    Ok(())
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
