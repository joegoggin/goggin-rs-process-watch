//! Core application modules.
//!
//! This module groups the command-line parser, config schema, and shared error
//! types used by the library entry point.
//!
//! # Modules
//!
//! - [`cli`] — Clap parser definitions for the terminal command.
//! - [`config`] — TOML config schema, loading, and validation.
//! - [`error`] — Application result and validation error types.

pub mod cli;
pub mod config;
pub mod error;
