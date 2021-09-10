extern crate itertools;
/// Applies the math in annealing.rs to keyboard layouts.
extern crate rand;

use self::rand::random;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use std::cmp::Ordering;

use crate::annealing;
use crate::app;
use crate::layout;
use crate::penalty;
use crate::utils;
use crate::Result;

#[derive(Debug, PartialEq, Clone)]
struct BestLayoutsEntry {
    penalty: f64,
    layout: layout::Layout,
}

impl PartialOrd for BestLayoutsEntry {
    fn partial_cmp(&self, other: &BestLayoutsEntry) -> Option<Ordering> {
        self.penalty.partial_cmp(&other.penalty)
    }
}

pub fn run<'a>(corpus: &penalty::Corpus, config: &app::Config) -> Result<()> {
    let init_penalty = config.layout.penalize_with_details(corpus);

    // Keep track of the best layouts we've encountered.
    let mut best_layouts = utils::ConstrainedSortedList::new(config.top_layouts);
    let entry = BestLayoutsEntry {
        layout: config.layout.clone(),
        penalty: init_penalty.scaled,
    };
    best_layouts.insert_maybe(&entry);

    for n in 0..config.repetition {
        println!("Started run {}/{}", n + 1, config.repetition);

        let mut accepted_layout = config.layout.clone();
        let mut accepted_penalty = init_penalty.scaled;

        for i in annealing::get_simulation_range() {
            // Copy and shuffle this iteration of the layout.
            let mut curr_layout = accepted_layout.clone();
            curr_layout.shuffle(random::<usize>() % config.swaps + 1);

            // Calculate penalty.
            let curr_penalty = curr_layout.par_penalize(corpus) / corpus.len as f64;

            // Probabilistically accept worse transitions; always accept better
            // transitions.
            if annealing::accept_transition(curr_penalty - accepted_penalty, i) {
                if config.debug {
                    println!(
                        "Iteration {} accepted with penalty {}. Current: {}",
                        i, curr_penalty, accepted_penalty
                    );
                }

                // Maybe insert this layout into best layouts.
                let entry = BestLayoutsEntry {
                    layout: curr_layout,
                    penalty: curr_penalty,
                };
                best_layouts.insert_maybe(&entry);

                accepted_layout = entry.layout;
                accepted_penalty = curr_penalty;
            } else {
                if config.debug {
                    println!(
                        "Iteration {} not accepted with penalty {}. Current: {}",
                        i, curr_penalty, accepted_penalty
                    );
                }
            }
        }
    }

    println!("Initial layout:");
    println!("{}", config.layout);
    println!("{}", init_penalty);
    println!("");

    println!("BestLayouts:");
    for (i, bl) in best_layouts.iter().enumerate() {
        let BestLayoutsEntry { layout, .. } = bl;
        if i == 0 {
            layout.write_to_file(&"winner.layout")?;
        }
        println!("");
        println!("Place {}:", i + 1);
        println!("{}", layout);
        println!("{}", layout.penalize_with_details(corpus));
    }

    Ok(())
}

pub fn refine<'a>(corpus: &penalty::Corpus, config: &app::Config) -> Result<()> {
    let mut curr_layout = config.layout.clone();
    let mut curr_penalty = curr_layout.penalize(corpus);

    println!(
        "Start refining with {} swaps and initial layout:",
        config.swaps
    );
    println!("{}", curr_layout);
    println!("{}", curr_layout.penalize_with_details(corpus));

    let mut permutations = layout::LayoutPermutations::from_config(&config);

    let mut count = 0;
    loop {
        count += 1;

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
            .unwrap();

        // Keep going until swapping doesn't get us any more improvements.
        if curr_penalty <= best_penalty {
            break;
        } else {
            println!("Result of iteration {}:", count);
            println!("{}", best_layout);
            println!("{}", best_layout.penalize_with_details(corpus));

            curr_layout = best_layout;
            curr_penalty = best_penalty;
        }
    }

    println!("");
    println!("Ultimate winner:");
    println!("{}", curr_layout);
    println!("{}", curr_layout.penalize_with_details(corpus));
    curr_layout.write_to_file(&"refined.layout")?;
    Ok(())
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
