/*****************
 * Configuration *
 *****************/

// Default layout
pub static INIT_LAYOUT: &Layout = &layout::RSTHD_LAYOUT;

// Base penalty.
const BASE_PENALTY_MULTIPLICATOR: f64 = 1.0;
static BASE_PENALTY: KeyMap<f64> = KeyMap([
    2.5, 0.5, 0.5, 1.0, 2.5, 2.5, 1.0, 0.5, 0.5, 2.5, //
    1.0, 0.0, 0.0, 0.0, 2.0, 2.0, 0.0, 0.0, 0.0, 1.0, //
    1.5, 1.5, 1.0, 0.5, 3.0, 3.0, 0.5, 1.0, 1.5, 1.5, //
    0.0, 0.0,
]);
//
// Penalise 5 points for using the same finger twice on different keys.
// An extra 5 points for each usage of the center row.
const SAME_FINGER_PENALTY: Option<f64> = Some(10.0);
//
// Penalise 10 points for jumping from top to bottom row or from bottom to
// top row on the same finger.
const LONG_JUMP_PENALTY: Option<f64> = Some(5.0);
//
// Penalise 1 point for jumping from top to bottom row or from bottom to
// top row on the same hand.
const LONG_JUMP_HAND_PENALTY: Option<f64> = Some(1.0);
//
// Penalise 5 points for jumping from top to bottom row or from bottom to
// top row on consecutive fingers, except for middle finger-top row ->
// index finger-bottom row.
const LONG_JUMP_CONSECUTIVE_PENALTY: Option<f64> = Some(5.0);
//
// TODO
const RING_STRETCH_PENALTY: Option<f64> = Some(20.0);
//
// Penalise if pinky follows ring finger (inprecise)
const PINKY_RING_PENALTY: Option<f64> = Some(5.0);
//
// Penalise 10 points for awkward pinky/ring combination where the pinky
// reaches above the ring finger, e.g. QA/AQ, PL/LP, ZX/XZ, ;./.; on Qwerty.
const PINKY_RING_TWIST_PENALTY: Option<f64> = Some(10.0);
//
// Penalise 20 points for reversing a roll at the end of the hand, i.e.
// using the ring, pinky, then middle finger of the same hand, or the
// middle, pinky, then ring of the same hand.
const ROLL_REVERSAL_PENALTY: Option<f64> = Some(10.0);
//
// Penalise 0.5 points for using the same hand four times in a row.
const SAME_HAND_PENALTY: Option<f64> = Some(0.5);
//
// Penalise 0.5 points for alternating hands three times in a row.
const ALTERNATING_HAND_PENALTY: Option<f64> = Some(0.5);
//
// Penalise 0.125 points for rolling outwards.
const ROLL_OUT_PENALTY: Option<f64> = Some(0.125);
//
// Award 0.125 points for rolling inwards.
const ROLL_IN_PENALTY: Option<f64> = Some(-0.125);
//
// Penalise 3 points for using the same finger on different keys
// with one key in between ("detached same finger bigram").
// An extra 3 points for each usage of the center row.
const SFB_SANDWICH_PENALTY: Option<f64> = Some(10.0);
//
// Penalise 3 points for jumping from top to bottom row or from bottom to
// top row on the same finger with a keystroke in between.
const LONG_JUMP_SANDWICH_PENALTY: Option<f64> = Some(5.0);
//
// Penalise 10 points for three consecutive keystrokes going up or down
// (currently only down) the three rows of the keyboard in a roll.
const TWIST_PENALTY: Option<f64> = Some(10.0);

/*********************
 * Configuration end *
 *********************/

use rayon::prelude::*;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Range;
/// Methods for calculating the penalty of a keyboard layout given an input
/// corpus string.
use std::vec::Vec;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::layout::{self, Finger, KeyMap, KeyPress, Layout, LayoutPosMap, Row};

pub struct KeyPressQuartad<'a> {
    curr: &'a KeyPress,
    old1: &'a KeyPress,
    old2: &'a KeyPress,
    old3: &'a KeyPress,
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct Quartad<'a>(&'a str);
impl Quartad<'_> {
    pub fn get_kp_quartad<'a>(&self, pos_map: &'a LayoutPosMap) -> Option<KeyPressQuartad<'a>> {
        let mut chars = self.0.chars().rev().map(|c| pos_map.get_key_position(c));
        Some(KeyPressQuartad {
            curr: chars.next().flatten()?,
            old1: chars.next().flatten()?,
            old2: chars.next().flatten()?,
            old3: chars.next().flatten()?,
        })
    }
}

pub struct Corpus<'a> {
    pub quartads: HashMap<Quartad<'a>, usize>,
    pub len: usize,
}
impl<'a> From<&'a str> for Corpus<'a> {
    fn from(string: &'a str) -> Self {
        let mut range: Range<usize> = 0..0;
        let mut quartads: HashMap<Quartad<'a>, usize> = HashMap::new();
        let position_map = INIT_LAYOUT.get_position_map();

        for (i, c) in string.chars().enumerate() {
            match position_map.get_key_position(c) {
                Some(_) => {
                    range.end = i + 1;
                    if range.end - range.start >= 4 {
                        range.start = range.end - 4;
                        let quartad = Quartad(&string[range.clone()]);
                        let entry = quartads.entry(quartad).or_insert(0);
                        *entry += 1;
                    };
                }
                None => {
                    range = (i + 1)..(i + 1);
                }
            }
        }

        Corpus {
            len: string.len(),
            quartads,
        }
    }
}

impl Layout {
    #[inline(always)]
    pub fn penalize(&self, corpus: &Corpus) -> f64 {
        let pos_map = self.get_position_map();
        corpus
            .quartads
            .iter()
            .map(|(quartad, &count)| {
                let mut total = TotalPenalty::new();
                let kp_quartad = quartad.get_kp_quartad(&pos_map)?;
                penalize_kp_quartad(&kp_quartad, &mut total);
                Some(count as f64 * total.value)
            })
            .flatten()
            .sum()
    }

    #[inline(always)]
    pub fn par_penalize(&self, corpus: &Corpus) -> f64 {
        let pos_map = self.get_position_map();
        corpus
            .quartads
            .par_iter()
            .map(|(quartad, &count)| {
                let mut total = TotalPenalty::new();
                let kp_quartad = quartad.get_kp_quartad(&pos_map)?;
                penalize_kp_quartad(&kp_quartad, &mut total);
                Some(count as f64 * total.value)
            })
            .flatten()
            .sum()
    }

    pub fn penalize_with_details<'a>(&self, corpus: &'a Corpus) -> LayoutPenalty<'a> {
        let pos_map = self.get_position_map();

        let mut total = 0.0;
        let mut high_keys: HashMap<PenaltyVar, HashMap<&'a str, f64>> = HashMap::new();

        corpus
            .quartads
            .iter()
            .map(|(quartad, &count)| {
                let kc_quartad = quartad.get_kp_quartad(&pos_map)?;
                let mut details = DetailedPenalty::new(&quartad, count);

                penalize_kp_quartad(&kc_quartad, &mut details);
                Some(details)
            })
            .flatten()
            .for_each(|details| {
                details.value.into_iter().for_each(|(pen, (s, v))| {
                    let pen_high_keys = high_keys.entry(pen).or_insert(HashMap::new());
                    let entry = pen_high_keys.entry(s).or_insert(0.0);
                    *entry += v;
                    total += v;
                });
            });

        LayoutPenalty {
            total,
            high_keys,
            scaled: total / corpus.len as f64,
        }
    }
}

pub struct LayoutPenalty<'a> {
    pub total: f64,
    pub scaled: f64,
    pub high_keys: HashMap<PenaltyVar, HashMap<&'a str, f64>>,
}

impl Display for LayoutPenalty<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "total: {}, scaled: {}", self.total, self.scaled)?;

        PenaltyVar::iter()
            .map(|var| self.high_keys.get(&var).zip(Some(var)))
            .flatten()
            .map(|(pen_high_keys, var)| {
                let mut pen_high_keys: Vec<(&str, f64)> =
                    pen_high_keys.iter().map(|(&s, &v)| (s, v)).collect();
                pen_high_keys.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                let pen_total: f64 = pen_high_keys.iter().map(|&(_, x)| -> f64 { x }).sum();
                write!(f, "{:22}: {:11} ", format!("{}", var), pen_total)?;
                pen_high_keys
                    .iter()
                    .take(5)
                    .map(|(chars, pen)| write!(f, " | {:4} : {:10}", chars, pen))
                    .collect::<Result<(), std::fmt::Error>>()?;
                writeln!(f, "")
            })
            .collect()
    }
}

fn penalize_kp_quartad<T>(kp_quartad: &KeyPressQuartad, total_penalty: &mut T)
where
    T: PenaltyAccumulator,
{
    let KeyPressQuartad {
        curr,
        old1,
        old2,
        old3,
    } = kp_quartad;

    total_penalty.add(penalties::base(&kp_quartad));

    if curr.hand == old1.hand {
        total_penalty.add(penalties::same_finger(&kp_quartad));
        total_penalty.add(penalties::long_jump(&kp_quartad));
        total_penalty.add(penalties::long_jump_hand(&kp_quartad));
        total_penalty.add(penalties::long_jump_consecutive(&kp_quartad));
        total_penalty.add(penalties::pinky_ring(&kp_quartad));
        total_penalty.add(penalties::pinky_ring_twist(&kp_quartad));
        total_penalty.add(penalties::roll_out(&kp_quartad));
        total_penalty.add(penalties::roll_in(&kp_quartad));
    }

    if curr.hand == old1.hand && old1.hand == old2.hand {
        total_penalty.add(penalties::ring_stretch(&kp_quartad));
        total_penalty.add(penalties::roll_reversal(&kp_quartad));
        total_penalty.add(penalties::twist(&kp_quartad));
    }

    if curr.hand == old2.hand && curr.finger == old2.finger {
        total_penalty.add(penalties::same_finger_sandwich(&kp_quartad));
        total_penalty.add(penalties::long_jump_sandwich(&kp_quartad));
    }

    if curr.hand == old1.hand && old1.hand == old2.hand && old2.hand == old3.hand {
        total_penalty.add(penalties::same_hand(&kp_quartad));
    } else if curr.hand != old1.hand && old1.hand != old2.hand && old2.hand != old3.hand {
        total_penalty.add(penalties::alternating_hand(&kp_quartad));
    }
}

#[derive(EnumIter, PartialEq, Eq, Hash)]
pub enum PenaltyVar {
    Base,
    SameFinger,
    LongJump,
    LongJumpHand,
    LongJumpConsecutive,
    PinkyRing,
    PinkyRingTwist,
    RollOut,
    RollIn,
    RingStretch,
    RollReversal,
    Twist,
    SameFingerSandwich,
    LongJumpSandwich,
    SameHand,
    AlternatingHand,
}

impl Display for PenaltyVar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use PenaltyVar::*;
        match self {
            Base => write!(f, "{}", "Base"),
            SameFinger => write!(f, "{}", "Same Finger"),
            LongJump => write!(f, "{}", "Long Jump"),
            LongJumpHand => write!(f, "{}", "Long Jump Hand"),
            LongJumpConsecutive => write!(f, "{}", "Long Jump Consecutive"),
            PinkyRing => write!(f, "{}", "Pinky Follows Ring"),
            PinkyRingTwist => write!(f, "{}", "Pinky Ring Twist"),
            RollOut => write!(f, "{}", "Roll Out"),
            RollIn => write!(f, "{}", "Roll In"),
            RingStretch => write!(f, "{}", "Ring Stretch"),
            RollReversal => write!(f, "{}", "Roll Reversal"),
            Twist => write!(f, "{}", "Twist"),
            SameFingerSandwich => write!(f, "{}", "Same Finger Sandwich"),
            LongJumpSandwich => write!(f, "{}", "Long Jump Sandwich"),
            SameHand => write!(f, "{}", "Same Hand"),
            AlternatingHand => write!(f, "{}", "Alternating Hand"),
        }
    }
}

mod penalties {
    use super::*;
    use PenaltyVar::*;

    #[inline(always)]
    pub fn base(kp_quartad: &KeyPressQuartad) -> Penalty {
        let KeyPressQuartad { curr, .. } = kp_quartad;
        Penalty {
            kind: Base,
            relevant_keys: 1,
            value: Some(BASE_PENALTY_MULTIPLICATOR * BASE_PENALTY.0[curr.pos]),
        }
    }

    #[inline(always)]
    pub fn same_finger(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old1.hand
        let KeyPressQuartad { curr, old1, .. } = kp_quartad;
        Penalty {
            kind: SameFinger,
            relevant_keys: 2,
            value: if curr.finger == old1.finger && curr.pos != old1.pos {
                SAME_FINGER_PENALTY.map(|p| {
                    p * (1.0
                        + if curr.center { 1.0 } else { 0.0 }
                        + if old1.center { 1.0 } else { 0.0 })
                })
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn long_jump(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old1.hand
        let KeyPressQuartad { curr, old1, .. } = kp_quartad;
        Penalty {
            kind: LongJump,
            relevant_keys: 2,
            value: if curr.finger == old1.finger
                && (curr.row == Row::Top && old1.row == Row::Bottom
                    || curr.row == Row::Bottom && old1.row == Row::Top)
            {
                LONG_JUMP_PENALTY
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn long_jump_hand(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old1.hand
        let KeyPressQuartad { curr, old1, .. } = kp_quartad;
        Penalty {
            kind: LongJumpHand,
            relevant_keys: 2,
            value: if curr.row == Row::Top && old1.row == Row::Bottom
                || curr.row == Row::Bottom && old1.row == Row::Top
            {
                LONG_JUMP_HAND_PENALTY
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn long_jump_consecutive(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old1.hand
        let KeyPressQuartad { curr, old1, .. } = kp_quartad;
        Penalty {
            kind: LongJumpConsecutive,
            relevant_keys: 2,
            value: if curr.row == Row::Top && old1.row == Row::Bottom
                || curr.row == Row::Bottom && old1.row == Row::Top
            {
                if curr.finger == Finger::Ring && old1.finger == Finger::Pinky
                    || curr.finger == Finger::Pinky && old1.finger == Finger::Ring
                    || curr.finger == Finger::Middle && old1.finger == Finger::Ring
                    || curr.finger == Finger::Ring && old1.finger == Finger::Middle
                    || (curr.finger == Finger::Index
                        && (old1.finger == Finger::Middle || old1.finger == Finger::Ring)
                        && curr.row == Row::Top
                        && old1.row == Row::Bottom)
                {
                    LONG_JUMP_CONSECUTIVE_PENALTY
                } else {
                    None
                }
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn pinky_ring(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old1.hand
        let KeyPressQuartad { curr, old1, .. } = kp_quartad;
        Penalty {
            kind: PinkyRing,
            relevant_keys: 2,
            value: if curr.finger == Finger::Pinky && old1.finger == Finger::Ring {
                PINKY_RING_PENALTY
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn pinky_ring_twist(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old1.hand
        let KeyPressQuartad { curr, old1, .. } = kp_quartad;
        Penalty {
            kind: PinkyRingTwist,
            relevant_keys: 2,
            value: if (curr.finger == Finger::Ring
                && old1.finger == Finger::Pinky
                && (curr.row < old1.row))
                || (curr.finger == Finger::Pinky
                    && old1.finger == Finger::Ring
                    && (old1.row < curr.row))
            {
                PINKY_RING_TWIST_PENALTY
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn roll_out(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old1.hand
        let KeyPressQuartad { curr, old1, .. } = kp_quartad;
        Penalty {
            kind: RollOut,
            relevant_keys: 2,
            value: if old1.finger != Finger::Thumb && curr.finger > old1.finger {
                ROLL_OUT_PENALTY
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn roll_in(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old1.hand
        let KeyPressQuartad { curr, old1, .. } = kp_quartad;
        Penalty {
            kind: RollIn,
            relevant_keys: 2,
            value: if old1.finger > curr.finger {
                ROLL_IN_PENALTY
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn ring_stretch(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old1.hand == old2.hand
        let KeyPressQuartad {
            curr, old1, old2, ..
        } = kp_quartad;
        Penalty {
            kind: RingStretch,
            relevant_keys: 3,
            value: if ((curr.finger == Finger::Middle
                && old1.finger == Finger::Ring
                && old2.finger == Finger::Pinky)
                || (curr.finger == Finger::Pinky
                    && old1.finger == Finger::Ring
                    && old2.finger == Finger::Middle))
                && old1.row > curr.row
                && old1.row > old2.row
            {
                RING_STRETCH_PENALTY
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn roll_reversal(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old1.hand == old2.hand
        let KeyPressQuartad {
            curr, old1, old2, ..
        } = kp_quartad;
        Penalty {
            kind: RollReversal,
            relevant_keys: 3,
            value: if (curr.finger == Finger::Middle
                && old1.finger == Finger::Pinky
                && old2.finger == Finger::Ring)
                || curr.finger == Finger::Ring
                    && old1.finger == Finger::Pinky
                    && old2.finger == Finger::Middle
            {
                ROLL_REVERSAL_PENALTY
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn twist(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old1.hand == old2.hand
        let KeyPressQuartad {
            curr, old1, old2, ..
        } = kp_quartad;
        Penalty {
            kind: Twist,
            relevant_keys: 3,
            value: if (curr.row == Row::Bottom && old1.row == Row::Home && old2.row == Row::Top
                || curr.row == Row::Top && old1.row == Row::Home && old2.row == Row::Bottom)
                && ((is_roll_out(curr.finger, old1.finger)
                    && is_roll_out(old1.finger, old2.finger))
                    || (is_roll_in(curr.finger, old1.finger)
                        && is_roll_in(old1.finger, old2.finger)))
            {
                TWIST_PENALTY
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn same_finger_sandwich(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old2.hand && curr.finger == old2.finger
        let KeyPressQuartad { curr, old2, .. } = kp_quartad;
        Penalty {
            kind: SameFingerSandwich,
            relevant_keys: 3,
            value: if curr.pos != old2.pos {
                SFB_SANDWICH_PENALTY.map(|p| {
                    p * (1.0
                        + if curr.center { 1.0 } else { 0.0 }
                        + if old2.center { 1.0 } else { 0.0 })
                })
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn long_jump_sandwich(kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old2.hand && curr.finger == old2.finger
        let KeyPressQuartad { curr, old2, .. } = kp_quartad;
        Penalty {
            kind: LongJumpSandwich,
            relevant_keys: 3,
            value: if curr.row == Row::Top && old2.row == Row::Bottom
                || curr.row == Row::Bottom && old2.row == Row::Top
            {
                LONG_JUMP_SANDWICH_PENALTY
            } else {
                None
            },
        }
    }

    #[inline(always)]
    pub fn same_hand(_kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand == old1.hand == old2.hand == old3.hand
        Penalty {
            kind: SameHand,
            relevant_keys: 4,
            value: SAME_HAND_PENALTY,
        }
    }

    #[inline(always)]
    pub fn alternating_hand(_kp_quartad: &KeyPressQuartad) -> Penalty {
        // Assumes curr.hand != old1.hand != old2.hand != old3.hand
        Penalty {
            kind: AlternatingHand,
            relevant_keys: 4,
            value: ALTERNATING_HAND_PENALTY,
        }
    }
}

pub struct Penalty {
    kind: PenaltyVar,
    relevant_keys: usize,
    value: Option<f64>,
}

trait PenaltyAccumulator {
    fn add(&mut self, penalty: Penalty) -> Option<()>;
}

struct TotalPenalty {
    value: f64,
}
impl TotalPenalty {
    pub fn new() -> TotalPenalty {
        TotalPenalty { value: 0.0 }
    }
}

impl PenaltyAccumulator for TotalPenalty {
    fn add(&mut self, penalty: Penalty) -> Option<()> {
        self.value += penalty.value?;
        Some(())
    }
}

struct DetailedPenalty<'a> {
    quartad: &'a Quartad<'a>,
    count: usize,
    value: Vec<(PenaltyVar, (&'a str, f64))>,
}
impl<'a> DetailedPenalty<'a> {
    pub fn new(quartad: &'a Quartad<'a>, count: usize) -> DetailedPenalty<'a> {
        DetailedPenalty {
            count,
            quartad,
            value: Vec::new(),
        }
    }
}

impl<'a> PenaltyAccumulator for DetailedPenalty<'a> {
    fn add(&mut self, penalty: Penalty) -> Option<()> {
        let s = &self.quartad.0[4 - penalty.relevant_keys..4];
        self.value
            .push((penalty.kind, (s, self.count as f64 * penalty.value?)));
        Some(())
    }
}

fn is_roll_out(curr: Finger, prev: Finger) -> bool {
    curr > prev
}

fn is_roll_in(curr: Finger, prev: Finger) -> bool {
    prev > curr
}
