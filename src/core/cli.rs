use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "goggin-rs-process-watch",
    about = "Watch and manage configured development processes"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start process watching using the discovered config or an explicit config path.
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

    let cli = Cli::try_parse_from([
        "goggin-rs-process-watch",
        "run",
        "--config",
        "process-watch.local.toml",
    ])
    .unwrap();

    let Command::Run { config } = cli.command;
    assert_eq!(config.unwrap(), PathBuf::from("process-watch.local.toml"));
}
