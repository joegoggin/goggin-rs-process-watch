use std::path::PathBuf;

use assert_cmd::Command;
use toml::Value;

fn sample_config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/api-web-common.toml")
}

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

#[test]
fn api_web_common_sample_config_loads_through_cli() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("process-watch.toml");
    std::fs::copy(sample_config_path(), &config).unwrap();
    create_sample_watch_paths(dir.path());

    let mut cmd = Command::cargo_bin("goggin-rs-process-watch").unwrap();

    cmd.args(["run", "--config"]).arg(config).assert().success();
}

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
