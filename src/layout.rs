#![allow(dead_code)]
use crate::penalty;
use crate::Result;
use itertools::Itertools;
use rand::{Rng, StdRng};
/// Data structures and methods for creating and shuffling keyboard layouts.
use std::fmt::{self, Display};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use strum_macros::EnumIter;

// KeyMap format:
//    LEFT HAND   |    RIGHT HAND
//  0  1  2  3  4 |  5  6  7  8  9 x
// 10 11 12 13 14 | 15 16 17 18 19 y
// 20 21 22 23 24 | 25 26 27 28 29
//
//             30 | 31 (thumb keys)
//             32 | 33 (extra thumb keys)

#[derive(Debug, PartialEq, Clone)]
pub struct KeyMap<T>(pub [T; 36]);

#[derive(Debug, Clone, PartialEq)]
pub struct Layer(KeyMap<char>);

#[derive(Debug, Clone, PartialEq)]
pub struct Layout(Layer, Layer);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Swap(usize, usize);

pub struct LayoutPosMap([Option<KeyPress>; 128]);

// #[derive(Clone)]
// pub struct LayoutShuffleMask(pub KeyMap<bool>);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Debug, EnumIter)]
pub enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}
impl Display for Finger {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Finger::*;
        match self {
            Thumb => write!(f, "Thumb"),
            Index => write!(f, "Index"),
            Middle => write!(f, "Middle"),
            Ring => write!(f, "Ring"),
            Pinky => write!(f, "Pinky"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter)]
pub enum Hand {
    Left,
    Right,
}
impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Hand::*;
        match self {
            Left => write!(f, "Left"),
            Right => write!(f, "Right"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Row {
    Thumb,
    Bottom,
    Home,
    Top,
}

#[derive(Clone, Copy)]
pub struct KeyPress {
    pub kc: char,
    pub pos: usize,
    pub finger: Finger,
    pub hand: Hand,
    pub row: Row,
    pub center: bool,
}

/* ------- *
 * STATICS *
 * ------- */

pub static RSTHD_LAYOUT: Layout = Layout(
    Layer(KeyMap([
        'j', 'c', 'y', 'f', 'k', 'z', 'l', ',', 'u', 'q','\\', //
        'r', 's', 't', 'h', 'd', 'm', 'n', 'a', 'i', 'o','\'', //
        '/', 'v', 'g', 'p', 'b', 'x', 'w', '.', ';', '-', //
        'e', ' ',
        '*', '*', 
    ])),
    Layer(KeyMap([
        'J', 'C', 'Y', 'F', 'K', 'Z', 'L', '<', 'U', 'Q','|', //
        'R', 'S', 'T', 'H', 'D', 'M', 'N', 'A', 'I', 'O','"', //
        '?', 'V', 'G', 'P', 'B', 'X', 'W', '>', ':', '_', //
        'E', ' ',
        '*', '*', 
    ])),
);

pub static QWERTY_LAYOUT: Layout = Layout(
    Layer(KeyMap([
        'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p','*', //
        'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';','*', //
        'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', //
        '*', ' ',
        '*', '*', 
    ])),
    Layer(KeyMap([
        'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P','*', //
        'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':','*', //
        'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?', //
        '*', ' ',
        '*', '*', 
    ])),
);

pub static DVORAK_LAYOUT: Layout = Layout(
    Layer(KeyMap([
        '/', ',', '.', 'p', 'y', 'f', 'g', 'c', 'r', 'l','*', //
        'a', 'o', 'e', 'u', 'i', 'd', 'h', 't', 'n', 's','*', //
        ';', 'q', 'j', 'k', 'x', 'b', 'm', 'w', 'v', 'z', //
        '*', ' ',
        '*', '*', 
    ])),
    Layer(KeyMap([
        '?', '<', '>', 'P', 'Y', 'F', 'G', 'C', 'R', 'L','*', //
        'A', 'O', 'E', 'U', 'I', 'D', 'H', 'T', 'N', 'S','*', //
        ':', 'Q', 'J', 'K', 'X', 'B', 'M', 'W', 'V', 'Z', //
        '*', ' ',
        '*', '*', 
    ])),
);

pub static COLEMAK_LAYOUT: Layout = Layout(
    Layer(KeyMap([
        'q', 'w', 'f', 'p', 'g', 'j', 'l', 'u', 'y', ';','*', //
        'a', 'r', 's', 't', 'd', 'h', 'n', 'e', 'i', 'o','*', //
        'z', 'x', 'c', 'v', 'b', 'k', 'm', ',', '.', '/', //
        '*', ' ',
        '*', '*', 
    ])),
    Layer(KeyMap([
        'Q', 'W', 'F', 'P', 'G', 'J', 'L', 'U', 'Y', ':','*', //
        'A', 'R', 'S', 'T', 'D', 'H', 'N', 'E', 'I', 'O','*', //
        'Z', 'X', 'C', 'V', 'B', 'K', 'M', '<', '>', '?', //
        '*', ' ',
        '*', '*', 
    ])),
);

pub static COLEMAK_DH_LAYOUT: Layout = Layout(
    Layer(KeyMap([
        'q', 'w', 'f', 'p', 'b', 'j', 'l', 'u', 'y', ';','*', //
        'a', 'r', 's', 't', 'g', 'm', 'n', 'e', 'i', 'o','*', //
        'z', 'x', 'c', 'd', 'v', 'k', 'h', ',', '.', '/', //
        '*', ' ',
        '*', '*', 
    ])),
    Layer(KeyMap([
        'Q', 'W', 'F', 'P', 'B', 'J', 'L', 'U', 'Y', ':','*', //
        'A', 'R', 'S', 'T', 'G', 'M', 'N', 'E', 'I', 'O','*', //
        'Z', 'X', 'C', 'D', 'V', 'K', 'H', '<', '>', '?', //
        '*', ' ',
        '*', '*', 
    ])),
);

pub static QGMLWY_LAYOUT: Layout = Layout(
    Layer(KeyMap([
        'q', 'g', 'm', 'l', 'w', 'y', 'f', 'u', 'b', ';','*', //
        'd', 's', 't', 'n', 'r', 'i', 'a', 'e', 'o', 'h','*', //
        'z', 'x', 'c', 'v', 'j', 'k', 'p', ',', '.', '/', //
        '*', ' ',
        '*', '*', 
    ])),
    Layer(KeyMap([
        'Q', 'G', 'M', 'L', 'W', 'Y', 'F', 'U', 'B', ':','*', //
        'D', 'S', 'T', 'N', 'R', 'I', 'A', 'E', 'O', 'H','*', //
        'Z', 'X', 'C', 'V', 'J', 'K', 'P', '<', '>', '?', //
        '*', ' ',
        '*', '*', 
    ])),
);

pub static WORKMAN_LAYOUT: Layout = Layout(
    Layer(KeyMap([
        'q', 'd', 'r', 'w', 'b', 'j', 'f', 'u', 'p', ';','*', //
        'a', 's', 'h', 't', 'g', 'y', 'n', 'e', 'o', 'i','*', //
        'z', 'x', 'm', 'c', 'v', 'k', 'l', ',', '.', '/', //
        '*', ' ',
        '*', '*', 
    ])),
    Layer(KeyMap([
        'Q', 'D', 'R', 'W', 'B', 'J', 'F', 'U', 'P', ':','*', //
        'A', 'S', 'H', 'T', 'G', 'Y', 'N', 'E', 'O', 'I','*', //
        'Z', 'X', 'M', 'C', 'V', 'K', 'L', '<', '>', '?', //
        '*', ' ',
        '*', '*', 
    ])),
);

pub static MALTRON_LAYOUT: Layout = Layout(
    Layer(KeyMap([
        'q', 'p', 'y', 'c', 'b', 'v', 'm', 'u', 'z', 'l','*', //
        'a', 'n', 'i', 's', 'f', 'd', 't', 'h', 'o', 'r','*', //
        ',', '.', 'j', 'g', '/', ';', 'w', 'k', '-', 'x', //
        'e', ' ',
        '*', '*', 
    ])),
    Layer(KeyMap([
        'Q', 'P', 'Y', 'C', 'B', 'V', 'M', 'U', 'Z', 'L','*', //
        'A', 'N', 'I', 'S', 'F', 'D', 'T', 'H', 'O', 'R','*', //
        '<', '>', 'J', 'G', '?', ':', 'W', 'K', '_', 'X', //
        'E', ' ',
        '*', '*', 
    ])),
);

pub static MTGAP_LAYOUT: Layout = Layout(
    Layer(KeyMap([
        'y', 'p', 'o', 'u', 'j', 'b', 'd', 'l', 'c', 'k','*', //
        'i', 'n', 'e', 'a', ',', 'm', 'h', 't', 's', 'r','*', //
        '-', ';', '/', '.', 'v', 'q', 'f', 'w', 'g', 'x', //
        'z', ' ',
        '*', '*', 
    ])),
    Layer(KeyMap([
        'Y', 'P', 'O', 'U', 'J', 'B', 'D', 'L', 'C', 'K','*', //
        'I', 'N', 'E', 'A', '<', 'M', 'H', 'T', 'S', 'R','*', //
        '_', ':', '?', '>', 'V', 'Q', 'F', 'W', 'G', 'X', //
        'Z', ' ',
        '*', '*', 
    ])),
);

pub static CAPEWELL_LAYOUT: Layout = Layout(
    Layer(KeyMap([
        '.', 'y', 'w', 'd', 'f', 'j', 'p', 'l', 'u', 'q','*', //
        'a', 'e', 'r', 's', 'g', 'b', 't', 'n', 'i', 'o','*', //
        'x', 'z', 'c', 'v', ';', 'k', 'w', 'h', ',', '/', //
        '*', ' ',
        '*', '*', 
    ])),
    Layer(KeyMap([
        '>', 'Y', 'W', 'D', 'F', 'J', 'P', 'L', 'U', 'Q','*', //
        'A', 'E', 'R', 'S', 'G', 'B', 'T', 'N', 'I', 'O', '*',//
        'X', 'Z', 'C', 'V', ':', 'K', 'W', 'H', '<', '?', //
        '*', ' ',
        '*', '*', 
    ])),
);

pub static ARENSITO_LAYOUT: Layout = Layout(
    Layer(KeyMap([
        'q', 'l', ',', 'p', ';', '/', 'f', 'u', 'd', 'k','*', //
        'a', 'r', 'e', 'n', 'b', 'g', 's', 'i', 't', 'o','*', //
        'z', 'w', '.', 'h', 'j', 'v', 'c', 'y', 'm', 'x', //
        '*', ' ',
        '*', '*', 
    ])),
    Layer(KeyMap([
        'Q', 'L', '<', 'P', ':', '?', 'F', 'U', 'D', 'K','*', //
        'A', 'R', 'E', 'N', 'B', 'G', 'S', 'I', 'T', 'O','*', //
        'Z', 'W', '>', 'H', 'J', 'V', 'C', 'Y', 'M', 'X', //
        '*', ' ',
        '*', '*', 
    ])),
);

static LAYOUT_MASK_NUM_SWAPPABLE: usize = 36;

#[rustfmt::skip]
static KEY_FINGERS: KeyMap<Finger> = KeyMap([
    Finger::Pinky, Finger::Ring, Finger::Middle, Finger::Index, Finger::Index, Finger::Index, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky, Finger::Pinky,
    Finger::Pinky, Finger::Ring, Finger::Middle, Finger::Index, Finger::Index, Finger::Index, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky, Finger::Pinky,
    Finger::Pinky, Finger::Ring, Finger::Middle, Finger::Index, Finger::Index, Finger::Index, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky, 
    Finger::Thumb, Finger::Thumb, 
    Finger::Thumb, Finger::Thumb, 
]);

#[rustfmt::skip]
static KEY_HANDS: KeyMap<Hand> = KeyMap([
    Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right, 
    Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right, 
    Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right, 
    Hand::Left, Hand::Right, 
    Hand::Left, Hand::Right, 
]);

#[rustfmt::skip]
static KEY_ROWS: KeyMap<Row> = KeyMap([
    Row::Top,    Row::Top,    Row::Top,    Row::Top,    Row::Top,    Row::Top,    Row::Top,    Row::Top,    Row::Top,    Row::Top, Row::Top, 
    Row::Home,   Row::Home,   Row::Home,   Row::Home,   Row::Home,   Row::Home,   Row::Home,   Row::Home,   Row::Home,   Row::Home, Row::Home,
    Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom, 
    Row::Thumb,  Row::Thumb,
    Row::Thumb,  Row::Thumb,
]);

#[rustfmt::skip]
static KEY_CENTER_COLUMN: KeyMap<bool> = KeyMap([
    false, false, false, false, true, true, false, false, false, false, false,
    false, false, false, false, true, true, false, false, false, false, false,
    false, false, false, false, true, true, false, false, false, false,
    false, false,
    false, false,
]);

#[rustfmt::skip]
static LAYOUT_FILE_IDXS: KeyMap<usize> = KeyMap([
    0,  1,  2,  3,  4,  6,  7,  8,  9,  10, 11,
    13, 14, 15, 16, 17, 19, 20, 21, 22, 23, 24,
    26, 27, 28, 29, 30, 32, 33, 34, 35, 36, 
    42, 44,
    50, 52,
]);

impl From<&Layout> for LayoutPosMap {
    fn from(layout: &Layout) -> LayoutPosMap {
        let Layout(ref lower, ref upper) = *layout;
        let mut map = [None; 128];
        lower.fill_position_map(&mut map);
        upper.fill_position_map(&mut map);
        LayoutPosMap(map)
    }
}

impl Layout {
    #[rustfmt::skip]
    pub fn write_to_file<P: AsRef<Path>>(&self, path: &P) -> Result<()> {
        let mut file = File::create(path)?;

        let lower = self.0.0.0;
        let upper = self.1.0.0;
        write!(file, "{}{}{}{}{} {}{}{}{}{}{}\n{}{}{}{}{} {}{}{}{}{}{}\n{}{}{}{}{} {}{}{}{}{}\n    {} {}\n    {} {}\n",
            lower[0],  lower[1],  lower[2],  lower[3],  lower[4],  lower[5],  lower[6],  lower[7],  lower[8],  lower[9],
            lower[10], lower[11], lower[12], lower[13], lower[14], lower[15], lower[16], lower[17], lower[18], lower[19],
            lower[20], lower[21], lower[22], lower[23], lower[24], lower[25], lower[26], lower[27], lower[28], lower[29],
            lower[30], lower[31],
            lower[32], lower[33],
            lower[34], lower[35],
        )?;
        write!(file, "{}{}{}{}{} {}{}{}{}{}{}\n{}{}{}{}{} {}{}{}{}{}{}\n{}{}{}{}{} {}{}{}{}{}\n    {} {}\n    {} {}\n",
            upper[0],  upper[1],  upper[2],  upper[3],  upper[4],  upper[5],  upper[6],  upper[7],  upper[8],  upper[9],
            upper[10], upper[11], upper[12], upper[13], upper[14], upper[15], upper[16], upper[17], upper[18], upper[19],
            upper[20], upper[21], upper[22], upper[23], upper[24], upper[25], upper[26], upper[27], upper[28], upper[29],
            upper[30], upper[31],
            upper[32], upper[33],
            upper[34], upper[35],
        )?;
        Ok(())
    }

    pub fn from_string(s: &str) -> Option<Layout> {
        let s: Vec<char> = s.chars().collect();
        let mut lower: [char; 36] = ['\0'; 36];
        let mut upper: [char; 36] = ['\0'; 36];

        for i in 0..36 {
            let file_i = LAYOUT_FILE_IDXS.0[i];
            lower[i] = *s.get(file_i)?;
            upper[i] = *s.get(file_i + 54)?;
        }

        Some(Layout(Layer(KeyMap(lower)), Layer(KeyMap(upper))))
    }

    pub fn shuffle(&mut self, times: usize, rng: &mut StdRng) {
        for _ in 0..times {
            let (i, j) = Layout::shuffle_position(rng);
            if penalty::LAYOUT_MASK.0[i] && penalty::LAYOUT_MASK.0[j] {
                let Layout(ref mut lower, ref mut upper) = *self;
                lower.swap(i, j);
                upper.swap(i, j);
            }
        }
    }

    pub fn get_position_map(&self) -> LayoutPosMap {
        let Layout(ref lower, ref upper) = *self;
        let mut map = [None; 128];
        lower.fill_position_map(&mut map);
        upper.fill_position_map(&mut map);

        LayoutPosMap(map)
    }

    fn shuffle_position(rng: &mut StdRng) -> (usize, usize) {
        let i = rng.gen_range(0, LAYOUT_MASK_NUM_SWAPPABLE);
        let mut j = rng.gen_range(0, LAYOUT_MASK_NUM_SWAPPABLE-1);
        if j >= i {
            j += 1;
        }

        (i, j)
    }
}

impl Layer {
    fn swap(&mut self, i: usize, j: usize) {
        let Layer(KeyMap(ref mut layer)) = *self;
        layer.swap(i, j);
    }

    fn fill_position_map(&self, map: &mut [Option<KeyPress>; 128]) {
        let Layer(KeyMap(ref layer)) = *self;
        let KeyMap(ref fingers) = KEY_FINGERS;
        let KeyMap(ref hands) = KEY_HANDS;
        let KeyMap(ref rows) = KEY_ROWS;
        let KeyMap(ref centers) = KEY_CENTER_COLUMN;
        // Ignore null characters since non-existing keys are internally
        // represented by such.
        for (i, c) in layer.into_iter().enumerate() {
            if (0 as char) < *c && *c < (128 as char) {
                map[*c as usize] = Some(KeyPress {
                    kc: *c,
                    pos: i,
                    finger: fingers[i],
                    hand: hands[i],
                    row: rows[i],
                    center: centers[i],
                });
            }
        }
    }
}

impl LayoutPosMap {
    pub fn get_key_position(&self, kc: char) -> Option<&KeyPress> {
        self.0.get(kc as usize).unwrap_or(&None).as_ref()
    }
}

pub struct LayoutPermutations {
    orig_layout: Layout,
    swaps_per_iteration: usize,
    swaps: Vec<(usize, usize)>,
}

impl LayoutPermutations {
    pub fn from_config(config: &crate::app::Config) -> LayoutPermutations {
        LayoutPermutations {
            orig_layout: config.layout.clone(),
            swaps: std::iter::once((0, 0))
                .chain(
                    (0..LAYOUT_MASK_NUM_SWAPPABLE)
                        .map(|n| (0..n).zip(std::iter::repeat(n)))
                        .flatten(),
                )
                .filter(|(i, j)| penalty::LAYOUT_MASK.0[*i] && penalty::LAYOUT_MASK.0[*j])
                .collect(),
            swaps_per_iteration: config.swaps,
        }
    }

    pub fn set_layout(&mut self, layout: &Layout) {
        self.orig_layout = layout.clone();
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Layout> + Send + 'a {
        self.swaps
            .iter()
            .permutations(self.swaps_per_iteration)
            .map(move |perm: Vec<&(usize, usize)>| {
                let mut layout = self.orig_layout.clone();
                let ref mut lower = ((layout.0).0).0;
                let ref mut upper = ((layout.1).0).0;
                perm.iter().for_each(|(i, j)| {
                    lower.swap(*i, *j);
                    upper.swap(*i, *j);
                });
                layout
            })
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Layout(ref lower, ref _upper) = *self;
        lower.fmt(f)
        // lower.fmt(f)?;
        // writeln!(f)?;
        // _upper.fmt(f)
    }
}

impl Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Layer(KeyMap(ref layer)) = *self;
        let layer: Vec<_> = layer
            .iter()
            .map(|c| if *c != '\0' { *c } else { '☐' })
            .collect();
        write!(
            f,
            "\
{} {} {} {} {} | {} {} {} {} {} {}
{} {} {} {} {} | {} {} {} {} {} {}
{} {} {} {} {} | {} {} {} {} {}
        {} | {}
        {} | {}",
            layer[0],
            layer[1],
            layer[2],
            layer[3],
            layer[4],
            layer[5],
            layer[6],
            layer[7],
            layer[8],
            layer[9],
            layer[10],
            layer[11],
            layer[12],
            layer[13],
            layer[14],
            layer[15],
            layer[16],
            layer[17],
            layer[18],
            layer[19],
            layer[20],
            layer[21],
            layer[22],
            layer[23],
            layer[24],
            layer[25],
            layer[26],
            layer[27],
            layer[28],
            layer[29],
            layer[30],
            layer[31],
            layer[32],
            layer[33],
            layer[34],
            layer[35]
        )
    }
}
