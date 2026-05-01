mod core;

use core::cli::{Cli, Command};
use core::error::ConsoleError;

use clap::Parser;

pub fn run() -> Result<(), ConsoleError> {
    run_with(std::env::args_os())
}

pub fn run_with<I, T>(args: I) -> Result<(), ConsoleError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::try_parse_from(args)?;
    dispatch(cli)
}

fn dispatch(cli: Cli) -> Result<(), ConsoleError> {
    match cli.command {
        Command::Run { config } => {
            // Issue #4 will load and validate this path before starting the UI.
            let _config_override = config;
            Ok(())
        }
    }
}
