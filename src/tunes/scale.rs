use super::{Buf, beat, hour_to_root, note_freq};
use rodio::Player;

const DEFAULT_BPM: f32 = 150.0;

/// Scale style: 5-note ascending run.
/// Hours 0-11 use major intervals, 12-23 use minor intervals.
pub fn play_scale(player: &Player, vol: f32, hour: u32) {
    let root = hour_to_root(hour);
    let intervals: &[i32] = if hour >= 12 {
        &[0, 2, 3, 5, 7]
    } else {
        &[0, 2, 4, 5, 7]
    };

    let mut buf = Buf::new();
    for (i, &interval) in intervals.iter().enumerate() {
        let beats = if i == intervals.len() - 1 { 1.0 } else { 0.5 };
        buf.sine_lp(
            note_freq(root + interval),
            beat(beats, DEFAULT_BPM),
            3000.0,
            vol,
        );
    }
    buf.play(player);
}
