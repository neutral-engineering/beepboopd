use super::{Buf, beat, note_freq};
use clap::ValueEnum;
use rodio::Player;
use std::fmt;

const DEFAULT_BPM: f32 = 110.0;

/// Ocarina note pitches as semitones from A4.
#[derive(Clone, Copy)]
enum Pitch {
    D4 = -7,
    F4 = -4,
    A4 = 0,
    B4 = 2,
    D5 = 5,
}

/// Musical note durations in beats.
#[derive(Clone, Copy)]
enum Len {
    Eighth,     // 0.5 beats
    Quarter,    // 1.0 beats
    DotQuarter, // 1.5 beats
    Half,       // 2.0 beats
    DotHalf,    // 3.0 beats
}

impl Len {
    const fn beats(self) -> f32 {
        match self {
            Len::Eighth => 0.5,
            Len::Quarter => 1.0,
            Len::DotQuarter => 1.5,
            Len::Half => 2.0,
            Len::DotHalf => 3.0,
        }
    }
}

use Len::*;
use Pitch::*;

#[derive(Clone, Copy, ValueEnum)]
pub enum ZeldaSong {
    Lullaby,
    Epona,
    Saria,
    Sun,
    Time,
    Storms,
    Minuet,
    Bolero,
    Serenade,
    Nocturne,
    Requiem,
    Prelude,
}

impl fmt::Display for ZeldaSong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZeldaSong::Lullaby => write!(f, "lullaby"),
            ZeldaSong::Epona => write!(f, "epona"),
            ZeldaSong::Saria => write!(f, "saria"),
            ZeldaSong::Sun => write!(f, "sun"),
            ZeldaSong::Time => write!(f, "time"),
            ZeldaSong::Storms => write!(f, "storms"),
            ZeldaSong::Minuet => write!(f, "minuet"),
            ZeldaSong::Bolero => write!(f, "bolero"),
            ZeldaSong::Serenade => write!(f, "serenade"),
            ZeldaSong::Nocturne => write!(f, "nocturne"),
            ZeldaSong::Requiem => write!(f, "requiem"),
            ZeldaSong::Prelude => write!(f, "prelude"),
        }
    }
}

pub const ZELDA_BY_HOUR: [ZeldaSong; 12] = [
    ZeldaSong::Time,     // 00, 12
    ZeldaSong::Nocturne, // 01, 13
    ZeldaSong::Requiem,  // 02, 14
    ZeldaSong::Lullaby,  // 03, 15
    ZeldaSong::Serenade, // 04, 16
    ZeldaSong::Minuet,   // 05, 17
    ZeldaSong::Prelude,  // 06, 18
    ZeldaSong::Sun,      // 07, 19
    ZeldaSong::Epona,    // 08, 20
    ZeldaSong::Saria,    // 09, 21
    ZeldaSong::Bolero,   // 10, 22
    ZeldaSong::Storms,   // 11, 23
];

impl ZeldaSong {
    fn notes(&self) -> &'static [(Pitch, Len)] {
        match self {
            // 3/4 waltz: half, quarter, dotted half
            ZeldaSong::Lullaby => &[
                (B4, Half),
                (D5, Quarter),
                (A4, DotHalf),
                (B4, Half),
                (D5, Quarter),
                (A4, DotHalf),
            ],
            // 3/4: half, half, dotted half
            ZeldaSong::Epona => &[
                (D5, Half),
                (B4, Half),
                (A4, DotHalf),
                (D5, Half),
                (B4, Half),
                (A4, DotHalf),
            ],
            // 4/4 bouncy: eighth, eighth, dotted quarter
            ZeldaSong::Saria => &[
                (F4, Eighth),
                (A4, Eighth),
                (B4, DotQuarter),
                (F4, Eighth),
                (A4, Eighth),
                (B4, DotQuarter),
            ],
            // 4/4 march: quarter, quarter, half
            ZeldaSong::Sun => &[
                (A4, Quarter),
                (F4, Quarter),
                (D5, Half),
                (A4, Quarter),
                (F4, Quarter),
                (D5, Half),
            ],
            // 3/4 stately: half, quarter, dotted half
            ZeldaSong::Time => &[
                (A4, Half),
                (D4, Quarter),
                (F4, DotHalf),
                (A4, Half),
                (D4, Quarter),
                (F4, DotHalf),
            ],
            // 6/8 driving: eighth, eighth, dotted quarter
            ZeldaSong::Storms => &[
                (D4, Eighth),
                (F4, Eighth),
                (D5, DotQuarter),
                (D4, Eighth),
                (F4, Eighth),
                (D5, DotQuarter),
            ],
            // quarters with held final
            ZeldaSong::Minuet => &[
                (D4, Quarter),
                (D5, Quarter),
                (B4, Quarter),
                (A4, Quarter),
                (B4, Quarter),
                (A4, Half),
            ],
            // even quarters, held final
            ZeldaSong::Bolero => &[
                (F4, Quarter),
                (D4, Quarter),
                (F4, Quarter),
                (D4, Quarter),
                (A4, Quarter),
                (F4, Quarter),
                (A4, Quarter),
                (F4, Half),
            ],
            // flowing halves, pickup quarter, held final
            ZeldaSong::Serenade => &[
                (D4, Half),
                (F4, Half),
                (A4, Half),
                (A4, Quarter),
                (B4, DotHalf),
            ],
            // alternating half/quarter, held final
            ZeldaSong::Nocturne => &[
                (B4, Half),
                (A4, Quarter),
                (A4, Quarter),
                (D4, Half),
                (B4, Quarter),
                (A4, Quarter),
                (F4, Half),
            ],
            // three quarters then descending halves
            ZeldaSong::Requiem => &[
                (D4, Quarter),
                (F4, Quarter),
                (D4, Quarter),
                (A4, Half),
                (F4, Half),
                (D4, Half),
            ],
            // quick quarters, held final
            ZeldaSong::Prelude => &[
                (D5, Quarter),
                (A4, Quarter),
                (D5, Quarter),
                (A4, Quarter),
                (B4, Quarter),
                (D5, Half),
            ],
        }
    }
}

/// Ocarina of Time songs.
pub fn play_zelda(player: &Player, vol: f32, song: &ZeldaSong) {
    let notes = song.notes();

    let mut buf = Buf::new();
    for &(pitch, len) in notes {
        buf.sine_lp(
            note_freq(pitch as i32),
            beat(len.beats(), DEFAULT_BPM),
            2000.0,
            vol,
        );
    }
    buf.play(player);
}
