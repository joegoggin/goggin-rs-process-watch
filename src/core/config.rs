use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use camino::Utf8PathBuf;
use serde::Deserialize;

use crate::core::error::{ConfigError, ValidationError, ValidationResult, validation_error};

pub const DEFAULT_CONFIG_FILE: &str = "process-watch.toml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessWatchConfig {
    pub services: BTreeMap<String, ServiceConfig>,
    #[serde(default)]
    pub workflows: BTreeMap<String, WorkflowConfig>,
    #[serde(default)]
    pub docs: BTreeMap<String, DocsConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfig {
    pub label: Option<String>,
    pub command: Vec<String>,
    #[serde(default)]
    pub watch: Vec<Utf8PathBuf>,
    pub port: Option<u16>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub readiness: Option<ReadinessCheck>,
    pub log_relay: Option<LogRelayConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReadinessCheck {
    Http {
        url: String,
        #[serde(default)]
        expected_status: Option<u16>,
    },
    Tcp {
        host: String,
        port: u16,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogRelayConfig {
    pub enabled: bool,
    pub target: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowConfig {
    pub label: Option<String>,
    pub command: Vec<String>,
    #[serde(default)]
    pub watch: Vec<Utf8PathBuf>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocsConfig {
    pub label: Option<String>,
    pub path: Option<Utf8PathBuf>,
    pub url: Option<String>,
    pub workflow: Option<String>,
}

impl ProcessWatchConfig {
    pub fn validate(&self, base_dir: &Path) -> ValidationResult {
        let mut errors = Vec::new();

        if self.services.is_empty() {
            errors.push(validation_error(
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
            Err(errors)
        }
    }
}

impl ServiceConfig {
    fn validate(&self, name: &str, base_dir: &Path, errors: &mut Vec<ValidationError>) {
        validate_command(&format!("services.{name}.command"), &self.command, errors);

        for (index, path) in self.watch.iter().enumerate() {
            if !resolve_config_path(base_dir, path).exists() {
                errors.push(validation_error(
                    format!("services.{name}.watch[{index}]"),
                    format!("path does not exist: {path}"),
                ));
            }
        }

        if let Some(readiness) = &self.readiness {
            readiness.validate(&format!("services.{name}.readiness"), errors);
        }

        if let Some(log_relay) = &self.log_relay {
            log_relay.validate(&format!("services.{name}.log_relay"), errors);
        }
    }
}

impl ReadinessCheck {
    fn validate(&self, field: &str, errors: &mut Vec<ValidationError>) {
        match self {
            ReadinessCheck::Http {
                url,
                expected_status,
            } => {
                if !(url.starts_with("http://") || url.starts_with("https://")) {
                    errors.push(validation_error(
                        format!("{field}.url"),
                        "HTTP readiness URL must start with http:// or https://",
                    ));
                }

                if expected_status.is_some_and(|status| !(100..=599).contains(&status)) {
                    errors.push(validation_error(
                        format!("{field}.expected_status"),
                        "expected status must be between 100 and 599",
                    ));
                }
            }
            ReadinessCheck::Tcp { host, port } => {
                if host.trim().is_empty() {
                    errors.push(validation_error(
                        format!("{field}.host"),
                        "TCP readiness host must not be empty",
                    ));
                }

                if *port == 0 {
                    errors.push(validation_error(
                        format!("{field}.port"),
                        "TCP readiness port must be greater than 0",
                    ));
                }
            }
        }
    }
}

impl LogRelayConfig {
    fn validate(&self, field: &str, errors: &mut Vec<ValidationError>) {
        if self.enabled
            && self
                .target
                .as_deref()
                .is_some_and(|target| target.trim().is_empty())
        {
            errors.push(validation_error(
                format!("{field}.target"),
                "target must not be empty when log relay is enabled",
            ));
        }
    }
}

impl WorkflowConfig {
    fn validate(&self, name: &str, base_dir: &Path, errors: &mut Vec<ValidationError>) {
        validate_command(&format!("workflows.{name}.command"), &self.command, errors);

        for (index, path) in self.watch.iter().enumerate() {
            if !resolve_config_path(base_dir, path).exists() {
                errors.push(validation_error(
                    format!("workflows.{name}.watch[{index}]"),
                    format!("path does not exist: {path}"),
                ));
            }
        }
    }
}

impl DocsConfig {
    fn validate(
        &self,
        name: &str,
        workflows: &BTreeMap<String, WorkflowConfig>,
        errors: &mut Vec<ValidationError>,
    ) {
        if self.path.is_none() && self.url.is_none() {
            errors.push(validation_error(
                format!("docs.{name}"),
                "docs entry must define either path or url",
            ));
        }

        if let Some(workflow) = &self.workflow
            && !workflows.contains_key(workflow)
        {
            errors.push(validation_error(
                format!("docs.{name}.workflow"),
                format!("unknown workflow reference: {workflow}"),
            ));
        }
    }
}

fn validate_command(field: &str, command: &[String], errors: &mut Vec<ValidationError>) {
    if command.is_empty() {
        errors.push(validation_error(
            field,
            "command must include at least one argument",
        ));
    }

    for (index, arg) in command.iter().enumerate() {
        if arg.trim().is_empty() {
            errors.push(validation_error(
                format!("{field}[{index}]"),
                "command arguments must not be empty",
            ));
        }
    }
}

fn resolve_config_path(base_dir: &Path, path: &camino::Utf8Path) -> PathBuf {
    let path = Path::new(path.as_str());

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

#[derive(Debug)]
pub struct LoadedConfig {
    pub path: PathBuf,
    pub base_dir: PathBuf,
    pub config: ProcessWatchConfig,
}

impl LoadedConfig {
    pub fn new(path_override: Option<&Path>) -> Result<LoadedConfig, ConfigError> {
        let path = path_override
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE));

        let source = std::fs::read_to_string(&path).map_err(|source| ConfigError::Read {
            path: path.clone(),
            source,
        })?;

        let config =
            toml::from_str::<ProcessWatchConfig>(&source).map_err(|source| ConfigError::Parse {
                path: path.clone(),
                source,
            })?;

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

    pub fn resolve_path(&self, path: &camino::Utf8Path) -> PathBuf {
        resolve_config_path(&self.base_dir, path)
    }
}
