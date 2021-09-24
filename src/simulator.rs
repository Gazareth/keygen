extern crate itertools;
/// Applies the math in annealing.rs to keyboard layouts.
extern crate rand;

use self::rand::random;
use itertools::{FoldWhile, Itertools};
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use std::path::PathBuf;

use crate::annealing;
use crate::app::{self, Config};
use crate::layout::{self, Layout};
use crate::penalty::{self, Corpus};
use crate::Result;

pub fn run<'a>(corpus: &Corpus, config: &Config) -> Result<()> {
    let init_penalty = config.layout.penalize_with_details(corpus);

    let best_layout = (0..config.repetition)
        .map(|n| {
            println!("Started run {}/{}", n + 1, config.repetition);
            simulated_annealing(corpus, config) // Returns best layout found during simulation
        })
        .min_by(|l1, l2| {
            let p1 = l1.par_penalize(corpus);
            let p2 = l2.par_penalize(corpus);
            p1.partial_cmp(&p2).unwrap()
        })
        .unwrap();
    // .unwrap_or(config.layout.clone());

    println!("Initial layout:");
    println!("{}", config.layout);
    println!("{}", init_penalty);
    println!("");

    println!("BestLayout:");
    println!("{}", best_layout);
    println!("{}", best_layout.penalize_with_details(corpus));
    best_layout.write_to_file(
        config
            .output
            .as_ref()
            .unwrap_or(&PathBuf::from("winner.layout")),
    )?;

    Ok(())
}

// Returns best layout found during the simulation, not neccessarily the last one
fn simulated_annealing(corpus: &Corpus, config: &Config) -> Layout {
    let init_penalty = config.layout.par_penalize(corpus) / corpus.len as f64;

    let mut best_layout = config.layout.clone();
    let mut best_penalty = init_penalty;

    annealing::get_simulation_range().fold(
        (config.layout.clone(), init_penalty),
        |(accepted_layout, accepted_penalty), i| {
            // Copy and shuffle this iteration of the layout.
            let mut new_layout = accepted_layout.clone();
            new_layout.shuffle(random::<usize>() % config.swaps + 1);

            // Probabilistically accept worse transitions; always accept better
            // transitions.
            let new_penalty = new_layout.par_penalize(corpus) / corpus.len as f64;

            if annealing::accept_transition(new_penalty - accepted_penalty, i) {
                if new_penalty < best_penalty {
                    best_layout = new_layout.clone();
                    best_penalty = new_penalty;
                }

                if config.debug {
                    println!(
                        "Iteration {} accepted with penalty {}. Current penalty: {}",
                        i, new_penalty, accepted_penalty
                    );
                }
                (new_layout, new_penalty)
            } else {
                if config.debug {
                    println!(
                        "Iteration {} not accepted with penalty {}. Current penalty: {}",
                        i, new_penalty, accepted_penalty
                    );
                }
                (accepted_layout, accepted_penalty)
            }
        },
    );
    best_layout
}

pub fn refine<'a>(corpus: &Corpus, config: &Config) -> Result<()> {
    println!(
        "Start refining with {} swaps and initial layout:",
        config.swaps
    );
    println!("{}", config.layout);
    println!("{}", config.layout.penalize_with_details(corpus));

    let mut permutations = layout::LayoutPermutations::from_config(&config);

    let best_layout = (1..)
        .fold_while(config.layout.clone(), |curr_layout, n| {
            let curr_penalty = curr_layout.penalize(corpus);

            // Test every layout within `num_swaps` swaps of the initial layout.
            permutations.set_layout(&curr_layout);
            let (best_layout, best_penalty) = permutations
                .iter()
                .par_bridge()
                .map(|layout| {
                    let penalty = layout.penalize(corpus);
                    (layout, penalty)
                })
                .min_by(|(_, p1), (_, p2)| p1.partial_cmp(&p2).unwrap())
                .unwrap_or((curr_layout.clone(), curr_penalty));

            // Keep going until swapping doesn't get us any more improvements.
            if curr_layout.penalize(corpus) <= best_penalty {
                FoldWhile::Done(curr_layout)
            } else {
                println!("Result of iteration {}:", n);
                println!("{}", best_layout);
                println!("{}", best_layout.penalize_with_details(corpus));
                FoldWhile::Continue(best_layout)
            }
        })
        .into_inner();

    println!("");
    println!("Ultimate winner:");
    println!("{}", best_layout);
    println!("{}", best_layout.penalize_with_details(corpus));
    best_layout.write_to_file(
        config
            .output
            .as_ref()
            .unwrap_or(&PathBuf::from("refined.layout")),
    )?;
    Ok(())
}

pub fn analyze(corpus: &Corpus) -> Result<()> {
    let mut path = std::env::current_dir()?;
    path.push("analyze");

    std::fs::read_dir(path)
        .map_err(|_| "No directory 'analyze' found in current path")?
        .flatten()
        .map(|file| -> Result<()> {
            if file.file_type()?.is_file() {
                let path = file.path();
                let layout = Layout::from_string(&std::fs::read_to_string(&path)?).ok_or(
                    format!("File {} does not contain a valid layout", path.display()),
                )?;
                println!(
                    "Layout: {}",
                    file.path().file_name().unwrap().to_str().unwrap()
                );
                println!("{}", layout);
                println!("{}", layout.penalize_with_details(corpus));
            };
            Ok(())
        })
        .collect()
}

pub fn run_refs(corpus: &penalty::Corpus, config: &app::Config) -> Result<()> {
    let penalize_and_print = |name, layout: &layout::Layout| {
        println!("");
        let penalty = layout.penalize_with_details(corpus);
        println!("Reference: {}", name);
        println!("{}", layout);
        println!("{}", penalty);
    };

    penalize_and_print("QWERTY", &layout::QWERTY_LAYOUT);
    // penalize_and_print("DVORAK",&layout::DVORAK_LAYOUT);
    penalize_and_print("COLEMAK", &layout::COLEMAK_LAYOUT);
    penalize_and_print("COLEMAK-DH", &layout::COLEMAK_DH_LAYOUT);
    // penalize_and_print("QGMLWY", &layout::QGMLWY_LAYOUT);
    penalize_and_print("WORKMAN", &layout::WORKMAN_LAYOUT);
    // penalize_and_print("MALTRON",&layout::MALTRON_LAYOUT);
    penalize_and_print("MTGAP", &layout::MTGAP_LAYOUT);
    // penalize_and_print("CAPEWELL",&layout::CAPEWELL_LAYOUT);
    // penalize_and_print("ARENSITO",&layout::ARENSITO_LAYOUT);
    penalize_and_print("RSTHD", &layout::RSTHD_LAYOUT);
    penalize_and_print("INIT", &config.layout);
    Ok(())
}
