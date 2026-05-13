//! Process-watch configuration loading and validation.
//!
//! This module defines the TOML schema used by `process-watch.toml`, loads
//! configs from disk, resolves config-relative paths, and accumulates
//! field-specific validation errors for user-facing diagnostics.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use camino::Utf8PathBuf;
use serde::Deserialize;

use anyhow::Context;

use crate::core::error::{AppResult, ValidationError, ValidationErrors, ValidationResult};

/// Default config file name used when no explicit path is provided.
pub const DEFAULT_CONFIG_FILE: &str = "process-watch.toml";

/// Root process-watch configuration loaded from TOML.
///
/// The top-level tables use dynamic names, so each map key is the service,
/// workflow, or documentation shortcut identifier from the config file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessWatchConfig {
    /// Long-running services managed by process watch.
    #[serde(default)]
    pub services: BTreeMap<String, ServiceConfig>,
    /// One-shot workflows that can run on demand or from watched paths.
    #[serde(default)]
    pub workflows: BTreeMap<String, WorkflowConfig>,
    /// Documentation and preview shortcuts.
    #[serde(default)]
    pub docs: BTreeMap<String, DocsConfig>,
}

impl ProcessWatchConfig {
    /// Validates the complete process-watch config.
    ///
    /// # Arguments
    ///
    /// * `base_dir` — Directory used to resolve relative watched paths.
    ///
    /// # Returns
    ///
    /// A [`ValidationResult`] indicating whether the config is valid.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationErrors`] if one or more config fields are invalid.
    pub fn validate(&self, base_dir: &Path) -> ValidationResult {
        let mut errors = Vec::new();

        if self.services.is_empty() {
            errors.push(ValidationError::new(
                "services",
                "at least one service is required",
            ));
        }

        for (name, service) in &self.services {
            service.validate(name, base_dir, &mut errors);
        }

        for (name, workflow) in &self.workflows {
            workflow.validate(name, base_dir, &mut errors);
        }

        for (name, docs) in &self.docs {
            docs.validate(name, &self.workflows, &mut errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors(errors))
        }
    }
}

/// Long-running process configuration.
///
/// Services are intended for processes such as API servers, frontend dev
/// servers, databases, and cache instances that should be started and monitored.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfig {
    /// Optional human-readable label shown in the terminal UI.
    pub label: Option<String>,
    /// Command and arguments used to start the service.
    pub command: Vec<String>,
    /// Files or directories that should trigger a service restart.
    #[serde(default)]
    pub watch: Vec<Utf8PathBuf>,
    /// Primary port exposed by the service.
    pub port: Option<u16>,
    /// Environment variables passed to the service process.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Optional readiness check used to decide when the service is available.
    pub readiness: Option<ReadinessCheck>,
    /// Optional log relay configuration for service output.
    pub log_relay: Option<LogRelayConfig>,
}

impl ServiceConfig {
    /// Adds validation errors for a service config.
    ///
    /// # Arguments
    ///
    /// * `name` — Service table name from the config.
    /// * `base_dir` — Directory used to resolve relative watched paths.
    /// * `errors` — Collection that receives validation failures.
    fn validate(&self, name: &str, base_dir: &Path, errors: &mut Vec<ValidationError>) {
        validate_command(&format!("services.{name}.command"), &self.command, errors);

        validate_watch_paths(
            &format!("services.{name}.watch"),
            base_dir,
            &self.watch,
            errors,
        );

        if self.port == Some(0) {
            errors.push(ValidationError::new(
                format!("services.{name}.port"),
                "service port must be greater than 0",
            ));
        }

        if let Some(readiness) = &self.readiness {
            readiness.validate(&format!("services.{name}.readiness"), errors);
        }

        if let Some(log_relay) = &self.log_relay {
            log_relay.validate(&format!("services.{name}.log_relay"), errors);
        }
    }
}

/// Readiness check used to decide whether a service is available.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReadinessCheck {
    /// Checks readiness by requesting an HTTP or HTTPS URL.
    Http {
        /// URL requested by the readiness check.
        url: String,
        /// Optional expected HTTP status code.
        #[serde(default)]
        expected_status: Option<u16>,
    },
    /// Checks readiness by opening a TCP connection.
    Tcp {
        /// Hostname or IP address used for the TCP connection.
        host: String,
        /// Port used for the TCP connection.
        port: u16,
    },
}

impl ReadinessCheck {
    /// Adds validation errors for a readiness check.
    ///
    /// # Arguments
    ///
    /// * `field` — Config field prefix used in validation diagnostics.
    /// * `errors` — Collection that receives validation failures.
    fn validate(&self, field: &str, errors: &mut Vec<ValidationError>) {
        match self {
            ReadinessCheck::Http {
                url,
                expected_status,
            } => {
                if !(url.starts_with("http://") || url.starts_with("https://")) {
                    errors.push(ValidationError::new(
                        format!("{field}.url"),
                        "HTTP readiness URL must start with http:// or https://",
                    ));
                }

                if expected_status.is_some_and(|status| !(100..=599).contains(&status)) {
                    errors.push(ValidationError::new(
                        format!("{field}.expected_status"),
                        "expected status must be between 100 and 599",
                    ));
                }
            }
            ReadinessCheck::Tcp { host, port } => {
                if host.trim().is_empty() {
                    errors.push(ValidationError::new(
                        format!("{field}.host"),
                        "TCP readiness host must not be empty",
                    ));
                }

                if *port == 0 {
                    errors.push(ValidationError::new(
                        format!("{field}.port"),
                        "TCP readiness port must be greater than 0",
                    ));
                }
            }
        }
    }
}

/// Log relay configuration for a service.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogRelayConfig {
    /// Whether relay forwarding is enabled.
    pub enabled: bool,
    /// Optional relay target name.
    pub target: Option<String>,
}

impl LogRelayConfig {
    /// Adds validation errors for a log relay config.
    ///
    /// # Arguments
    ///
    /// * `field` — Config field prefix used in validation diagnostics.
    /// * `errors` — Collection that receives validation failures.
    fn validate(&self, field: &str, errors: &mut Vec<ValidationError>) {
        if self.enabled
            && self
                .target
                .as_deref()
                .is_some_and(|target| target.trim().is_empty())
        {
            errors.push(ValidationError::new(
                format!("{field}.target"),
                "target must not be empty when log relay is enabled",
            ));
        }
    }
}

/// One-shot workflow configuration.
///
/// Workflows use the same command, watch, and environment concepts as services
/// but are intended for commands such as checks, tests, and documentation
/// builds.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowConfig {
    /// Optional human-readable label shown in the terminal UI.
    pub label: Option<String>,
    /// Command and arguments used to run the workflow.
    pub command: Vec<String>,
    /// Files or directories that should trigger the workflow.
    #[serde(default)]
    pub watch: Vec<Utf8PathBuf>,
    /// Environment variables passed to the workflow process.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

impl WorkflowConfig {
    /// Adds validation errors for a workflow config.
    ///
    /// # Arguments
    ///
    /// * `name` — Workflow table name from the config.
    /// * `base_dir` — Directory used to resolve relative watched paths.
    /// * `errors` — Collection that receives validation failures.
    fn validate(&self, name: &str, base_dir: &Path, errors: &mut Vec<ValidationError>) {
        validate_command(&format!("workflows.{name}.command"), &self.command, errors);

        validate_watch_paths(
            &format!("workflows.{name}.watch"),
            base_dir,
            &self.watch,
            errors,
        );
    }
}

/// Documentation or preview shortcut configuration.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocsConfig {
    /// Optional human-readable label shown in the terminal UI.
    pub label: Option<String>,
    /// Optional local documentation path.
    pub path: Option<Utf8PathBuf>,
    /// Optional documentation or preview URL.
    pub url: Option<String>,
    /// Optional workflow name that builds or refreshes this docs target.
    pub workflow: Option<String>,
}

impl DocsConfig {
    /// Adds validation errors for a docs shortcut config.
    ///
    /// # Arguments
    ///
    /// * `name` — Docs table name from the config.
    /// * `workflows` — Workflow definitions available for reference checks.
    /// * `errors` — Collection that receives validation failures.
    fn validate(
        &self,
        name: &str,
        workflows: &BTreeMap<String, WorkflowConfig>,
        errors: &mut Vec<ValidationError>,
    ) {
        if self.path.is_none() && self.url.is_none() {
            errors.push(ValidationError::new(
                format!("docs.{name}"),
                "docs entry must define either path or url",
            ));
        }

        if self.path.is_some() && self.url.is_some() {
            errors.push(ValidationError::new(
                format!("docs.{name}"),
                "docs entry must define only one of path or url",
            ));
        }

        if self
            .path
            .as_ref()
            .is_some_and(|path| path.as_str().trim().is_empty())
        {
            errors.push(ValidationError::new(
                format!("docs.{name}.path"),
                "docs path must not be empty",
            ));
        }

        if self.url.as_deref().is_some_and(|url| url.trim().is_empty()) {
            errors.push(ValidationError::new(
                format!("docs.{name}.url"),
                "docs URL must not be empty",
            ));
        }

        if let Some(workflow) = &self.workflow
            && !workflows.contains_key(workflow)
        {
            errors.push(ValidationError::new(
                format!("docs.{name}.workflow"),
                format!("unknown workflow reference: {workflow}"),
            ));
        }
    }
}

/// Loaded process-watch config and its path context.
#[derive(Debug)]
pub struct LoadedConfig {
    /// Config file path that was loaded.
    pub path: PathBuf,
    /// Base directory used to resolve config-relative paths.
    pub base_dir: PathBuf,
    /// Parsed process-watch config.
    pub config: ProcessWatchConfig,
}

impl LoadedConfig {
    /// Creates a loaded config from an optional path override.
    ///
    /// # Arguments
    ///
    /// * `path_override` — Optional config path used instead of
    ///   [`DEFAULT_CONFIG_FILE`].
    ///
    /// # Returns
    ///
    /// A [`LoadedConfig`] with parsed TOML and path context.
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if the config cannot be read or parsed.
    pub fn new(path_override: Option<&Path>) -> AppResult<LoadedConfig> {
        let path = path_override
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE));

        let source = std::fs::read_to_string(&path)
            .with_context(|| format!("could not read config at {}", path.display()))?;

        let config = toml::from_str::<ProcessWatchConfig>(&source)
            .with_context(|| format!("could not parse config at {}", path.display()))?;

        let base_dir = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        Ok(LoadedConfig {
            path,
            base_dir,
            config,
        })
    }

    /// Resolves a path relative to the loaded config file.
    ///
    /// Absolute paths are returned unchanged. Relative paths are resolved from
    /// the directory containing the loaded config file.
    ///
    /// # Arguments
    ///
    /// * `path` — Path from the loaded config.
    ///
    /// # Returns
    ///
    /// A [`PathBuf`] containing the resolved filesystem path.
    pub fn resolve_path(&self, path: &camino::Utf8Path) -> PathBuf {
        resolve_config_path(&self.base_dir, path)
    }
}

/// Adds validation errors for a command argument list.
///
/// # Arguments
///
/// * `field` — Config field prefix used in validation diagnostics.
/// * `command` — Command and argument values from the config.
/// * `errors` — Collection that receives validation failures.
fn validate_command(field: &str, command: &[String], errors: &mut Vec<ValidationError>) {
    if command.is_empty() {
        errors.push(ValidationError::new(
            field,
            "command must include at least one argument",
        ));
    }

    for (index, arg) in command.iter().enumerate() {
        if arg.trim().is_empty() {
            errors.push(ValidationError::new(
                format!("{field}[{index}]"),
                "command arguments must not be empty",
            ));
        }
    }
}

/// Adds validation errors for watch paths.
///
/// # Arguments
///
/// * `field` — Config field prefix used in validation diagnostics.
/// * `base_dir` — Directory used when a watch path is relative.
/// * `paths` — Watch path values from the config.
/// * `errors` — Collection that receives validation failures.
fn validate_watch_paths(
    field: &str,
    base_dir: &Path,
    paths: &[Utf8PathBuf],
    errors: &mut Vec<ValidationError>,
) {
    for (index, path) in paths.iter().enumerate() {
        let field = format!("{field}[{index}]");

        if path.as_str().trim().is_empty() {
            errors.push(ValidationError::new(field, "watch path must not be empty"));
        } else if !resolve_config_path(base_dir, path).exists() {
            errors.push(ValidationError::new(
                field,
                format!("path does not exist: {path}"),
            ));
        }
    }
}

/// Resolves a config path against a base directory.
///
/// # Arguments
///
/// * `base_dir` — Directory used when `path` is relative.
/// * `path` — Absolute or relative config path.
///
/// # Returns
///
/// A [`PathBuf`] containing the resolved filesystem path.
fn resolve_config_path(base_dir: &Path, path: &camino::Utf8Path) -> PathBuf {
    let path = Path::new(path.as_str());

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}
