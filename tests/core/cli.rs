//! Integration coverage for the process-watch command line interface.
//!
//! These tests document the visible CLI contract for help text and config
//! override parsing. The config examples are intentionally minimal because the
//! CLI layer only needs a valid file to prove argument handling reaches config
//! loading successfully.

use assert_cmd::Command;
use predicates::prelude::*;

// CLI help text behavior.

/// Verifies the top-level help output advertises the run command.
///
/// # Example Under Test
///
/// ```text
/// goggin-rs-process-watch --help
/// ```
///
/// # Assertions
///
/// - The command exits successfully.
/// - Standard output contains `run`.
/// - Standard output contains `Watch and manage configured development processes`.
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

// CLI config override behavior.

/// Verifies `run --config` accepts a non-default config file name.
///
/// # Example Under Test
///
/// ```toml
/// [services.api]
/// command = ["cargo", "run"]
/// ```
///
/// ```text
/// goggin-rs-process-watch run --config process-watch.local.toml
/// ```
///
/// # Assertions
///
/// - The command exits successfully.
///
/// # Why
///
/// This proves the non-default file name is parsed and passed through to
/// config loading.
#[test]
fn run_accepts_config_override_argument() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("process-watch.local.toml"),
        r#"
        [services.api]
        command = ["cargo", "run"]
        "#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("goggin-rs-process-watch").unwrap();

    cmd.current_dir(dir.path())
        .args(["run", "--config", "process-watch.local.toml"])
        .assert()
        .success();
}
