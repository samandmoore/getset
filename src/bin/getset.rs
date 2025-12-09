use clap::Parser;
use getset::cli;

#[tokio::main]
async fn main() {
    env_logger::init();

    let app = cli::App::parse();

    if let Err(e) = app.run().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
