use assert_cmd::Command;
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
