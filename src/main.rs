use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::path::PathBuf;

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
    // Print the title
    println!("{}", cmd_entry.title);

    // Split command into program and args
    let parts: Vec<&str> = cmd_entry.command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    let program = parts[0];
    let args = &parts[1..];

    // Start the command
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {}", e))?;

    // Capture output lines for potential clearing
    let mut output_lines = Vec::new();

    // Stream stdout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Error reading stdout: {}", e))?;
            println!("{}", line);
            output_lines.push(line);
        }
    }

    // Stream stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Error reading stderr: {}", e))?;
            eprintln!("{}", line);
            output_lines.push(line);
        }
    }

    // Wait for command to complete
    let status = child.wait()
        .map_err(|e| format!("Failed to wait for command: {}", e))?;

    if status.success() {
        // Clear the output lines (move cursor up and clear each line)
        let lines_to_clear = output_lines.len() + 1; // +1 for title
        for _ in 0..lines_to_clear {
            print!("\x1B[1A"); // Move cursor up one line
            print!("\x1B[2K"); // Clear the line
        }

        // Print title with success emoji
        println!("{} ✓", cmd_entry.title);
        Ok(())
    } else {
        Err(format!("Command exited with status: {}", status))
    }
}
