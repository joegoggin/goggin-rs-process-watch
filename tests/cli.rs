use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_lists_run_command() {
    let mut cmd = Command::cargo_bin("goggin-rs-process-watch").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains(
            "Watch and manage configured development processes",
        ));
}

#[test]
fn run_help_describes_config_override() {
    let mut cmd = Command::cargo_bin("goggin-rs-process-watch").unwrap();

    cmd.args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--config"))
        .stdout(predicate::str::contains("default discovery path"))
        .stdout(predicate::str::contains("GigLog").not());
}
