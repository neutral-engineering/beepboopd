use super::{Buf, beat};
use rodio::Player;

const DEFAULT_BPM: f32 = 200.0;

/// Wee woo wee woo (for --help).
pub fn play_wee_woo(player: &Player, vol: f32) {
    let mut buf = Buf::new();
    for _ in 0..2 {
        buf.sine_lp(880.0, beat(0.5, DEFAULT_BPM), 2000.0, vol);
        buf.sine_lp(660.0, beat(0.5, DEFAULT_BPM), 2000.0, vol);
    }
    buf.play(player);
}
