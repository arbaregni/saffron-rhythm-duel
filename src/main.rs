mod keycode_serde;
mod user_settings;
mod logging;
mod song;
mod lane;
mod judgement;
mod layout;
mod record;
mod input;
mod team_markers;
mod remote;
mod widgets;
mod selector_menu;

use std::path::PathBuf;
use std::net::IpAddr;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use anyhow::Result;
use bevy::prelude::*;
use clap::{
    Subcommand,
    Parser,
};
use directories::ProjectDirs;

use layout::BBox;

pub const BACKGROUND_COLOR: Color = Color::rgb(27.0 / 255.0, 32.0 / 255.0, 33.0 / 255.0); // eerie black 

pub fn world() -> BBox {
    BBox::from_size(1600.0, 800.0) // cut in half from world size;
}

pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "arbaregni", "saffron-rhythm-duel")
}

#[derive(Parser)]
#[derive(Resource)]
#[derive(Debug)]
#[command(version, about, arg_required_else_help=true, long_about = None)]
struct CliArgs {
    #[arg(long, value_name = "FILE")]
    /// Supply to override the default settings directory.
    settings: Option<PathBuf>,

    #[arg(long, default_value_t = logging::LevelFilter::INFO)]
    /// Sets the default log level to report to stdout.
    log_level: logging::LevelFilter,

    #[arg(long, value_parser=logging::parse_log_filter, num_args = 0.., value_delimiter = ',')]
    /// Specify one or more log filters, separated by commas.
    /// Log filters are a rust module path (excluding the project name), e.g. `judgement::metrics'.
    /// Optionally, you may include an equal sign and one of OFF, DEBUG, INFO, WARN, or ERROR to specify the level of logging to filter out.
    /// For example, `judgement::metrics=OFF`.
    log_filters: Option<Vec<logging::LogFilter>>,

    #[arg(short, long)]
    /// Enable showing the beat numbers on the notes
    show_beat_numbers: bool,

    #[arg(short, long)]
    /// Enable showing the debug inspector windows
    debug_inspector: bool,

    #[arg(long)]
    /// Force the program to reset the settings to defaults on load.
    reset_to_default_settings: bool,

    #[command(subcommand)]
    /// What mode to run in
    mode: ConnectionMode,
}

#[derive(Subcommand)]
#[derive(Debug,Clone)]
#[command(arg_required_else_help=true)]
enum ConnectionMode {
    /// Listen for the remote user to connect to you.
    Listen {
        /// Supply this parameter to override the configured settings default port.
        /// Your remote partner will need this port and your IP address to connect.
        #[arg(long)]
        port: Option<u16>,
    },
    /// Connect to a remote host.
    Connect {
        /// Attempts to connect to a remote IP address.
        remote_addr: IpAddr,

        /// Optionally, specify a port to connect to on the remote machine.
        /// By default, this will take from the configured port in settings.toml
        #[arg(long)]
        remote_port: Option<u16>,
    },
    /// Record a song locally
    Record {
        // TODO
    }
}

const BASE_FONT_NAME: &str = "fonts/FiraSans-Bold.ttf";

fn make_window_plugin(settings: &user_settings::UserSettings) -> bevy::window::WindowPlugin {
    use bevy::window::*;

    let primary_window = Window {
        title: "Saffron Rhythm Duel".to_string(),
        resolution: (world().width(), world().height()).into(),
        window_theme: Some(WindowTheme::Dark),
        present_mode: PresentMode::AutoVsync,
        mode: settings.window_mode.into(),
        visible: false, // turn it on in a few frames to let the gpu catch up
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

    logging::configure_logging(&cli)?;

    let settings = user_settings::load_settings(&cli)?;

    log::info!("Initializing app...");


    let window_plugin = make_window_plugin(&settings);

    App::new()
        // Configure default plugins
        .add_plugins(DefaultPlugins
            .set(window_plugin)
            .disable::<bevy::log::LogPlugin>()
        )
        // Load custom plugins
        .add_plugins((
            song::ArrowsPlugin,
            judgement::JudgementPlugin,
            layout::UiPlugin,
            input::InputPlugin,
            widgets::WidgetsPlugin,
            selector_menu::ChartSelectorPlugin,
            remote::RemoteUserPlugin,
            record::RecordingPlugin,
        ))

        .config_if(cli.debug_inspector, |app| {
            app.add_plugins(WorldInspectorPlugin::new())
        })

        // Load resources
        .insert_resource(cli)
        .insert_resource(settings)
        .insert_resource(ClearColor(BACKGROUND_COLOR))

        // Systems
        .add_systems(Startup, setup)
        .add_systems(OnEnter(layout::LayoutState::Done), make_window_visible)
        .add_systems(Update, close_on_esc)
        .add_systems(Update, close_on_window_close_requested)
        .run();
    Ok(())
}

trait AppExt {
    fn app_mut(&mut self) -> &mut App;

    fn config_if<F>(&mut self, cond: bool, mut config: F) -> &mut App
    where F: FnMut(&mut App) -> &mut App {
        if cond {
            config(self.app_mut());
        }
        self.app_mut()
    }

}
impl AppExt for App {
    fn app_mut(&mut self) -> &mut App {
        self
    }
}

fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = bevy::render::camera::ScalingMode::Fixed {
        width: world().width(),
        height: world().height(), 
    };
    commands.spawn(cam);
}

fn make_window_visible(mut window: Query<&mut Window>) {
    window.single_mut().visible = true;
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
