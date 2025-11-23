use crate::config::CommandEntry;
use std::io::{self, IsTerminal};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Determines if we should use PTY mode based on the current context
fn should_use_pty() -> bool {
    // Check if stdout is a terminal - if so, favor PTY mode
    io::stdout().is_terminal()
}

/// Print command start message
fn print_command_start(cmd_entry: &CommandEntry, verbose: bool) {
    println!(
        "{} {}",
        console::style("===>").bold().dim(),
        console::style(&cmd_entry.title).bold().dim()
    );

    if verbose {
        println!("{}", console::style(&cmd_entry.command).yellow().dim());
    }
}

/// Print command result
fn print_command_result(cmd_entry: &CommandEntry, elapsed: Duration, success: bool) {
    if success {
        println!(
            "{} {} {} {}",
            console::style("└──▶").dim(),
            console::style("✓").green(),
            console::style(&cmd_entry.title).bold(),
            console::style(format!("({:.2}s)", elapsed.as_secs_f64())).dim()
        );
    } else {
        println!(
            "{} {} {} {}",
            console::style("└──▶").dim(),
            console::style("✗").red(),
            console::style(&cmd_entry.title).bold(),
            console::style(format!("({:.2}s)", elapsed.as_secs_f64())).dim()
        );
    }
}

/// Run a command using PTY for better terminal support
fn run_with_pty(cmd_entry: &CommandEntry) -> Result<(bool, Duration), String> {
    let timer = Instant::now();
    let (_, pts) =
        pty_process::blocking::open().map_err(|e| format!("Failed to open PTY: {}", e))?;

    // Execute command through shell to support multiline scripts and shell features
    let mut child = pty_process::blocking::Command::new("sh")
        .arg("-c")
        .arg(&cmd_entry.command)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn(pts)
        .map_err(|e| format!("Failed to spawn command: {}", e))?;

    // Wait for command to complete
    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for command: {}", e))?;

    let elapsed = timer.elapsed();
    Ok((status.success(), elapsed))
}

/// Run a command without PTY (for non-terminal contexts)
fn run_without_pty(cmd_entry: &CommandEntry) -> Result<(bool, Duration), String> {
    let timer = Instant::now();

    // Execute command through shell to support multiline scripts and shell features
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&cmd_entry.command)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {}", e))?;

    // Wait for command to complete
    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for command: {}", e))?;

    let elapsed = timer.elapsed();
    Ok((status.success(), elapsed))
}

/// Run a command, automatically detecting whether to use PTY or not
pub fn run_command(cmd_entry: &CommandEntry, verbose: bool) -> Result<Duration, String> {
    print_command_start(cmd_entry, verbose);

    let (success, elapsed) = if should_use_pty() {
        // PTY is favored when available (in terminal contexts)
        // But if it fails, gracefully fall back to non-PTY mode
        match run_with_pty(cmd_entry) {
            Ok(result) => result,
            Err(e) if e.contains("Failed to open PTY") || e.contains("Failed to spawn command") => {
                // PTY failed, fall back to non-PTY mode
                run_without_pty(cmd_entry)?
            }
            Err(e) => return Err(e),
        }
    } else {
        // Fall back to non-PTY mode in non-terminal contexts
        run_without_pty(cmd_entry)?
    };

    print_command_result(cmd_entry, elapsed, success);

    if success {
        Ok(elapsed)
    } else {
        Err("Command exited with non-zero status".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_success() {
        let cmd = CommandEntry {
            title: "Test echo".to_string(),
            command: "echo 'test'".to_string(),
        };

        let result = run_command(&cmd, false);
        assert!(result.is_ok(), "Command should succeed");
    }

    #[test]
    fn test_run_command_failure() {
        let cmd = CommandEntry {
            title: "Test false".to_string(),
            command: "false".to_string(),
        };

        let result = run_command(&cmd, false);
        assert!(result.is_err(), "Command should fail");
        assert!(result.unwrap_err().contains("non-zero status"));
    }

    #[test]
    fn test_run_command_with_output() {
        let cmd = CommandEntry {
            title: "Test ls".to_string(),
            command: "ls -la".to_string(),
        };

        let result = run_command(&cmd, false);
        assert!(result.is_ok(), "Command should succeed");
    }

    #[test]
    fn test_run_without_pty_success() {
        let cmd = CommandEntry {
            title: "Test non-PTY echo".to_string(),
            command: "echo 'non-pty test'".to_string(),
        };

        let result = run_without_pty(&cmd);
        assert!(result.is_ok(), "Non-PTY command should succeed");
        let (success, _) = result.unwrap();
        assert!(success, "Command should return success");
    }

    #[test]
    fn test_run_without_pty_failure() {
        let cmd = CommandEntry {
            title: "Test non-PTY false".to_string(),
            command: "exit 1".to_string(),
        };

        let result = run_without_pty(&cmd);
        assert!(result.is_ok(), "Non-PTY command should return a result");
        let (success, _) = result.unwrap();
        assert!(!success, "Command should return failure");
    }

    #[test]
    fn test_should_use_pty_detection() {
        // This test just verifies the function is callable
        // The actual behavior depends on the terminal context
        let _ = should_use_pty();
    }
}
