mod core;

use core::cli::{Cli, Command};
use core::error::ProcessWatchError;

use clap::Parser;

use crate::core::config::LoadedConfig;

pub fn run() -> Result<(), ProcessWatchError> {
    run_with(std::env::args_os())
}

pub fn run_with<I, T>(args: I) -> Result<(), ProcessWatchError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::try_parse_from(args)?;
    dispatch(cli)
}

fn dispatch(cli: Cli) -> Result<(), ProcessWatchError> {
    match cli.command {
        Command::Run { config } => {
            let _config = LoadedConfig::new(config.as_deref())?;
            Ok(())
        }
    }
}
