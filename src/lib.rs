//! Process watch library entry point.
//!
//! This crate exposes the command-line runner and the configuration and error
//! modules used by the `goggin-rs-process-watch` binary. The current runtime
//! loads a process-watch TOML config, validates it, and reports user-facing
//! errors through the binary.
//!
//! # Modules
//!
//! - [`config`] — Process-watch configuration structures and validation.
//! - [`error`] — Shared application and validation error types.

mod core;

pub use crate::core::{config, error};

use core::cli::{Cli, Command};
use core::error::AppResult;

use clap::Parser;

use crate::core::config::LoadedConfig;

/// Runs the application with arguments from the current process environment.
///
/// # Returns
///
/// An empty [`AppResult`] on success.
///
/// # Errors
///
/// Returns [`anyhow::Error`] if config loading or validation fails.
pub fn run() -> AppResult {
    run_with(std::env::args_os())
}

/// Runs the application with an explicit argument iterator.
///
/// This is the testable entry point used by the binary-level runner. Clap parse
/// errors exit through clap's standard error path before command dispatch.
///
/// # Arguments
///
/// * `args` — Argument values to parse as command-line input.
///
/// # Returns
///
/// An empty [`AppResult`] on success.
///
/// # Errors
///
/// Returns [`anyhow::Error`] if config loading or validation fails.
pub fn run_with<I, T>(args: I) -> AppResult
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::try_parse_from(args).unwrap_or_else(|error| error.exit());
    dispatch(cli)
}

/// Dispatches a parsed command to its implementation.
///
/// # Arguments
///
/// * `cli` — Parsed command-line arguments.
///
/// # Returns
///
/// An empty [`AppResult`] on success.
///
/// # Errors
///
/// Returns [`anyhow::Error`] if the selected command cannot load or validate
/// its config.
fn dispatch(cli: Cli) -> AppResult {
    match cli.command {
        Command::Run { config } => {
            let loaded = LoadedConfig::new(config.as_deref())?;
            loaded.config.validate(&loaded.base_dir)?;
            Ok(())
        }
    }
}
