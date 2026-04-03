use super::{Buf, beat};
use clap::ValueEnum;
use rodio::Player;
use std::fmt;

const DEFAULT_BPM: f32 = 200.0;

#[derive(Clone, ValueEnum)]
pub enum BeepPattern {
    Success,
    Failure,
}

impl fmt::Display for BeepPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BeepPattern::Success => write!(f, "success"),
            BeepPattern::Failure => write!(f, "failure"),
        }
    }
}

/// The OG beep. Ascending tones for success, descending for failure.
/// (frequency_hz, beats)
const SUCCESS: &[(f32, f32)] = &[
    (440.0, 1.0),
    (460.0, 1.0),
    (480.0, 1.0),
    (500.0, 1.0),
    (520.0, 2.33),
];

const FAILURE: &[(f32, f32)] = &[
    (520.0, 1.0),
    (500.0, 1.0),
    (480.0, 1.0),
    (460.0, 1.0),
    (440.0, 2.33),
];

pub fn play_beep(player: &Player, vol: f32, bpm: Option<f32>, pattern: &BeepPattern) {
    let bpm = bpm.unwrap_or(DEFAULT_BPM);
    let tones = match pattern {
        BeepPattern::Success => SUCCESS,
        BeepPattern::Failure => FAILURE,
    };

    let mut buf = Buf::new();
    for &(freq, beats) in tones {
        buf.sine_lp(freq, beat(beats, bpm), 2000.0, vol);
    }
    buf.play(player);
}
