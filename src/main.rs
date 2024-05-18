#![allow(dead_code)]

mod arrow;
mod lane;
mod judgement;
mod layout;
mod record;
mod input;
mod team_markers;
mod remote;
mod widgets;

use std::path::PathBuf;
use std::net::SocketAddr;

use anyhow::{Result, Context};
use bevy::prelude::*;
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

    #[arg(long)]
    connect_to: SocketAddr,

    #[arg(long)]
    disable_remote_listener: bool,

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

fn make_window_plugin() -> bevy::window::WindowPlugin {
    use bevy::window::*;

    let primary_window = Window {
        title: "Saffron Rhythm Duel".to_string(),
        resolution: (world().width(), world().height()).into(),
        window_theme: Some(WindowTheme::Dark),
        ..default()
    };
    WindowPlugin {
        primary_window: Some(primary_window),
        exit_condition: ExitCondition::OnPrimaryClosed,
        close_when_requested: false,
    }
}

fn main() -> Result<()> {
    let cli = CliArgs::parse();

    // configure the logging
    bevy::log::tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_level(true)
        .with_thread_names(true)
        .try_init()
        .expect("unable to initialize tracing subscriber");



    log::info!("Reading config file...");
    let config_str = std::fs::read_to_string(&cli.config)
        .with_context(|| format!("Parsing config file"))?;
    let config: Config = toml::from_str(config_str.as_ref())?;

    log::info!("Initializing app...");

    App::new()
        // Load resources
        .insert_resource(cli)
        .insert_resource(config)
        .insert_resource(ClearColor(BACKGROUND_COLOR))

        // Configure default plugins
        .add_plugins(DefaultPlugins
            .set(make_window_plugin())
            .disable::<bevy::log::LogPlugin>()
        )
        // Load custom plugins
        .add_plugins(arrow::ArrowsPlugin)
        .add_plugins(judgement::JudgementPlugin)
        .add_plugins(layout::UiPlugin)
        .add_plugins(input::InputPlugin)
        .add_plugins(remote::RemoteUserPlugin)
        .add_plugins(widgets::WidgetsPlugin)

        // Systems
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .add_systems(Update, close_on_window_close_requested)
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


fn close_on_esc(
    input: Res<ButtonInput<KeyCode>>,
    mut app_exit: ResMut<Events<bevy::app::AppExit>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        teardown(app_exit.as_mut());
    }
}

fn close_on_window_close_requested(
    mut close_requested: EventReader<bevy::window::WindowCloseRequested>,
    mut app_exit: ResMut<Events<bevy::app::AppExit>>,
) {
    if close_requested.is_empty() {
        return;
    }
    close_requested.clear();
    teardown(app_exit.as_mut());
}

fn teardown(app_exit_events: &mut Events<bevy::app::AppExit>) {
    log::info!("tearing down - sending app exit event");
    app_exit_events.send(bevy::app::AppExit);
}
