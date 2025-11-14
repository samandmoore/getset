use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use indicatif::{ProgressBar, ProgressStyle};

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
    let yaml_content = fs::read_to_string(&cli.file)
        .unwrap_or_else(|e| {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        });

    let config: Config = serde_yaml::from_str(&yaml_content)
        .unwrap_or_else(|e| {
            eprintln!("Error parsing YAML: {}", e);
            std::process::exit(1);
        });

    // Run commands sequentially
    for cmd_entry in config.commands {
        if let Err(e) = run_command(&cmd_entry) {
            eprintln!("\nCommand failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_command(cmd_entry: &CommandEntry) -> Result<(), String> {
    // Create a spinner with the title
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.dim} {msg}")
            .unwrap()
    );
    pb.set_message(format!("\x1B[1m\x1B[90m{}\x1B[0m", cmd_entry.title));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

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
            if let Ok(line) = line {
                let _ = tx_stdout.send(Output::Stdout(line));
            }
        }
    });

    // Spawn thread for stderr
    let stderr = child.stderr.take().expect("Failed to capture stderr");
    let tx_stderr = tx.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                let _ = tx_stderr.send(Output::Stderr(line));
            }
        }
    });

    // Drop the original sender so the channel closes when both threads finish
    drop(tx);

    // Capture all output lines for potential display on failure
    let mut output_lines = Vec::new();

    // Receive and update spinner with the latest output line
    for output in rx {
        match output {
            Output::Stdout(line) => {
                // Update spinner message with title and latest line
                pb.set_message(format!(
                    "\x1B[1m\x1B[90m{}\x1B[0m {}",
                    cmd_entry.title,
                    line
                ));
                output_lines.push(line);
            }
            Output::Stderr(line) => {
                // Update spinner message with title and latest line
                pb.set_message(format!(
                    "\x1B[1m\x1B[90m{}\x1B[0m {}",
                    cmd_entry.title,
                    line
                ));
                output_lines.push(line);
            }
        }
    }

    // Wait for command to complete
    let status = child.wait()
        .map_err(|e| format!("Failed to wait for command: {}", e))?;

    if status.success() {
        // Clear the spinner and print title with success emoji
        pb.finish_and_clear();
        println!("\x1B[1m\x1B[90m{}\x1B[0m \x1B[32m✓\x1B[0m", cmd_entry.title);
        Ok(())
    } else {
        // Clear the spinner and print title with failure marker
        pb.finish_and_clear();
        println!("\x1B[1m\x1B[90m{}\x1B[0m \x1B[31m✗\x1B[0m", cmd_entry.title);

        // Print all output for debugging
        for line in output_lines {
            println!("{}", line);
        }

        Err(format!("Command exited with status: {}", status))
    }
}
