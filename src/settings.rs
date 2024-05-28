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
    lane::Lane,
    CliArgs,
    project_dirs,
};

#[derive(Debug, Serialize, Deserialize)]
#[derive(Resource)]
/// For user defined settings
pub struct Config {
    /// The key bindings for common keys
    pub keybindings: KeyBindings,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host_addr")]
    pub host_addr: IpAddr,
}

fn default_port() -> u16 {
    8080
}
fn default_host_addr() -> IpAddr {
    IpAddr::from([0,0,0,0])
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            port: default_port(),
            host_addr: default_host_addr(),
            ..default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct KeyBindings {
    pub lane_hit_L1: String,
    pub lane_hit_L2: String,
    pub lane_hit_R1: String,
    pub lane_hit_R2: String,
}
impl KeyBindings {
    pub fn key_name(&self, lane: Lane) -> &str {
        match lane {
            Lane::L1 => self.lane_hit_L1.as_str(),
            Lane::L2 => self.lane_hit_L2.as_str(),
            Lane::R1 => self.lane_hit_R1.as_str(),
            Lane::R2 => self.lane_hit_R2.as_str(),
        }
    }
}
// TODO: use the default somehow
impl std::default::Default for KeyBindings {
    fn default() -> Self {
        Self {
            lane_hit_L1: "a".to_string(),
            lane_hit_L2: "s".to_string(),
            lane_hit_R1: "d".to_string(),
            lane_hit_R2: "f".to_string(),
        }
    }
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

pub fn load_settings(cli: &CliArgs) -> Result<Config> {
    let path = settings_path(cli);
    let display_path = path.display();

    let config = if path.exists() {
        log::info!("Reading settings from {display_path}...");
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("reading settings file at {display_path}"))?;


        let config = toml::from_str(contents.as_ref())?;

        config
    } else {
        log::info!("settings file does not exist at {display_path}, using defaults");
        Config::default()
    };

    log::debug!("loaded settings: {config:?}");

    // we write them back in case we picked up any defaults or fields were missing
    store_settings(cli, &config)?;

    Ok(config)
}

pub fn store_settings(cli: &CliArgs, config: &Config) -> Result<()> {
    let path = settings_path(cli);

    let parent = path.parent().unwrap_or(path.as_path());

    fs::create_dir_all(parent)
        .with_context(|| format!("storing settings to {}", parent.display()))?;

    let contents = toml::to_string(config)?;
    fs::write(&path, contents.as_str())
        .with_context(|| format!("writing to {}", path.display()))?; 
    Ok(())
}
