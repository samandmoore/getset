use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "getset")]
#[command(about = "Run commands from a YAML file sequentially", long_about = None)]
struct Cli {
    /// Path to the YAML file containing commands
    file: PathBuf,

    /// Show verbose logging
    #[arg(long)]
    verbose: bool,
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

    let yaml_content = fs::read_to_string(&cli.file).unwrap_or_else(|e| {
        eprintln!("Error reading file: {}", e);
        std::process::exit(1);
    });

    let config: Config = serde_yaml::from_str(&yaml_content).unwrap_or_else(|e| {
        eprintln!("Error parsing YAML: {}", e);
        std::process::exit(1);
    });

    let timer = Instant::now();

    for cmd_entry in config.commands.iter() {
        if let Err(e) = run_command(cmd_entry, cli.verbose) {
            eprintln!(
                "\n{} A command failed",
                console::style("Error:").red().bold(),
            );

            if cli.verbose {
                eprintln!("{}", console::style(e).red());
            }
            std::process::exit(1);
        }
    }

    let elapsed = timer.elapsed();

    println!("{}", console::style("│").dim());
    println!(
        "{} All set! {}",
        console::style("└─▶").dim(),
        console::style(format!("({:.2}s)", elapsed.as_secs_f64())).dim()
    );
}

fn run_command(cmd_entry: &CommandEntry, verbose: bool) -> Result<(), String> {
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
            console::style("└─▶").dim(),
            console::style("✓").green(),
            console::style(&cmd_entry.title).bold(),
            console::style(format!("({:.2}s)", elapsed.as_secs_f64())).dim()
        );
        Ok(())
    } else {
        println!(
            "{} {} {} {}",
            console::style("└─▶").dim(),
            console::style("✗").red(),
            console::style(&cmd_entry.title).bold(),
            console::style(format!("({:.2}s)", elapsed.as_secs_f64())).dim()
        );
        Err(format!("Command exited with status: {}", status))
    }
}
