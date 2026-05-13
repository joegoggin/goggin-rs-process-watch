use std::path::Path;

use assert_cmd::Command;
use goggin_rs_process_watch::config::{LoadedConfig, ProcessWatchConfig, ReadinessCheck};
use predicates::prelude::*;

fn valid_config() -> &'static str {
    r#"
    [services.api]
    command = ["cargo", "run"]
    "#
}

#[test]
fn run_uses_default_config_in_current_directory() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("process-watch.toml"), valid_config()).unwrap();

    let mut cmd = Command::cargo_bin("goggin-rs-process-watch").unwrap();

    cmd.current_dir(dir.path()).arg("run").assert().success();
}

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

#[test]
fn validation_rejects_empty_services() {
    let config: ProcessWatchConfig = toml::from_str("services = {}").unwrap();

    let errors = config.validate(Path::new(".")).unwrap_err();

    assert!(errors.iter().any(|error| error.field == "services"));
}

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
