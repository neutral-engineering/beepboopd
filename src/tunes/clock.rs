use super::{Buf, beat, note_freq};
use rodio::Player;

const DEFAULT_BPM: f32 = 240.0;

/// Westminster-style clock: melodic chime intro then deep bongs for the hour.
pub fn play_clock(player: &Player, vol: f32, hour: u32) {
    let e5 = note_freq(7);
    let c5 = note_freq(3);
    let d5 = note_freq(5);
    let g4 = note_freq(-2);

    // 3/4 time: notes 1-3 are quarter (1 beat), note 4 is half (2 beats)
    let chime: &[(f32, f32)] = &[
        (e5, 1.0),
        (c5, 1.0),
        (d5, 1.0),
        (g4, 2.0),
        (g4, 1.0),
        (d5, 1.0),
        (e5, 1.0),
        (c5, 2.0),
    ];

    let mut buf = Buf::new();
    for (i, &(freq, beats)) in chime.iter().enumerate() {
        buf.sine_lp(freq, beat(beats, DEFAULT_BPM), 3000.0, vol * 0.8);
        if i == 3 {
            buf.silence(beat(1.6, DEFAULT_BPM));
        }
    }

    buf.silence(beat(2.0, DEFAULT_BPM));

    // BONG strikes
    let bong = note_freq(-12);
    let count = match hour % 12 {
        0 => 12,
        h => h,
    };
    for i in 0..count {
        buf.sine_lp(bong, beat(2.4, DEFAULT_BPM), 1000.0, vol);
        if i < count - 1 {
            buf.silence(beat(2.0, DEFAULT_BPM));
        }
    }
    buf.play(player);
}
