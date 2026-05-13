use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ProcessWatchError {
    #[error(transparent)]
    Cli(#[from] clap::Error),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Validation(#[from] ValidationErrors),
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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{field}: {message}")]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub type ValidationResult = Result<(), Vec<ValidationError>>;

pub(crate) fn validation_error(
    field: impl Into<String>,
    message: impl Into<String>,
) -> ValidationError {
    ValidationError {
        field: field.into(),
        message: message.into(),
    }
}

#[derive(Debug)]
pub struct ValidationErrors(pub Vec<ValidationError>);

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(formatter, "invalid config:")?;

        for error in &self.0 {
            writeln!(formatter, "{error}")?;
        }

        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

impl From<Vec<ValidationError>> for ProcessWatchError {
    fn from(errors: Vec<ValidationError>) -> ProcessWatchError {
        ProcessWatchError::Validation(ValidationErrors(errors))
    }
}
