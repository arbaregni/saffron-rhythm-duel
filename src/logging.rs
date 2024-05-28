use std::path::{
    Path,
    PathBuf
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
    let stdout_log = tracing_subscriber::fmt::layer()
        .compact()
        .with_level(true)
        .with_thread_names(true)
        .with_file(true);

    let project_name = "saffron_rhythm_duel";

    let default_level = cli.log_level;
    let mut targets = Targets::new()
        .with_target(project_name, default_level);

    if let Some(log_filters) = &cli.log_filters {
        // if we specify something, that means by default it needs to be off
        targets = targets.with_default(LevelFilter::OFF);

        for filter in log_filters {

            let module = filter.module.as_str();
            let level = filter.level.unwrap_or(default_level);

            let t = format!("{project_name}::{module}");
            targets = targets.with_target(t, level);
        }

    };

    // TODO: log more things to a file

    tracing_subscriber::registry()
        .with(
            stdout_log
                .with_filter(targets)
        )
        .init();
    
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

