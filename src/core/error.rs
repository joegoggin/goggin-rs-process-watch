#[derive(Debug, thiserror::Error)]
pub enum ProcessWatchError {
    #[error(transparent)]
    Cli(#[from] clap::Error),
}

impl ProcessWatchError {
    pub fn exit(self) -> ! {
        match self {
            Self::Cli(error) => error.exit(),
        }
    }
}
