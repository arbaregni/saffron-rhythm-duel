use anyhow::Result;

use bevy::log::tracing_subscriber::{
    self,
    filter::{
        Targets,
        LevelFilter,
    },
    prelude::*,
};

use crate::{
    CliArgs
};

pub const LOG_FILTER_HELP_MESSAGE: &'static str = "Specify one or more log filters, separated by commas.\n\
                                                   Log filters are a rust module path (excluding the project name), e.g. `judgement::metrics'.\n\
                                                   Optionally, you may include an equal sign and one of OFF, DEBUG, INFO, WARN, or ERROR to specify the level of logging to filter out.\n\
                                                     For example, `judgement::metrics=OFF`.";

#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub struct LogFilter {
    /// A name of a source file module
    module: String,
    /// A `LevelFilter`, i.e. OFF, DEBUG, INFO, WARN, or ERROR
    level: Option<LevelFilter>,
}
pub fn parse_log_filter(source: &str) -> Result<LogFilter> {
    if let Some((module, level)) = source.split_once('=') {
        let level = match level.trim().to_uppercase().as_str() {
            "OFF" => LevelFilter::OFF,
            "DEBUG" => LevelFilter::DEBUG,
            "INFO" => LevelFilter::INFO,
            "WARN" => LevelFilter::WARN,
            "ERROR" => LevelFilter::ERROR,
            _ => anyhow::bail!("not a valid level filter: {level}"),
        };

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

    // specify the default level to INFO for everything
    let default_level = LevelFilter::INFO;
    let mut targets = Targets::new().with_default(default_level);

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

    tracing_subscriber::registry()
        .with(
            stdout_log
                .with_filter(targets)
        )
        .init();
    
    Ok(())
}

