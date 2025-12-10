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
        .stdout(predicate::str::contains("up"));
}

#[test]
fn test_up_subcommand_help() {
    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("up")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Run commands from a TOML file"))
        .stdout(predicate::str::contains("--verbose"))
        .stdout(predicate::str::contains("--report"))
        .stdout(predicate::str::contains("--step"));
}

#[test]
fn test_valid_file_execution() {
    let fixture = get_fixture_path("valid.toml");

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("up")
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
        .arg("up")
        .arg(&fixture)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error parsing TOML"));
}

#[test]
fn test_nonexistent_file_error() {
    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("up")
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
        .arg("up")
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
        .arg("up")
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
        .arg("up")
        .arg("--report")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("📊 Report").or(predicate::str::contains("Report")));
}

#[test]
fn test_default_config_file() {
    // Change to the fixtures directory so getset.toml is found
    let fixtures_dir = get_fixture_path(".");

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("up")
        .current_dir(fixtures_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Default config test 1"))
        .stdout(predicate::str::contains("Default config test 2"))
        .stdout(predicate::str::contains("All set!"));
}

#[test]
fn test_step_flag_single_match() {
    let fixture = get_fixture_path("step-test.toml");

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("up")
        .arg("--step")
        .arg("production")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploy to production"))
        .stdout(predicate::str::contains("Deploying..."))
        .stdout(predicate::str::contains("Build frontend").not())
        .stdout(predicate::str::contains("Build backend").not())
        .stdout(predicate::str::contains("Run tests").not())
        .stdout(predicate::str::contains("All set!"));
}

#[test]
fn test_step_flag_multiple_matches() {
    let fixture = get_fixture_path("step-test.toml");

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("up")
        .arg("--step")
        .arg("build")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 3 steps matching 'build'"))
        .stdout(predicate::str::contains("Build frontend"))
        .stdout(predicate::str::contains("Build backend"))
        .stdout(predicate::str::contains("Build documentation"))
        .stdout(predicate::str::contains("Building frontend..."))
        .stdout(predicate::str::contains("Building backend..."))
        .stdout(predicate::str::contains("Building docs..."))
        .stdout(predicate::str::contains("Run tests").not())
        .stdout(predicate::str::contains("Deploy to production").not())
        .stdout(predicate::str::contains("All set!"));
}

#[test]
fn test_step_flag_no_matches() {
    let fixture = get_fixture_path("step-test.toml");

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("up")
        .arg("--step")
        .arg("nonexistent")
        .arg(&fixture)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No steps found matching 'nonexistent'",
        ));
}

#[test]
fn test_step_flag_case_insensitive() {
    let fixture = get_fixture_path("step-test.toml");

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("up")
        .arg("--step")
        .arg("PRODUCTION")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploy to production"))
        .stdout(predicate::str::contains("Deploying..."));
}

#[test]
fn test_step_flag_partial_match() {
    let fixture = get_fixture_path("step-test.toml");

    std::process::Command::new(assert_cmd::cargo::cargo_bin!("getset"))
        .arg("up")
        .arg("--step")
        .arg("front")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("Build frontend"))
        .stdout(predicate::str::contains("Building frontend..."))
        .stdout(predicate::str::contains("Build backend").not())
        .stdout(predicate::str::contains("All set!"));
}
