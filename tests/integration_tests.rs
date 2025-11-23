use std::path::PathBuf;
use std::process::Command;

/// Helper function to get the path to the compiled binary
fn get_binary_path() -> PathBuf {
    // Try to use CARGO_BIN_EXE environment variable first (more reliable)
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_getset") {
        return PathBuf::from(path);
    }

    // Fallback to computing path from test executable
    let mut path = std::env::current_exe().expect("Failed to get current executable path");
    path.pop(); // Remove test executable name
    path.pop(); // Remove 'deps'
    path.push("getset");
    path
}

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
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify that help output contains expected information
    assert!(
        stdout.contains("Run commands from a TOML file sequentially"),
        "Help should contain description"
    );
    assert!(
        stdout.contains("Usage:"),
        "Help should contain usage information"
    );
    assert!(
        stdout.contains("--verbose"),
        "Help should mention verbose flag"
    );
    assert!(
        stdout.contains("--report"),
        "Help should mention report flag"
    );
    assert!(stdout.contains("--help"), "Help should mention help flag");
}

#[test]
fn test_valid_file_execution() {
    let binary = get_binary_path();
    let fixture = get_fixture_path("valid.toml");

    let output = Command::new(&binary)
        .arg(&fixture)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Valid file execution should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify that each command title shows up in the output
    assert!(
        stdout.contains("Echo test 1"),
        "Output should contain first command title"
    );
    assert!(
        stdout.contains("Echo test 2"),
        "Output should contain second command title"
    );
    assert!(
        stdout.contains("Echo test 3"),
        "Output should contain third command title"
    );

    // Verify success indicators
    assert!(
        stdout.contains("✓"),
        "Output should contain success checkmarks"
    );
    assert!(
        stdout.contains("All set!"),
        "Output should contain completion message"
    );
}

#[test]
fn test_invalid_file_error() {
    let binary = get_binary_path();
    let fixture = get_fixture_path("invalid.toml");

    let output = Command::new(&binary)
        .arg(&fixture)
        .output()
        .expect("Failed to execute command");

    assert!(
        !output.status.success(),
        "Invalid file should cause failure"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify that error message is present
    assert!(
        stderr.contains("Error parsing TOML"),
        "Error message should mention TOML parsing error"
    );
}

#[test]
fn test_nonexistent_file_error() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("nonexistent-file-that-does-not-exist.toml")
        .output()
        .expect("Failed to execute command");

    assert!(
        !output.status.success(),
        "Nonexistent file should cause failure"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify that error message mentions the file
    assert!(
        stderr.contains("Error reading file") || stderr.contains("No such file"),
        "Error message should mention file reading error"
    );
}

#[test]
fn test_verbose_flag_shows_command_text() {
    let binary = get_binary_path();
    let fixture = get_fixture_path("verbose-test.toml");

    let output = Command::new(&binary)
        .arg("--verbose")
        .arg(&fixture)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Verbose execution should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify that the actual command text is shown
    assert!(
        stdout.contains("echo 'Hello from verbose test'"),
        "Verbose output should contain the command text"
    );

    // Verify that the title is shown
    assert!(
        stdout.contains("Simple echo"),
        "Verbose output should contain command title"
    );
}

#[test]
fn test_verbose_flag_without_verbose() {
    let binary = get_binary_path();
    let fixture = get_fixture_path("verbose-test.toml");

    let output = Command::new(&binary)
        .arg(&fixture)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Normal execution should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify that command text is NOT shown without verbose flag
    assert!(
        !stdout.contains("echo 'Hello from verbose test'"),
        "Non-verbose output should NOT contain the command text"
    );

    // But title should still be shown
    assert!(
        stdout.contains("Simple echo"),
        "Output should still contain command title"
    );
}

#[test]
fn test_report_flag() {
    let binary = get_binary_path();
    let fixture = get_fixture_path("valid.toml");

    let output = Command::new(&binary)
        .arg("--report")
        .arg(&fixture)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Execution with report flag should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify that report section is shown
    assert!(
        stdout.contains("📊 Report") || stdout.contains("Report"),
        "Output with --report should contain report section"
    );
}
