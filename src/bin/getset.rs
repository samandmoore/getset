use clap::Parser;
use getset::cli;

fn main() {
    let app = cli::App::parse();

    if let Err(e) = app.run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
