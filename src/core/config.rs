use std::collections::BTreeMap;

use camino::Utf8PathBuf;
use serde::Deserialize;

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
}
