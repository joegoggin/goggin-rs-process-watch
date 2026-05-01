use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "goggin-rs-console",
    about = "Run a configurable Rust development console"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the console using the discovered config or an explicit config path.
    Run {
        #[arg(
            short,
            long,
            value_name = "PATH",
            help = "Use this config file instead of the default discovery path"
        )]
        config: Option<PathBuf>,
    },
}

#[test]
fn parses_run_config_override() {
    use clap::Parser;

    let cli = Cli::try_parse_from(["goggin-rs-console", "run", "--config", "console.local.toml"])
        .unwrap();

    let Command::Run { config } = cli.command;
    assert_eq!(config.unwrap(), PathBuf::from("console.local.toml"));
}
