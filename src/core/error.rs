#[derive(Debug, thiserror::Error)]
pub enum ConsoleError {
    #[error(transparent)]
    Cli(#[from] clap::Error),
}

impl ConsoleError {
    pub fn exit(self) -> ! {
        match self {
            Self::Cli(error) => error.exit(),
        }
    }
}
