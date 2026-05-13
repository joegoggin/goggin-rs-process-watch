//! Contract tests for the checked-in API/web/common sample config.
//!
//! The tests keep `examples/api-web-common.toml` useful as documentation by
//! loading the real file and asserting that it demonstrates the expected
//! project shapes: API, web, database, and cache services; docs and frontend
//! workflows; and a Rustdoc shortcut wired to the docs workflow.

use std::path::PathBuf;

use assert_cmd::Command;
use toml::Value;

/// Returns the path to the real `examples/api-web-common.toml` fixture.
///
/// The example under test is the checked-in TOML file rather than a copied
/// inline string, so this test fails if the public sample drifts from the
/// supported config schema.
fn sample_config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/api-web-common.toml")
}

/// Creates placeholder paths referenced by `examples/api-web-common.toml`.
///
/// The sample demonstrates watch entries for files and directories such as:
///
/// ```text
/// Cargo.toml
/// Cargo.lock
/// compose.yaml
/// apps/web
/// crates/api
/// crates/common
/// migrations
/// tests
/// ```
///
/// The placeholders let CLI validation exercise the real sample config without
/// depending on those example project directories existing in this crate.
fn create_sample_watch_paths(dir: &std::path::Path) {
    for file in ["Cargo.toml", "Cargo.lock", "compose.yaml"] {
        std::fs::write(dir.join(file), "").unwrap();
    }

    for directory in [
        "apps/web",
        "crates/api",
        "crates/common",
        "migrations",
        "tests",
    ] {
        std::fs::create_dir_all(dir.join(directory)).unwrap();
    }
}

/// Verifies the checked-in API/web/common sample loads through the CLI.
///
/// # Example Under Test
///
/// Real config file:
///
/// ```text
/// examples/api-web-common.toml
/// ```
///
/// Placeholder watch paths created beside the copied config:
///
/// ```text
/// Cargo.toml
/// Cargo.lock
/// compose.yaml
/// apps/web
/// crates/api
/// crates/common
/// migrations
/// tests
/// ```
///
/// ```text
/// goggin-rs-process-watch run --config process-watch.toml
/// ```
///
/// # Assertions
///
/// - The command exits successfully.
///
/// # Why
///
/// Loading the real sample through the CLI keeps the public example aligned
/// with config parsing and validation behavior.
#[test]
fn api_web_common_sample_config_loads_through_cli() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("process-watch.toml");
    std::fs::copy(sample_config_path(), &config).unwrap();
    create_sample_watch_paths(dir.path());

    let mut cmd = Command::cargo_bin("goggin-rs-process-watch").unwrap();

    cmd.args(["run", "--config"]).arg(config).assert().success();
}

/// Verifies the checked-in API/web/common sample demonstrates expected sections.
///
/// # Example Under Test
///
/// Real config file:
///
/// ```text
/// examples/api-web-common.toml
/// ```
///
/// Expected service sections:
///
/// ```toml
/// [services.api]
/// [services.web]
/// [services.database]
/// [services.cache]
/// ```
///
/// Expected workflow sections:
///
/// ```toml
/// [workflows.docs]
/// [workflows.frontend]
/// ```
///
/// Expected docs relationship:
///
/// ```toml
/// [docs.rustdoc]
/// workflow = "docs"
/// ```
///
/// # Assertions
///
/// - The sample has a `services` table.
/// - The `services` table contains `api`, `web`, `database`, and `cache`.
/// - The `api` service contains a `readiness` section.
/// - The `api` service contains a `log_relay` section.
/// - The `web` service has a command array.
/// - The `web` command includes `trunk`.
/// - The sample has a `workflows` table.
/// - The `workflows` table contains `docs` and `frontend`.
/// - The sample has a `docs` table.
/// - The `rustdoc` docs entry uses workflow `docs`.
///
/// # Why
///
/// These assertions keep the checked-in sample useful as documentation for a
/// common API, frontend, database, cache, workflow, and docs configuration.
#[test]
fn api_web_common_sample_config_demonstrates_expected_sections() {
    let source = std::fs::read_to_string(sample_config_path()).unwrap();
    let config: Value = toml::from_str(&source).unwrap();

    let services = config
        .get("services")
        .and_then(Value::as_table)
        .expect("sample has services");

    for service in ["api", "web", "database", "cache"] {
        assert!(
            services.contains_key(service),
            "sample should include {service} service"
        );
    }

    let api = services.get("api").unwrap();
    assert!(api.get("readiness").is_some());
    assert!(api.get("log_relay").is_some());

    let web = services.get("web").unwrap();
    let web_command = web
        .get("command")
        .and_then(Value::as_array)
        .expect("web service has command");
    assert!(
        web_command
            .iter()
            .any(|part| part.as_str() == Some("trunk")),
        "web service should demonstrate Trunk"
    );

    let workflows = config
        .get("workflows")
        .and_then(Value::as_table)
        .expect("sample has workflows");
    assert!(workflows.contains_key("docs"));
    assert!(workflows.contains_key("frontend"));

    let docs = config
        .get("docs")
        .and_then(Value::as_table)
        .expect("sample has docs");
    assert_eq!(
        docs.get("rustdoc")
            .and_then(|doc| doc.get("workflow"))
            .and_then(Value::as_str),
        Some("docs")
    );
}
