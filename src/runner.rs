use crate::config::CommandEntry;
use std::process::Stdio;
use std::time::{Duration, Instant};

pub fn run_command(cmd_entry: &CommandEntry, verbose: bool) -> Result<Duration, String> {
    let timer = Instant::now();
    let (_, pts) = pty_process::blocking::open().unwrap();

    println!(
        "{} {}",
        console::style("===>").bold().dim(),
        console::style(&cmd_entry.title).bold().dim()
    );

    if verbose {
        println!("{}", console::style(&cmd_entry.command).yellow().dim());
    }

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

    if status.success() {
        println!(
            "{} {} {} {}",
            console::style("└──▶").dim(),
            console::style("✓").green(),
            console::style(&cmd_entry.title).bold(),
            console::style(format!("({:.2}s)", elapsed.as_secs_f64())).dim()
        );
        Ok(elapsed)
    } else {
        println!(
            "{} {} {} {}",
            console::style("└──▶").dim(),
            console::style("✗").red(),
            console::style(&cmd_entry.title).bold(),
            console::style(format!("({:.2}s)", elapsed.as_secs_f64())).dim()
        );
        Err(format!("Command exited with status: {}", status))
    }
}
