use super::{Buf, beat, hour_to_root, note_freq};
use rodio::Player;

const DEFAULT_BPM: f32 = 160.0;

/// The Lick — the most famous jazz phrase, transposed to all 12 keys.
/// D E F G E C D (in D dorian) = intervals 0, 2, 3, 5, 2, -2, 0 from root.
/// Swing eighths: long-short pairs (2/3 + 1/3 of a beat).
const LICK: &[(i32, f32)] = &[
    (0, 0.67), // D  — long swing eighth
    (2, 0.33), // E  — short swing eighth
    (3, 0.67), // F  — long
    (5, 0.33), // G  — short
    (2, 1.0),  // E  — quarter
    (-2, 1.0), // C  — quarter
    (0, 2.0),  // D  — quarter
];

pub fn play_jazz(player: &Player, vol: f32, bpm: Option<f32>, hour: u32) {
    let root = hour_to_root(hour);

    let mut buf = Buf::new();
    for &(interval, beats) in LICK {
        buf.sine_lp(
            note_freq(root + interval),
            beat(beats, bpm.unwrap_or(DEFAULT_BPM)),
            2500.0,
            vol,
        );
    }
    buf.play(player);
}
