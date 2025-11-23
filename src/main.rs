mod config;
mod runner;

use clap::Parser;
use config::Config;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "getset")]
#[command(about = "Run commands from a TOML file sequentially", long_about = None)]
struct Cli {
    /// Path to the TOML file containing commands
    file: PathBuf,

    /// Show verbose logging
    #[arg(long)]
    verbose: bool,

    /// Show profiling report at the end
    #[arg(long)]
    report: bool,
}

#[derive(Debug)]
struct CommandResult {
    title: String,
    duration: std::time::Duration,
}

fn main() {
    let cli = Cli::parse();

    let config = Config::from_file(&cli.file).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let timer = Instant::now();
    let mut results = Vec::new();

    for cmd_entry in config.commands.iter() {
        match runner::run_command(cmd_entry, cli.verbose) {
            Ok(duration) => {
                results.push(CommandResult {
                    title: cmd_entry.title.clone(),
                    duration,
                });
            }
            Err(e) => {
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
    }

    let elapsed = timer.elapsed();

    println!("{}", console::style("│").dim());
    println!(
        "{} All set! {}",
        console::style("└─▶").dim(),
        console::style(format!("({:.2}s)", elapsed.as_secs_f64())).dim()
    );

    if cli.report {
        print_profile_report(&results, elapsed);
    }
}



fn print_profile_report(results: &[CommandResult], total: std::time::Duration) {
    println!("\n{}", console::style("📊 Report").bold().cyan());

    for result in results {
        println!(
            "{} {} {}",
            console::style("├──▶").dim(),
            console::style(format!("{:.2}s", result.duration.as_secs_f64())).dim(),
            console::style(&result.title).bold(),
        );
    }

    println!(
        "{} {}",
        console::style("└─▶").dim(),
        console::style(format!("{:.2}s", total.as_secs_f64())).dim()
    );
}
