use std::{
    fs,
    path::{Path, PathBuf},
    net::IpAddr,
};

use anyhow::{
    Result,
    Context,
};
use bevy::prelude::*;
use serde::{
    Deserialize,Serialize
};

use crate::{
    CliArgs,
    project_dirs,
};

#[derive(Debug, Serialize, Deserialize)]
#[derive(Resource)]
/// For user defined settings
pub struct UserSettings {
    /// The key bindings for common keys
    pub keybindings: KeyBindings,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host_addr")]
    pub host_addr: IpAddr,
    #[serde(default = "default_window_mode")]
    pub window_mode: WindowMode,
    #[serde(default = "default_latency_tolerance")]
    pub latency_tolerance: f32
}

/// Default latency in milli seconds
fn default_latency_tolerance() -> f32 {
    1000.0
}

fn default_port() -> u16 {
    8080
}
fn default_host_addr() -> IpAddr {
    IpAddr::from([0,0,0,0])
}

fn default_window_mode() -> WindowMode {
    WindowMode::Borderless
}

#[derive(Debug,Clone,Copy,Eq,PartialEq,Serialize,Deserialize)]
pub enum WindowMode {
    Borderless,
    Windowed
}
impl std::convert::Into<bevy::window::WindowMode> for WindowMode {
    fn into(self: Self) -> bevy::window::WindowMode {
        match self {
            WindowMode::Borderless => bevy::window::WindowMode::BorderlessFullscreen,
            WindowMode::Windowed => bevy::window::WindowMode::Windowed,
        }
    }
}

impl std::default::Default for UserSettings {
    fn default() -> Self {
        Self {
            latency_tolerance: default_latency_tolerance(),
            window_mode: default_window_mode(),
            host_addr: default_host_addr(),
            port: default_port(),
            keybindings: KeyBindings::default(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct KeyBindings {
    #[serde(default)]
    pub lane_hit_keymap: crate::input::LaneHitControls,
    #[serde(default)]
    pub recording_keymap: crate::record::controls::RecordingKeymap,
}

const SETTINGS_FILENAME: &str = "settings.toml";
fn settings_path(cli: &CliArgs) -> PathBuf {
    // first try the cli arguments
    cli.settings.clone()
        // if that doesn't work, then check the project directory
        .or_else(|| {
            project_dirs()
                .map(|p| p.config_dir().to_path_buf())
        })
        // and if that fails, then we just default to the current working directory
        .unwrap_or(Path::new(".").to_path_buf())
        // and then we join it with the settings file
        .join(SETTINGS_FILENAME)
}

pub fn load_settings(cli: &CliArgs) -> Result<UserSettings> {
    let path = settings_path(cli);
    let display_path = path.display();

    let settings = if !path.exists() {
        log::info!("settings file does not exist at {display_path}, using defaults");
        UserSettings::default()
    } else if cli.reset_to_default_settings {
        log::info!("cli argument reset-to-default-settings was passed, using the defaults");
        UserSettings::default()
    } else {
        log::info!("Reading settings from {display_path}...");
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("reading settings file at {display_path}"))?;

        let settings = toml::from_str(contents.as_ref())
            .with_context(|| format!("deserializing settings.toml file at {display_path}"))?;

        settings
    };

    log::debug!("loaded settings: {settings:?}");

    log::info!("storing settings");
    // we write them back in case we picked up any defaults or fields were missing
    store_settings(cli, &settings)?;

    Ok(settings)
}

pub fn store_settings(cli: &CliArgs, settings: &UserSettings) -> Result<()> {
    let path = settings_path(cli);

    let parent = path.parent().unwrap_or(path.as_path());

    fs::create_dir_all(parent)
        .with_context(|| format!("storing settings to {}", parent.display()))?;

    log::debug!("about to deserialize contents");

    let contents = toml::to_string(settings)
        .with_context(|| format!("serializing current settings"))?;

    fs::write(&path, contents.as_str())
        .with_context(|| format!("writing settings to {}", path.display()))?; 

    Ok(())
}
