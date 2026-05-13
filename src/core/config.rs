use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use camino::Utf8PathBuf;
use serde::Deserialize;

use crate::core::error::ConfigError;

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
        let path = Path::new(path.as_str());

        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_dir.join(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_dynamic_service_names() {
        let config: ProcessWatchConfig = toml::from_str(
            r#"
            [services.api]
            label = "API"
            command = ["cargo", "run", "-p", "api"]
            watch = ["crates/api", "crates/common"]
            port = 8080
            env = { RUST_LOG = "info" }

            [services.api.readiness]
            kind = "http"
            url = "http://localhost:8080/health"
            expected_status = 200

            [services.api.log_relay]
            enabled = true
            target = "process_watch"

            [workflows.check]
            label = "Check"
            command = ["cargo", "check", "--workspace"]

            [docs.rustdoc]
            label = "Rustdoc"
            path = "target/doc"
            workflow = "check"
            "#,
        )
        .unwrap();

        let service = config.services.get("api").unwrap();
        assert_eq!(service.label.as_deref(), Some("API"));
        assert_eq!(service.command[0], "cargo");
        assert_eq!(service.watch.len(), 2);
        assert!(matches!(
            service.readiness,
            Some(ReadinessCheck::Http { .. })
        ));
        assert!(config.workflows.contains_key("check"));
        assert!(config.docs.contains_key("rustdoc"));
    }

    #[test]
    fn resolves_relative_paths_from_config_parent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("process-watch.toml");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            r#"
        [services.api]
        command = ["cargo", "run"]
        watch = ["crates/api"]
        "#,
        )
        .unwrap();

        let loaded = LoadedConfig::new(Some(&path)).unwrap();
        let service = loaded.config.services.get("api").unwrap();
        let resolved = loaded.resolve_path(&service.watch[0]);

        assert_eq!(resolved, path.parent().unwrap().join("crates/api"));
    }

    #[test]
    fn resolve_path_leaves_absolute_paths_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("process-watch.toml");
        let absolute_watch = dir.path().join("src");
        let absolute_watch = camino::Utf8PathBuf::from_path_buf(absolute_watch).unwrap();

        std::fs::write(
            &path,
            format!(
                r#"
            [services.api]
            command = ["cargo", "run"]
            watch = ["{}"]
            "#,
                absolute_watch
            ),
        )
        .unwrap();

        let loaded = LoadedConfig::new(Some(&path)).unwrap();
        let service = loaded.config.services.get("api").unwrap();

        assert_eq!(
            loaded.resolve_path(&service.watch[0]),
            absolute_watch.as_std_path()
        );
    }
}
