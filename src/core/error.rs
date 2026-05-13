use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ProcessWatchError {
    #[error(transparent)]
    Cli(#[from] clap::Error),
    #[error(transparent)]
    Config(#[from] ConfigError),
}

impl ProcessWatchError {
    pub fn exit(self) -> ! {
        match self {
            Self::Cli(error) => error.exit(),
            error => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("could not read config at {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not parse config at {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}
