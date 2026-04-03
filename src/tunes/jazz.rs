// TODO: Autumn Leaves — the descending ii-V-I melody
// TODO: All Blues (Miles Davis) — 6/8 blues head
// TODO: Misty (Erroll Garner) — ballad standard
// TODO: Blue Bossa (Kenny Dorham) — latin jazz head
// TODO: Donna Lee (Charlie Parker) — bebop head
// TODO: Cantaloupe Island (Herbie Hancock) — funky vamp

use super::{Buf, beat, hour_to_root, note_freq};
use clap::ValueEnum;
use rodio::Player;
use std::fmt;

#[derive(Clone, Copy, ValueEnum)]
pub enum JazzTune {
    Lick,
    SoWhat,
    BlueMonk,
    TakeFive,
    GiantSteps,
    Moanin,
}

impl fmt::Display for JazzTune {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JazzTune::Lick => write!(f, "lick"),
            JazzTune::SoWhat => write!(f, "so_what"),
            JazzTune::BlueMonk => write!(f, "blue_monk"),
            JazzTune::TakeFive => write!(f, "take_five"),
            JazzTune::GiantSteps => write!(f, "giant_steps"),
            JazzTune::Moanin => write!(f, "moanin"),
        }
    }
}

pub const JAZZ_BY_HOUR: [JazzTune; 6] = [
    JazzTune::Lick,
    JazzTune::SoWhat,
    JazzTune::BlueMonk,
    JazzTune::TakeFive,
    JazzTune::GiantSteps,
    JazzTune::Moanin,
];

/// The Lick — the most famous jazz phrase.
/// D E F G E C D (in D dorian). Swing eighths.
const LICK: &[(i32, f32)] = &[
    (0, 0.67),  // D  — long swing eighth
    (2, 0.33),  // E  — short swing eighth
    (3, 0.67),  // F  — long
    (5, 0.33),  // G  — short
    (2, 1.0),   // E  — quarter
    (-2, 1.0),  // C  — quarter
    (0, 2.0),   // D  — half
];

/// So What — Miles Davis. The bass riff ascending D dorian.
const SO_WHAT: &[(i32, f32)] = &[
    (-5, 1.0),  // A  — quarter (bass pickup)
    (0, 0.5),   // D  — eighth
    (2, 0.5),   // E  — eighth
    (3, 1.0),   // F  — quarter
    (5, 1.0),   // G  — quarter
    (7, 2.0),   // A  — half
    (5, 1.0),   // G  — quarter
    (3, 1.0),   // F  — quarter
    (0, 2.0),   // D  — half
];

/// Blue Monk — Thelonious Monk. Bluesy chromatic walk-up in Bb.
const BLUE_MONK: &[(i32, f32)] = &[
    (0, 1.0),   // Bb — quarter
    (2, 1.0),   // C  — quarter
    (3, 0.5),   // Db — eighth
    (4, 0.5),   // D  — eighth
    (0, 2.0),   // Bb — half
    (0, 1.0),   // Bb — quarter
    (2, 1.0),   // C  — quarter
    (3, 0.5),   // Db — eighth
    (4, 0.5),   // D  — eighth
    (0, 2.0),   // Bb — half
];

/// Take Five — Dave Brubeck. The alto sax riff in 5/4, Eb minor.
const TAKE_FIVE: &[(i32, f32)] = &[
    (0, 1.0),   // Eb — quarter
    (-5, 0.5),  // Bb — eighth (below)
    (-7, 0.5),  // Ab — eighth
    (-5, 1.0),  // Bb — quarter
    (0, 1.0),   // Eb — quarter (beat 4)
    (-2, 1.0),  // Db — quarter (beat 5)
    (-3, 1.0),  // C  — quarter
    (-5, 2.0),  // Bb — half
];

/// Giant Steps — John Coltrane. The head that haunts jazz students.
/// B major → G major → Eb major, cycling in major thirds.
const GIANT_STEPS: &[(i32, f32)] = &[
    (0, 0.5),   // B  — eighth
    (4, 0.5),   // D# — eighth
    (-4, 1.0),  // G  — quarter
    (-1, 0.5),  // Bb — eighth
    (3, 0.5),   // Eb — eighth
    (3, 1.0),   // Eb — quarter
    (-2, 0.5),  // A  — eighth
    (1, 0.5),   // C  — eighth
    (-5, 2.0),  // F# — half
];

/// Moanin' — Bobby Timmons / Art Blakey. Gospel-blues call in F minor.
const MOANIN: &[(i32, f32)] = &[
    (0, 0.67),  // F  — long swing eighth
    (3, 0.33),  // Ab — short
    (5, 0.67),  // Bb — long
    (6, 0.33),  // B  — short (blue note)
    (7, 2.0),   // C  — half (response)
    (5, 0.5),   // Bb — eighth
    (3, 0.5),   // Ab — eighth
    (0, 2.0),   // F  — half
];

impl JazzTune {
    fn notes(self) -> &'static [(i32, f32)] {
        match self {
            JazzTune::Lick => LICK,
            JazzTune::SoWhat => SO_WHAT,
            JazzTune::BlueMonk => BLUE_MONK,
            JazzTune::TakeFive => TAKE_FIVE,
            JazzTune::GiantSteps => GIANT_STEPS,
            JazzTune::Moanin => MOANIN,
        }
    }

    fn default_bpm(self) -> f32 {
        match self {
            JazzTune::Lick => 160.0,
            JazzTune::SoWhat => 136.0,
            JazzTune::BlueMonk => 120.0,
            JazzTune::TakeFive => 172.0,
            JazzTune::GiantSteps => 280.0,
            JazzTune::Moanin => 140.0,
        }
    }
}

pub fn play_jazz(player: &Player, vol: f32, bpm: Option<f32>, hour: u32, tune: &JazzTune) {
    let root = hour_to_root(hour);
    let bpm = bpm.unwrap_or(tune.default_bpm());

    let mut buf = Buf::new();
    for &(interval, beats) in tune.notes() {
        buf.sine_lp(
            note_freq(root + interval),
            beat(beats, bpm),
            2500.0,
            vol,
        );
    }
    buf.play(player);
}
