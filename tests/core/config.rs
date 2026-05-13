//! Integration coverage for process-watch config behavior.
//!
//! These tests exercise the config-facing CLI path, TOML deserialization,
//! config-relative path resolution, and accumulated validation diagnostics.
//! The inline TOML snippets are intentionally small examples of the config
//! shapes under test, while the CLI cases write those examples to temporary
//! config files to verify the same behavior users get from `run --config`.

use std::path::Path;

use assert_cmd::Command;
use goggin_rs_process_watch::config::{LoadedConfig, ProcessWatchConfig, ReadinessCheck};
use predicates::prelude::*;

/// Returns a minimal valid process-watch config.
///
/// This fixture exercises the smallest accepted service table:
///
/// ```toml
/// [services.api]
/// command = ["cargo", "run"]
/// ```
fn valid_config() -> &'static str {
    r#"
    [services.api]
    command = ["cargo", "run"]
    "#
}

// CLI config loading behavior.

/// Verifies `run` loads `process-watch.toml` from the current directory.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = ["cargo", "run"]
/// ```
///
/// ```text
/// goggin-rs-process-watch run
/// ```
///
/// # Assertions
///
/// - The command exits successfully.
#[test]
fn run_uses_default_config_in_current_directory() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("process-watch.toml"), valid_config()).unwrap();

    let mut cmd = Command::cargo_bin("goggin-rs-process-watch").unwrap();

    cmd.current_dir(dir.path()).arg("run").assert().success();
}

/// Verifies `run --config` takes precedence over default config discovery.
///
/// # Example Under Test
///
/// Default `process-watch.toml`:
///
/// ```text
/// not = [valid
/// ```
///
/// Explicit `custom.toml`:
///
/// ```toml
/// [services.api]
/// command = ["cargo", "run"]
/// ```
///
/// ```text
/// goggin-rs-process-watch run --config custom.toml
/// ```
///
/// # Assertions
///
/// - The command exits successfully.
///
/// # Why
///
/// The invalid default file proves the explicit config path is used instead
/// of the normal `process-watch.toml` discovery path.
#[test]
fn run_uses_explicit_config_instead_of_default_file() {
    let dir = tempfile::tempdir().unwrap();
    let explicit = dir.path().join("custom.toml");

    std::fs::write(dir.path().join("process-watch.toml"), "not = [valid").unwrap();
    std::fs::write(&explicit, valid_config()).unwrap();

    let mut cmd = Command::cargo_bin("goggin-rs-process-watch").unwrap();

    cmd.current_dir(dir.path())
        .args(["run", "--config"])
        .arg(&explicit)
        .assert()
        .success();
}

/// Verifies `run --config` reports a missing explicit config file.
///
/// # Example Under Test
///
/// ```text
/// goggin-rs-process-watch run --config missing.toml
/// ```
///
/// # Assertions
///
/// - The command exits with failure.
/// - Standard error contains `could not read config`.
/// - Standard error contains `missing.toml`.
#[test]
fn run_reports_missing_explicit_config() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.toml");

    let mut cmd = Command::cargo_bin("goggin-rs-process-watch").unwrap();

    cmd.args(["run", "--config"])
        .arg(&missing)
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not read config"))
        .stderr(predicate::str::contains("missing.toml"));
}

/// Verifies `run --config` reports a malformed explicit config file.
///
/// # Example Under Test
///
/// `broken.toml`:
///
/// ```text
/// not = [valid
/// ```
///
/// ```text
/// goggin-rs-process-watch run --config broken.toml
/// ```
///
/// # Assertions
///
/// - The command exits with failure.
/// - Standard error contains `could not parse config`.
/// - Standard error contains `broken.toml`.
#[test]
fn run_reports_malformed_explicit_config() {
    let dir = tempfile::tempdir().unwrap();
    let broken = dir.path().join("broken.toml");
    std::fs::write(&broken, "not = [valid").unwrap();

    let mut cmd = Command::cargo_bin("goggin-rs-process-watch").unwrap();

    cmd.args(["run", "--config"])
        .arg(&broken)
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not parse config"))
        .stderr(predicate::str::contains("broken.toml"));
}

/// Verifies CLI validation diagnostics include config field names.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = []
/// ```
///
/// ```text
/// goggin-rs-process-watch run --config bad.toml
/// ```
///
/// # Assertions
///
/// - The command exits with failure.
/// - Standard error contains `invalid config:`.
/// - Standard error contains `services.api.command`.
///
/// # Why
///
/// Empty command arrays should be reported with the full user-facing config
/// field path.
#[test]
fn run_reports_validation_field_names() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("bad.toml");

    std::fs::write(
        &config,
        r#"
        [services.api]
        command = []
        "#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("goggin-rs-process-watch").unwrap();

    cmd.args(["run", "--config"])
        .arg(&config)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid config:"))
        .stderr(predicate::str::contains("services.api.command"));
}

/// Verifies CLI validation reports multiple invalid fields at once.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = []
///
/// [docs.rustdoc]
/// workflow = "missing"
/// ```
///
/// ```text
/// goggin-rs-process-watch run --config bad.toml
/// ```
///
/// # Assertions
///
/// - The command exits with failure.
/// - Standard error contains `services.api.command`.
/// - Standard error contains `docs.rustdoc`.
/// - Standard error contains `docs.rustdoc.workflow`.
///
/// # Why
///
/// The CLI should surface all validation failures instead of stopping after
/// the first invalid field.
#[test]
fn run_reports_multiple_validation_errors() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("bad.toml");

    std::fs::write(
        &config,
        r#"
        [services.api]
        command = []

        [docs.rustdoc]
        workflow = "missing"
        "#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("goggin-rs-process-watch").unwrap();

    cmd.args(["run", "--config"])
        .arg(&config)
        .assert()
        .failure()
        .stderr(predicate::str::contains("services.api.command"))
        .stderr(predicate::str::contains("docs.rustdoc"))
        .stderr(predicate::str::contains("docs.rustdoc.workflow"));
}

// TOML deserialization coverage.

/// Verifies dynamic service, workflow, and docs names deserialize correctly.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// label = "API"
/// command = ["cargo", "run", "-p", "api"]
/// watch = ["crates/api", "crates/common"]
/// port = 8080
/// env = { RUST_LOG = "info" }
///
/// [services.api.readiness]
/// kind = "http"
/// url = "http://localhost:8080/health"
/// expected_status = 200
///
/// [services.api.log_relay]
/// enabled = true
/// target = "process_watch"
///
/// [workflows.check]
/// label = "Check"
/// command = ["cargo", "check", "--workspace"]
///
/// [docs.rustdoc]
/// label = "Rustdoc"
/// path = "target/doc"
/// workflow = "check"
/// ```
///
/// # Assertions
///
/// - The TOML deserializes into [`ProcessWatchConfig`] successfully.
/// - The `api` service label is `API`.
/// - The first `api` command part is `cargo`.
/// - The `api` service has two watch paths.
/// - The `api` readiness check is HTTP.
/// - The config contains the `check` workflow.
/// - The config contains the `rustdoc` docs entry.
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

// Config-relative path resolution.

/// Verifies relative watch paths resolve from the config file parent.
///
/// # Example Under Test
///
/// Config file path:
///
/// ```text
/// nested/process-watch.toml
/// ```
///
/// ```toml
/// [services.api]
/// command = ["cargo", "run"]
/// watch = ["crates/api"]
/// ```
///
/// # Assertions
///
/// - Loading the config succeeds.
/// - The loaded config contains the `api` service.
/// - Resolving the first `api` watch path returns `nested/crates/api`.
///
/// # Why
///
/// Relative watch paths should resolve from the config file's parent
/// directory instead of the process current directory.
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

/// Verifies absolute watch paths stay unchanged during path resolution.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = ["cargo", "run"]
/// watch = ["/absolute/path/to/src"]
/// ```
///
/// # Assertions
///
/// - Loading the config succeeds.
/// - The loaded config contains the `api` service.
/// - Resolving the first `api` watch path returns the same absolute `src`
///   path.
///
/// # Why
///
/// Absolute paths should not be joined to the config file directory.
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

// Validation rejection cases.

/// Verifies validation rejects a config with no services.
///
/// # Example Under Test
///
/// ```toml
/// services = {}
/// ```
///
/// # Assertions
///
/// - The TOML deserializes into [`ProcessWatchConfig`] successfully.
/// - Validation fails.
/// - The validation errors include the `services` field.
#[test]
fn validation_rejects_empty_services() {
    let config: ProcessWatchConfig = toml::from_str("services = {}").unwrap();

    let errors = config.validate(Path::new(".")).unwrap_err();

    assert!(errors.iter().any(|error| error.field == "services"));
}

/// Verifies validation rejects an empty service command array.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = []
/// ```
///
/// # Assertions
///
/// - The TOML deserializes into [`ProcessWatchConfig`] successfully.
/// - Validation fails.
/// - The validation errors include the `services.api.command` field.
#[test]
fn validation_rejects_empty_service_command() {
    let config: ProcessWatchConfig = toml::from_str(
        r#"
        [services.api]
        command = []
        "#,
    )
    .unwrap();

    let errors = config.validate(Path::new(".")).unwrap_err();

    assert!(
        errors
            .iter()
            .any(|error| error.field == "services.api.command")
    );
}

/// Verifies validation rejects blank command arguments.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = ["cargo", " "]
/// ```
///
/// # Assertions
///
/// - The TOML deserializes into [`ProcessWatchConfig`] successfully.
/// - Validation fails.
/// - The validation errors include the `services.api.command[1]` field.
#[test]
fn validation_rejects_empty_command_argument() {
    let config: ProcessWatchConfig = toml::from_str(
        r#"
        [services.api]
        command = ["cargo", " "]
        "#,
    )
    .unwrap();

    let errors = config.validate(Path::new(".")).unwrap_err();

    assert!(
        errors
            .iter()
            .any(|error| error.field == "services.api.command[1]")
    );
}

/// Verifies validation rejects missing relative watch paths.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = ["cargo", "run"]
/// watch = ["missing"]
/// ```
///
/// # Assertions
///
/// - The TOML deserializes into [`ProcessWatchConfig`] successfully.
/// - Validation fails.
/// - The validation errors include the `services.api.watch[0]` field.
///
/// # Why
///
/// Validation checks watch paths relative to the supplied base directory.
#[test]
fn validation_rejects_missing_watch_path_relative_to_base_dir() {
    let dir = tempfile::tempdir().unwrap();
    let config: ProcessWatchConfig = toml::from_str(
        r#"
        [services.api]
        command = ["cargo", "run"]
        watch = ["missing"]
        "#,
    )
    .unwrap();

    let errors = config.validate(dir.path()).unwrap_err();

    assert!(
        errors
            .iter()
            .any(|error| error.field == "services.api.watch[0]")
    );
}

/// Verifies validation rejects invalid HTTP readiness settings.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = ["cargo", "run"]
///
/// [services.api.readiness]
/// kind = "http"
/// url = "localhost:3000/health"
/// expected_status = 99
/// ```
///
/// # Assertions
///
/// - The TOML deserializes into [`ProcessWatchConfig`] successfully.
/// - Validation fails.
/// - The validation errors include the `services.api.readiness.url` field.
/// - The validation errors include the
///   `services.api.readiness.expected_status` field.
#[test]
fn validation_rejects_invalid_http_readiness() {
    let config: ProcessWatchConfig = toml::from_str(
        r#"
        [services.api]
        command = ["cargo", "run"]

        [services.api.readiness]
        kind = "http"
        url = "localhost:3000/health"
        expected_status = 99
        "#,
    )
    .unwrap();

    let errors = config.validate(Path::new(".")).unwrap_err();

    assert!(
        errors
            .iter()
            .any(|error| error.field == "services.api.readiness.url")
    );
    assert!(
        errors
            .iter()
            .any(|error| error.field == "services.api.readiness.expected_status")
    );
}

/// Verifies validation rejects invalid TCP readiness settings.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = ["cargo", "run"]
///
/// [services.api.readiness]
/// kind = "tcp"
/// host = " "
/// port = 0
/// ```
///
/// # Assertions
///
/// - The TOML deserializes into [`ProcessWatchConfig`] successfully.
/// - Validation fails.
/// - The validation errors include the `services.api.readiness.host` field.
/// - The validation errors include the `services.api.readiness.port` field.
#[test]
fn validation_rejects_invalid_tcp_readiness() {
    let config: ProcessWatchConfig = toml::from_str(
        r#"
        [services.api]
        command = ["cargo", "run"]

        [services.api.readiness]
        kind = "tcp"
        host = " "
        port = 0
        "#,
    )
    .unwrap();

    let errors = config.validate(Path::new(".")).unwrap_err();

    assert!(
        errors
            .iter()
            .any(|error| error.field == "services.api.readiness.host")
    );
    assert!(
        errors
            .iter()
            .any(|error| error.field == "services.api.readiness.port")
    );
}

/// Verifies validation rejects an enabled log relay with a blank target.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = ["cargo", "run"]
///
/// [services.api.log_relay]
/// enabled = true
/// target = " "
/// ```
///
/// # Assertions
///
/// - The TOML deserializes into [`ProcessWatchConfig`] successfully.
/// - Validation fails.
/// - The validation errors include the `services.api.log_relay.target` field.
#[test]
fn validation_rejects_empty_log_relay_target() {
    let config: ProcessWatchConfig = toml::from_str(
        r#"
        [services.api]
        command = ["cargo", "run"]

        [services.api.log_relay]
        enabled = true
        target = " "
        "#,
    )
    .unwrap();

    let errors = config.validate(Path::new(".")).unwrap_err();

    assert!(
        errors
            .iter()
            .any(|error| error.field == "services.api.log_relay.target")
    );
}

/// Verifies validation rejects docs entries that reference unknown workflows.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = ["cargo", "run"]
///
/// [docs.rustdoc]
/// path = "target/doc"
/// workflow = "missing"
/// ```
///
/// # Assertions
///
/// - The TOML deserializes into [`ProcessWatchConfig`] successfully.
/// - Validation fails.
/// - The validation errors include the `docs.rustdoc.workflow` field.
#[test]
fn validation_rejects_unknown_docs_workflow() {
    let config: ProcessWatchConfig = toml::from_str(
        r#"
        [services.api]
        command = ["cargo", "run"]

        [docs.rustdoc]
        path = "target/doc"
        workflow = "missing"
        "#,
    )
    .unwrap();

    let errors = config.validate(Path::new(".")).unwrap_err();

    assert!(
        errors
            .iter()
            .any(|error| error.field == "docs.rustdoc.workflow")
    );
}

/// Verifies validation accumulates multiple errors in one pass.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = []
///
/// [docs.rustdoc]
/// workflow = "missing"
/// ```
///
/// # Assertions
///
/// - The TOML deserializes into [`ProcessWatchConfig`] successfully.
/// - Validation fails with at least three errors.
/// - The validation errors include the `services.api.command` field.
/// - The validation errors include the `docs.rustdoc` field.
/// - The validation errors include the `docs.rustdoc.workflow` field.
///
/// # Why
///
/// A single validation pass should collect each invalid field so users can fix
/// the whole config in one edit cycle.
#[test]
fn validation_collects_multiple_errors() {
    let config: ProcessWatchConfig = toml::from_str(
        r#"
        [services.api]
        command = []

        [docs.rustdoc]
        workflow = "missing"
        "#,
    )
    .unwrap();

    let errors = config.validate(Path::new(".")).unwrap_err();

    assert!(errors.len() >= 3);
    assert!(
        errors
            .iter()
            .any(|error| error.field == "services.api.command")
    );
    assert!(errors.iter().any(|error| error.field == "docs.rustdoc"));
    assert!(
        errors
            .iter()
            .any(|error| error.field == "docs.rustdoc.workflow")
    );
}

// Validation acceptance cases.

/// Verifies validation accepts existing relative watch paths.
///
/// # Example Under Test
///
/// Existing path:
///
/// ```text
/// src
/// ```
///
/// ```toml
/// [services.api]
/// command = ["cargo", "run"]
/// watch = ["src"]
/// ```
///
/// # Assertions
///
/// - The TOML deserializes into [`ProcessWatchConfig`] successfully.
/// - Validation succeeds.
#[test]
fn validation_accepts_existing_watch_paths() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();

    let config: ProcessWatchConfig = toml::from_str(
        r#"
        [services.api]
        command = ["cargo", "run"]
        watch = ["src"]
        "#,
    )
    .unwrap();

    config.validate(dir.path()).unwrap();
}
