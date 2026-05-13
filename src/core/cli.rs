//! Command-line interface definitions.
//!
//! This module defines the clap parser used by the binary and library entry
//! points. It keeps CLI shape separate from command execution so parsing can be
//! tested directly.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Parsed command-line interface for process watch.
#[derive(Debug, Parser)]
#[command(
    name = "goggin-rs-process-watch",
    about = "Watch and manage configured development processes"
)]
pub struct Cli {
    /// Command selected by the user.
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level command variants supported by process watch.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start process watching using the discovered config or an explicit config path.
    Run {
        /// Optional config file path used instead of the default discovery path.
        #[arg(
            short,
            long,
            value_name = "PATH",
            help = "Use this config file instead of the default discovery path"
        )]
        config: Option<PathBuf>,
    },
}
