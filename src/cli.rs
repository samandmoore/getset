use crate::config::{CommandEntry, Config};
use crate::platformx::{self, PlatformXClient};
use crate::runner;
use clap::{Parser, Subcommand};
use color_eyre::Section;
use color_eyre::eyre::{Result, eyre};
use console::style;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "getset")]
#[command(about = "Run commands from a TOML file sequentially", long_about = None)]
pub struct App {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run commands from a TOML file
    Up(UpCommand),
}

#[derive(Parser)]
pub struct UpCommand {
    /// Path to the TOML file containing commands (defaults to getset.toml)
    #[arg(default_value = "getset.toml")]
    pub file: PathBuf,

    /// Show verbose logging
    #[arg(long)]
    pub verbose: bool,

    /// Show profiling report at the end
    #[arg(long)]
    pub report: bool,

    /// Run only steps matching this substring (case-insensitive)
    #[arg(long)]
    pub step: Option<String>,
}

#[derive(Debug)]
struct CommandResult {
    title: String,
    duration: std::time::Duration,
}

impl App {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Up(cmd) => cmd.run().await,
        }
    }
}

impl UpCommand {
    pub async fn run(self) -> Result<()> {
        let config = Config::from_file(&self.file)?;

        // Get default metadata for telemetry
        let default_metadata = platformx::get_globals();

        // Initialize PlatformX client if configured
        let platformx_client = config
            .platformx
            .as_ref()
            .map(|px_config| PlatformXClient::new(px_config.clone(), default_metadata.clone()));

        let timer = Instant::now();

        if let Some(ref client) = platformx_client {
            // ignore errors to avoid failing due to tracking
            let _ = client.send_start().await;
        }

        // Filter commands based on --step argument if provided
        let commands_to_run: Vec<&CommandEntry> = if let Some(ref step_filter) = self.step {
            let matches: Vec<&CommandEntry> = config
                .commands
                .iter()
                .filter(|cmd| {
                    cmd.title
                        .to_lowercase()
                        .contains(&step_filter.to_lowercase())
                })
                .collect();

            if matches.is_empty() {
                return Err(eyre!(
                    "{} No steps found matching '{}'",
                    style("Error:").red().bold(),
                    step_filter
                ));
            }

            if matches.len() > 1 {
                println!(
                    "{} Found {} steps matching '{}':",
                    style("Info:").cyan().bold(),
                    matches.len(),
                    step_filter
                );
                for (i, cmd) in matches.iter().enumerate() {
                    println!("  {}. {}", i + 1, style(&cmd.title).cyan());
                }
                println!();
            }

            matches
        } else {
            config.commands.iter().collect()
        };

        let mut results = Vec::new();

        for cmd_entry in commands_to_run.iter() {
            match runner::run_command(cmd_entry, self.verbose) {
                Ok(duration) => {
                    results.push(CommandResult {
                        title: cmd_entry.title.clone(),
                        duration,
                    });
                }
                Err(e) => {
                    let elapsed = timer.elapsed();

                    if let Some(ref client) = platformx_client {
                        let error_msg = e.clone();
                        // ignore errors to avoid failing due to tracking
                        let _ = client.send_error(elapsed, error_msg).await;
                    }

                    return Err(eyre!("A command failed").with_note(|| e));
                }
            }
        }

        let elapsed = timer.elapsed();

        println!(
            "\n🎯 All set! {}",
            style(format!("({:.2}s)", elapsed.as_secs_f64())).dim()
        );

        if self.report {
            print_report(&results, elapsed);
        }

        if let Some(ref client) = platformx_client {
            // ignore errors to avoid failing due to tracking
            let _ = client.send_complete(elapsed).await;
        }

        Ok(())
    }
}

fn print_report(results: &[CommandResult], total: std::time::Duration) {
    println!("\n{}", style("📊 Report").bold());

    for result in results {
        println!(
            "{} {} {}",
            style("├──▶").dim(),
            style(format!("{:.2}s", result.duration.as_secs_f64())).dim(),
            &result.title,
        );
    }

    println!(
        "{} {} {}",
        style("└─▶").dim(),
        style(format!("{:.2}s", total.as_secs_f64())).dim().bold(),
        style("Total").bold(),
    );
}
