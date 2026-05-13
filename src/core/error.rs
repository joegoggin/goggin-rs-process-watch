use std::{fmt::Display, ops::Deref};

pub type AppResult<T = ()> = anyhow::Result<T>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{field}: {message}")]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> ValidationError {
        ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
}

pub type ValidationResult = Result<(), ValidationErrors>;

#[derive(Debug, thiserror::Error)]
pub struct ValidationErrors(pub Vec<ValidationError>);

impl Deref for ValidationErrors {
    type Target = [ValidationError];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ValidationErrors {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(formatter, "invalid config:")?;

        for error in &self.0 {
            writeln!(formatter, "{error}")?;
        }

        Ok(())
    }
}
