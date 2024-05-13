#![allow(dead_code)]

mod arrow;
mod lane;
mod judgement;
mod layout;
mod record;
mod input;
mod team_markers;

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
use lane::Lane;

pub const BACKGROUND_COLOR: Color = Color::rgb(27.0 / 255.0, 32.0 / 255.0, 33.0 / 255.0); // eerie black 

pub fn world() -> BBox {
    BBox::from_size(1600.0, 800.0) // cut in hhalf from world size;
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
impl KeyBindings {
    fn key_name(&self, lane: Lane) -> &str {
        match lane {
            Lane::L1 => self.lane_hit_L1.as_str(),
            Lane::L2 => self.lane_hit_L2.as_str(),
            Lane::R1 => self.lane_hit_R1.as_str(),
            Lane::R2 => self.lane_hit_R2.as_str(),
        }
    }
}

const BASE_FONT_NAME: &str = "fonts/FiraSans-Bold.ttf";

fn main() -> Result<()> {
    let cli = CliArgs::parse();

    //pretty_env_logger::init();
    /*
    env_logger::builder()
        .format(|f, record| {
            use std::io::Write;
            use log::Level::*;

            let target = record.target();

            let level = match record.level() {
                Trace => "TRACE",
                Debug => "DEBUG",
                Info  => "INFO",
                Warn  => "WARN",
                Error => "ERROR",
            };

            let module = record.module_path().unwrap_or("");

            let time = f.timestamp_millis();

            writeln!(f, " {time} {level} {target} {module} > {}", record.args(),)?;

            Ok(())
        })
        .try_init()
        .expect("failed to initialize logger");
        */


    log::info!("Reading config file...");
    let config_str = std::fs::read_to_string(&cli.config)
        .with_context(|| format!("Parsing config file"))?;
    let config: Config = toml::from_str(config_str.as_ref())?;

    log::info!("Initializing...");

    App::new()
        .insert_resource(cli)
        .insert_resource(config)
        .add_systems(Startup, setup)
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Rhythm Tutorial".to_string(),
                        resolution: (world().width(), world().height()).into(),
                        window_theme: Some(WindowTheme::Dark),
                        ..default()
                    }),
                    ..default()
            })
            //.disable::<bevy::log::LogPlugin>()
        )
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(arrow::ArrowsPlugin)
        .add_plugins(judgement::TargetsPlugin)
        .add_plugins(layout::UiPlugin)
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
