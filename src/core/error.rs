//! Shared application error types.
//!
//! This module defines the crate-wide result alias and validation error
//! structures used when config validation finds one or more field-specific
//! problems.

use std::{fmt::Display, ops::Deref};

/// Application result type used by command entry points.
pub type AppResult<T = ()> = anyhow::Result<T>;

/// Validation error for a single config field.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{field}: {message}")]
pub struct ValidationError {
    /// Field path that failed validation.
    pub field: String,
    /// Human-readable validation failure message.
    pub message: String,
}

impl ValidationError {
    /// Creates a validation error.
    ///
    /// # Arguments
    ///
    /// * `field` — Field path that failed validation.
    /// * `message` — Human-readable validation failure message.
    ///
    /// # Returns
    ///
    /// A [`ValidationError`] containing the provided field and message.
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> ValidationError {
        ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Validation result returned by config validation routines.
pub type ValidationResult = Result<(), ValidationErrors>;

/// Collection of config validation errors.
#[derive(Debug, thiserror::Error)]
pub struct ValidationErrors(pub Vec<ValidationError>);

impl Deref for ValidationErrors {
    /// Slice target used when reading validation errors through deref coercion.
    type Target = [ValidationError];

    /// Returns the validation errors as a slice.
    ///
    /// # Returns
    ///
    /// A [`Self::Target`] slice of validation errors.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ValidationErrors {
    /// Formats validation errors for user-facing diagnostics.
    ///
    /// # Arguments
    ///
    /// * `formatter` — Formatter receiving the diagnostic output.
    ///
    /// # Returns
    ///
    /// A [`std::fmt::Result`] indicating whether formatting succeeded.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(formatter, "invalid config:")?;

        for error in &self.0 {
            writeln!(formatter, "{error}")?;
        }

        Ok(())
    }
}
