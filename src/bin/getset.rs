use clap::Parser;
use console::style;
use getset::cli;

#[tokio::main]
async fn main() {
    env_logger::init();

    let app = cli::App::parse();

    if let Err(e) = app.run().await {
        eprintln!("\n{} {}", style("Error:").red().bold(), e);

        std::process::exit(1);
    }
}
