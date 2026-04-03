use super::{Buf, SR, beat, hour_to_root, note_freq};
use rodio::Player;
use std::f32::consts::PI;

const DEFAULT_BPM: f32 = 150.0;

/// Chord style: unique chord per hour.
/// Hours 0-11 get major chords, 12-23 get minor chords.
pub fn play_chords(player: &Player, vol: f32, hour: u32) {
    let root = hour_to_root(hour);
    let third = if hour >= 12 { 3 } else { 4 };
    let f1 = note_freq(root);
    let f2 = note_freq(root + third);
    let f3 = note_freq(root + 7);
    let amp = vol / 1.7;

    let dur = beat(2.0, DEFAULT_BPM);
    let mut buf = Buf::new();
    let n = (SR * dur) as usize;
    let rc = 1.0 / (2.0 * PI * 3000.0);
    let dt = 1.0 / SR;
    let alpha = dt / (rc + dt);
    let mut lp = 0.0_f32;
    let mut p1 = 0.0_f32;
    let mut p2 = 0.0_f32;
    let mut p3 = 0.0_f32;
    let inc1 = 2.0 * PI * f1 / SR;
    let inc2 = 2.0 * PI * f2 / SR;
    let inc3 = 2.0 * PI * f3 / SR;
    for _ in 0..n {
        let raw = amp * (p1.sin() + p2.sin() + p3.sin()) / 3.0;
        lp += alpha * (raw - lp);
        buf.samples.push(lp);
        p1 += inc1;
        p2 += inc2;
        p3 += inc3;
    }
    buf.play(player);
}
