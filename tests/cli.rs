use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_lists_run_command() {
    let mut cmd = Command::cargo_bin("goggin-rs-console").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains(
            "configurable Rust development console",
        ));
}

#[test]
fn run_help_describes_config_override() {
    let mut cmd = Command::cargo_bin("goggin-rs-console").unwrap();

    cmd.args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--config"))
        .stdout(predicate::str::contains("default discovery path"))
        .stdout(predicate::str::contains("GigLog").not());
}
