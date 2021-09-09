#![feature(linked_list_cursors)]

mod annealing;
mod app;
mod layout;
mod penalty;
mod simulator;
mod utils;

use app::{Command, Config};
use penalty::Corpus;
use std::fs::File;
use std::io::Read;
use std::process::exit;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;


fn main() {
    let config = Config::from_env().unwrap_or_else(|err| {
        println!("Error: {}", err);
        exit(1);
    });

    let buffer = read_file(&config.corpus_path).unwrap_or_else(|err| {
        println!("Error: {}", err);
        exit(1);
    });

    let corpus = Corpus::from(&buffer[..]);

    match config.command {
        Command::Run => simulator::run(&corpus, &config),
        Command::RunRefs => simulator::run_refs(&corpus, &config),
        Command::Refine => simulator::refine(&corpus, &config),
    }
    .unwrap_or_else(|err| {
        println!("Error: {}", err);
        exit(1);
    });
}

fn read_file<T: AsRef<std::path::Path>>(path: &T) -> Result<String> {
    let path = path.as_ref();
    let mut f = File::open(path).unwrap_or_else(|err| {
        println!("Can't open file {}: {}", &path.display(), err);
        exit(1);
    });

    let mut buffer = String::new();
    match f.read_to_string(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(err) => Err(format!(
            "Error while reading from file {}: {}",
            path.display(),
            err
        ))?,
    }
}
