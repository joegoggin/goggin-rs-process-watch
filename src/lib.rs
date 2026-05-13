mod core;

pub use crate::core::{config, error};

use core::cli::{Cli, Command};
use core::error::AppResult;

use clap::Parser;

use crate::core::config::LoadedConfig;

pub fn run() -> AppResult {
    run_with(std::env::args_os())
}

pub fn run_with<I, T>(args: I) -> AppResult
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::try_parse_from(args).unwrap_or_else(|error| error.exit());
    dispatch(cli)
}

fn dispatch(cli: Cli) -> AppResult {
    match cli.command {
        Command::Run { config } => {
            let loaded = LoadedConfig::new(config.as_deref())?;
            loaded.config.validate(&loaded.base_dir)?;
            Ok(())
        }
    }
}
