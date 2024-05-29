use std::{
    path::{
        Path,
        PathBuf
    },
    fs::{
        self,
        File
    },
};

use anyhow::{
    Result,
    Context
};

use bevy::log::tracing_subscriber::{
    self,
    filter::{
        Targets,
    },
    prelude::*,
};

use crate::{
    CliArgs
};

pub type LevelFilter = tracing_subscriber::filter::LevelFilter;

#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub struct LogFilter {
    /// A name of a source file module
    module: String,
    /// A `LevelFilter`, i.e. OFF, DEBUG, INFO, WARN, or ERROR
    level: Option<LevelFilter>,
}
pub fn parse_log_level(source: &str) -> Result<LevelFilter> {
    let level = match source.trim().to_uppercase().as_str() {
        "OFF" => LevelFilter::OFF,
        "DEBUG" => LevelFilter::DEBUG,
        "INFO" => LevelFilter::INFO,
        "WARN" => LevelFilter::WARN,
        "ERROR" => LevelFilter::ERROR,
        _ => anyhow::bail!("not a valid level filter: {source}"),
    };
    Ok(level)
}
pub fn parse_log_filter(source: &str) -> Result<LogFilter> {
    if let Some((module, level)) = source.split_once('=') {
        let level = parse_log_level(level)
            .with_context(|| format!("while parsing log filter"))?;
        Ok(LogFilter {
            module: module.trim().to_string(),
            level: Some(level)
        })
    } else {
        Ok(LogFilter {
            module: source.trim().to_string(),
            level: None
        })
    }
}

pub fn configure_logging(cli: &CliArgs) -> Result<()> {
    let project_name = "saffron_rhythm_duel";

    let default_level = cli.log_level;

    let stdout_log = {

        let targets = match &cli.log_filters {
            None => Targets::new().with_target(project_name, default_level),
            Some(log_filters) => {

                let mut targets = Targets::new()
                                    .with_default(LevelFilter::OFF);

                for filter in log_filters {

                    let module = filter.module.as_str();
                    let level = filter.level.unwrap_or(default_level);

                    let t = format!("{project_name}::{module}");
                    targets = targets.with_target(t, level);
                }

                targets
            }
        };

        tracing_subscriber::fmt::layer()
            .compact()
            .with_level(true)
            .with_thread_names(true)
            .with_file(true)
            .with_filter(targets)
    };


    let log_filepath;

    let debug_log = {
        // logging to a rolling file
        let (log_file, path) = rolling_log_file(cli)?;
        log_filepath = path;

        let targets = Targets::new().with_target(project_name, LevelFilter::DEBUG);

        tracing_subscriber::fmt::layer()
            .compact()
            .with_ansi(false)
            .with_level(true)
            .with_thread_names(true)
            .with_file(true)
            .with_writer(std::sync::Arc::new(log_file))
            .with_filter(targets)
    };


    tracing_subscriber::registry()
        .with(stdout_log)
        .with(debug_log)
        .init();
    
    log::info!("logging to {}", log_filepath.display());

    Ok(())
}


const MAX_LOG_RUNS_SAVED: usize = 10;
const LOG_FOLDER_NAME: &str = "logs";

fn log_folder_path(cli: &CliArgs) -> PathBuf {
    // first try the cli arguments
    cli.settings.clone()
        // if that doesn't work, then check the project directory
        .or_else(|| {
            crate::project_dirs()
                .map(|p| p.cache_dir().to_path_buf())
        })
        // and if that fails, then we just default to the current working directory
        .unwrap_or(Path::new(".").to_path_buf())
        // and then we join it with the settings file
        .join("logs")
}

fn rolling_log_file(cli: &CliArgs) -> Result<(File, PathBuf)> {
    let log_folder = log_folder_path(cli);

    if !log_folder.exists() {
        // make sure it exists
        fs::create_dir_all(&log_folder)
            .with_context(|| format!("creating log folder at {}", log_folder.display()))?;
    }

    // clear out space for the new file

    let mut read_dir = fs::read_dir(&log_folder)
        .with_context(|| format!("reading log folder at {}", log_folder.display()))?
        .collect::<Result<Vec<_>, _>>()
            .with_context(|| format!("reading log folder at {}", log_folder.display()))?;

    read_dir.sort_by_key(|entry| {
        entry.path()
             .file_stem()
             .and_then(|s| s.to_str())
             .unwrap_or("")
             .to_string()
    });
    read_dir.reverse();

    // include the last item because we need to make room for 1 more entry
    let to_be_deleted = (MAX_LOG_RUNS_SAVED - 1)..read_dir.len();

    for i in to_be_deleted {
        let path = read_dir[i].path();
        fs::remove_file(&path)
            .with_context(|| format!("attempting to remove old log file"))?;
    }


    // create the new file

    let now = chrono::Utc::now();
    let filename = format!("game-run-{}.log", now.format("%Y-%m-%d-%H-%M-%S"));
    let filepath = log_folder.join(filename);

    let file = File::create(&filepath)
        .with_context(|| format!("creating new log in {}", log_folder.display()))?;

    Ok((file, filepath))
}

