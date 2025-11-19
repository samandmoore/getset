use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

#[derive(Parser)]
#[command(name = "getset")]
#[command(about = "Run commands from a YAML file sequentially", long_about = None)]
struct Cli {
    /// Path to the YAML file containing commands
    file: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Config {
    commands: Vec<CommandEntry>,
}

#[derive(Debug, Deserialize)]
struct CommandEntry {
    title: String,
    command: String,
}

#[derive(Debug)]
enum Output {
    Stdout(String),
    Stderr(String),
}

fn main() {
    let cli = Cli::parse();

    // Read and parse YAML file
    let yaml_content = fs::read_to_string(&cli.file).unwrap_or_else(|e| {
        eprintln!("Error reading file: {}", e);
        std::process::exit(1);
    });

    let config: Config = serde_yaml::from_str(&yaml_content).unwrap_or_else(|e| {
        eprintln!("Error parsing YAML: {}", e);
        std::process::exit(1);
    });

    // Run commands sequentially
    for cmd_entry in config.commands.iter() {
        if let Err(e) = run_command(cmd_entry) {
            eprintln!("\nCommand failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_command(cmd_entry: &CommandEntry) -> Result<(), String> {
    // Create a progress bar with spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.dim} {msg:.bold.dim}")
            .unwrap(),
    );
    pb.set_message(cmd_entry.title.clone());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    // Execute command through shell to support multiline scripts and shell features
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&cmd_entry.command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {}", e))?;

    // Create a channel for output
    let (tx, rx) = mpsc::channel();

    // Spawn thread for stdout
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let tx_stdout = tx.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let Ok(line) = line else { continue };
            let _ = tx_stdout.send(Output::Stdout(line));
        }
    });

    // Spawn thread for stderr
    let stderr = child.stderr.take().expect("Failed to capture stderr");
    let tx_stderr = tx.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let Ok(line) = line else { continue };
            let _ = tx_stderr.send(Output::Stderr(line));
        }
    });

    // Drop the original sender so the channel closes when both threads finish
    drop(tx);

    // Receive and print output in real-time
    for output in rx {
        match output {
            Output::Stdout(line) | Output::Stderr(line) => {
                pb.println(line);
            }
        }
    }

    // Wait for command to complete
    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for command: {}", e))?;

    // Calculate elapsed time
    let elapsed = pb.elapsed();

    if status.success() {
        // TODO: replace with custom ProgressTracker for elapsed time that has this higher
        // precision
        pb.finish_with_message(format!(
            "{} {} {}",
            console::style("✓").green(),
            console::style(&cmd_entry.title).bold().dim(),
            console::style(format!("({:.2}s)", elapsed.as_secs_f64())).dim()
        ));
        Ok(())
    } else {
        pb.finish_with_message(format!(
            "{} {} {}",
            console::style("✗").red(),
            console::style(&cmd_entry.title).bold().dim(),
            console::style(format!("({:.2}s)", elapsed.as_secs_f64())).dim()
        ));
        Err(format!("Command exited with status: {}", status))
    }
}
