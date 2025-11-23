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

/// Helper function to get the binary path without using deprecated cargo_bin
fn get_bin() -> std::process::Command {
    // Use CARGO_BIN_EXE_<name> environment variable which is set by cargo test
    let bin_path = env!("CARGO_BIN_EXE_getset");
    std::process::Command::new(bin_path)
}

#[test]
fn test_help_output() {
    get_bin()
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

    get_bin()
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("Echo test 1"))
        .stdout(predicate::str::contains("Echo test 2"))
        .stdout(predicate::str::contains("Echo test 3"))
        .stdout(predicate::str::contains("✓"))
        .stdout(predicate::str::contains("All set!"));
}

#[test]
fn test_invalid_file_error() {
    let fixture = get_fixture_path("invalid.toml");

    get_bin()
        .arg(&fixture)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error parsing TOML"));
}

#[test]
fn test_nonexistent_file_error() {
    get_bin()
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

    get_bin()
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

    get_bin()
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("echo 'Hello from verbose test'").not())
        .stdout(predicate::str::contains("Simple echo"));
}

#[test]
fn test_report_flag() {
    let fixture = get_fixture_path("valid.toml");

    get_bin()
        .arg("--report")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("📊 Report").or(predicate::str::contains("Report")));
}
