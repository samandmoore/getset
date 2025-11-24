use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::path::PathBuf;

/// Helper function to get the path to test fixtures
fn get_fixture_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(filename);
    path
}

#[test]
fn test_help_output() {
    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Run commands from a TOML file sequentially",
        ))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--verbose"))
        .stdout(predicate::str::contains("--report"))
        .stdout(predicate::str::contains("--help"));
}

#[test]
fn test_valid_file_execution() {
    let fixture = get_fixture_path("valid.toml");

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("Echo test 1"))
        .stdout(predicate::str::contains("Echo test 2"))
        .stdout(predicate::str::contains("Echo test 3"))
        .stdout(predicate::str::contains("All set!"));
}

#[test]
fn test_invalid_file_error() {
    let fixture = get_fixture_path("invalid.toml");

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg(&fixture)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error parsing TOML"));
}

#[test]
fn test_nonexistent_file_error() {
    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("nonexistent-file-that-does-not-exist.toml")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Error reading file")
                .or(predicate::str::contains("No such file")),
        );
}

#[test]
fn test_verbose_flag_shows_command_text() {
    let fixture = get_fixture_path("verbose-test.toml");

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("--verbose")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("echo 'Hello from verbose test'"))
        .stdout(predicate::str::contains("Simple echo"));
}

#[test]
fn test_verbose_flag_without_verbose() {
    let fixture = get_fixture_path("verbose-test.toml");

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("echo 'Hello from verbose test'").not())
        .stdout(predicate::str::contains("Simple echo"));
}

#[test]
fn test_report_flag() {
    let fixture = get_fixture_path("valid.toml");

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("--report")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("📊 Report").or(predicate::str::contains("Report")));
}
