use std::path::PathBuf;
use std::process::Command;

/// Helper function to get the path to the compiled binary
fn get_binary_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Note: This test may fail in CI/non-PTY environments due to PTY requirements
    // In such cases, we verify that:
    // 1. The file was successfully read and parsed (command titles appear)
    // 2. Command execution was attempted (no file parsing errors)

    if output.status.success() {
        // If execution succeeded (PTY available), verify full output
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
        assert!(
            stdout.contains("✓"),
            "Output should contain success checkmarks"
        );
        assert!(
            stdout.contains("All set!"),
            "Output should contain completion message"
        );
    } else {
        // In non-PTY environments, verify the file was at least parsed correctly
        // The first command title should appear before any error
        assert!(
            stdout.contains("Echo test 1") || stderr.contains("command failed"),
            "Should show attempt to run first command or PTY error"
        );
        // Should NOT be a file parsing error
        assert!(
            !stderr.contains("Error parsing TOML") && !stderr.contains("Error reading file"),
            "Should not be a file parsing error - file should be valid"
        );
    }
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

    let stdout = String::from_utf8_lossy(&output.stdout);

    // In PTY environments, this should succeed. In non-PTY environments,
    // we can still verify the verbose flag shows command text before PTY failure

    // Verify that the actual command text is shown (this happens before PTY spawn)
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

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Even in non-PTY environments where execution fails,
    // we can verify the commands were parsed and execution was attempted
    assert!(
        stdout.contains("Echo test 1"),
        "Output should show first command was processed"
    );
}
