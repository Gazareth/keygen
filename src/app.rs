use clap::{App, Arg, ArgMatches};
use std::path::PathBuf;

use crate::layout;
use crate::layout::Layout;
use crate::Result;

const DEFAULT_SWAPS: usize = 2;

#[derive(Debug)]
pub enum Command {
    Run,
    RunRefs,
    Refine,
    Analyze,
}

#[derive(Debug)]
pub struct Config {
    pub debug: bool,
    pub swaps: usize,
    pub command: Command,
    pub corpus_path: PathBuf,
    pub repetition: usize,
    pub layout: Layout,
    pub output: Option<PathBuf>,
}

fn print_usage_and_exit(matches: &ArgMatches) -> ! {
    println!("{}", matches.usage());
    std::process::exit(1);
}

impl Config {
    pub fn from_env() -> Result<Config> {
        let matches = App::new("keygen")
            .arg(Arg::with_name("debug").long("debug").short("d"))
            .arg(
                Arg::with_name("swaps")
                    .long("swaps-per-iteration")
                    .short("s")
                    .value_name("COUNT"),
            )
            .arg(
                Arg::with_name("command")
                    .index(1)
                    .required(true)
                    .value_name("COMMAND"),
            )
            .arg(
                Arg::with_name("corpus")
                    .index(2)
                    .required(true)
                    .value_name("PATH"),
            )
            .arg(
                Arg::with_name("repetitions")
                    .long("repititions")
                    .short("r")
                    .takes_value(true)
                    .value_name("COUNT"),
            )
            .arg(
                Arg::with_name("layout")
                    .long("layout")
                    .short("l")
                    .takes_value(true)
                    .value_name("PATH"),
            )
            .arg(
                Arg::with_name("output")
                .long("output")
                .short("o")
                .takes_value(true)
                .value_name("PATH")
            )
            .get_matches();

        let layout = match matches.value_of("layout") {
            None => crate::penalty::INIT_LAYOUT.clone(),
            Some(path) => match path {
                "colemak" => layout::COLEMAK_LAYOUT.clone(),
                "colemak-dh" => layout::COLEMAK_DH_LAYOUT.clone(),
                "dvorak" => layout::DVORAK_LAYOUT.clone(),
                "mtgap" => layout::MTGAP_LAYOUT.clone(),
                "qwerty" => layout::QWERTY_LAYOUT.clone(),
                "rsthd" => layout::RSTHD_LAYOUT.clone(),
                "workman" => layout::WORKMAN_LAYOUT.clone(),
                _ => {
                    let s = std::fs::read_to_string(&path)?;
                    Layout::from_string(&s)
                        .ok_or(format!("File {} does not contain a valid layout.", path))?
                }
            },
        };

        Ok(Config {
            debug: matches.is_present("debug"),

            swaps: match matches.value_of("swaps") {
                Some(s) => str::parse::<usize>(s)
                    .map_err(|_| format!("Invalid option for '--swaps-per-iteration': '{}'", s))?,
                None => DEFAULT_SWAPS,
            },

            repetition: match matches.value_of("repetitions") {
                Some(s) => str::parse::<usize>(s)
                    .map_err(|_| format!("Invalid option for 'repititions': '{}'", s))?,
                None => 1,
            },

            command: match matches.value_of("command").unwrap() {
                "run" => Command::Run,
                "run-refs" => Command::RunRefs,
                "refine" => Command::Refine,
                "analyze" => Command::Analyze,
                _ => print_usage_and_exit(&matches),
            },

            corpus_path: PathBuf::from(matches.value_of("corpus").unwrap()),

            output: matches.value_of("output").map(|s| PathBuf::from(s)),

            layout,
        })
    }
}
