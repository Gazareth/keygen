mod annealing;
mod app;
mod layout;
mod penalty;
mod simulator;

use app::{Command, Config};
use penalty::Corpus;
use std::process::exit;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    run().unwrap_or_else(|err| {
        println!("Error: {}", err);
        exit(1);
    });
}

fn run() -> Result<()> {
    let config = Config::from_env()?;

    let buffer = std::fs::read_to_string(&config.corpus_path)?;

    let corpus = Corpus::from(&buffer[..]);

    match config.command {
        Command::Run => simulator::run(&corpus, &config)?,
        Command::RunRefs => simulator::run_refs(&corpus, &config)?,
        Command::Refine => simulator::refine(&corpus, &config)?,
        Command::Analyze => simulator::analyze(&corpus)?,
    };
    Ok(())
}
